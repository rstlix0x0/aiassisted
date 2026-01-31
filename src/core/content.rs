//! Content domain trait abstractions.
//!
//! These traits define the interfaces for content management operations
//! like loading manifests and downloading content files.

use std::path::{Path, PathBuf};

use async_trait::async_trait;

use super::types::{ManifestEntry, Result};

/// Abstraction for loading and saving manifests.
#[async_trait]
#[allow(dead_code)] // Used in Phase 3 (content domain)
pub trait ManifestStore: Send + Sync {
    /// Load manifest from a local path.
    async fn load_local(&self, path: &Path) -> Result<Vec<ManifestEntry>>;

    /// Load manifest from a remote URL.
    async fn load_remote(&self, url: &str) -> Result<Vec<ManifestEntry>>;

    /// Save manifest to a local path.
    async fn save(&self, path: &Path, entries: &[ManifestEntry]) -> Result<()>;
}

/// Abstraction for downloading content files.
#[async_trait]
#[allow(dead_code)] // Used in Phase 3 (content domain)
pub trait ContentDownloader: Send + Sync {
    /// Download a single file and return its content.
    ///
    /// Verifies the checksum if provided.
    async fn download_file(&self, url: &str, expected_checksum: Option<&str>) -> Result<String>;

    /// Download multiple files in batch.
    ///
    /// Returns a list of (path, content) tuples.
    async fn download_batch(
        &self,
        base_url: &str,
        entries: &[ManifestEntry],
    ) -> Result<Vec<(PathBuf, String)>>;
}
