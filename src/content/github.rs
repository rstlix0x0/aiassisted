//! GitHub API utilities for downloading .aiassisted content.

use std::path::{Path, PathBuf};

use crate::core::infra::{Checksum, FileSystem, HttpClient};
use crate::core::types::{Error, ManifestEntry, Result};

/// Base URL for raw GitHub content.
pub const GITHUB_RAW_BASE: &str = "https://raw.githubusercontent.com/rstlix0x0/aiassisted/main";

/// Manifest file path relative to repository root.
pub const MANIFEST_PATH: &str = ".aiassisted/manifest.json";

/// Get the full URL for the manifest file.
pub fn manifest_url() -> String {
    format!("{}/{}", GITHUB_RAW_BASE, MANIFEST_PATH)
}

/// Get the full URL for a content file.
pub fn content_url(path: &Path) -> String {
    format!(
        "{}/.aiassisted/{}",
        GITHUB_RAW_BASE,
        path.display()
    )
}

/// Download a single file from GitHub with checksum verification.
pub async fn download_file<H, C, F>(
    http: &H,
    checksum: &C,
    fs: &F,
    entry: &ManifestEntry,
    dest_dir: &Path,
) -> Result<()>
where
    H: HttpClient,
    C: Checksum,
    F: FileSystem,
{
    let url = content_url(&entry.path);
    let dest_path = dest_dir.join(".aiassisted").join(&entry.path);

    // Download content
    let content = http.get(&url).await?;

    // Verify checksum
    let actual_checksum = checksum.sha256(content.as_bytes());
    if actual_checksum != entry.checksum {
        return Err(Error::ChecksumMismatch {
            expected: entry.checksum.clone(),
            actual: actual_checksum,
        });
    }

    // Ensure parent directory exists
    if let Some(parent) = dest_path.parent() {
        fs.create_dir_all(parent).await?;
    }

    // Write file
    fs.write(&dest_path, &content).await?;

    Ok(())
}

