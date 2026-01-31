//! Command implementations for the config domain.

use std::io::Write;
use std::process::{Command, Stdio};

use crate::core::config::ConfigStore;
use crate::core::infra::Logger;
use crate::core::types::{Error, Result};

/// Show command - displays all configuration values.
pub struct ShowCommand;

impl ShowCommand {
    /// Execute the show command.
    pub async fn execute<C, L>(&self, config_store: &C, logger: &L) -> Result<()>
    where
        C: ConfigStore,
        L: Logger,
    {
        let config = config_store.load().await?;

        logger.info("Current configuration:");
        println!();
        println!("  default_tool      = {}", config.default_tool);
        println!("  verbosity         = {}", config.verbosity);
        println!("  auto_update       = {}", config.auto_update);
        println!("  prefer_project    = {}", config.prefer_project);
        println!();
        logger.info(&format!(
            "Configuration file: {}",
            config_store.config_path().display()
        ));

        Ok(())
    }
}

/// Get command - retrieves a specific configuration value.
pub struct GetCommand {
    pub key: String,
}

impl GetCommand {
    /// Execute the get command.
    pub async fn execute<C, L>(&self, config_store: &C, logger: &L) -> Result<()>
    where
        C: ConfigStore,
        L: Logger,
    {
        match config_store.get(&self.key).await {
            Some(value) => {
                println!("{}", value);
                Ok(())
            }
            None => {
                logger.error(&format!("Unknown configuration key: {}", self.key));
                Err(Error::Config(format!(
                    "Unknown configuration key: {}",
                    self.key
                )))
            }
        }
    }
}

/// Edit command - opens configuration file in editor.
pub struct EditCommand;

impl EditCommand {
    /// Execute the edit command.
    pub async fn execute<C, L>(&self, config_store: &C, logger: &L) -> Result<()>
    where
        C: ConfigStore,
        L: Logger,
    {
        // Ensure config file exists with defaults
        let config = config_store.load().await?;
        config_store.save(&config).await?;

        let config_path = config_store.config_path();

        // Determine editor
        let editor = std::env::var("EDITOR")
            .or_else(|_| std::env::var("VISUAL"))
            .unwrap_or_else(|_| {
                if cfg!(windows) {
                    "notepad".to_string()
                } else {
                    // Try common editors
                    for ed in ["vim", "vi", "nano", "emacs"] {
                        if Command::new("which")
                            .arg(ed)
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .status()
                            .map(|s| s.success())
                            .unwrap_or(false)
                        {
                            return ed.to_string();
                        }
                    }
                    "vi".to_string() // Last resort
                }
            });

        logger.info(&format!("Opening config in {}...", editor));

        // Open editor
        let status = Command::new(&editor)
            .arg(&config_path)
            .status()
            .map_err(|e| Error::Config(format!("Failed to open editor: {}", e)))?;

        if !status.success() {
            return Err(Error::Config(format!(
                "Editor exited with status: {}",
                status
            )));
        }

        // Validate the edited config
        match config_store.load().await {
            Ok(_) => {
                logger.info("Configuration updated successfully");
                Ok(())
            }
            Err(e) => {
                logger.error(&format!("Invalid configuration: {}", e));
                logger.warn("Please fix the configuration file manually");
                Err(e)
            }
        }
    }
}

/// Reset command - resets configuration to defaults.
pub struct ResetCommand {
    pub force: bool,
}

impl ResetCommand {
    /// Execute the reset command.
    pub async fn execute<C, L>(&self, config_store: &C, logger: &L) -> Result<()>
    where
        C: ConfigStore,
        L: Logger,
    {
        if !self.force {
            // Prompt for confirmation
            logger.warn("This will reset all configuration to defaults.");
            print!("Continue? [y/N] ");
            std::io::stdout()
                .flush()
                .map_err(|e| Error::Config(format!("Failed to flush stdout: {}", e)))?;

            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .map_err(|e| Error::Config(format!("Failed to read input: {}", e)))?;

            let input = input.trim().to_lowercase();
            if input != "y" && input != "yes" {
                logger.info("Reset cancelled");
                return Ok(());
            }
        }

        config_store.reset().await?;
        logger.info("Configuration reset to defaults");
        Ok(())
    }
}

