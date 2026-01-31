//! Migration domain for upgrading from shell-based to Rust version.
//!
//! This module handles migrating user installations from the old shell-based
//! version (POSIX shell scripts) to the new Rust version.

pub mod commands;
pub mod shell_config;

pub use commands::MigrateCommand;
