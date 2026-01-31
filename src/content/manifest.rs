//! Manifest parsing and verification.
//!
//! The manifest.json file contains a list of all files in the .aiassisted
//! directory along with their SHA256 checksums.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::infra::{Checksum, FileSystem, HttpClient};
use crate::core::types::{Error, ManifestEntry, Result};

/// Manifest structure matching the JSON format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub files: Vec<ManifestEntry>,
}

impl Manifest {
    /// Load manifest from a local file.
    pub async fn load_local<F: FileSystem>(fs: &F, path: &Path) -> Result<Self> {
        let content = fs.read(path).await?;
        serde_json::from_str(&content).map_err(|e| Error::Serialization(e.to_string()))
    }

    /// Load manifest from a remote URL.
    pub async fn load_remote<H: HttpClient>(http: &H, url: &str) -> Result<Self> {
        let content = http.get(url).await?;
        serde_json::from_str(&content).map_err(|e| Error::Serialization(e.to_string()))
    }

    /// Save manifest to a local file.
    pub async fn save<F: FileSystem>(&self, fs: &F, path: &Path) -> Result<()> {
        let content =
            serde_json::to_string_pretty(self).map_err(|e| Error::Serialization(e.to_string()))?;
        fs.write(path, &content).await
    }

    /// Verify checksums of all files in the manifest.
    pub fn verify_checksums<C: Checksum, F: FileSystem>(
        &self,
        checksum: &C,
        fs: &F,
        base_path: &Path,
    ) -> Result<Vec<(ManifestEntry, bool)>> {
        let mut results = Vec::new();

        for entry in &self.files {
            let file_path = base_path.join(&entry.path);

            if !fs.exists(&file_path) {
                results.push((entry.clone(), false));
                continue;
            }

            match checksum.sha256_file(&file_path) {
                Ok(actual_checksum) => {
                    let matches = actual_checksum == entry.checksum;
                    results.push((entry.clone(), matches));
                }
                Err(_) => {
                    results.push((entry.clone(), false));
                }
            }
        }

        Ok(results)
    }

    /// Compare this manifest with another to find differences.
    pub fn diff(&self, other: &Manifest) -> ManifestDiff {
        let mut new_files = Vec::new();
        let mut modified_files = Vec::new();
        let mut unchanged_files = Vec::new();

        for other_entry in &other.files {
            match self
                .files
                .iter()
                .find(|e| e.path == other_entry.path)
            {
                Some(local_entry) => {
                    if local_entry.checksum == other_entry.checksum {
                        unchanged_files.push(other_entry.clone());
                    } else {
                        modified_files.push(other_entry.clone());
                    }
                }
                None => {
                    new_files.push(other_entry.clone());
                }
            }
        }

        ManifestDiff {
            new_files,
            modified_files,
            unchanged_files,
        }
    }
}

/// Difference between two manifests.
#[derive(Debug)]
pub struct ManifestDiff {
    pub new_files: Vec<ManifestEntry>,
    pub modified_files: Vec<ManifestEntry>,
    pub unchanged_files: Vec<ManifestEntry>,
}

impl ManifestDiff {
    /// Check if there are any changes.
    pub fn has_changes(&self) -> bool {
        !self.new_files.is_empty() || !self.modified_files.is_empty()
    }

