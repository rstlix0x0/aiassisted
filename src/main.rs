//! aiassisted CLI - Embed AI assistant guidelines and templates into projects.

use clap::Parser;

mod cli;
mod content;
mod core;
mod infra;

use cli::{Cli, Commands, ConfigCommands, TemplatesCommands};
use content::{CheckCommand, InstallCommand, UpdateCommand};
use core::infra::{Checksum, FileSystem, HttpClient, Logger};
use infra::{ColoredLogger, ReqwestClient, Sha2Checksum, StdFileSystem};

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
            ctx.logger.info(&format!(
                "Setting up skills for {}{}",
                tool,
                if args.dry_run { " (dry run)" } else { "" }
            ));
            ctx.logger.warn("Setup-skills command not yet implemented");
            Ok(())
        }

        Commands::SetupAgents(args) => {
            let tool: core::ToolType = args.tool.into();
            ctx.logger.info(&format!(
                "Setting up agents for {}{}",
                tool,
                if args.dry_run { " (dry run)" } else { "" }
            ));
            ctx.logger.warn("Setup-agents command not yet implemented");
            Ok(())
        }

        Commands::Templates(args) => match args.command {
            TemplatesCommands::List { tool } => {
                let tool: core::ToolType = tool.into();
                ctx.logger.info(&format!("Listing templates for {}", tool));
                ctx.logger.warn("Templates list command not yet implemented");
                Ok(())
            }
            TemplatesCommands::Show { path } => {
                ctx.logger.info(&format!("Showing template: {}", path));
                ctx.logger.warn("Templates show command not yet implemented");
                Ok(())
            }
            TemplatesCommands::Init { force } => {
                ctx.logger.info(&format!(
                    "Initializing project templates{}",
                    if force { " (forced)" } else { "" }
                ));
                ctx.logger.warn("Templates init command not yet implemented");
                Ok(())
            }
            TemplatesCommands::Sync { force } => {
                ctx.logger.info(&format!(
                    "Syncing templates{}",
                    if force { " (forced)" } else { "" }
                ));
                ctx.logger.warn("Templates sync command not yet implemented");
                Ok(())
            }
            TemplatesCommands::Path => {
                ctx.logger.info("Template paths:");
                ctx.logger.warn("Templates path command not yet implemented");
                Ok(())
            }
            TemplatesCommands::Diff { path } => {
                if let Some(p) = path {
                    ctx.logger.info(&format!("Diffing template: {}", p));
                } else {
                    ctx.logger.info("Diffing all templates");
                }
                ctx.logger.warn("Templates diff command not yet implemented");
                Ok(())
            }
        },

        Commands::Config(args) => match args.command {
            ConfigCommands::Show => {
                ctx.logger.info("Current configuration:");
                ctx.logger.warn("Config show command not yet implemented");
                Ok(())
            }
            ConfigCommands::Get { key } => {
                ctx.logger.info(&format!("Getting config key: {}", key));
                ctx.logger.warn("Config get command not yet implemented");
                Ok(())
            }
            ConfigCommands::Edit => {
                ctx.logger.info("Opening config in editor");
                ctx.logger.warn("Config edit command not yet implemented");
                Ok(())
            }
            ConfigCommands::Reset => {
                ctx.logger.info("Resetting config to defaults");
                ctx.logger.warn("Config reset command not yet implemented");
                Ok(())
            }
            ConfigCommands::Path => {
                ctx.logger.info("Config file path:");
                ctx.logger.warn("Config path command not yet implemented");
                Ok(())
            }
        },

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
