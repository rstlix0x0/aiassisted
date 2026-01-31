//! Integration tests for config module.
//!
//! These tests verify that configuration management works correctly end-to-end
//! using real implementations.

use aiassisted::config::{GetCommand, PathCommand, ResetCommand, ShowCommand, TomlConfigStore};
use aiassisted::core::config::ConfigStore;
use aiassisted::core::infra::{FileSystem, Logger};
use aiassisted::core::types::{AppConfig, ToolType};
use aiassisted::infra::StdFileSystem;
use tempfile::TempDir;

// Simple test logger
#[derive(Debug, Clone, Default)]
struct TestLogger;

impl Logger for TestLogger {
    fn info(&self, _msg: &str) {}
    fn warn(&self, _msg: &str) {}
    fn error(&self, _msg: &str) {}
    fn debug(&self, _msg: &str) {}
    fn success(&self, _msg: &str) {}
}

#[tokio::test]
async fn test_config_load_default() {
    let fs = StdFileSystem::new();
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let store = TomlConfigStore::with_path(fs, config_path.clone());

    // Load should return defaults when file doesn't exist
    let config = store.load().await.unwrap();
    assert_eq!(config.default_tool, ToolType::Auto);
    assert_eq!(config.verbosity, 1);
    assert_eq!(config.auto_update, true);
    assert_eq!(config.prefer_project, true);
}

#[tokio::test]
async fn test_config_save_and_load() {
    let fs = StdFileSystem::new();
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let store = TomlConfigStore::with_path(fs.clone(), config_path.clone());

    // Create custom config
    let mut config = AppConfig::default();
    config.default_tool = ToolType::Claude;
    config.verbosity = 2;
    config.auto_update = false;

    // Save it
    store.save(&config).await.unwrap();

    // Verify file was created
    assert!(fs.exists(&config_path));

    // Load it back
    let loaded = store.load().await.unwrap();
    assert_eq!(loaded.default_tool, ToolType::Claude);
    assert_eq!(loaded.verbosity, 2);
    assert_eq!(loaded.auto_update, false);
}

#[tokio::test]
async fn test_config_get_set() {
    let fs = StdFileSystem::new();
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let store = TomlConfigStore::with_path(fs, config_path);

    // Set a value
    store.set("default_tool", "claude").await.unwrap();

    // Get it back
    let value = store.get("default_tool").await;
    assert_eq!(value, Some("claude".to_string()));
}

#[tokio::test]
async fn test_config_get_unknown_key() {
    let fs = StdFileSystem::new();
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let store = TomlConfigStore::with_path(fs, config_path);

    // Get unknown key
    let value = store.get("unknown_key").await;
    assert_eq!(value, None);
}

#[tokio::test]
async fn test_config_reset() {
    let fs = StdFileSystem::new();
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let store = TomlConfigStore::with_path(fs.clone(), config_path.clone());

    // Set custom values
    let mut config = AppConfig::default();
    config.default_tool = ToolType::OpenCode;
    config.verbosity = 2; // Valid range is 0-2
    store.save(&config).await.unwrap();

    // Reset
    store.reset().await.unwrap();

    // Load should return defaults
    let loaded = store.load().await.unwrap();
    assert_eq!(loaded.default_tool, ToolType::Auto);
    assert_eq!(loaded.verbosity, 1); // Default verbosity is 1
}

#[tokio::test]
async fn test_config_path_command() {
    let fs = StdFileSystem::new();
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let store = TomlConfigStore::with_path(fs, config_path.clone());

    let cmd = PathCommand;
    let result = cmd.execute(&store).await;

    assert!(result.is_ok());
    assert_eq!(store.config_path(), config_path);
}

