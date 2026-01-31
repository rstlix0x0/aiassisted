//! Self-update domain for CLI binary updates.
//!
//! This module handles checking for and downloading new CLI versions from GitHub Releases.

pub mod commands;
pub mod github_releases;
pub mod platform;
pub mod version;

pub use commands::SelfUpdateCommand;
pub use github_releases::GithubReleasesProvider;
