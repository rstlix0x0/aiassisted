//! Sync logic for installing and updating .aiassisted content.

use std::path::Path;

use crate::core::infra::{Checksum, FileSystem, HttpClient, Logger};
use crate::core::types::Result;

use super::github;
use super::manifest::Manifest;

/// Install .aiassisted to a target directory.
pub async fn install<F, H, C, L>(
    fs: &F,
    http: &H,
    checksum: &C,
    logger: &L,
    target_dir: &Path,
) -> Result<()>
where
    F: FileSystem,
    H: HttpClient,
    C: Checksum,
    L: Logger,
{
    let aiassisted_dir = target_dir.join(".aiassisted");

    // Check if already installed
    if fs.exists(&aiassisted_dir) {
        logger.warn("Directory .aiassisted already exists. Use 'update' to update it.");
        return Ok(());
    }

    logger.info("Downloading manifest...");
    let manifest = Manifest::load_remote(http, &github::manifest_url()).await?;

    logger.info(&format!(
        "Manifest loaded: version {}, {} files",
        manifest.version,
        manifest.files.len()
    ));

    // Create .aiassisted directory
    fs.create_dir_all(&aiassisted_dir).await?;

    // Download all files
    logger.info("Downloading files...");
    let downloaded = github::download_batch(http, checksum, fs, &manifest.files, target_dir).await?;

    logger.success(&format!(
        "Successfully installed {} files to {}",
        downloaded.len(),
        aiassisted_dir.display()
    ));

    // Save manifest locally
    let manifest_path = aiassisted_dir.join("manifest.json");
    manifest.save(fs, &manifest_path).await?;

    Ok(())
}

/// Update existing .aiassisted installation.
pub async fn update<F, H, C, L>(
    fs: &F,
    http: &H,
    checksum: &C,
    logger: &L,
    target_dir: &Path,
    force: bool,
) -> Result<()>
where
    F: FileSystem,
    H: HttpClient,
    C: Checksum,
    L: Logger,
{
    let aiassisted_dir = target_dir.join(".aiassisted");

    // Check if installed
    if !fs.exists(&aiassisted_dir) {
        logger.warn("Directory .aiassisted not found. Use 'install' first.");
        return Ok(());
    }

    logger.info("Checking for updates...");

    // Load local and remote manifests
    let local_manifest_path = aiassisted_dir.join("manifest.json");
    let local_manifest = Manifest::load_local(fs, &local_manifest_path).await?;
    let remote_manifest = Manifest::load_remote(http, &github::manifest_url()).await?;

    logger.info(&format!(
        "Local: v{}, Remote: v{}",
        local_manifest.version, remote_manifest.version
    ));

    if force {
        logger.info("Force update: downloading all files...");
        let downloaded =
            github::download_batch(http, checksum, fs, &remote_manifest.files, target_dir).await?;

        logger.success(&format!("Updated {} files (forced)", downloaded.len()));
    } else {
        // Compare manifests
        let diff = local_manifest.diff(&remote_manifest);

        if !diff.has_changes() {
            logger.info("No updates available.");
            return Ok(());
        }

        logger.info(&format!(
            "Updates available: {} new, {} modified",
            diff.new_files.len(),
            diff.modified_files.len()
        ));

        // Download only changed files
        let files_to_download = diff.files_to_download();
        let downloaded =
            github::download_batch(http, checksum, fs, &files_to_download, target_dir).await?;

        logger.success(&format!("Updated {} files", downloaded.len()));
    }

    // Save updated manifest
    remote_manifest.save(fs, &local_manifest_path).await?;

    Ok(())
}

/// Check for updates without downloading.
pub async fn check<F, H, L>(
    fs: &F,
    http: &H,
    logger: &L,
    target_dir: &Path,
) -> Result<()>
where
    F: FileSystem,
    H: HttpClient,
    L: Logger,
{
    let aiassisted_dir = target_dir.join(".aiassisted");

    // Check if installed
    if !fs.exists(&aiassisted_dir) {
        logger.warn("Directory .aiassisted not found. Use 'install' first.");
        return Ok(());
    }

    logger.info("Checking for updates...");

    // Load local and remote manifests
    let local_manifest_path = aiassisted_dir.join("manifest.json");
    let local_manifest = Manifest::load_local(fs, &local_manifest_path).await?;
    let remote_manifest = Manifest::load_remote(http, &github::manifest_url()).await?;

    logger.info(&format!(
        "Local: v{}, Remote: v{}",
        local_manifest.version, remote_manifest.version
    ));

    // Compare manifests
    let diff = local_manifest.diff(&remote_manifest);

    if !diff.has_changes() {
        logger.success("No updates available. You're up to date!");
        return Ok(());
    }

    logger.info(&format!(
        "Updates available: {} new, {} modified",
        diff.new_files.len(),
        diff.modified_files.len()
    ));

    // List new files
    if !diff.new_files.is_empty() {
        logger.info("New files:");
        for entry in &diff.new_files {
            logger.info(&format!("  + {}", entry.path.display()));
        }
    }

    // List modified files
    if !diff.modified_files.is_empty() {
        logger.info("Modified files:");
        for entry in &diff.modified_files {
            logger.info(&format!("  ~ {}", entry.path.display()));
        }
    }

    logger.info("Run 'aiassisted update' to download updates.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::{mock, predicate::*};
    use std::path::PathBuf;
    use tempfile::TempDir;

    // Mock implementations
    mock! {
        pub FileSystem {}

        #[async_trait::async_trait]
        impl crate::core::infra::FileSystem for FileSystem {
            async fn read(&self, path: &Path) -> Result<String>;
            async fn write(&self, path: &Path, content: &str) -> Result<()>;
            fn exists(&self, path: &Path) -> bool;
            fn is_dir(&self, path: &Path) -> bool;
            fn is_file(&self, path: &Path) -> bool;
            async fn create_dir_all(&self, path: &Path) -> Result<()>;
            async fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
            async fn copy(&self, from: &Path, to: &Path) -> Result<()>;
        }
    }

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
        pub Logger {}

        impl crate::core::infra::Logger for Logger {
            fn info(&self, msg: &str);
            fn warn(&self, msg: &str);
            fn error(&self, msg: &str);
            fn debug(&self, msg: &str);
            fn success(&self, msg: &str);
        }
    }

    #[tokio::test]
    async fn test_install_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let mut mock_fs = MockFileSystem::new();
        let mock_http = MockHttpClient::new();
        let mock_checksum = MockChecksum::new();
        let mut mock_logger = MockLogger::new();

        // Directory already exists
        mock_fs.expect_exists().times(1).returning(|_| true);

        mock_logger
            .expect_warn()
            .times(1)
            .withf(|msg: &str| msg.contains("already exists"))
            .return_const(());

        let result = install(
            &mock_fs,
            &mock_http,
            &mock_checksum,
            &mock_logger,
            temp_dir.path(),
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_not_installed() {
        let temp_dir = TempDir::new().unwrap();
        let mut mock_fs = MockFileSystem::new();
        let mock_http = MockHttpClient::new();
        let mut mock_logger = MockLogger::new();

        // Directory doesn't exist
        mock_fs.expect_exists().times(1).returning(|_| false);

        mock_logger
            .expect_warn()
            .times(1)
            .withf(|msg: &str| msg.contains("not found"))
            .return_const(());

        let result = check(&mock_fs, &mock_http, &mock_logger, temp_dir.path()).await;

        assert!(result.is_ok());
    }
}
