//! Config domain trait abstractions.
//!
//! These traits define the interfaces for configuration management.

use async_trait::async_trait;

use super::types::{AppConfig, Result};

/// Abstraction for configuration persistence.
#[async_trait]
pub trait ConfigStore: Send + Sync {
    /// Load the application configuration.
    async fn load(&self) -> Result<AppConfig>;

    /// Save the application configuration.
    async fn save(&self, config: &AppConfig) -> Result<()>;

    /// Get a specific configuration value by key.
    ///
    /// Keys use dot notation (e.g., "default_tool", "verbosity").
    async fn get(&self, key: &str) -> Option<String>;

    /// Reset configuration to defaults.
    async fn reset(&self) -> Result<()>;

    /// Get the path to the configuration file.
    fn config_path(&self) -> std::path::PathBuf;
}
