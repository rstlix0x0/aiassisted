//! Core module containing all abstractions.
//!
//! This module defines the shared types and trait abstractions that
//! all domains depend on. Following the dependency inversion principle,
//! domains depend on these abstractions, not on concrete implementations.
//!
//! # Module Structure
//!
//! - [`types`] - Shared types (Error, Result, ToolType, DTOs)
//! - [`infra`] - Infrastructure traits (FileSystem, HttpClient, Checksum, Logger)
//! - [`templates`] - Templates domain traits (TemplateEngine, TemplateResolver)
//! - [`config`] - Config domain traits (ConfigStore)
//! - [`selfupdate`] - Self-update domain traits (ReleaseProvider)

pub mod config;
pub mod infra;
pub mod selfupdate;
pub mod templates;
pub mod types;

// Re-export commonly used types for convenience
pub use types::ToolType;
