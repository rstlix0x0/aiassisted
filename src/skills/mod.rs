//! Skills domain - install and manage AI skills

mod commands;
mod copier;
mod diff;
mod discovery;

pub use commands::{SetupSkillsCommand, SkillsListCommand, SkillsUpdateCommand};
pub use copier::{SkillCopier, SkillInfo};
pub use diff::{FileStatus, SkillDiff, SkillDiffer, SkillStatus, SkillsUpdateDiff};
pub use discovery::ToolDetector;
