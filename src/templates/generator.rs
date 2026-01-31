//! Skill and agent file generation.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::core::infra::{FileSystem, Logger};
use crate::core::templates::{TemplateEngine, TemplateResolver};
use crate::core::types::{Error, Result, ToolType};

use super::discovery::ToolDetector;

/// Generates skill files from templates.
pub struct SkillGenerator;

impl SkillGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate skills for a project.
    #[allow(clippy::too_many_arguments)]
    pub async fn generate<F, E, R, L>(
        &self,
        fs: &F,
        engine: &E,
        resolver: &R,
        logger: &L,
        project_path: &Path,
        tool: ToolType,
        dry_run: bool,
    ) -> Result<Vec<PathBuf>>
    where
        F: FileSystem,
        E: TemplateEngine,
        R: TemplateResolver,
        L: Logger,
    {
        let detector = ToolDetector::new();
        let output_dir = project_path.join(detector.skills_dir(tool));

        // Get available templates
        let templates = resolver.list_templates(tool)?;
        let skill_templates: Vec<_> = templates
            .iter()
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.contains(".SKILL.md.template"))
                    .unwrap_or(false)
            })
            .collect();

        if skill_templates.is_empty() {
            logger.warn(&format!("No skill templates found for {}", tool));
            return Ok(Vec::new());
        }

        logger.info(&format!(
            "Found {} skill template(s) for {}",
            skill_templates.len(),
            tool
        ));

        let mut generated = Vec::new();

        for template_path in skill_templates {
            let template_name = template_path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| Error::Template("Invalid template filename".to_string()))?;

            // Remove .template extension to get output filename
            let output_name = template_name.replace(".template", "");
            let output_path = output_dir.join(&output_name);

            logger.info(&format!("Generating skill: {}", output_name));

            if !dry_run {
                // Read template
                let template_content = fs.read(template_path).await?;

                // Prepare variables
                let vars = self.prepare_variables(fs, project_path, tool).await?;

                // Render template
                let rendered = engine.render(&template_content, &vars)?;

                // Create output directory
                fs.create_dir_all(&output_dir).await?;

                // Write skill file
                fs.write(&output_path, &rendered).await?;

                logger.success(&format!("Created: {}", output_path.display()));
            } else {
                logger.info(&format!("Would create: {}", output_path.display()));
            }

            generated.push(output_path);
        }

        Ok(generated)
    }

    /// Prepare template variables.
    async fn prepare_variables<F: FileSystem>(
        &self,
        fs: &F,
        project_path: &Path,
        tool: ToolType,
    ) -> Result<HashMap<String, String>> {
        let mut vars = HashMap::new();

        // PROJECT_ROOT
        vars.insert(
            "PROJECT_ROOT".to_string(),
            project_path
                .to_str()
                .ok_or_else(|| Error::Template("Invalid project path".to_string()))?
                .to_string(),
        );

        // RUST_GUIDELINES_LIST
        let rust_list = self
            .list_guidelines(fs, project_path, "rust")
            .await
            .unwrap_or_default();
        vars.insert("RUST_GUIDELINES_LIST".to_string(), rust_list);

        // ARCH_GUIDELINES_LIST
        let arch_list = self
            .list_guidelines(fs, project_path, "architecture")
            .await
            .unwrap_or_default();
        vars.insert("ARCH_GUIDELINES_LIST".to_string(), arch_list);

        // tool
        vars.insert("tool".to_string(), tool.to_string());

        // project_name
        let project_name = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .to_string();
        vars.insert("project_name".to_string(), project_name);

        // timestamp
        let timestamp = chrono::Utc::now().to_rfc3339();
        vars.insert("timestamp".to_string(), timestamp);

        Ok(vars)
    }

    /// List guidelines in a category.
    async fn list_guidelines<F: FileSystem>(
        &self,
        fs: &F,
        project_path: &Path,
        category: &str,
    ) -> Result<String> {
        let guidelines_dir = project_path
            .join(".aiassisted")
            .join("guidelines")
            .join(category);

        if !fs.exists(&guidelines_dir) || !fs.is_dir(&guidelines_dir) {
            return Ok(String::new());
        }

        let entries = fs.list_dir(&guidelines_dir).await?;
        let mut md_files: Vec<_> = entries
            .into_iter()
            .filter(|p| {
                p.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "md")
                    .unwrap_or(false)
            })
            .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(String::from))
            .collect();

        md_files.sort();

        Ok(md_files
            .into_iter()
            .map(|f| format!("- {}", f))
            .collect::<Vec<_>>()
            .join("\n"))
    }
}

