//! Self-update domain trait abstractions.
//!
//! These traits define the interfaces for CLI self-update operations.

use std::path::Path;

use async_trait::async_trait;

use super::types::{ReleaseInfo, Result};

/// Abstraction for checking and downloading releases.
#[async_trait]
pub trait ReleaseProvider: Send + Sync {
    /// Get the latest release information.
    async fn get_latest(&self) -> Result<ReleaseInfo>;

    /// Check if an update is available.
    ///
    /// Compares the current version with the latest release.
    async fn is_update_available(&self, current_version: &str) -> Result<bool>;

    /// Download a release to the specified destination.
    async fn download_release(&self, release: &ReleaseInfo, dest: &Path) -> Result<()>;
}
