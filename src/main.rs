//! aiassisted CLI - Embed AI assistant guidelines and templates into projects.

use std::sync::Arc;

use clap::Parser;

mod cli;
mod core;
mod infra;

use cli::{Cli, Commands, ConfigCommands, TemplatesCommands};
use core::infra::{Checksum, FileSystem, HttpClient, Logger};
use infra::{ColoredLogger, ReqwestClient, Sha2Checksum, StdFileSystem};

/// Application context holding all infrastructure dependencies.
#[allow(dead_code)]
struct AppContext {
    fs: Arc<dyn FileSystem>,
    http: Arc<dyn HttpClient>,
    checksum: Arc<dyn Checksum>,
    logger: Arc<dyn Logger>,
}

impl AppContext {
    fn new(verbosity: u8) -> Self {
        Self {
            fs: Arc::new(StdFileSystem::new()),
            http: Arc::new(ReqwestClient::new()),
            checksum: Arc::new(Sha2Checksum::new()),
            logger: Arc::new(ColoredLogger::new(verbosity)),
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let verbosity = cli.verbose.max(1); // Default to 1 if not specified
    let ctx = AppContext::new(verbosity);

    match cli.command {
        Commands::Install(args) => {
            ctx.logger.info(&format!(
                "Installing .aiassisted to {}",
                args.path.display()
            ));
            ctx.logger.warn("Install command not yet implemented");
        }

        Commands::Update(args) => {
            ctx.logger.info(&format!(
                "Updating .aiassisted in {}{}",
                args.path.display(),
                if args.force { " (forced)" } else { "" }
            ));
            ctx.logger.warn("Update command not yet implemented");
        }

        Commands::Check(args) => {
            ctx.logger.info(&format!(
                "Checking for updates in {}",
                args.path.display()
            ));
            ctx.logger.warn("Check command not yet implemented");
        }

        Commands::SetupSkills(args) => {
            let tool: core::ToolType = args.tool.into();
            ctx.logger.info(&format!(
                "Setting up skills for {}{}",
                tool,
                if args.dry_run { " (dry run)" } else { "" }
            ));
            ctx.logger.warn("Setup-skills command not yet implemented");
        }

        Commands::SetupAgents(args) => {
            let tool: core::ToolType = args.tool.into();
            ctx.logger.info(&format!(
                "Setting up agents for {}{}",
                tool,
                if args.dry_run { " (dry run)" } else { "" }
            ));
            ctx.logger.warn("Setup-agents command not yet implemented");
        }

        Commands::Templates(args) => match args.command {
            TemplatesCommands::List { tool } => {
                let tool: core::ToolType = tool.into();
                ctx.logger.info(&format!("Listing templates for {}", tool));
                ctx.logger.warn("Templates list command not yet implemented");
            }
            TemplatesCommands::Show { path } => {
                ctx.logger.info(&format!("Showing template: {}", path));
                ctx.logger.warn("Templates show command not yet implemented");
            }
            TemplatesCommands::Init { force } => {
                ctx.logger.info(&format!(
                    "Initializing project templates{}",
                    if force { " (forced)" } else { "" }
                ));
                ctx.logger.warn("Templates init command not yet implemented");
            }
            TemplatesCommands::Sync { force } => {
                ctx.logger.info(&format!(
                    "Syncing templates{}",
                    if force { " (forced)" } else { "" }
                ));
                ctx.logger.warn("Templates sync command not yet implemented");
            }
            TemplatesCommands::Path => {
                ctx.logger.info("Template paths:");
                ctx.logger.warn("Templates path command not yet implemented");
            }
            TemplatesCommands::Diff { path } => {
                if let Some(p) = path {
                    ctx.logger.info(&format!("Diffing template: {}", p));
                } else {
                    ctx.logger.info("Diffing all templates");
                }
                ctx.logger.warn("Templates diff command not yet implemented");
            }
        },

        Commands::Config(args) => match args.command {
            ConfigCommands::Show => {
                ctx.logger.info("Current configuration:");
                ctx.logger.warn("Config show command not yet implemented");
            }
            ConfigCommands::Get { key } => {
                ctx.logger.info(&format!("Getting config key: {}", key));
                ctx.logger.warn("Config get command not yet implemented");
            }
            ConfigCommands::Edit => {
                ctx.logger.info("Opening config in editor");
                ctx.logger.warn("Config edit command not yet implemented");
            }
            ConfigCommands::Reset => {
                ctx.logger.info("Resetting config to defaults");
                ctx.logger.warn("Config reset command not yet implemented");
            }
            ConfigCommands::Path => {
                ctx.logger.info("Config file path:");
                ctx.logger.warn("Config path command not yet implemented");
            }
        },

        Commands::SelfUpdate => {
            ctx.logger.info("Checking for CLI updates...");
            ctx.logger.warn("Self-update command not yet implemented");
        }

        Commands::Version => {
            println!("aiassisted {}", env!("CARGO_PKG_VERSION"));
        }
    }
}
