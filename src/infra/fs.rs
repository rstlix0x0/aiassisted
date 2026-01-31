//! Standard file system implementation.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::core::infra::FileSystem;
use crate::core::types::Result;

/// File system implementation using standard library.
#[derive(Debug, Clone, Default)]
pub struct StdFileSystem;

impl StdFileSystem {
    /// Create a new StdFileSystem instance.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl FileSystem for StdFileSystem {
    async fn read(&self, path: &Path) -> Result<String> {
        Ok(fs::read_to_string(path).await?)
    }

    async fn read_bytes(&self, path: &Path) -> Result<Vec<u8>> {
        Ok(fs::read(path).await?)
    }

    async fn write(&self, path: &Path, content: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let mut file = fs::File::create(path).await?;
        file.write_all(content.as_bytes()).await?;
        Ok(())
    }

    async fn write_bytes(&self, path: &Path, content: &[u8]) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let mut file = fs::File::create(path).await?;
        file.write_all(content).await?;
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    async fn create_dir_all(&self, path: &Path) -> Result<()> {
        Ok(fs::create_dir_all(path).await?)
    }

    async fn remove_file(&self, path: &Path) -> Result<()> {
        Ok(fs::remove_file(path).await?)
    }

    async fn remove_dir_all(&self, path: &Path) -> Result<()> {
        Ok(fs::remove_dir_all(path).await?)
    }

    async fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let mut entries = Vec::new();
        let mut dir = fs::read_dir(path).await?;
        while let Some(entry) = dir.next_entry().await? {
            entries.push(entry.path());
        }
        Ok(entries)
    }

    async fn copy(&self, from: &Path, to: &Path) -> Result<()> {
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::copy(from, to).await?;
        Ok(())
    }
}