/// Download multiple files in batch.
pub async fn download_batch<H, C, F>(
    http: &H,
    checksum: &C,
    fs: &F,
    entries: &[ManifestEntry],
    dest_dir: &Path,
) -> Result<Vec<PathBuf>>
where
    H: HttpClient,
    C: Checksum,
    F: FileSystem,
{
    let mut downloaded = Vec::new();

    for entry in entries {
        download_file(http, checksum, fs, entry, dest_dir).await?;
        downloaded.push(dest_dir.join(".aiassisted").join(&entry.path));
    }

    Ok(downloaded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::{mock, predicate::*};
    use tempfile::TempDir;

    // Mock implementations (reuse from manifest tests)
    mock! {
        pub HttpClient {}

        #[async_trait::async_trait]
        impl crate::core::infra::HttpClient for HttpClient {
            async fn get(&self, url: &str) -> Result<String>;
            async fn get_bytes(&self, url: &str) -> Result<Vec<u8>>;
            async fn download(&self, url: &str, dest: &Path) -> Result<()>;
        }
    }

    mock! {
        pub Checksum {}

        impl crate::core::infra::Checksum for Checksum {
            fn sha256(&self, content: &[u8]) -> String;
            fn sha256_file(&self, path: &Path) -> Result<String>;
        }
    }

    mock! {
        pub FileSystem {}

        #[async_trait::async_trait]
        impl crate::core::infra::FileSystem for FileSystem {
            async fn read(&self, path: &Path) -> Result<String>;
            async fn read_bytes(&self, path: &Path) -> Result<Vec<u8>>;
            async fn write(&self, path: &Path, content: &str) -> Result<()>;
            async fn write_bytes(&self, path: &Path, content: &[u8]) -> Result<()>;
            fn exists(&self, path: &Path) -> bool;
            fn is_dir(&self, path: &Path) -> bool;
            fn is_file(&self, path: &Path) -> bool;
            async fn create_dir_all(&self, path: &Path) -> Result<()>;
            async fn remove_file(&self, path: &Path) -> Result<()>;
            async fn remove_dir_all(&self, path: &Path) -> Result<()>;
            async fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
            async fn copy(&self, from: &Path, to: &Path) -> Result<()>;
        }
    }

    #[test]
    fn test_manifest_url() {
        let url = manifest_url();
        assert_eq!(
            url,
            "https://raw.githubusercontent.com/rstlix0x0/aiassisted/main/.aiassisted/manifest.json"
        );
    }

    #[test]
    fn test_content_url() {
        let path = Path::new("guidelines/architecture.md");
        let url = content_url(path);
        assert_eq!(
            url,
            "https://raw.githubusercontent.com/rstlix0x0/aiassisted/main/.aiassisted/guidelines/architecture.md"
        );
    }

    #[test]
    fn test_content_url_nested_path() {
        let path = Path::new("templates/skills/commit.md");
        let url = content_url(path);
        assert_eq!(
            url,
            "https://raw.githubusercontent.com/rstlix0x0/aiassisted/main/.aiassisted/templates/skills/commit.md"
        );
    }

    #[test]
    fn test_content_url_root_file() {
        let path = Path::new("README.md");
        let url = content_url(path);
        assert_eq!(
            url,
            "https://raw.githubusercontent.com/rstlix0x0/aiassisted/main/.aiassisted/README.md"
        );
    }

    #[tokio::test]
    async fn test_download_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let entry = ManifestEntry {
            path: PathBuf::from("test.txt"),
            checksum: "abc123".to_string(),
        };

        let mut mock_http = MockHttpClient::new();
        let mut mock_checksum = MockChecksum::new();
        let mut mock_fs = MockFileSystem::new();

        // Expect HTTP GET
        mock_http
            .expect_get()
            .times(1)
            .returning(|_| Ok("file content".to_string()));

        // Expect checksum calculation
        mock_checksum
            .expect_sha256()
            .times(1)
            .returning(|_| "abc123".to_string());

        // Expect directory creation
        mock_fs
            .expect_create_dir_all()
            .times(1)
            .returning(|_| Ok(()));

        // Expect file write
        mock_fs
            .expect_write()
            .times(1)
            .returning(|_, _| Ok(()));

        let result = download_file(
            &mock_http,
            &mock_checksum,
            &mock_fs,
            &entry,
            temp_dir.path(),
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_download_file_checksum_mismatch() {
        let temp_dir = TempDir::new().unwrap();
        let entry = ManifestEntry {
            path: PathBuf::from("test.txt"),
            checksum: "expected_checksum".to_string(),
        };

        let mut mock_http = MockHttpClient::new();
        let mut mock_checksum = MockChecksum::new();
        let mock_fs = MockFileSystem::new();

        mock_http
            .expect_get()
            .times(1)
            .returning(|_| Ok("file content".to_string()));

        mock_checksum
            .expect_sha256()
            .times(1)
            .returning(|_| "wrong_checksum".to_string());

        let result = download_file(
            &mock_http,
            &mock_checksum,
            &mock_fs,
            &entry,
            temp_dir.path(),
        )
        .await;

        assert!(result.is_err());
        match result {
            Err(Error::ChecksumMismatch { expected, actual }) => {
                assert_eq!(expected, "expected_checksum");
                assert_eq!(actual, "wrong_checksum");
            }
            _ => panic!("Expected ChecksumMismatch error"),
        }
    }

    #[tokio::test]
    async fn test_download_file_network_error() {
        let temp_dir = TempDir::new().unwrap();
        let entry = ManifestEntry {
            path: PathBuf::from("test.txt"),
            checksum: "abc123".to_string(),
        };

        let mut mock_http = MockHttpClient::new();
        let mock_checksum = MockChecksum::new();
        let mock_fs = MockFileSystem::new();

        mock_http
            .expect_get()
            .times(1)
            .returning(|_| Err(Error::Network("Connection failed".to_string())));

        let result = download_file(
            &mock_http,
            &mock_checksum,
            &mock_fs,
            &entry,
            temp_dir.path(),
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_download_batch_success() {
        let temp_dir = TempDir::new().unwrap();
        let entries = vec![
            ManifestEntry {
                path: PathBuf::from("file1.txt"),
                checksum: "checksum1".to_string(),
            },
            ManifestEntry {
                path: PathBuf::from("file2.txt"),
                checksum: "checksum2".to_string(),
            },
        ];

        let mut mock_http = MockHttpClient::new();
        let mut mock_checksum = MockChecksum::new();
        let mut mock_fs = MockFileSystem::new();

        // Expect 2 HTTP GETs
        mock_http
            .expect_get()
            .times(2)
            .returning(|_| Ok("content".to_string()));

        // Expect 2 checksum calculations
        mock_checksum
            .expect_sha256()
            .times(1)
            .returning(|_| "checksum1".to_string());
        mock_checksum
            .expect_sha256()
            .times(1)
            .returning(|_| "checksum2".to_string());

        // Expect directory creations and file writes
        mock_fs
            .expect_create_dir_all()
            .times(2)
            .returning(|_| Ok(()));

        mock_fs
            .expect_write()
            .times(2)
            .returning(|_, _| Ok(()));

        let result = download_batch(
            &mock_http,
            &mock_checksum,
            &mock_fs,
            &entries,
            temp_dir.path(),
        )
        .await;

        assert!(result.is_ok());
        let downloaded = result.unwrap();
        assert_eq!(downloaded.len(), 2);
    }

    #[tokio::test]
    async fn test_download_batch_partial_failure() {
        let temp_dir = TempDir::new().unwrap();
        let entries = vec![
            ManifestEntry {
                path: PathBuf::from("file1.txt"),
                checksum: "checksum1".to_string(),
            },
            ManifestEntry {
                path: PathBuf::from("file2.txt"),
                checksum: "checksum2".to_string(),
            },
        ];

        let mut mock_http = MockHttpClient::new();
        let mut mock_checksum = MockChecksum::new();
        let mut mock_fs = MockFileSystem::new();

        // First file succeeds
        mock_http
            .expect_get()
            .times(1)
            .returning(|_| Ok("content".to_string()));

        mock_checksum
            .expect_sha256()
            .times(1)
            .returning(|_| "checksum1".to_string());

        mock_fs
            .expect_create_dir_all()
            .times(1)
            .returning(|_| Ok(()));

        mock_fs
            .expect_write()
            .times(1)
            .returning(|_, _| Ok(()));

        // Second file fails
        mock_http
            .expect_get()
            .times(1)
            .returning(|_| Err(Error::Network("Failed".to_string())));

        let result = download_batch(
            &mock_http,
            &mock_checksum,
            &mock_fs,
            &entries,
            temp_dir.path(),
        )
        .await;

        // Should fail on the second file
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_download_batch_empty() {
        let temp_dir = TempDir::new().unwrap();
        let entries: Vec<ManifestEntry> = vec![];

        let mock_http = MockHttpClient::new();
        let mock_checksum = MockChecksum::new();
        let mock_fs = MockFileSystem::new();

        let result = download_batch(
            &mock_http,
            &mock_checksum,
            &mock_fs,
            &entries,
            temp_dir.path(),
        )
        .await;

        assert!(result.is_ok());
        let downloaded = result.unwrap();
        assert_eq!(downloaded.len(), 0);
    }
}
