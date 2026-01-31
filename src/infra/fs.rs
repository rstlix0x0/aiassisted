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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_write_and_read_string() {
        let fs = StdFileSystem::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Write
        fs.write(&file_path, "Hello, World!").await.unwrap();

        // Read
        let content = fs.read(&file_path).await.unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[tokio::test]
    async fn test_write_and_read_bytes() {
        let fs = StdFileSystem::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.bin");

        let data = vec![0u8, 1, 2, 3, 255];

        // Write bytes
        fs.write_bytes(&file_path, &data).await.unwrap();

        // Read bytes
        let content = fs.read_bytes(&file_path).await.unwrap();
        assert_eq!(content, data);
    }

    #[tokio::test]
    async fn test_write_creates_parent_dirs() {
        let fs = StdFileSystem::new();
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("a/b/c/test.txt");

        // Parent directories don't exist yet
        assert!(!fs.exists(&temp_dir.path().join("a")));

        // Write should create them
        fs.write(&nested_path, "test").await.unwrap();

        // Verify file exists
        assert!(fs.exists(&nested_path));
        assert!(fs.is_file(&nested_path));
    }

    #[tokio::test]
    async fn test_read_nonexistent_file() {
        let fs = StdFileSystem::new();
        let result = fs.read(Path::new("/nonexistent/file.txt")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_exists() {
        let fs = StdFileSystem::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("exists_test.txt");

        assert!(!fs.exists(&file_path));

        fs.write(&file_path, "content").await.unwrap();

        assert!(fs.exists(&file_path));
    }

    #[tokio::test]
    async fn test_is_dir_and_is_file() {
        let fs = StdFileSystem::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("file.txt");
        let dir_path = temp_dir.path().join("subdir");

        fs.write(&file_path, "content").await.unwrap();
        fs.create_dir_all(&dir_path).await.unwrap();

        assert!(fs.is_file(&file_path));
        assert!(!fs.is_dir(&file_path));

        assert!(fs.is_dir(&dir_path));
        assert!(!fs.is_file(&dir_path));
    }

    #[tokio::test]
    async fn test_create_dir_all() {
        let fs = StdFileSystem::new();
        let temp_dir = TempDir::new().unwrap();
        let nested_dir = temp_dir.path().join("a/b/c/d");

        fs.create_dir_all(&nested_dir).await.unwrap();

        assert!(fs.exists(&nested_dir));
        assert!(fs.is_dir(&nested_dir));
    }

    #[tokio::test]
    async fn test_remove_file() {
        let fs = StdFileSystem::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("to_delete.txt");

        fs.write(&file_path, "delete me").await.unwrap();
        assert!(fs.exists(&file_path));

        fs.remove_file(&file_path).await.unwrap();
        assert!(!fs.exists(&file_path));
    }

    #[tokio::test]
    async fn test_remove_nonexistent_file() {
        let fs = StdFileSystem::new();
        let result = fs.remove_file(Path::new("/nonexistent/file.txt")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_dir_all() {
        let fs = StdFileSystem::new();
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("dir_to_remove");
        let file_path = dir_path.join("file.txt");

        fs.write(&file_path, "content").await.unwrap();
        assert!(fs.exists(&dir_path));

        fs.remove_dir_all(&dir_path).await.unwrap();
        assert!(!fs.exists(&dir_path));
    }

    #[tokio::test]
    async fn test_list_dir() {
        let fs = StdFileSystem::new();
        let temp_dir = TempDir::new().unwrap();

        // Create some files
        fs.write(&temp_dir.path().join("file1.txt"), "1")
            .await
            .unwrap();
        fs.write(&temp_dir.path().join("file2.txt"), "2")
            .await
            .unwrap();
        fs.create_dir_all(&temp_dir.path().join("subdir"))
            .await
            .unwrap();

        let entries = fs.list_dir(temp_dir.path()).await.unwrap();

        assert_eq!(entries.len(), 3);
        assert!(entries.iter().any(|p| p.ends_with("file1.txt")));
        assert!(entries.iter().any(|p| p.ends_with("file2.txt")));
        assert!(entries.iter().any(|p| p.ends_with("subdir")));
    }

    #[tokio::test]
    async fn test_list_empty_dir() {
        let fs = StdFileSystem::new();
        let temp_dir = TempDir::new().unwrap();

        let entries = fs.list_dir(temp_dir.path()).await.unwrap();
        assert_eq!(entries.len(), 0);
    }

    #[tokio::test]
    async fn test_list_nonexistent_dir() {
        let fs = StdFileSystem::new();
        let result = fs.list_dir(Path::new("/nonexistent/dir")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_copy_file() {
        let fs = StdFileSystem::new();
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("dest.txt");

        fs.write(&source, "copy me").await.unwrap();
        fs.copy(&source, &dest).await.unwrap();

        let content = fs.read(&dest).await.unwrap();
        assert_eq!(content, "copy me");

        // Both files should exist
        assert!(fs.exists(&source));
        assert!(fs.exists(&dest));
    }

    #[tokio::test]
    async fn test_copy_creates_parent_dirs() {
        let fs = StdFileSystem::new();
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("a/b/c/dest.txt");

        fs.write(&source, "copy me").await.unwrap();
        fs.copy(&source, &dest).await.unwrap();

        assert!(fs.exists(&dest));
        let content = fs.read(&dest).await.unwrap();
        assert_eq!(content, "copy me");
    }

    #[tokio::test]
    async fn test_copy_nonexistent_source() {
        let fs = StdFileSystem::new();
        let temp_dir = TempDir::new().unwrap();
        let source = Path::new("/nonexistent/source.txt");
        let dest = temp_dir.path().join("dest.txt");

        let result = fs.copy(source, &dest).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_write_unicode_content() {
        let fs = StdFileSystem::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("unicode.txt");

        let unicode_text = "Hello, 世界! 🦀 Rust";
        fs.write(&file_path, unicode_text).await.unwrap();

        let content = fs.read(&file_path).await.unwrap();
        assert_eq!(content, unicode_text);
    }
}
