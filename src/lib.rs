//! aiassisted - CLI tool for embedding AI assistant guidelines into projects
//!
//! This library provides core functionality for managing `.aiassisted/` directories
//! that contain curated guidelines, instructions, prompts, and skills for AI assistants.

// Public modules for external use and testing
pub mod config;
pub mod content;
pub mod core;
pub mod infra;
pub mod migration;
pub mod selfupdate;
pub mod skills;

// Re-export commonly used types
pub use core::types::{Error, ManifestEntry, Result};
pub use content::manifest::Manifest;
