//! Content domain for managing .aiassisted directory content.
//!
//! This module handles installing, updating, and checking the .aiassisted
//! directory structure that contains guidelines, templates, and instructions.

pub mod commands;
pub mod github;
pub mod manifest;
pub mod sync;

pub use commands::{CheckCommand, InstallCommand, UpdateCommand};
