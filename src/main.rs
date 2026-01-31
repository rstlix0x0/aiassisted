//! aiassisted CLI - Embed AI assistant guidelines and templates into projects.

use clap::Parser;

mod cli;
mod config;
mod content;
mod core;
mod infra;
mod templates;

use cli::{Cli, Commands, ConfigCommands, TemplatesCommands};
use config::{
    EditCommand as ConfigEditCommand, GetCommand as ConfigGetCommand,
    PathCommand as ConfigPathCommand, ResetCommand as ConfigResetCommand,
    ShowCommand as ConfigShowCommand, TomlConfigStore,
};
use content::{CheckCommand, InstallCommand, UpdateCommand};
use core::infra::{Checksum, FileSystem, HttpClient, Logger};
use infra::{ColoredLogger, ReqwestClient, Sha2Checksum, StdFileSystem};
use templates::{
    ListTemplatesCommand, SetupAgentsCommand, SetupSkillsCommand, ShowTemplateCommand,
    TemplatesDiffCommand, TemplatesInitCommand, TemplatesPathCommand, TemplatesSyncCommand,
};
use templates::engine::SimpleTemplateEngine;
use templates::resolver::CascadingResolver;

/// Application context holding all infrastructure dependencies.
/// Uses static dispatch (generics) for zero-cost abstractions.
struct AppContext<F, H, C, L>
where
    F: FileSystem,
    H: HttpClient,
    C: Checksum,
    L: Logger,
{
    #[allow(dead_code)] // Used in Phase 3 (content domain commands)
    fs: F,
    #[allow(dead_code)] // Used in Phase 3 (content domain commands)
    http: H,
    #[allow(dead_code)] // Used in Phase 3 (content domain commands)
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
            let tool: core::ToolType = args.tool.into();
            let cmd = SetupSkillsCommand {
                tool,
                dry_run: args.dry_run,
            };

            // Create template dependencies
            let engine = SimpleTemplateEngine::new();
            let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            let resolver = CascadingResolver::new(
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
                home_dir,
            );
            let project_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

            cmd.execute(&ctx.fs, &engine, &resolver, &ctx.logger, &project_path).await
        }

        Commands::SetupAgents(args) => {
            let tool: core::ToolType = args.tool.into();
            let cmd = SetupAgentsCommand {
                tool,
                dry_run: args.dry_run,
            };

            // Create template dependencies
            let engine = SimpleTemplateEngine::new();
            let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            let resolver = CascadingResolver::new(
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
                home_dir,
            );
            let project_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

            cmd.execute(&ctx.fs, &engine, &resolver, &ctx.logger, &project_path).await
        }

        Commands::Templates(args) => {
            let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            let project_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let resolver = CascadingResolver::new(project_path.clone(), home_dir);

            match args.command {
                TemplatesCommands::List { tool } => {
                    let tool: core::ToolType = tool.into();
                    let cmd = ListTemplatesCommand { tool };
                    cmd.execute(&ctx.fs, &resolver, &ctx.logger, &project_path).await
                }
                TemplatesCommands::Show { path } => {
                    let cmd = ShowTemplateCommand { path };
                    // Default to Claude for show command
                    cmd.execute(&ctx.fs, &resolver, &ctx.logger, core::ToolType::Claude).await
                }
                TemplatesCommands::Init { force } => {
                    let cmd = TemplatesInitCommand { force };
                    cmd.execute(&ctx.fs, &resolver, &ctx.logger, &project_path).await
                }
                TemplatesCommands::Sync { force } => {
                    let cmd = TemplatesSyncCommand { force };
                    cmd.execute(&ctx.fs, &resolver, &ctx.logger, &project_path).await
                }
                TemplatesCommands::Path => {
                    let cmd = TemplatesPathCommand;
                    cmd.execute(&resolver, &ctx.logger, &project_path).await
                }
                TemplatesCommands::Diff { path } => {
                    let cmd = TemplatesDiffCommand { path };
                    cmd.execute(&ctx.fs, &resolver, &ctx.logger, &ctx.checksum, &project_path).await
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
            ctx.logger.info("Checking for CLI updates...");
            ctx.logger.warn("Self-update command not yet implemented");
            Ok(())
        }

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
