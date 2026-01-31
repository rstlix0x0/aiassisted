//! Command implementations for the content domain.

use std::path::PathBuf;

use crate::core::infra::{Checksum, FileSystem, HttpClient, Logger};
use crate::core::types::Result;

use super::sync;

/// Install command - installs .aiassisted to a target directory.
pub struct InstallCommand {
    pub path: PathBuf,
}

impl InstallCommand {
    /// Execute the install command.
    pub async fn execute<F, H, C, L>(
        &self,
        fs: &F,
        http: &H,
        checksum: &C,
        logger: &L,
    ) -> Result<()>
    where
        F: FileSystem,
        H: HttpClient,
        C: Checksum,
        L: Logger,
    {
        logger.info(&format!(
            "Installing .aiassisted to {}",
            self.path.display()
        ));

        sync::install(fs, http, checksum, logger, &self.path).await
    }
}

/// Update command - updates existing .aiassisted installation.
pub struct UpdateCommand {
    pub path: PathBuf,
    pub force: bool,
}

impl UpdateCommand {
    /// Execute the update command.
    pub async fn execute<F, H, C, L>(
        &self,
        fs: &F,
        http: &H,
        checksum: &C,
        logger: &L,
    ) -> Result<()>
    where
        F: FileSystem,
        H: HttpClient,
        C: Checksum,
        L: Logger,
    {
        logger.info(&format!(
            "Updating .aiassisted in {}{}",
            self.path.display(),
            if self.force { " (forced)" } else { "" }
        ));

        sync::update(fs, http, checksum, logger, &self.path, self.force).await
    }
}

/// Check command - checks for updates without downloading.
pub struct CheckCommand {
    pub path: PathBuf,
}

impl CheckCommand {
    /// Execute the check command.
    pub async fn execute<F, H, L>(&self, fs: &F, http: &H, logger: &L) -> Result<()>
    where
        F: FileSystem,
        H: HttpClient,
        L: Logger,
    {
        logger.info(&format!(
            "Checking for updates in {}",
            self.path.display()
        ));

        sync::check(fs, http, logger, &self.path).await
    }
}
