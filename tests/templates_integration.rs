//! Integration tests for templates module.
//!
//! These tests verify that template operations work correctly end-to-end
//! using real implementations.

use aiassisted::core::infra::{Checksum, FileSystem, Logger};
use aiassisted::core::templates::{TemplateEngine, TemplateResolver};
use aiassisted::core::types::ToolType;
use aiassisted::infra::{Sha2Checksum, StdFileSystem};
use aiassisted::templates::{
    CascadingResolver, ListTemplatesCommand, SetupAgentsCommand, SetupSkillsCommand,
    ShowTemplateCommand, SimpleTemplateEngine, TemplatesDiffCommand, TemplatesInitCommand,
    TemplatesPathCommand, TemplatesSyncCommand,
};
use std::collections::HashMap;
use std::path::PathBuf;
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
async fn test_template_engine_render() {
    let engine = SimpleTemplateEngine::new();
    let mut vars = HashMap::new();
    vars.insert("PROJECT_ROOT".to_string(), "/home/user/project".to_string());
    vars.insert("TOOL".to_string(), "claude".to_string());

    let template = "Project: {{PROJECT_ROOT}}, Tool: {{TOOL}}";
    let result = engine.render(template, &vars).unwrap();

    assert_eq!(result, "Project: /home/user/project, Tool: claude");
}

#[tokio::test]
async fn test_template_engine_missing_variable() {
    let engine = SimpleTemplateEngine::new();
    let vars = HashMap::new();

    let template = "Project: {{PROJECT_ROOT}}";
    let result = engine.render(template, &vars);

    // Should error on missing variable
    assert!(result.is_err());
}

#[tokio::test]
async fn test_template_resolver_cascading() {
    let fs = StdFileSystem::new();
    let temp_dir = TempDir::new().unwrap();
    let global_dir = temp_dir.path().join("global");
    let project_dir = temp_dir.path().join("project");

    // Create global templates
    let global_template_dir = global_dir
        .join(".aiassisted")
        .join("templates")
        .join("skills")
        .join("claude");
    fs.create_dir_all(&global_template_dir).await.unwrap();

    let global_template_path = global_template_dir.join("test.SKILL.md.template");
    fs.write(&global_template_path, "Global template content")
        .await
        .unwrap();

    // Create project templates (overrides global)
    let project_template_dir = project_dir
        .join(".aiassisted")
        .join("templates")
        .join("skills")
        .join("claude");
    fs.create_dir_all(&project_template_dir).await.unwrap();

    let project_template_path = project_template_dir.join("test.SKILL.md.template");
    fs.write(&project_template_path, "Project template content")
        .await
        .unwrap();

    // Resolver should prefer project over global
    let resolver = CascadingResolver::new(project_dir.clone(), global_dir.clone());
    let resolved = resolver
        .resolve("test.SKILL.md", ToolType::Claude)
        .unwrap();

    let content = fs.read(&resolved).await.unwrap();
    assert_eq!(content, "Project template content");
}

#[tokio::test]
async fn test_template_resolver_fallback_to_global() {
    let fs = StdFileSystem::new();
    let temp_dir = TempDir::new().unwrap();
    let global_dir = temp_dir.path().join("global");
    let project_dir = temp_dir.path().join("project");

    // Create only global template
    let global_template_dir = global_dir
        .join(".aiassisted")
        .join("templates")
        .join("agents")
        .join("claude");
    fs.create_dir_all(&global_template_dir).await.unwrap();

    let global_template_path = global_template_dir.join("commit.md.template");
    fs.write(&global_template_path, "Global agent template")
        .await
        .unwrap();

    // Resolver should fall back to global when project doesn't have it
    let resolver = CascadingResolver::new(project_dir, global_dir);
    let resolved = resolver.resolve("commit.md", ToolType::Claude).unwrap();

    let content = fs.read(&resolved).await.unwrap();
    assert_eq!(content, "Global agent template");
}

