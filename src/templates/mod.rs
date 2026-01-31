//! Templates domain for AI skill/agent file generation.
//!
//! This module handles template processing, resolution, and generation
//! for AI assistant skills and agents.

pub mod commands;
pub mod discovery;
pub mod engine;
pub mod generator;
pub mod resolver;

pub use commands::{
    ListTemplatesCommand, SetupAgentsCommand, SetupSkillsCommand, ShowTemplateCommand,
    TemplatesDiffCommand, TemplatesInitCommand, TemplatesPathCommand, TemplatesSyncCommand,
};
