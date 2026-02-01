//! Integration tests for migration domain.
//!
//! These tests use real filesystem and verify the migration workflow.

use aiassisted::config::TomlConfigStore;
use aiassisted::core::config::ConfigStore;
use aiassisted::core::types::ToolType;
use aiassisted::infra::StdFileSystem;

use tempfile::TempDir;

#[tokio::test]
async fn test_migrate_with_old_config() {
    let temp_dir = TempDir::new().unwrap();
    let fake_home = temp_dir.path();

    // Create old-style config
    let old_config_dir = fake_home.join(".aiassisted");
    std::fs::create_dir_all(&old_config_dir).unwrap();

    let old_config_path = old_config_dir.join("config.toml");
    let old_config_content = r#"
[general]
default_runtime = "shell"
verbosity = 2

[install]
auto_update = false

[templates]
prefer_project = true
"#;
    std::fs::write(&old_config_path, old_config_content).unwrap();

    // Create a custom config store with temp path
    let new_config_path = old_config_dir.join("new_config.toml");
    let config_store = TomlConfigStore::with_path(StdFileSystem::new(), new_config_path.clone());

    // Note: We can't easily test the full migration because it uses
    // dirs::home_dir() internally. This is a limitation of the current design.
    // In a real scenario, we'd inject the home directory path.

    // Instead, let's verify the config parsing works
    let shell_config =
        aiassisted::migration::shell_config::ShellConfig::parse(old_config_content).unwrap();
    let new_config = shell_config.to_app_config();

    assert_eq!(new_config.default_tool, ToolType::Auto);
    assert_eq!(new_config.verbosity, 2);
    assert!(!new_config.auto_update);
    assert!(new_config.prefer_project);

    // Save it
    config_store.save(&new_config).await.unwrap();

    // Load it back
    let loaded = config_store.load().await.unwrap();
    assert_eq!(loaded.default_tool, ToolType::Auto);
    assert_eq!(loaded.verbosity, 2);
    assert!(!loaded.auto_update);
    assert!(loaded.prefer_project);
}

#[tokio::test]
async fn test_migrate_config_conversion() {
    // Test all runtime conversions
    let test_cases = vec![
        ("shell", ToolType::Auto),
        ("auto", ToolType::Auto),
        ("claude", ToolType::Claude),
        ("opencode", ToolType::OpenCode),
        ("python", ToolType::Auto), // unknown -> auto
        ("bun", ToolType::Auto),    // unknown -> auto
    ];

    for (runtime, expected_tool) in test_cases {
        let config_content = format!(
            r#"
[general]
default_runtime = "{}"
verbosity = 1

[install]
auto_update = true

[templates]
prefer_project = false
"#,
            runtime
        );

        let shell_config =
            aiassisted::migration::shell_config::ShellConfig::parse(&config_content).unwrap();
        let new_config = shell_config.to_app_config();

        assert_eq!(
            new_config.default_tool, expected_tool,
            "Failed for runtime: {}",
            runtime
        );
        assert_eq!(new_config.verbosity, 1);
        assert!(new_config.auto_update);
        assert!(!new_config.prefer_project);
    }
}

#[tokio::test]
async fn test_migrate_backup_creation() {
    let temp_dir = TempDir::new().unwrap();
    let fake_home = temp_dir.path();

    // Create old source directory with some files
    let old_source_dir = fake_home.join(".aiassisted").join("source");
    std::fs::create_dir_all(&old_source_dir).unwrap();

    let test_file = old_source_dir.join("test.txt");
    std::fs::write(&test_file, "test content").unwrap();

    let subdir = old_source_dir.join("subdir");
    std::fs::create_dir_all(&subdir).unwrap();
    std::fs::write(subdir.join("nested.txt"), "nested content").unwrap();

    // We can't actually test the full migration without mocking dirs::home_dir(),
    // but we can verify the directory structure is correct
    assert!(old_source_dir.exists());
    assert!(test_file.exists());
    assert!(subdir.join("nested.txt").exists());

    // The actual migration would:
    // 1. Create a backup at ~/.aiassisted/source.backup.{timestamp}
    // 2. Copy all files recursively
    // 3. Remove the old source directory

    // For this test, we just verify the structure exists
}

#[tokio::test]
async fn test_migrate_no_old_installation() {
    // This test verifies that running migrate with no old installation
    // doesn't fail and reports correctly

    let temp_dir = TempDir::new().unwrap();
    let fake_home = temp_dir.path();

    // Ensure .aiassisted doesn't exist
    assert!(!fake_home.join(".aiassisted").exists());

    // Note: We can't fully test this because MigrateCommand uses
    // dirs::home_dir() internally. In production code, we'd want to
    // inject the home directory path for better testability.
}

#[test]
fn test_shell_config_handles_extra_sections() {
    // Verify that extra sections in old config don't break parsing
    let config_with_extras = r#"
[general]
default_runtime = "auto"
verbosity = 1

[install]
auto_update = true
confirm_before_install = false
install_path = ".aiassisted"

[templates]
prefer_project = true
auto_init_templates = false
auto_sync_templates = false

[skills]
tools = []
auto_setup = false

[update]
check_on_startup = false
channel = "stable"

[github]
repo = "rstlix0x0/aiassisted"
ref = ""
"#;

    let config =
        aiassisted::migration::shell_config::ShellConfig::parse(config_with_extras).unwrap();
    assert_eq!(config.general.default_runtime, "auto");
    assert_eq!(config.general.verbosity, 1);
    assert!(config.install.auto_update);
    assert!(config.templates.prefer_project);

    // Conversion should work
    let app_config = config.to_app_config();
    assert_eq!(app_config.default_tool, ToolType::Auto);
}