#[tokio::test]
async fn test_template_init_command() {
    let fs = StdFileSystem::new();
    let logger = TestLogger;
    let temp_dir = TempDir::new().unwrap();
    let global_dir = temp_dir.path().join("global");
    let project_dir = temp_dir.path().join("project");

    // Create global templates with some files
    let global_template_dir = global_dir.join(".aiassisted").join("templates");
    let skills_dir = global_template_dir.join("skills").join("claude");
    fs.create_dir_all(&skills_dir).await.unwrap();

    fs.write(&skills_dir.join("test.SKILL.md.template"), "Test skill")
        .await
        .unwrap();

    // Create resolver
    let resolver = CascadingResolver::new(project_dir.clone(), global_dir);

    // Execute init command
    let cmd = TemplatesInitCommand { force: false };
    cmd.execute(&fs, &resolver, &logger, &project_dir)
        .await
        .unwrap();

    // Verify project templates were created
    let project_template_file = project_dir
        .join(".aiassisted")
        .join("templates")
        .join("skills")
        .join("claude")
        .join("test.SKILL.md.template");

    assert!(fs.exists(&project_template_file));
    let content = fs.read(&project_template_file).await.unwrap();
    assert_eq!(content, "Test skill");
}

#[tokio::test]
async fn test_template_init_command_force() {
    let fs = StdFileSystem::new();
    let logger = TestLogger;
    let temp_dir = TempDir::new().unwrap();
    let global_dir = temp_dir.path().join("global");
    let project_dir = temp_dir.path().join("project");

    // Create global template
    let global_template_dir = global_dir.join(".aiassisted").join("templates");
    let skills_dir = global_template_dir.join("skills").join("claude");
    fs.create_dir_all(&skills_dir).await.unwrap();
    fs.write(&skills_dir.join("test.SKILL.md.template"), "New content")
        .await
        .unwrap();

    // Create existing project template with different content
    let project_template_dir = project_dir.join(".aiassisted").join("templates");
    let project_skills_dir = project_template_dir.join("skills").join("claude");
    fs.create_dir_all(&project_skills_dir).await.unwrap();
    fs.write(
        &project_skills_dir.join("test.SKILL.md.template"),
        "Old content",
    )
    .await
    .unwrap();

    let resolver = CascadingResolver::new(project_dir.clone(), global_dir);

    // Init with force should overwrite
    let cmd = TemplatesInitCommand { force: true };
    cmd.execute(&fs, &resolver, &logger, &project_dir)
        .await
        .unwrap();

    let project_file = project_skills_dir.join("test.SKILL.md.template");
    let content = fs.read(&project_file).await.unwrap();
    assert_eq!(content, "New content");
}

#[tokio::test]
async fn test_template_sync_command() {
    let fs = StdFileSystem::new();
    let logger = TestLogger;
    let temp_dir = TempDir::new().unwrap();
    let global_dir = temp_dir.path().join("global");
    let project_dir = temp_dir.path().join("project");

    // Create project templates
    let project_template_dir = project_dir.join(".aiassisted").join("templates");
    fs.create_dir_all(&project_template_dir).await.unwrap();

    // Create global template (newer)
    let global_template_dir = global_dir.join(".aiassisted").join("templates");
    let skills_dir = global_template_dir.join("skills").join("claude");
    fs.create_dir_all(&skills_dir).await.unwrap();

    let global_file = skills_dir.join("new.SKILL.md.template");
    fs.write(&global_file, "New skill").await.unwrap();

    // Wait a bit to ensure different modification times
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let resolver = CascadingResolver::new(project_dir.clone(), global_dir);

    // Execute sync command
    let cmd = TemplatesSyncCommand { force: false };
    cmd.execute(&fs, &resolver, &logger, &project_dir)
        .await
        .unwrap();

    // Verify file was synced
    let synced_file = project_template_dir
        .join("skills")
        .join("claude")
        .join("new.SKILL.md.template");
    assert!(fs.exists(&synced_file));
    let content = fs.read(&synced_file).await.unwrap();
    assert_eq!(content, "New skill");
}

