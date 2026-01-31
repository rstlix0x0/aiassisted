//! Config domain for application settings management.
//!
//! This module handles reading, writing, and validating application
//! configuration stored in TOML format at `~/.aiassisted/config.toml`.

pub mod commands;
pub mod settings;
pub mod toml_store;

pub use commands::{EditCommand, GetCommand, PathCommand, ResetCommand, ShowCommand};
pub use toml_store::TomlConfigStore;
