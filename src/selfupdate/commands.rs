//! Self-update command implementation.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use tar::Archive;

use crate::core::infra::Logger;
use crate::core::selfupdate::ReleaseProvider;
use crate::core::types::{Error, Result};

/// Self-update command for updating the CLI binary.
pub struct SelfUpdateCommand;

impl SelfUpdateCommand {
    /// Execute the self-update command.
    ///
    /// Checks for updates, downloads if available, and replaces the current binary.
    pub async fn execute<R: ReleaseProvider, L: Logger>(
        &self,
        provider: &R,
        logger: &L,
    ) -> Result<()> {
        let current_version = env!("CARGO_PKG_VERSION");
        logger.info(&format!("Current version: v{}", current_version));

        // Check if update is available
        if !provider
            .is_update_available(&format!("v{}", current_version))
            .await?
        {
            logger.success("Already up to date!");
            return Ok(());
        }

        // Get latest release info
        let release = provider.get_latest().await?;
        logger.info(&format!("New version available: {}", release.version));

        // Download to temp directory
        let temp_dir = env::temp_dir();
        let archive_path = temp_dir.join(format!("aiassisted-{}.archive", release.version));

        logger.info(&format!(
            "Downloading {} ...",
            release.download_url.rsplit('/').next().unwrap_or("binary")
        ));
        provider.download_release(&release, &archive_path).await?;

        // Extract binary
        let binary_path = Self::extract_binary(&archive_path)?;
        logger.info("Binary extracted successfully");

        // Replace current binary
        Self::replace_binary(&binary_path, logger)?;

        logger.success(&format!("Updated to version {}", release.version));

        // Cleanup
        let _ = fs::remove_file(&archive_path);
        let _ = fs::remove_file(&binary_path);

        Ok(())
    }

    /// Extract the binary from the downloaded archive.
    ///
    /// Handles both .tar.gz and .zip formats.
    fn extract_binary(archive_path: &Path) -> Result<PathBuf> {
        let extension = archive_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match extension {
            "gz" | "archive" => Self::extract_tar_gz(archive_path),
            "zip" => Self::extract_zip(archive_path),
            _ => Err(Error::Parse(format!(
                "Unsupported archive format: {}",
                extension
            ))),
        }
    }

    /// Extract binary from .tar.gz archive.
    fn extract_tar_gz(archive_path: &Path) -> Result<PathBuf> {
        let file = fs::File::open(archive_path)
            .map_err(Error::from)?;

        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);

        let temp_dir = env::temp_dir();

        for entry in archive
            .entries()
            .map_err(Error::from)?
        {
            let mut entry =
                entry.map_err(Error::from)?;

            let path = entry
                .path()
                .map_err(Error::from)?;

            // Look for the binary file (skip directories and other files)
            if let Some(filename) = path.file_name() {
                let filename_str = filename.to_string_lossy();
                if filename_str.starts_with("aiassisted")
                    && !filename_str.ends_with(".md")
                    && !filename_str.ends_with(".txt")
                {
                    let dest_path = temp_dir.join(filename);
                    entry
                        .unpack(&dest_path)
                        .map_err(Error::from)?;

                    // Make executable on Unix
                    #[cfg(unix)]
                    Self::make_executable(&dest_path)?;

                    return Ok(dest_path);
                }
            }
        }