#[tokio::test]
async fn test_show_command() {
    let fs = StdFileSystem::new();
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    let logger = TestLogger;

    let store = TomlConfigStore::with_path(fs, config_path);

    // Set some values
    let mut config = AppConfig::default();
    config.default_tool = ToolType::Claude;
    config.verbosity = 1;
    store.save(&config).await.unwrap();

    let cmd = ShowCommand;
    let result = cmd.execute(&store, &logger).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_command_existing_key() {
    let fs = StdFileSystem::new();
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    let logger = TestLogger;

    let store = TomlConfigStore::with_path(fs, config_path);

    // Set default_tool
    store.set("default_tool", "claude").await.unwrap();

    let cmd = GetCommand {
        key: "default_tool".to_string(),
    };
    let result = cmd.execute(&store, &logger).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_command_nonexistent_key() {
    let fs = StdFileSystem::new();
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    let logger = TestLogger;

    let store = TomlConfigStore::with_path(fs, config_path);

    let cmd = GetCommand {
        key: "nonexistent".to_string(),
    };
    let result = cmd.execute(&store, &logger).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_reset_command_with_force() {
    let fs = StdFileSystem::new();
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    let logger = TestLogger;

    let store = TomlConfigStore::with_path(fs.clone(), config_path.clone());

    // Set custom values
    let mut config = AppConfig::default();
    config.default_tool = ToolType::OpenCode;
    store.save(&config).await.unwrap();

    // Reset with force (no prompt)
    let cmd = ResetCommand { force: true };
    let result = cmd.execute(&store, &logger).await;

    assert!(result.is_ok());

    // Verify reset to defaults
    let loaded = store.load().await.unwrap();
    assert_eq!(loaded.default_tool, ToolType::Auto);
}

#[tokio::test]
async fn test_config_validation() {
    let fs = StdFileSystem::new();
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let store = TomlConfigStore::with_path(fs, config_path);

    // Try to set invalid verbosity (should fail validation)
    let result = store.set("verbosity", "10").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_config_toml_roundtrip() {
    let fs = StdFileSystem::new();
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let store = TomlConfigStore::with_path(fs.clone(), config_path.clone());

    // Create config with all fields set
    let mut config = AppConfig::default();
    config.default_tool = ToolType::Claude;
    config.verbosity = 2;
    config.auto_update = false;
    config.prefer_project = false;

    // Save
    store.save(&config).await.unwrap();

    // Read the file content directly to verify TOML format
    let content = fs.read(&config_path).await.unwrap();
    assert!(content.contains("default_tool"));
    assert!(content.contains("verbosity"));
    assert!(content.contains("auto_update"));
    assert!(content.contains("prefer_project"));

    // Load back and verify
    let loaded = store.load().await.unwrap();
    assert_eq!(loaded.default_tool, config.default_tool);
    assert_eq!(loaded.verbosity, config.verbosity);
    assert_eq!(loaded.auto_update, config.auto_update);
    assert_eq!(loaded.prefer_project, config.prefer_project);
}

#[tokio::test]
async fn test_config_multiple_sets() {
    let fs = StdFileSystem::new();
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let store = TomlConfigStore::with_path(fs, config_path);

    // Set multiple values
    store.set("default_tool", "claude").await.unwrap();
    store.set("verbosity", "2").await.unwrap();
    store.set("auto_update", "false").await.unwrap();

    // Verify all are set
    assert_eq!(store.get("default_tool").await, Some("claude".to_string()));
    assert_eq!(store.get("verbosity").await, Some("2".to_string()));
    assert_eq!(
        store.get("auto_update").await,
        Some("false".to_string())
    );
}

#[tokio::test]
async fn test_config_persistence() {
    let fs = StdFileSystem::new();
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // First store instance
    {
        let store1 = TomlConfigStore::with_path(fs.clone(), config_path.clone());
        store1.set("default_tool", "claude").await.unwrap();
    }

    // Second store instance (simulating app restart)
    {
        let store2 = TomlConfigStore::with_path(fs, config_path);
        let value = store2.get("default_tool").await;
        assert_eq!(value, Some("claude".to_string()));
    }
}

#[tokio::test]
async fn test_config_empty_file_handling() {
    let fs = StdFileSystem::new();
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // Create empty file
    fs.write(&config_path, "").await.unwrap();

    let store = TomlConfigStore::with_path(fs, config_path);

    // Load should handle empty file gracefully
    let result = store.load().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_config_invalid_toml() {
    let fs = StdFileSystem::new();
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // Write invalid TOML
    fs.write(&config_path, "this is not valid toml [[[")
        .await
        .unwrap();

    let store = TomlConfigStore::with_path(fs, config_path);

    // Load should fail with parse error
    let result = store.load().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_config_creates_parent_directory() {
    let fs = StdFileSystem::new();
    let temp_dir = TempDir::new().unwrap();
    let nested_path = temp_dir.path().join("a/b/c/config.toml");

    let store = TomlConfigStore::with_path(fs.clone(), nested_path.clone());

    // Save should create parent directories
    let config = AppConfig::default();
    store.save(&config).await.unwrap();

    // Verify file exists
    assert!(fs.exists(&nested_path));
}