    /// Get all files that need to be downloaded.
    pub fn files_to_download(&self) -> Vec<ManifestEntry> {
        let mut files = Vec::new();
        files.extend(self.new_files.clone());
        files.extend(self.modified_files.clone());
        files
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::{mock, predicate::*};
    use std::path::PathBuf;
    use tempfile::TempDir;

    // Mock FileSystem for testing
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

    // Mock HttpClient for testing
    mock! {
        pub HttpClient {}

        #[async_trait::async_trait]
        impl crate::core::infra::HttpClient for HttpClient {
            async fn get(&self, url: &str) -> Result<String>;
            async fn get_bytes(&self, url: &str) -> Result<Vec<u8>>;
            async fn download(&self, url: &str, dest: &Path) -> Result<()>;
        }
    }

    // Mock Checksum for testing
    mock! {
        pub Checksum {}

        impl crate::core::infra::Checksum for Checksum {
            fn sha256(&self, content: &[u8]) -> String;
            fn sha256_file(&self, path: &Path) -> Result<String>;
        }
    }

    #[test]
    fn test_manifest_diff_no_changes() {
        let manifest1 = Manifest {
            version: "1.0.0".to_string(),
            files: vec![ManifestEntry {
                path: PathBuf::from("file1.txt"),
                checksum: "abc123".to_string(),
            }],
        };

        let manifest2 = manifest1.clone();
        let diff = manifest1.diff(&manifest2);

        assert!(!diff.has_changes());
        assert_eq!(diff.new_files.len(), 0);
        assert_eq!(diff.modified_files.len(), 0);
        assert_eq!(diff.unchanged_files.len(), 1);
    }

    #[test]
    fn test_manifest_diff_new_file() {
        let manifest1 = Manifest {
            version: "1.0.0".to_string(),
            files: vec![],
        };

        let manifest2 = Manifest {
            version: "1.0.1".to_string(),
            files: vec![ManifestEntry {
                path: PathBuf::from("file1.txt"),
                checksum: "abc123".to_string(),
            }],
        };

        let diff = manifest1.diff(&manifest2);

        assert!(diff.has_changes());
        assert_eq!(diff.new_files.len(), 1);
        assert_eq!(diff.modified_files.len(), 0);
    }

    #[test]
    fn test_manifest_diff_modified_file() {
        let manifest1 = Manifest {
            version: "1.0.0".to_string(),
            files: vec![ManifestEntry {
                path: PathBuf::from("file1.txt"),
                checksum: "abc123".to_string(),
            }],
        };

        let manifest2 = Manifest {
            version: "1.0.1".to_string(),
            files: vec![ManifestEntry {
                path: PathBuf::from("file1.txt"),
                checksum: "def456".to_string(),
            }],
        };

        let diff = manifest1.diff(&manifest2);

        assert!(diff.has_changes());
        assert_eq!(diff.new_files.len(), 0);
        assert_eq!(diff.modified_files.len(), 1);
    }

    #[test]
    fn test_manifest_diff_multiple_changes() {
        let manifest1 = Manifest {
            version: "1.0.0".to_string(),
            files: vec![
                ManifestEntry {
                    path: PathBuf::from("unchanged.txt"),
                    checksum: "same123".to_string(),
                },
                ManifestEntry {
                    path: PathBuf::from("modified.txt"),
                    checksum: "old456".to_string(),
                },
            ],
        };

        let manifest2 = Manifest {
            version: "1.0.1".to_string(),
            files: vec![
                ManifestEntry {
                    path: PathBuf::from("unchanged.txt"),
                    checksum: "same123".to_string(),
                },
                ManifestEntry {
                    path: PathBuf::from("modified.txt"),
                    checksum: "new456".to_string(),
                },
                ManifestEntry {
                    path: PathBuf::from("new.txt"),
                    checksum: "new789".to_string(),
                },
            ],
        };

        let diff = manifest1.diff(&manifest2);

        assert!(diff.has_changes());
        assert_eq!(diff.new_files.len(), 1);
        assert_eq!(diff.modified_files.len(), 1);
        assert_eq!(diff.unchanged_files.len(), 1);
    }

    #[test]
    fn test_files_to_download() {
        let manifest1 = Manifest {
            version: "1.0.0".to_string(),
            files: vec![ManifestEntry {
                path: PathBuf::from("file1.txt"),
                checksum: "abc123".to_string(),
            }],
        };

        let manifest2 = Manifest {
            version: "1.0.1".to_string(),
            files: vec![
                ManifestEntry {
                    path: PathBuf::from("file1.txt"),
                    checksum: "def456".to_string(),
                },
                ManifestEntry {
                    path: PathBuf::from("file2.txt"),
                    checksum: "ghi789".to_string(),
                },
            ],
        };

        let diff = manifest1.diff(&manifest2);
        let to_download = diff.files_to_download();

        assert_eq!(to_download.len(), 2); // 1 modified + 1 new
    }

    #[test]
    fn test_diff_empty_manifests() {
        let manifest1 = Manifest {
            version: "1.0.0".to_string(),
            files: vec![],
        };

        let manifest2 = Manifest {
            version: "1.0.1".to_string(),
            files: vec![],
        };

        let diff = manifest1.diff(&manifest2);

        assert!(!diff.has_changes());
        assert_eq!(diff.new_files.len(), 0);
        assert_eq!(diff.modified_files.len(), 0);
    }

    #[tokio::test]
    async fn test_load_local_success() {
        let mut mock_fs = MockFileSystem::new();
        let manifest_json = r#"{"version":"1.0.0","files":[{"path":"test.txt","checksum":"abc123"}]}"#;

        mock_fs
            .expect_read()
            .times(1)
            .returning(move |_| Ok(manifest_json.to_string()));

        let result = Manifest::load_local(&mock_fs, Path::new("manifest.json")).await;
        assert!(result.is_ok());

        let manifest = result.unwrap();
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.files.len(), 1);
        assert_eq!(manifest.files[0].path, PathBuf::from("test.txt"));
    }

