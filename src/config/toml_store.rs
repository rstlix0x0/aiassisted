//! TOML-based configuration store implementation.

use std::path::PathBuf;

use async_trait::async_trait;

use crate::core::config::ConfigStore;
use crate::core::infra::FileSystem;
use crate::core::types::{AppConfig, Error, Result};

use super::settings;

/// TOML-based configuration store.
pub struct TomlConfigStore<F: FileSystem> {
    fs: F,
    config_path: PathBuf,
}

impl<F: FileSystem> TomlConfigStore<F> {
    /// Create a new TOML config store.
    ///
    /// The config file is stored at `~/.aiassisted/config.toml`.
    pub fn new(fs: F) -> Result<Self> {
        let config_dir = dirs::home_dir()
            .ok_or_else(|| Error::Config("Unable to determine home directory".to_string()))?
            .join(".aiassisted");

        let config_path = config_dir.join("config.toml");

        Ok(Self { fs, config_path })
    }

    /// Create a new TOML config store with a custom path.
    ///
    /// Useful for testing or when you need a custom config file location.
    pub fn with_path(fs: F, config_path: PathBuf) -> Self {
        Self { fs, config_path }
    }

    /// Ensure the config directory exists.
    async fn ensure_config_dir(&self) -> Result<()> {
        if let Some(parent) = self.config_path.parent().filter(|p| !self.fs.exists(p)) {
            self.fs.create_dir_all(parent).await?;
        }
        Ok(())
    }

    /// Load config from file, or return defaults if file doesn't exist.
    async fn load_or_default(&self) -> Result<AppConfig> {
        if !self.fs.exists(&self.config_path) {
            return Ok(AppConfig::default());
        }

        let contents = self.fs.read(&self.config_path).await?;
        let config: AppConfig = toml::from_str(&contents)
            .map_err(|e| Error::Serialization(format!("Failed to parse config: {}", e)))?;

        settings::validate_config(&config)?;
        Ok(config)
    }
}

#[async_trait]
impl<F: FileSystem> ConfigStore for TomlConfigStore<F> {
    async fn load(&self) -> Result<AppConfig> {
        self.load_or_default().await
    }