impl Default for SkillGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Generates agent files from templates.
pub struct AgentGenerator;

impl AgentGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate agents for a project.
    #[allow(clippy::too_many_arguments)]
    pub async fn generate<F, E, R, L>(
        &self,
        fs: &F,
        engine: &E,
        resolver: &R,
        logger: &L,
        project_path: &Path,
        tool: ToolType,
        dry_run: bool,
    ) -> Result<Vec<PathBuf>>
    where
        F: FileSystem,
        E: TemplateEngine,
        R: TemplateResolver,
        L: Logger,
    {
        let detector = ToolDetector::new();
        let output_dir = project_path.join(detector.agents_dir(tool));

        // Get available templates
        let templates = resolver.list_templates(tool)?;
        let agent_templates: Vec<_> = templates
            .iter()
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.ends_with(".md.template") && !n.contains(".SKILL.md.template"))
                    .unwrap_or(false)
            })
            .collect();

        if agent_templates.is_empty() {
            logger.warn(&format!("No agent templates found for {}", tool));
            return Ok(Vec::new());
        }

        logger.info(&format!(
            "Found {} agent template(s) for {}",
            agent_templates.len(),
            tool
        ));

        let mut generated = Vec::new();

        for template_path in agent_templates {
            let template_name = template_path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| Error::Template("Invalid template filename".to_string()))?;

            // Remove .template extension to get output filename
            let output_name = template_name.replace(".template", "");
            let output_path = output_dir.join(&output_name);

            logger.info(&format!("Generating agent: {}", output_name));

            if !dry_run {
                // Read template
                let template_content = fs.read(template_path).await?;

                // Prepare variables
                let vars = self.prepare_variables(fs, project_path, tool).await?;

                // Render template
                let rendered = engine.render(&template_content, &vars)?;

                // Create output directory
                fs.create_dir_all(&output_dir).await?;

                // Write agent file
                fs.write(&output_path, &rendered).await?;

                logger.success(&format!("Created: {}", output_path.display()));
            } else {
                logger.info(&format!("Would create: {}", output_path.display()));
            }

            generated.push(output_path);
        }

        Ok(generated)
    }

    /// Prepare template variables.
    async fn prepare_variables<F: FileSystem>(
        &self,
        fs: &F,
        project_path: &Path,
        tool: ToolType,
    ) -> Result<HashMap<String, String>> {
        let mut vars = HashMap::new();

        // PROJECT_ROOT
        vars.insert(
            "PROJECT_ROOT".to_string(),
            project_path
                .to_str()
                .ok_or_else(|| Error::Template("Invalid project path".to_string()))?
                .to_string(),
        );

        // RUST_GUIDELINES_LIST
        let rust_list = self
            .list_guidelines(fs, project_path, "rust")
            .await
            .unwrap_or_default();
        vars.insert("RUST_GUIDELINES_LIST".to_string(), rust_list);

        // ARCH_GUIDELINES_LIST
        let arch_list = self
            .list_guidelines(fs, project_path, "architecture")
            .await
            .unwrap_or_default();
        vars.insert("ARCH_GUIDELINES_LIST".to_string(), arch_list);

        // tool
        vars.insert("tool".to_string(), tool.to_string());

        // project_name
        let project_name = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .to_string();
        vars.insert("project_name".to_string(), project_name);

        // timestamp
        let timestamp = chrono::Utc::now().to_rfc3339();
        vars.insert("timestamp".to_string(), timestamp);

        Ok(vars)
    }

    /// List guidelines in a category.
    async fn list_guidelines<F: FileSystem>(
        &self,
        fs: &F,
        project_path: &Path,
        category: &str,
    ) -> Result<String> {
        let guidelines_dir = project_path
            .join(".aiassisted")
            .join("guidelines")
            .join(category);

        if !fs.exists(&guidelines_dir) || !fs.is_dir(&guidelines_dir) {
            return Ok(String::new());
        }

        let entries = fs.list_dir(&guidelines_dir).await?;
        let mut md_files: Vec<_> = entries
            .into_iter()
            .filter(|p| {
                p.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "md")
                    .unwrap_or(false)
            })
            .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(String::from))
            .collect();

        md_files.sort();

        Ok(md_files
            .into_iter()
            .map(|f| format!("- {}", f))
            .collect::<Vec<_>>()
            .join("\n"))
    }
}