    #[tokio::test]
    async fn test_load_local_invalid_json() {
        let mut mock_fs = MockFileSystem::new();

        mock_fs
            .expect_read()
            .times(1)
            .returning(|_| Ok("invalid json".to_string()));

        let result = Manifest::load_local(&mock_fs, Path::new("manifest.json")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_local_file_not_found() {
        let mut mock_fs = MockFileSystem::new();

        mock_fs
            .expect_read()
            .times(1)
            .returning(|_| Err(Error::NotFound("File not found".to_string())));

        let result = Manifest::load_local(&mock_fs, Path::new("missing.json")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_remote_success() {
        let mut mock_http = MockHttpClient::new();
        let manifest_json = r#"{"version":"2.0.0","files":[{"path":"remote.txt","checksum":"def456"}]}"#;

        mock_http
            .expect_get()
            .times(1)
            .returning(move |_| Ok(manifest_json.to_string()));

        let result = Manifest::load_remote(&mock_http, "https://example.com/manifest.json").await;
        assert!(result.is_ok());

        let manifest = result.unwrap();
        assert_eq!(manifest.version, "2.0.0");
        assert_eq!(manifest.files.len(), 1);
    }

    #[tokio::test]
    async fn test_load_remote_network_error() {
        let mut mock_http = MockHttpClient::new();

        mock_http
            .expect_get()
            .times(1)
            .returning(|_| Err(Error::Network("Connection failed".to_string())));

        let result = Manifest::load_remote(&mock_http, "https://example.com/manifest.json").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_save_success() {
        let mut mock_fs = MockFileSystem::new();
        let manifest = Manifest {
            version: "1.0.0".to_string(),
            files: vec![ManifestEntry {
                path: PathBuf::from("test.txt"),
                checksum: "abc123".to_string(),
            }],
        };

        mock_fs
            .expect_write()
            .times(1)
            .returning(|_, _| Ok(()));

        let result = manifest.save(&mock_fs, Path::new("output.json")).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_checksums_all_match() {
        let manifest = Manifest {
            version: "1.0.0".to_string(),
            files: vec![ManifestEntry {
                path: PathBuf::from("test.txt"),
                checksum: "abc123".to_string(),
            }],
        };

        let mut mock_checksum = MockChecksum::new();
        let mut mock_fs = MockFileSystem::new();

        mock_fs.expect_exists().times(1).returning(|_| true);

        mock_checksum
            .expect_sha256_file()
            .times(1)
            .returning(|_| Ok("abc123".to_string()));

        let temp_dir = TempDir::new().unwrap();
        let results = manifest
            .verify_checksums(&mock_checksum, &mock_fs, temp_dir.path())
            .unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].1); // Checksum matches
    }

    #[tokio::test]
    async fn test_verify_checksums_mismatch() {
        let manifest = Manifest {
            version: "1.0.0".to_string(),
            files: vec![ManifestEntry {
                path: PathBuf::from("test.txt"),
                checksum: "abc123".to_string(),
            }],
        };

        let mut mock_checksum = MockChecksum::new();
        let mut mock_fs = MockFileSystem::new();

        mock_fs.expect_exists().times(1).returning(|_| true);

        mock_checksum
            .expect_sha256_file()
            .times(1)
            .returning(|_| Ok("different".to_string()));

        let temp_dir = TempDir::new().unwrap();
        let results = manifest
            .verify_checksums(&mock_checksum, &mock_fs, temp_dir.path())
            .unwrap();

        assert_eq!(results.len(), 1);
        assert!(!results[0].1); // Checksum does not match
    }

    #[tokio::test]
    async fn test_verify_checksums_file_missing() {
        let manifest = Manifest {
            version: "1.0.0".to_string(),
            files: vec![ManifestEntry {
                path: PathBuf::from("missing.txt"),
                checksum: "abc123".to_string(),
            }],
        };

        let mock_checksum = MockChecksum::new();
        let mut mock_fs = MockFileSystem::new();

        mock_fs.expect_exists().times(1).returning(|_| false);

        let temp_dir = TempDir::new().unwrap();
        let results = manifest
            .verify_checksums(&mock_checksum, &mock_fs, temp_dir.path())
            .unwrap();

        assert_eq!(results.len(), 1);
        assert!(!results[0].1); // File doesn't exist
    }
}