    async fn save(&self, config: &AppConfig) -> Result<()> {
        settings::validate_config(config)?;

        self.ensure_config_dir().await?;

        let contents = toml::to_string_pretty(config)
            .map_err(|e| Error::Serialization(format!("Failed to serialize config: {}", e)))?;

        self.fs.write(&self.config_path, &contents).await?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Option<String> {
        let config = self.load_or_default().await.ok()?;
        settings::get_value(&config, key)
    }

    async fn set(&self, key: &str, value: &str) -> Result<()> {
        let mut config = self.load_or_default().await?;
        settings::set_value(&mut config, key, value)?;
        self.save(&config).await
    }

    async fn reset(&self) -> Result<()> {
        let default_config = AppConfig::default();
        self.save(&default_config).await
    }

    fn config_path(&self) -> PathBuf {
        self.config_path.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::ToolType;
    use mockall::mock;
    use std::path::Path;

    mock! {
        pub FileSystem {}

        #[async_trait]
        impl FileSystem for FileSystem {
            async fn read(&self, path: &Path) -> Result<String>;
            async fn read_bytes(&self, path: &Path) -> Result<Vec<u8>>;
            async fn write(&self, path: &Path, contents: &str) -> Result<()>;
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

    #[tokio::test]
    async fn test_load_default_when_file_missing() {
        let mut mock_fs = MockFileSystem::new();
        mock_fs.expect_exists().returning(|_| false);

        let config_path = PathBuf::from("/test/config.toml");
        let store = TomlConfigStore::with_path(mock_fs, config_path);

        let config = store.load().await.unwrap();
        assert_eq!(config.default_tool, ToolType::Auto);
        assert_eq!(config.verbosity, 1);
        assert!(config.auto_update);
        assert!(config.prefer_project);
    }

    #[tokio::test]
    async fn test_load_existing_config() {
        let mut mock_fs = MockFileSystem::new();
        mock_fs.expect_exists().returning(|_| true);
        mock_fs.expect_read().returning(|_| {
            Ok(r#"
default_tool = "claude"
verbosity = 2
auto_update = false
prefer_project = false
"#
            .to_string())
        });

        let config_path = PathBuf::from("/test/config.toml");
        let store = TomlConfigStore::with_path(mock_fs, config_path);

        let config = store.load().await.unwrap();
        assert_eq!(config.default_tool, ToolType::Claude);
        assert_eq!(config.verbosity, 2);
        assert!(!config.auto_update);
        assert!(!config.prefer_project);
    }

    #[tokio::test]
    async fn test_load_invalid_toml() {
        let mut mock_fs = MockFileSystem::new();
        mock_fs.expect_exists().returning(|_| true);
        mock_fs
            .expect_read()
            .returning(|_| Ok("invalid toml content".to_string()));

        let config_path = PathBuf::from("/test/config.toml");
        let store = TomlConfigStore::with_path(mock_fs, config_path);

        let result = store.load().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_save_config() {
        let mut mock_fs = MockFileSystem::new();
        mock_fs.expect_exists().returning(|_| true);
        mock_fs.expect_create_dir_all().returning(|_| Ok(()));
        mock_fs
            .expect_write()
            .withf(|_, contents: &str| {
                contents.contains("default_tool = \"claude\"")
                    && contents.contains("verbosity = 2")
            })
            .returning(|_, _| Ok(()));

        let config_path = PathBuf::from("/test/config.toml");
        let store = TomlConfigStore::with_path(mock_fs, config_path);

        let config = AppConfig {
            default_tool: ToolType::Claude,
            verbosity: 2,
            auto_update: true,
            prefer_project: true,
        };

        let result = store.save(&config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_save_invalid_config() {
        let mock_fs = MockFileSystem::new();
        let config_path = PathBuf::from("/test/config.toml");
        let store = TomlConfigStore::with_path(mock_fs, config_path);

        let config = AppConfig {
            verbosity: 3, // Invalid
            ..Default::default()
        };

        let result = store.save(&config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_existing_key() {
        let mut mock_fs = MockFileSystem::new();
        mock_fs.expect_exists().returning(|_| true);
        mock_fs.expect_read().returning(|_| {
            Ok(r#"
default_tool = "opencode"
verbosity = 1
auto_update = true
prefer_project = true
"#
            .to_string())
        });

        let config_path = PathBuf::from("/test/config.toml");
        let store = TomlConfigStore::with_path(mock_fs, config_path);

        let value = store.get("default_tool").await;
        assert_eq!(value, Some("opencode".to_string()));
    }

    #[tokio::test]
    async fn test_get_nonexistent_key() {
        let mut mock_fs = MockFileSystem::new();
        mock_fs.expect_exists().returning(|_| false);

        let config_path = PathBuf::from("/test/config.toml");
        let store = TomlConfigStore::with_path(mock_fs, config_path);

        let value = store.get("unknown_key").await;
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_set_value() {
        let mut mock_fs = MockFileSystem::new();
        // First call for load
        mock_fs.expect_exists().returning(|_| false);
        // Second call for save
        mock_fs.expect_exists().returning(|_| true);
        mock_fs.expect_create_dir_all().returning(|_| Ok(()));
        mock_fs.expect_write().returning(|_, _| Ok(()));

        let config_path = PathBuf::from("/test/config.toml");
        let store = TomlConfigStore::with_path(mock_fs, config_path);

        let result = store.set("verbosity", "2").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_invalid_value() {
        let mut mock_fs = MockFileSystem::new();
        mock_fs.expect_exists().returning(|_| false);

        let config_path = PathBuf::from("/test/config.toml");
        let store = TomlConfigStore::with_path(mock_fs, config_path);

        let result = store.set("verbosity", "invalid").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reset() {
        let mut mock_fs = MockFileSystem::new();
        mock_fs.expect_exists().returning(|_| true);
        mock_fs.expect_create_dir_all().returning(|_| Ok(()));
        mock_fs
            .expect_write()
            .withf(|_, contents: &str| {
                contents.contains("default_tool = \"auto\"") && contents.contains("verbosity = 1")
            })
            .returning(|_, _| Ok(()));

        let config_path = PathBuf::from("/test/config.toml");
        let store = TomlConfigStore::with_path(mock_fs, config_path);

        let result = store.reset().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_config_path() {
        let mock_fs = MockFileSystem::new();
        let config_path = PathBuf::from("/test/config.toml");
        let store = TomlConfigStore::with_path(mock_fs, config_path.clone());

        assert_eq!(store.config_path(), config_path);
    }
}