impl Default for AgentGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::{mock, predicate::*};
    use std::collections::HashMap;

    // Mock definitions for testing
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
        pub Logger {}

        impl crate::core::infra::Logger for Logger {
            fn info(&self, msg: &str);
            fn warn(&self, msg: &str);
            fn error(&self, msg: &str);
            fn debug(&self, msg: &str);
            fn success(&self, msg: &str);
        }
    }

    mock! {
        pub TemplateEngine {}

        impl crate::core::templates::TemplateEngine for TemplateEngine {
            fn render(&self, template: &str, vars: &HashMap<String, String>) -> Result<String>;
        }
    }

    mock! {
        pub TemplateResolver {}

        impl crate::core::templates::TemplateResolver for TemplateResolver {
            fn resolve(&self, name: &str, tool: ToolType) -> Result<PathBuf>;
            fn list_templates(&self, tool: ToolType) -> Result<Vec<PathBuf>>;
            fn project_templates_dir(&self) -> Option<PathBuf>;
            fn global_templates_dir(&self) -> PathBuf;
        }
    }

    // SkillGenerator tests

    #[tokio::test]
    async fn test_skill_generate_success() {
        let mut mock_fs = MockFileSystem::new();
        let mut mock_engine = MockTemplateEngine::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");
        let template_path = PathBuf::from("/templates/test.SKILL.md.template");
        let output_path = PathBuf::from("/test/project/.claude/commands/test.SKILL.md");

        // Mock resolver returns one skill template
        let template_path_for_list = template_path.clone();
        mock_resolver
            .expect_list_templates()
            .with(eq(ToolType::Claude))
            .returning(move |_| Ok(vec![template_path_for_list.clone()]));

        // Mock reading template
        let template_path_for_read = template_path.clone();
        mock_fs
            .expect_read()
            .with(eq(template_path_for_read))
            .returning(|_| Ok("Template: {{project_name}}".to_string()));

        // Mock exists and is_dir for guidelines checking
        mock_fs.expect_exists().returning(|_| false);

        // Mock rendering
        mock_engine
            .expect_render()
            .returning(|_, _| Ok("Rendered content".to_string()));

        // Mock directory creation and file write
        mock_fs.expect_create_dir_all().returning(|_| Ok(()));
        mock_fs.expect_write().returning(|_, _| Ok(()));

        // Mock logger calls
        mock_logger.expect_info().returning(|_| ());
        mock_logger.expect_success().returning(|_| ());

        let generator = SkillGenerator::new();
        let result = generator
            .generate(
                &mock_fs,
                &mock_engine,
                &mock_resolver,
                &mock_logger,
                &project_path,
                ToolType::Claude,
                false,
            )
            .await;

        assert!(result.is_ok());
        let generated = result.unwrap();
        assert_eq!(generated.len(), 1);
        assert_eq!(generated[0], output_path);
    }

    #[tokio::test]
    async fn test_skill_generate_dry_run() {
        let mock_fs = MockFileSystem::new();
        let mock_engine = MockTemplateEngine::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");
        let template_path = PathBuf::from("/templates/test.SKILL.md.template");

        let template_path_for_list = template_path.clone();
        mock_resolver
            .expect_list_templates()
            .returning(move |_| Ok(vec![template_path_for_list.clone()]));

        mock_logger.expect_info().returning(|_| ());
        mock_logger.expect_warn().times(0);

        let generator = SkillGenerator::new();
        let result = generator
            .generate(
                &mock_fs,
                &mock_engine,
                &mock_resolver,
                &mock_logger,
                &project_path,
                ToolType::Claude,
                true, // dry_run
            )
            .await;

        assert!(result.is_ok());
        let generated = result.unwrap();
        assert_eq!(generated.len(), 1);
    }

    #[tokio::test]
    async fn test_skill_generate_no_templates() {
        let mock_fs = MockFileSystem::new();
        let mock_engine = MockTemplateEngine::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");

        mock_resolver
            .expect_list_templates()
            .returning(|_| Ok(vec![]));

        mock_logger.expect_warn().returning(|_| ());

        let generator = SkillGenerator::new();
        let result = generator
            .generate(
                &mock_fs,
                &mock_engine,
                &mock_resolver,
                &mock_logger,
                &project_path,
                ToolType::Claude,
                false,
            )
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_skill_prepare_variables() {
        let mut mock_fs = MockFileSystem::new();
        let project_path = PathBuf::from("/test/myproject");

        // Mock no guidelines directories
        mock_fs.expect_exists().returning(|_| false);

        let generator = SkillGenerator::new();
        let result = generator
            .prepare_variables(&mock_fs, &project_path, ToolType::Claude)
            .await;

        assert!(result.is_ok());
        let vars = result.unwrap();

        assert_eq!(vars.get("PROJECT_ROOT"), Some(&"/test/myproject".to_string()));
        assert_eq!(vars.get("project_name"), Some(&"myproject".to_string()));
        assert_eq!(vars.get("tool"), Some(&"claude".to_string()));
        assert!(vars.contains_key("timestamp"));
        assert_eq!(vars.get("RUST_GUIDELINES_LIST"), Some(&String::new()));
        assert_eq!(vars.get("ARCH_GUIDELINES_LIST"), Some(&String::new()));
    }

    #[tokio::test]
    async fn test_skill_list_guidelines_with_files() {
        let mut mock_fs = MockFileSystem::new();
        let project_path = PathBuf::from("/test/project");
        let guidelines_dir = project_path
            .join(".aiassisted")
            .join("guidelines")
            .join("rust");

        mock_fs
            .expect_exists()
            .with(eq(guidelines_dir.clone()))
            .returning(|_| true);
        mock_fs
            .expect_is_dir()
            .with(eq(guidelines_dir.clone()))
            .returning(|_| true);

        let file1 = PathBuf::from("file1.md");
        let file2 = PathBuf::from("file2.md");
        mock_fs
            .expect_list_dir()
            .with(eq(guidelines_dir))
            .returning(move |_| Ok(vec![file1.clone(), file2.clone()]));

        let generator = SkillGenerator::new();
        let result = generator
            .list_guidelines(&mock_fs, &project_path, "rust")
            .await;

        assert!(result.is_ok());
        let list = result.unwrap();
        assert!(list.contains("- file1.md"));
        assert!(list.contains("- file2.md"));
    }

    #[tokio::test]
    async fn test_skill_list_guidelines_no_directory() {
        let mut mock_fs = MockFileSystem::new();
        let project_path = PathBuf::from("/test/project");

        mock_fs.expect_exists().returning(|_| false);

        let generator = SkillGenerator::new();
        let result = generator
            .list_guidelines(&mock_fs, &project_path, "rust")
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), String::new());
    }

    #[tokio::test]
    async fn test_skill_list_guidelines_filters_md_only() {
        let mut mock_fs = MockFileSystem::new();
        let project_path = PathBuf::from("/test/project");
        let guidelines_dir = project_path
            .join(".aiassisted")
            .join("guidelines")
            .join("rust");

        mock_fs
            .expect_exists()
            .with(eq(guidelines_dir.clone()))
            .returning(|_| true);
        mock_fs
            .expect_is_dir()
            .with(eq(guidelines_dir.clone()))
            .returning(|_| true);

        let md_file = PathBuf::from("guide.md");
        let txt_file = PathBuf::from("readme.txt");
        mock_fs
            .expect_list_dir()
            .with(eq(guidelines_dir))
            .returning(move |_| Ok(vec![md_file.clone(), txt_file.clone()]));

        let generator = SkillGenerator::new();
        let result = generator
            .list_guidelines(&mock_fs, &project_path, "rust")
            .await;

        assert!(result.is_ok());
        let list = result.unwrap();
        assert!(list.contains("- guide.md"));
        assert!(!list.contains("readme.txt"));
    }

    // AgentGenerator tests

    #[tokio::test]
    async fn test_agent_generate_success() {
        let mut mock_fs = MockFileSystem::new();
        let mut mock_engine = MockTemplateEngine::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");
        let template_path = PathBuf::from("/templates/agent.md.template");
        let output_path = PathBuf::from("/test/project/.claude/agents/agent.md");

        let template_path_for_list = template_path.clone();
        mock_resolver
            .expect_list_templates()
            .with(eq(ToolType::Claude))
            .returning(move |_| Ok(vec![template_path_for_list.clone()]));

        let template_path_for_read = template_path.clone();
        mock_fs
            .expect_read()
            .with(eq(template_path_for_read))
            .returning(|_| Ok("Agent: {{project_name}}".to_string()));

        mock_fs.expect_exists().returning(|_| false);

        mock_engine
            .expect_render()
            .returning(|_, _| Ok("Rendered agent".to_string()));

        mock_fs.expect_create_dir_all().returning(|_| Ok(()));
        mock_fs.expect_write().returning(|_, _| Ok(()));

        mock_logger.expect_info().returning(|_| ());
        mock_logger.expect_success().returning(|_| ());

        let generator = AgentGenerator::new();
        let result = generator
            .generate(
                &mock_fs,
                &mock_engine,
                &mock_resolver,
                &mock_logger,
                &project_path,
                ToolType::Claude,
                false,
            )
            .await;

        assert!(result.is_ok());
        let generated = result.unwrap();
        assert_eq!(generated.len(), 1);
        assert_eq!(generated[0], output_path);
    }

    #[tokio::test]
    async fn test_agent_generate_dry_run() {
        let mock_fs = MockFileSystem::new();
        let mock_engine = MockTemplateEngine::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");
        let template_path = PathBuf::from("/templates/agent.md.template");

        let template_path_for_list = template_path.clone();
        mock_resolver
            .expect_list_templates()
            .returning(move |_| Ok(vec![template_path_for_list.clone()]));

        mock_logger.expect_info().returning(|_| ());

        let generator = AgentGenerator::new();
        let result = generator
            .generate(
                &mock_fs,
                &mock_engine,
                &mock_resolver,
                &mock_logger,
                &project_path,
                ToolType::Claude,
                true,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_agent_generate_no_templates() {
        let mock_fs = MockFileSystem::new();
        let mock_engine = MockTemplateEngine::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");

        mock_resolver
            .expect_list_templates()
            .returning(|_| Ok(vec![]));

        mock_logger.expect_warn().returning(|_| ());

        let generator = AgentGenerator::new();
        let result = generator
            .generate(
                &mock_fs,
                &mock_engine,
                &mock_resolver,
                &mock_logger,
                &project_path,
                ToolType::Claude,
                false,
            )
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_agent_filters_skill_templates() {
        let mock_fs = MockFileSystem::new();
        let mock_engine = MockTemplateEngine::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");
        let skill_template = PathBuf::from("/templates/skill.SKILL.md.template");
        let agent_template = PathBuf::from("/templates/agent.md.template");

        // Return both skill and agent templates
        mock_resolver.expect_list_templates().returning(move |_| {
            Ok(vec![skill_template.clone(), agent_template.clone()])
        });

        mock_logger.expect_info().returning(|_| ());
        mock_logger.expect_warn().returning(|_| ());

        let generator = AgentGenerator::new();
        let result = generator
            .generate(
                &mock_fs,
                &mock_engine,
                &mock_resolver,
                &mock_logger,
                &project_path,
                ToolType::Claude,
                true,
            )
            .await;

        assert!(result.is_ok());
        // Should only generate from agent template, not skill
        let generated = result.unwrap();
        assert_eq!(generated.len(), 1);
        assert!(generated[0]
            .to_str()
            .unwrap()
            .contains("agent.md"));
    }

    #[tokio::test]
    async fn test_agent_prepare_variables() {
        let mut mock_fs = MockFileSystem::new();
        let project_path = PathBuf::from("/test/agentproject");

        mock_fs.expect_exists().returning(|_| false);

        let generator = AgentGenerator::new();
        let result = generator
            .prepare_variables(&mock_fs, &project_path, ToolType::OpenCode)
            .await;

        assert!(result.is_ok());
        let vars = result.unwrap();

        assert_eq!(vars.get("PROJECT_ROOT"), Some(&"/test/agentproject".to_string()));
        assert_eq!(vars.get("project_name"), Some(&"agentproject".to_string()));
        assert_eq!(vars.get("tool"), Some(&"opencode".to_string()));
        assert!(vars.contains_key("timestamp"));
    }

    #[tokio::test]
    async fn test_agent_list_guidelines_with_files() {
        let mut mock_fs = MockFileSystem::new();
        let project_path = PathBuf::from("/test/project");
        let guidelines_dir = project_path
            .join(".aiassisted")
            .join("guidelines")
            .join("architecture");

        mock_fs
            .expect_exists()
            .with(eq(guidelines_dir.clone()))
            .returning(|_| true);
        mock_fs
            .expect_is_dir()
            .with(eq(guidelines_dir.clone()))
            .returning(|_| true);

        let file = PathBuf::from("patterns.md");
        mock_fs
            .expect_list_dir()
            .with(eq(guidelines_dir))
            .returning(move |_| Ok(vec![file.clone()]));

        let generator = AgentGenerator::new();
        let result = generator
            .list_guidelines(&mock_fs, &project_path, "architecture")
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "- patterns.md");
    }

    #[tokio::test]
    async fn test_agent_list_guidelines_empty() {
        let mut mock_fs = MockFileSystem::new();
        let project_path = PathBuf::from("/test/project");

        mock_fs.expect_exists().returning(|_| false);

        let generator = AgentGenerator::new();
        let result = generator
            .list_guidelines(&mock_fs, &project_path, "architecture")
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), String::new());
    }

    #[test]
    fn test_skill_generator_default() {
        let _generator = SkillGenerator::new();
    }

    #[test]
    fn test_agent_generator_default() {
        let _generator = AgentGenerator::new();
    }
}
