//! aiassisted CLI - Embed AI assistant guidelines and templates into projects.

use clap::Parser;

// Binary-only module for CLI argument parsing
mod cli;

// Import from library crate using package name
use cli::{Cli, Commands, ConfigCommands, SkillsCommands, AgentsCommands};
use aiassisted::agents::{AgentsListCommand, AgentsSetupCommand, AgentsUpdateCommand};
use aiassisted::config::{
    EditCommand as ConfigEditCommand, GetCommand as ConfigGetCommand,
    PathCommand as ConfigPathCommand, ResetCommand as ConfigResetCommand,
    ShowCommand as ConfigShowCommand, TomlConfigStore,
};
use aiassisted::content::{CheckCommand, InstallCommand, UpdateCommand};
use aiassisted::core::infra::{Checksum, FileSystem, HttpClient, Logger};
use aiassisted::infra::{ColoredLogger, ReqwestClient, Sha2Checksum, StdFileSystem};
use aiassisted::migration::MigrateCommand;
use aiassisted::selfupdate::{GithubReleasesProvider, SelfUpdateCommand};
use aiassisted::skills::{SetupSkillsCommand, SkillsListCommand, SkillsUpdateCommand};

/// Application context holding all infrastructure dependencies.
/// Uses static dispatch (generics) for zero-cost abstractions.
struct AppContext<F, H, C, L>
where
    F: FileSystem,
    H: HttpClient,
    C: Checksum,
    L: Logger,
{
    fs: F,
    http: H,
    checksum: C,
    logger: L,
}

impl<F, H, C, L> AppContext<F, H, C, L>
where
    F: FileSystem,
    H: HttpClient,
    C: Checksum,
    L: Logger,
{
    fn new(fs: F, http: H, checksum: C, logger: L) -> Self {
        Self {
            fs,
            http,
            checksum,
            logger,
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let verbosity = cli.verbose.max(1); // Default to 1 if not specified

    // Create infrastructure with concrete types (static dispatch)
    let fs = StdFileSystem::new();
    let http = ReqwestClient::new();
    let checksum = Sha2Checksum::new();
    let logger = ColoredLogger::new(verbosity);

    let ctx = AppContext::new(fs, http, checksum, logger);

    let result = match cli.command {
        Commands::Install(args) => {
            let cmd = InstallCommand { path: args.path };
            cmd.execute(&ctx.fs, &ctx.http, &ctx.checksum, &ctx.logger)
                .await
        }

        Commands::Update(args) => {
            let cmd = UpdateCommand {
                path: args.path,
                force: args.force,
            };
            cmd.execute(&ctx.fs, &ctx.http, &ctx.checksum, &ctx.logger)
                .await
        }

        Commands::Check(args) => {
            let cmd = CheckCommand { path: args.path };
            cmd.execute(&ctx.fs, &ctx.http, &ctx.logger).await
        }

        Commands::SetupSkills(args) => {
            // Deprecation warning
            ctx.logger.warn("'setup-skills' is deprecated. Use 'aiassisted skills setup' instead.");

            let tool: aiassisted::core::ToolType = args.tool.into();
            let cmd = SetupSkillsCommand {
                tool,
                dry_run: args.dry_run,
                force: args.force,
            };
            let project_path =
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

            cmd.execute(&ctx.fs, &ctx.logger, &project_path).await
        }

        Commands::Skills(args) => {
            let project_path =
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

            match args.command {
                SkillsCommands::Setup {
                    tool,
                    dry_run,
                    force,
                } => {
                    let tool: aiassisted::core::ToolType = tool.into();
                    let cmd = SetupSkillsCommand {
                        tool,
                        dry_run,
                        force,
                    };
                    cmd.execute(&ctx.fs, &ctx.logger, &project_path).await
                }
                SkillsCommands::List { tool } => {
                    let tool: aiassisted::core::ToolType = tool.into();
                    let cmd = SkillsListCommand { tool };
                    cmd.execute(&ctx.fs, &ctx.logger, &project_path).await
                }
                SkillsCommands::Update {
                    tool,
                    dry_run,
                    force,
                } => {
                    let tool: aiassisted::core::ToolType = tool.into();
                    let cmd = SkillsUpdateCommand {
                        tool,
                        dry_run,
                        force,
                    };
                    cmd.execute(&ctx.fs, &ctx.checksum, &ctx.logger, &project_path)
                        .await
                }
            }
        }

        Commands::Agents(args) => {
            let project_path =
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

            match args.command {
                None => {
                    // Default: list agents
                    let cmd = AgentsListCommand;
                    cmd.execute(&ctx.fs, &ctx.logger, &project_path).await
                }
                Some(AgentsCommands::Setup {
                    platform,
                    dry_run,
                    force,
                }) => {
                    let platform: aiassisted::agents::Platform = platform.into();
                    let cmd = AgentsSetupCommand {
                        platform,
                        dry_run,
                        force,
                    };
                    cmd.execute(&ctx.fs, &ctx.logger, &project_path).await
                }
                Some(AgentsCommands::Update {
                    platform,
                    dry_run,
                    force,
                }) => {
                    let platform: aiassisted::agents::Platform = platform.into();
                    let cmd = AgentsUpdateCommand {
                        platform,
                        dry_run,
                        force,
                    };
                    cmd.execute(&ctx.fs, &ctx.checksum, &ctx.logger, &project_path)
                        .await
                }
            }
        }

        Commands::Config(args) => async {
            // Create config store
            let config_store = TomlConfigStore::new(StdFileSystem::new())?;

            match args.command {
                ConfigCommands::Show => {
                    let cmd = ConfigShowCommand;
                    cmd.execute(&config_store, &ctx.logger).await
                }
                ConfigCommands::Get { key } => {
                    let cmd = ConfigGetCommand { key };
                    cmd.execute(&config_store, &ctx.logger).await
                }
                ConfigCommands::Edit => {
                    let cmd = ConfigEditCommand;
                    cmd.execute(&config_store, &ctx.logger).await
                }
                ConfigCommands::Reset { force } => {
                    let cmd = ConfigResetCommand { force };
                    cmd.execute(&config_store, &ctx.logger).await
                }
                ConfigCommands::Path => {
                    let cmd = ConfigPathCommand;
                    cmd.execute(&config_store).await
                }
            }
        }
        .await,

        Commands::SelfUpdate => {
            let provider = GithubReleasesProvider::new(ctx.http);
            let command = SelfUpdateCommand;
            command.execute(&provider, &ctx.logger).await
        }

        Commands::Migrate => async {
            let config_store = TomlConfigStore::new(StdFileSystem::new())?;
            let cmd = MigrateCommand;
            cmd.execute(&ctx.fs, &config_store, &ctx.logger).await.map(|_| ())
        }
        .await,

        Commands::Version => {
            println!("aiassisted {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    };

    // Handle errors
    if let Err(e) = result {
        ctx.logger.error(&format!("Error: {}", e));
        std::process::exit(1);
    }
}
