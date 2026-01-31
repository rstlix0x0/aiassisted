//! Infrastructure trait abstractions.
//!
//! These traits define the interfaces for infrastructure concerns like
//! file system operations, HTTP clients, checksums, and logging.

use std::path::{Path, PathBuf};

use async_trait::async_trait;

use super::types::Result;

/// Abstraction for file system operations.
#[async_trait]
pub trait FileSystem: Send + Sync {
    /// Read the contents of a file as a string.
    async fn read(&self, path: &Path) -> Result<String>;

    /// Read the contents of a file as bytes.
    async fn read_bytes(&self, path: &Path) -> Result<Vec<u8>>;

    /// Write content to a file.
    async fn write(&self, path: &Path, content: &str) -> Result<()>;

    /// Write bytes to a file.
    async fn write_bytes(&self, path: &Path, content: &[u8]) -> Result<()>;

    /// Check if a path exists.
    fn exists(&self, path: &Path) -> bool;

    /// Check if a path is a directory.
    fn is_dir(&self, path: &Path) -> bool;

    /// Check if a path is a file.
    fn is_file(&self, path: &Path) -> bool;

    /// Create a directory and all parent directories.
    async fn create_dir_all(&self, path: &Path) -> Result<()>;

    /// Remove a file.
    async fn remove_file(&self, path: &Path) -> Result<()>;

    /// Remove a directory and all its contents.
    async fn remove_dir_all(&self, path: &Path) -> Result<()>;

    /// List entries in a directory.
    async fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;

    /// Copy a file from source to destination.
    async fn copy(&self, from: &Path, to: &Path) -> Result<()>;
}

/// Abstraction for HTTP client operations.
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// Perform a GET request and return the response body as a string.
    async fn get(&self, url: &str) -> Result<String>;

    /// Perform a GET request and return the response body as bytes.
    async fn get_bytes(&self, url: &str) -> Result<Vec<u8>>;

    /// Download a file from a URL to a destination path.
    async fn download(&self, url: &str, dest: &Path) -> Result<()>;
}

/// Abstraction for checksum operations.
pub trait Checksum: Send + Sync {
    /// Calculate SHA256 checksum of content.
    fn sha256(&self, content: &[u8]) -> String;

    /// Calculate SHA256 checksum of a file.
    fn sha256_file(&self, path: &Path) -> Result<String>;
}

/// Abstraction for logging operations.
pub trait Logger: Send + Sync {
    /// Log an informational message.
    fn info(&self, msg: &str);

    /// Log a warning message.
    fn warn(&self, msg: &str);

    /// Log an error message.
    fn error(&self, msg: &str);

    /// Log a debug message.
    fn debug(&self, msg: &str);

    /// Log a success message.
    fn success(&self, msg: &str);
}