/// Path command - shows configuration file path.
pub struct PathCommand;

impl PathCommand {
    /// Execute the path command.
    pub async fn execute<C>(&self, config_store: &C) -> Result<()>
    where
        C: ConfigStore,
    {
        println!("{}", config_store.config_path().display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::AppConfig;
    use mockall::mock;
    use std::path::PathBuf;

    mock! {
        pub ConfigStore {}

        #[async_trait::async_trait]
        impl ConfigStore for ConfigStore {
            async fn load(&self) -> Result<AppConfig>;
            async fn save(&self, config: &AppConfig) -> Result<()>;
            async fn get(&self, key: &str) -> Option<String>;
            async fn reset(&self) -> Result<()>;
            fn config_path(&self) -> PathBuf;
        }
    }

    mock! {
        pub Logger {}

        impl Logger for Logger {
            fn info(&self, msg: &str);
            fn warn(&self, msg: &str);
            fn error(&self, msg: &str);
            fn debug(&self, msg: &str);
            fn success(&self, msg: &str);
        }
    }

    #[tokio::test]
    async fn test_show_command() {
        let mut mock_store = MockConfigStore::new();
        let mut mock_logger = MockLogger::new();

        mock_store
            .expect_load()
            .returning(|| Ok(AppConfig::default()));
        mock_store
            .expect_config_path()
            .returning(|| PathBuf::from("/test/config.toml"));

        mock_logger.expect_info().times(2).returning(|_| ());

        let cmd = ShowCommand;
        let result = cmd.execute(&mock_store, &mock_logger).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_show_command_load_error() {
        let mut mock_store = MockConfigStore::new();
        let mock_logger = MockLogger::new();

        mock_store
            .expect_load()
            .returning(|| Err(Error::Config("Load failed".to_string())));

        let cmd = ShowCommand;
        let result = cmd.execute(&mock_store, &mock_logger).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_command_existing_key() {
        let mut mock_store = MockConfigStore::new();
        let mock_logger = MockLogger::new();

        mock_store
            .expect_get()
            .with(mockall::predicate::eq("default_tool"))
            .returning(|_| Some("auto".to_string()));

        let cmd = GetCommand {
            key: "default_tool".to_string(),
        };
        let result = cmd.execute(&mock_store, &mock_logger).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_command_nonexistent_key() {
        let mut mock_store = MockConfigStore::new();
        let mut mock_logger = MockLogger::new();

        mock_store
            .expect_get()
            .with(mockall::predicate::eq("unknown"))
            .returning(|_| None);

        mock_logger.expect_error().returning(|_| ());

        let cmd = GetCommand {
            key: "unknown".to_string(),
        };
        let result = cmd.execute(&mock_store, &mock_logger).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reset_command_with_force() {
        let mut mock_store = MockConfigStore::new();
        let mut mock_logger = MockLogger::new();

        mock_store.expect_reset().returning(|| Ok(()));
        mock_logger.expect_info().returning(|_| ());

        let cmd = ResetCommand { force: true };
        let result = cmd.execute(&mock_store, &mock_logger).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reset_command_error() {
        let mut mock_store = MockConfigStore::new();
        let mock_logger = MockLogger::new();

        mock_store
            .expect_reset()
            .returning(|| Err(Error::Config("Reset failed".to_string())));

        let cmd = ResetCommand { force: true };
        let result = cmd.execute(&mock_store, &mock_logger).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_path_command() {
        let mut mock_store = MockConfigStore::new();

        mock_store
            .expect_config_path()
            .returning(|| PathBuf::from("/test/config.toml"));

        let cmd = PathCommand;
        let result = cmd.execute(&mock_store).await;
        assert!(result.is_ok());
    }
}