        Err(Error::NotFound(
            "Binary not found in archive".to_string(),
        ))
    }

    /// Extract binary from .zip archive.
    fn extract_zip(archive_path: &Path) -> Result<PathBuf> {
        let file = fs::File::open(archive_path)
            .map_err(Error::from)?;

        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| Error::Parse(format!("Failed to read zip: {}", e)))?;

        let temp_dir = env::temp_dir();

        for i in 0..archive.len() {
            let mut entry = archive
                .by_index(i)
                .map_err(|e| Error::Parse(format!("Failed to read zip entry: {}", e)))?;

            if let Some(filename) = entry.name().rsplit('/').next() {
                if filename.starts_with("aiassisted")
                    && !filename.ends_with(".md")
                    && !filename.ends_with(".txt")
                    && !entry.is_dir()
                {
                    let dest_path = temp_dir.join(filename);
                    let mut outfile = fs::File::create(&dest_path)
                        .map_err(Error::from)?;

                    std::io::copy(&mut entry, &mut outfile)
                        .map_err(Error::from)?;

                    // Make executable on Unix
                    #[cfg(unix)]
                    Self::make_executable(&dest_path)?;

                    return Ok(dest_path);
                }
            }
        }

        Err(Error::NotFound(
            "Binary not found in archive".to_string(),
        ))
    }

    /// Make the binary executable on Unix systems.
    #[cfg(unix)]
    fn make_executable(path: &Path) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = fs::metadata(path)
            .map_err(Error::from)?
            .permissions();

        perms.set_mode(0o755);

        fs::set_permissions(path, perms)
            .map_err(Error::from)?;

        Ok(())
    }

    /// Replace the current binary with the new one.
    fn replace_binary<L: Logger>(new_binary: &Path, logger: &L) -> Result<()> {
        let current_exe = env::current_exe()
            .map_err(Error::from)?;

        logger.debug(&format!(
            "Replacing {} with {}",
            current_exe.display(),
            new_binary.display()
        ));

        // On Windows, we can't replace a running executable directly.
        // On Unix, we can use atomic rename.
        #[cfg(unix)]
        {
            fs::rename(new_binary, &current_exe).map_err(|e| {
                Error::Network(format!("Failed to replace binary: {}. Try running with sudo if this is a permission issue.", e))
            })?;
        }

        #[cfg(not(unix))]
        {
            // On Windows, move the old binary to a .old file, then copy new one
            let old_binary = current_exe.with_extension("old");
            let _ = fs::remove_file(&old_binary); // Ignore if doesn't exist

            if let Err(e) = fs::rename(&current_exe, &old_binary) {
                logger.error(&format!(
                    "Failed to rename old binary: {}. Try closing all instances of the program.",
                    e
                ));
                return Err(Error::from(e));
            }

            fs::copy(new_binary, &current_exe)
                .map_err(Error::from)?;

            logger.info("Old binary saved as .old file");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::selfupdate::ReleaseProvider;
    use crate::core::types::ReleaseInfo;
    use async_trait::async_trait;
    use mockall::mock;
    use std::io::Write;
    use tempfile::TempDir;

    #[derive(Debug, Clone, Default)]
    struct TestLogger;

    impl Logger for TestLogger {
        fn info(&self, _msg: &str) {}
        fn warn(&self, _msg: &str) {}
        fn error(&self, _msg: &str) {}
        fn debug(&self, _msg: &str) {}
        fn success(&self, _msg: &str) {}
    }

    mock! {
        pub ReleaseProvider {}

        #[async_trait]
        impl ReleaseProvider for ReleaseProvider {
            async fn get_latest(&self) -> Result<ReleaseInfo>;
            async fn is_update_available(&self, current_version: &str) -> Result<bool>;
            async fn download_release(&self, release: &ReleaseInfo, dest: &Path) -> Result<()>;
        }
    }

    #[tokio::test]
    async fn test_execute_no_update_available() {
        let mut mock_provider = MockReleaseProvider::new();
        let logger = TestLogger;

        mock_provider
            .expect_is_update_available()
            .times(1)
            .returning(|_| Ok(false));

        let command = SelfUpdateCommand;
        let result = command.execute(&mock_provider, &logger).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_extract_tar_gz_creates_binary() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("test.tar.gz");

        // Create a simple tar.gz with a fake binary
        let file = fs::File::create(&archive_path).unwrap();
        let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut tar = tar::Builder::new(encoder);

        // Add a fake binary file
        let mut header = tar::Header::new_gnu();
        header.set_path("aiassisted").unwrap();
        header.set_size(4);
        header.set_mode(0o755);
        header.set_cksum();
        tar.append(&header, b"test" as &[u8]).unwrap();
        let encoder = tar.into_inner().unwrap();
        encoder.finish().unwrap();

        // Extract should succeed
        let result = SelfUpdateCommand::extract_tar_gz(&archive_path);
        assert!(result.is_ok());

        let extracted = result.unwrap();
        assert!(extracted.exists());
        assert_eq!(fs::read_to_string(&extracted).unwrap(), "test");
    }

    #[tokio::test]
    async fn test_extract_tar_gz_binary_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("test.tar.gz");

        // Create a tar.gz without the binary
        let file = fs::File::create(&archive_path).unwrap();
        let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut tar = tar::Builder::new(encoder);

        // Add a different file
        let mut header = tar::Header::new_gnu();
        header.set_path("README.md").unwrap();
        header.set_size(4);
        header.set_cksum();
        tar.append(&header, b"test" as &[u8]).unwrap();
        let encoder = tar.into_inner().unwrap();
        encoder.finish().unwrap();

        let result = SelfUpdateCommand::extract_tar_gz(&archive_path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::NotFound(_)));
    }

    #[tokio::test]
    async fn test_extract_zip_creates_binary() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("test.zip");

        // Create a simple zip with a fake binary
        let file = fs::File::create(&archive_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);

        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        zip.start_file("aiassisted.exe", options).unwrap();
        zip.write_all(b"test").unwrap();
        zip.finish().unwrap();

        // Extract should succeed
        let result = SelfUpdateCommand::extract_zip(&archive_path);
        assert!(result.is_ok());

        let extracted = result.unwrap();
        assert!(extracted.exists());
        assert_eq!(fs::read_to_string(&extracted).unwrap(), "test");
    }

    #[tokio::test]
    async fn test_extract_zip_binary_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("test.zip");

        // Create a zip without the binary
        let file = fs::File::create(&archive_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);

        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        zip.start_file("README.md", options).unwrap();
        zip.write_all(b"test").unwrap();
        zip.finish().unwrap();

        let result = SelfUpdateCommand::extract_zip(&archive_path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::NotFound(_)));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_make_executable() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_binary");

        fs::write(&file_path, "test").unwrap();

        // Initially not executable
        let perms = fs::metadata(&file_path).unwrap().permissions();
        assert_eq!(perms.mode() & 0o111, 0);

        // Make executable
        SelfUpdateCommand::make_executable(&file_path).unwrap();

        // Now should be executable
        let perms = fs::metadata(&file_path).unwrap().permissions();
        assert_ne!(perms.mode() & 0o111, 0);
    }
}
