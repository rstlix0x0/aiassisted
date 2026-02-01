//! Skills domain - install and manage AI skills

mod commands;
mod copier;
mod discovery;

pub use commands::{SetupSkillsCommand, SkillsListCommand};
pub use copier::{SkillCopier, SkillInfo};
pub use discovery::ToolDetector;
