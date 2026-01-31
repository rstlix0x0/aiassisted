//! aiassisted - CLI tool for embedding AI assistant guidelines into projects
//!
//! This library provides core functionality for managing `.aiassisted/` directories
//! that contain curated guidelines, instructions, prompts, and templates for AI assistants.

// Public modules for external use and testing
pub mod core;
pub mod infra;
pub mod content;
pub mod templates;
pub mod config;

// Re-export commonly used types
pub use core::types::{Error, ManifestEntry, Result};
pub use content::manifest::Manifest;
