//! CLI definitions using Clap.

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use aiassisted::core::ToolType;

/// CLI tool for embedding AI assistant guidelines and templates into projects.
#[derive(Parser, Debug)]
#[command(name = "aiassisted")]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Verbosity level (-v for info, -vv for debug)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Install .aiassisted directory to a project
    Install(InstallArgs),

    /// Update existing .aiassisted installation
    Update(UpdateArgs),

    /// Check if updates are available
    Check(CheckArgs),

    /// Set up AI skills (copy to tool directory)
    SetupSkills(SetupSkillsArgs),

    /// Manage skills
    Skills(SkillsArgs),

    /// Manage configuration
    Config(ConfigArgs),

    /// Update the CLI binary itself
    SelfUpdate,

    /// Migrate from old shell-based version
    Migrate,

    /// Show version information
    Version,
}

/// Arguments for the install command.
#[derive(Parser, Debug)]
pub struct InstallArgs {
    /// Target directory path
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,
}

/// Arguments for the update command.
#[derive(Parser, Debug)]
pub struct UpdateArgs {
    /// Force update without confirmation
    #[arg(short, long)]
    pub force: bool,

    /// Target directory path
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,
}

/// Arguments for the check command.
#[derive(Parser, Debug)]
pub struct CheckArgs {
    /// Target directory path
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,
}

/// Arguments for the setup-skills command.
#[derive(Parser, Debug)]
pub struct SetupSkillsArgs {
    /// AI tool to generate skills for
    #[arg(short, long, value_enum, default_value = "auto")]
    pub tool: CliToolType,

    /// Show what would be created without creating
    #[arg(long)]
    pub dry_run: bool,

    /// Overwrite existing skills
    #[arg(long)]
    pub force: bool,
}

/// Arguments for the skills command.
#[derive(Parser, Debug)]
pub struct SkillsArgs {
    #[command(subcommand)]
    pub command: SkillsCommands,
}

/// Skills subcommands.
#[derive(Subcommand, Debug)]
pub enum SkillsCommands {
    /// List available skills
    List {
        /// AI tool to list skills for
        #[arg(short, long, value_enum, default_value = "auto")]
        tool: CliToolType,
    },
}

/// Arguments for the config command.
#[derive(Parser, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

/// Config subcommands.
#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,

    /// Get a specific configuration value
    Get {
        /// Configuration key (e.g., default_tool, verbosity)
        key: String,
    },

    /// Edit configuration in $EDITOR
    Edit,

    /// Reset configuration to defaults
    Reset {
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Show configuration file path
    Path,
}

/// CLI tool type enum for Clap.
#[derive(ValueEnum, Clone, Debug, Default)]
pub enum CliToolType {
    #[default]
    Auto,
    OpenCode,
    Claude,
}

impl From<CliToolType> for ToolType {
    fn from(cli: CliToolType) -> Self {
        match cli {
            CliToolType::Auto => ToolType::Auto,
            CliToolType::OpenCode => ToolType::OpenCode,
            CliToolType::Claude => ToolType::Claude,
        }
    }
}