#[tokio::test]
async fn test_template_diff_command_no_differences() {
    let fs = StdFileSystem::new();
    let checksum = Sha2Checksum::new();
    let logger = TestLogger;
    let temp_dir = TempDir::new().unwrap();
    let global_dir = temp_dir.path().join("global");
    let project_dir = temp_dir.path().join("project");

    // Create identical templates in both
    let content = "Identical content";

    let global_template_dir = global_dir
        .join(".aiassisted")
        .join("templates")
        .join("skills")
        .join("claude");
    fs.create_dir_all(&global_template_dir).await.unwrap();
    fs.write(&global_template_dir.join("test.SKILL.md.template"), content)
        .await
        .unwrap();

    let project_template_dir = project_dir
        .join(".aiassisted")
        .join("templates")
        .join("skills")
        .join("claude");
    fs.create_dir_all(&project_template_dir).await.unwrap();
    fs.write(&project_template_dir.join("test.SKILL.md.template"), content)
        .await
        .unwrap();

    let resolver = CascadingResolver::new(project_dir.clone(), global_dir);

    // Execute diff command
    let cmd = TemplatesDiffCommand { path: None };
    let result = cmd
        .execute(&fs, &resolver, &logger, &checksum, &project_dir)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_template_diff_command_with_differences() {
    let fs = StdFileSystem::new();
    let checksum = Sha2Checksum::new();
    let logger = TestLogger;
    let temp_dir = TempDir::new().unwrap();
    let global_dir = temp_dir.path().join("global");
    let project_dir = temp_dir.path().join("project");

    // Create different content
    let global_template_dir = global_dir
        .join(".aiassisted")
        .join("templates")
        .join("skills")
        .join("claude");
    fs.create_dir_all(&global_template_dir).await.unwrap();
    fs.write(
        &global_template_dir.join("test.SKILL.md.template"),
        "Global content",
    )
    .await
    .unwrap();

    let project_template_dir = project_dir
        .join(".aiassisted")
        .join("templates")
        .join("skills")
        .join("claude");
    fs.create_dir_all(&project_template_dir).await.unwrap();
    fs.write(
        &project_template_dir.join("test.SKILL.md.template"),
        "Project content - modified",
    )
    .await
    .unwrap();

    let resolver = CascadingResolver::new(project_dir.clone(), global_dir);

    // Execute diff command
    let cmd = TemplatesDiffCommand { path: None };
    let result = cmd
        .execute(&fs, &resolver, &logger, &checksum, &project_dir)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_template_diff_single_file() {
    let fs = StdFileSystem::new();
    let checksum = Sha2Checksum::new();
    let logger = TestLogger;
    let temp_dir = TempDir::new().unwrap();
    let global_dir = temp_dir.path().join("global");
    let project_dir = temp_dir.path().join("project");

    // Create templates
    let global_template_dir = global_dir.join(".aiassisted").join("templates");
    fs.create_dir_all(&global_template_dir).await.unwrap();
    fs.write(
        &global_template_dir.join("test.txt"),
        "Global content",
    )
    .await
    .unwrap();

    let project_template_dir = project_dir.join(".aiassisted").join("templates");
    fs.create_dir_all(&project_template_dir).await.unwrap();
    fs.write(
        &project_template_dir.join("test.txt"),
        "Different content",
    )
    .await
    .unwrap();

    let resolver = CascadingResolver::new(project_dir.clone(), global_dir);

    // Diff specific file
    let cmd = TemplatesDiffCommand {
        path: Some("test.txt".to_string()),
    };
    let result = cmd
        .execute(&fs, &resolver, &logger, &checksum, &project_dir)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_templates_command() {
    let fs = StdFileSystem::new();
    let logger = TestLogger;
    let temp_dir = TempDir::new().unwrap();
    let global_dir = temp_dir.path().join("global");
    let project_dir = temp_dir.path().join("project");

    // Create some templates
    let skills_dir = global_dir
        .join(".aiassisted")
        .join("templates")
        .join("skills")
        .join("claude");
    fs.create_dir_all(&skills_dir).await.unwrap();
    fs.write(&skills_dir.join("test.SKILL.md.template"), "Skill")
        .await
        .unwrap();

    let agents_dir = global_dir
        .join(".aiassisted")
        .join("templates")
        .join("agents")
        .join("claude");
    fs.create_dir_all(&agents_dir).await.unwrap();
    fs.write(&agents_dir.join("commit.md.template"), "Agent")
        .await
        .unwrap();

    let resolver = CascadingResolver::new(project_dir.clone(), global_dir);

    // List templates
    let cmd = ListTemplatesCommand {
        tool: ToolType::Claude,
    };
    let result = cmd
        .execute(&fs, &resolver, &logger, &project_dir)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_show_template_command() {
    let fs = StdFileSystem::new();
    let logger = TestLogger;
    let temp_dir = TempDir::new().unwrap();
    let global_dir = temp_dir.path().join("global");
    let project_dir = temp_dir.path().join("project");

    // Create template
    let skills_dir = global_dir
        .join(".aiassisted")
        .join("templates")
        .join("skills")
        .join("claude");
    fs.create_dir_all(&skills_dir).await.unwrap();
    let template_content = "# Test Skill\nThis is a test skill template.";
    fs.write(&skills_dir.join("test.SKILL.md.template"), template_content)
        .await
        .unwrap();

    let resolver = CascadingResolver::new(project_dir, global_dir);

    // Show template
    let cmd = ShowTemplateCommand {
        path: "test.SKILL.md".to_string(),
    };
    let result = cmd
        .execute(&fs, &resolver, &logger, ToolType::Claude)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_templates_path_command() {
    let temp_dir = TempDir::new().unwrap();
    let global_dir = temp_dir.path().join("global");
    let project_dir = temp_dir.path().join("project");
    let logger = TestLogger;

    let resolver = CascadingResolver::new(project_dir.clone(), global_dir);

    let cmd = TemplatesPathCommand;
    let result = cmd
        .execute(&resolver, &logger, &project_dir)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_setup_skills_dry_run() {
    let fs = StdFileSystem::new();
    let engine = SimpleTemplateEngine::new();
    let logger = TestLogger;
    let temp_dir = TempDir::new().unwrap();
    let global_dir = temp_dir.path().join("global");
    let project_dir = temp_dir.path().join("project");

    // Create global templates directory
    let skills_dir = global_dir
        .join(".aiassisted")
        .join("templates")
        .join("skills")
        .join("claude");
    fs.create_dir_all(&skills_dir).await.unwrap();

    let resolver = CascadingResolver::new(project_dir.clone(), global_dir);

    // Setup skills with dry run
    let cmd = SetupSkillsCommand {
        tool: ToolType::Claude,
        dry_run: true,
    };

    // Should not error even if no templates exist
    let result = cmd
        .execute(&fs, &engine, &resolver, &logger, &project_dir)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_setup_agents_dry_run() {
    let fs = StdFileSystem::new();
    let engine = SimpleTemplateEngine::new();
    let logger = TestLogger;
    let temp_dir = TempDir::new().unwrap();
    let global_dir = temp_dir.path().join("global");
    let project_dir = temp_dir.path().join("project");

    // Create global templates directory
    let agents_dir = global_dir
        .join(".aiassisted")
        .join("templates")
        .join("agents")
        .join("claude");
    fs.create_dir_all(&agents_dir).await.unwrap();

    let resolver = CascadingResolver::new(project_dir.clone(), global_dir);

    // Setup agents with dry run
    let cmd = SetupAgentsCommand {
        tool: ToolType::Claude,
        dry_run: true,
    };

    let result = cmd
        .execute(&fs, &engine, &resolver, &logger, &project_dir)
        .await;

    assert!(result.is_ok());
}
