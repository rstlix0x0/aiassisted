//! Command implementations for the templates domain.

use std::path::{Path, PathBuf};

use crate::core::infra::{Checksum, FileSystem, Logger};
use crate::core::templates::{TemplateEngine, TemplateResolver};
use crate::core::types::{Result, ToolType};

use super::discovery::ToolDetector;
use super::generator::{AgentGenerator, SkillGenerator};

/// Copy all files recursively from source to destination.
/// Returns the number of files copied.
async fn copy_dir_all<F: FileSystem>(
    fs: &F,
    src: &Path,
    dst: &Path,
    force: bool,
) -> Result<usize> {
    let mut count = 0;

    if !fs.exists(src) || !fs.is_dir(src) {
        return Ok(0);
    }

    fs.create_dir_all(dst).await?;

    let entries = fs.list_dir(src).await?;

    for entry in entries {
        let file_name = entry
            .file_name()
            .ok_or_else(|| crate::core::types::Error::Io(std::io::Error::other(
                "Invalid file name",
            )))?;
        let dest_path = dst.join(file_name);

        if fs.is_dir(&entry) {
            // Recursively copy directory
            count += Box::pin(copy_dir_all(fs, &entry, &dest_path, force)).await?;
        } else if fs.is_file(&entry) {
            // Only copy if force=true or destination doesn't exist
            if force || !fs.exists(&dest_path) {
                fs.copy(&entry, &dest_path).await?;
                count += 1;
            }
        }
    }

    Ok(count)
}

/// Sync files from source to destination, only copying newer files.
/// Returns the number of files synced.
async fn sync_dir<F: FileSystem>(
    fs: &F,
    src: &Path,
    dst: &Path,
    force: bool,
) -> Result<usize> {
    let mut count = 0;

    if !fs.exists(src) || !fs.is_dir(src) {
        return Ok(0);
    }

    fs.create_dir_all(dst).await?;

    let entries = fs.list_dir(src).await?;

    for entry in entries {
        let file_name = entry
            .file_name()
            .ok_or_else(|| crate::core::types::Error::Io(std::io::Error::other(
                "Invalid file name",
            )))?;
        let dest_path = dst.join(file_name);

        if fs.is_dir(&entry) {
            // Recursively sync directory
            count += Box::pin(sync_dir(fs, &entry, &dest_path, force)).await?;
        } else if fs.is_file(&entry) {
            // Copy if force=true, or if destination doesn't exist, or if source is newer
            let should_copy = if force || !fs.exists(&dest_path) {
                true
            } else {
                // Compare modification times using std::fs metadata
                use std::fs;
                let src_meta = fs::metadata(&entry)?;
                let dst_meta = fs::metadata(&dest_path)?;
                src_meta.modified()? > dst_meta.modified()?
            };

            if should_copy {
                fs.copy(&entry, &dest_path).await?;
                count += 1;
            }
        }
    }

    Ok(count)
}

/// Collect all files recursively from a directory.
async fn collect_files<F: FileSystem>(
    fs: &F,
    dir: &Path,
    base: &Path,
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if !fs.exists(dir) || !fs.is_dir(dir) {
        return Ok(files);
    }

    let entries = fs.list_dir(dir).await?;

    for entry in entries {
        if fs.is_dir(&entry) {
            files.extend(Box::pin(collect_files(fs, &entry, base)).await?);
        } else if fs.is_file(&entry) {
            // Store relative path from base
            if let Ok(rel_path) = entry.strip_prefix(base) {
                files.push(rel_path.to_path_buf());
            }
        }
    }

    Ok(files)
}

/// Setup skills command - generates AI skill files.
pub struct SetupSkillsCommand {
    pub tool: ToolType,
    pub dry_run: bool,
}

impl SetupSkillsCommand {
    /// Execute the setup-skills command.
    pub async fn execute<F, E, R, L>(
        &self,
        fs: &F,
        engine: &E,
        resolver: &R,
        logger: &L,
        project_path: &Path,
    ) -> Result<()>
    where
        F: FileSystem,
        E: TemplateEngine,
        R: TemplateResolver,
        L: Logger,
    {
        let detector = ToolDetector::new();

        // Auto-detect tool if needed
        let tool = if self.tool == ToolType::Auto {
            let detected = detector.detect(fs, project_path);
            logger.info(&format!("Auto-detected tool: {}", detected));
            detected
        } else {
            self.tool
        };

        logger.info(&format!(
            "Setting up skills for {}{}",
            tool,
            if self.dry_run { " (dry run)" } else { "" }
        ));

        let generator = SkillGenerator::new();
        let skills = generator
            .generate(fs, engine, resolver, logger, project_path, tool, self.dry_run)
            .await?;

        if skills.is_empty() {
            logger.warn("No skills were generated");
        } else {
            logger.success(&format!(
                "Successfully {} {} skill(s)",
                if self.dry_run { "would generate" } else { "generated" },
                skills.len()
            ));
        }

        Ok(())
    }
}

/// Setup agents command - generates AI agent files.
pub struct SetupAgentsCommand {
    pub tool: ToolType,
    pub dry_run: bool,
}

impl SetupAgentsCommand {
    /// Execute the setup-agents command.
    pub async fn execute<F, E, R, L>(
        &self,
        fs: &F,
        engine: &E,
        resolver: &R,
        logger: &L,
        project_path: &Path,
    ) -> Result<()>
    where
        F: FileSystem,
        E: TemplateEngine,
        R: TemplateResolver,
        L: Logger,
    {
        let detector = ToolDetector::new();

        // Auto-detect tool if needed
        let tool = if self.tool == ToolType::Auto {
            let detected = detector.detect(fs, project_path);
            logger.info(&format!("Auto-detected tool: {}", detected));
            detected
        } else {
            self.tool
        };

        logger.info(&format!(
            "Setting up agents for {}{}",
            tool,
            if self.dry_run { " (dry run)" } else { "" }
        ));

        let generator = AgentGenerator::new();
        let agents = generator
            .generate(fs, engine, resolver, logger, project_path, tool, self.dry_run)
            .await?;

        if agents.is_empty() {
            logger.warn("No agents were generated");
        } else {
            logger.success(&format!(
                "Successfully {} {} agent(s)",
                if self.dry_run { "would generate" } else { "generated" },
                agents.len()
            ));
        }

        Ok(())
    }
}

/// List templates command.
pub struct ListTemplatesCommand {
    pub tool: ToolType,
}

impl ListTemplatesCommand {
    /// Execute the templates list command.
    pub async fn execute<F, R, L>(
        &self,
        fs: &F,
        resolver: &R,
        logger: &L,
        project_path: &Path,
    ) -> Result<()>
    where
        F: FileSystem,
        R: TemplateResolver,
        L: Logger,
    {
        let detector = ToolDetector::new();

        // Auto-detect tool if needed
        let tool = if self.tool == ToolType::Auto {
            let detected = detector.detect(fs, project_path);
            logger.info(&format!("Auto-detected tool: {}", detected));
            detected
        } else {
            self.tool
        };

        logger.info(&format!("Available templates for {}:", tool));

        let templates = resolver.list_templates(tool)?;

        if templates.is_empty() {
            logger.warn("No templates found");
            return Ok(());
        }

        // Group by category (skills vs agents)
        let mut skills = Vec::new();
        let mut agents = Vec::new();

        for template in templates {
            let name = template
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            if name.contains(".SKILL.md.template") {
                skills.push(name.replace(".template", ""));
            } else if name.ends_with(".md.template") {
                agents.push(name.replace(".template", ""));
            }
        }

        if !skills.is_empty() {
            logger.info("\nSkills:");
            for skill in skills {
                logger.info(&format!("  - {}", skill));
            }
        }

        if !agents.is_empty() {
            logger.info("\nAgents:");
            for agent in agents {
                logger.info(&format!("  - {}", agent));
            }
        }

        Ok(())
    }
}

/// Show template command.
pub struct ShowTemplateCommand {
    pub path: String,
}

impl ShowTemplateCommand {
    /// Execute the templates show command.
    pub async fn execute<F, R, L>(
        &self,
        fs: &F,
        resolver: &R,
        logger: &L,
        tool: ToolType,
    ) -> Result<()>
    where
        F: FileSystem,
        R: TemplateResolver,
        L: Logger,
    {
        logger.info(&format!("Showing template: {}", self.path));

        let template_path = resolver.resolve(&self.path, tool)?;
        let content = fs.read(&template_path).await?;

        println!("\n{}\n", content);

        logger.info(&format!("Template path: {}", template_path.display()));

        Ok(())
    }
}

/// Initialize project templates command.
pub struct TemplatesInitCommand {
    pub force: bool,
}

impl TemplatesInitCommand {
    /// Execute the templates init command.
    pub async fn execute<F, R, L>(
        &self,
        fs: &F,
        resolver: &R,
        logger: &L,
        project_path: &Path,
    ) -> Result<()>
    where
        F: FileSystem,
        R: TemplateResolver,
        L: Logger,
    {
        logger.info(&format!(
            "Initializing project templates{}",
            if self.force { " (forced)" } else { "" }
        ));

        let project_templates_dir = project_path.join(".aiassisted").join("templates");

        // Check if already exists
        if fs.exists(&project_templates_dir) && !self.force {
            logger.warn("Project templates directory already exists. Use --force to overwrite.");
            return Ok(());
        }

        // Create directory
        fs.create_dir_all(&project_templates_dir).await?;

        // Copy from global
        let global_dir = resolver.global_templates_dir();

        if !fs.exists(&global_dir) {
            logger.warn("Global templates directory not found. Creating empty project templates.");
            logger.success(&format!(
                "Created empty templates directory: {}",
                project_templates_dir.display()
            ));
            return Ok(());
        }

        // Copy recursively from global to project
        let copied_count = copy_dir_all(fs, &global_dir, &project_templates_dir, self.force).await?;

        logger.success(&format!(
            "Initialized templates directory with {} file(s): {}",
            copied_count,
            project_templates_dir.display()
        ));

        Ok(())
    }
}

/// Sync project templates with global command.
pub struct TemplatesSyncCommand {
    pub force: bool,
}

impl TemplatesSyncCommand {
    /// Execute the templates sync command.
    pub async fn execute<F, R, L>(
        &self,
        fs: &F,
        resolver: &R,
        logger: &L,
        project_path: &Path,
    ) -> Result<()>
    where
        F: FileSystem,
        R: TemplateResolver,
        L: Logger,
    {
        logger.info(&format!(
            "Syncing templates{}",
            if self.force { " (forced)" } else { "" }
        ));

        let project_templates_dir = project_path.join(".aiassisted").join("templates");

        if !fs.exists(&project_templates_dir) {
            logger.warn("Project templates not initialized. Run 'templates init' first.");
            return Ok(());
        }

        let global_dir = resolver.global_templates_dir();

        if !fs.exists(&global_dir) {
            logger.warn("Global templates directory not found. Nothing to sync.");
            return Ok(());
        }

        // Sync newer files from global to project
        let synced_count = sync_dir(fs, &global_dir, &project_templates_dir, self.force).await?;

        if synced_count > 0 {
            logger.success(&format!("Synced {} file(s) successfully", synced_count));
        } else {
            logger.info("No files needed syncing (all up to date)");
        }

        Ok(())
    }
}

/// Show template paths command.
pub struct TemplatesPathCommand;

impl TemplatesPathCommand {
    /// Execute the templates path command.
    pub async fn execute<R, L>(
        &self,
        resolver: &R,
        logger: &L,
        project_path: &Path,
    ) -> Result<()>
    where
        R: TemplateResolver,
        L: Logger,
    {
        logger.info("Template directories:");

        // Project templates
        if let Some(project_dir) = resolver.project_templates_dir() {
            logger.info(&format!("  Project: {}", project_dir.display()));
        } else {
            logger.info(&format!(
                "  Project: {} (not found)",
                project_path.join(".aiassisted/templates").display()
            ));
        }

        // Global templates
        let global_dir = resolver.global_templates_dir();
        logger.info(&format!("  Global:  {}", global_dir.display()));

        Ok(())
    }
}

/// Show template differences command.
pub struct TemplatesDiffCommand {
    pub path: Option<String>,
}

impl TemplatesDiffCommand {
    /// Execute the templates diff command.
    pub async fn execute<F, R, L, C>(
        &self,
        fs: &F,
        resolver: &R,
        logger: &L,
        checksum: &C,
        project_path: &Path,
    ) -> Result<()>
    where
        F: FileSystem,
        R: TemplateResolver,
        L: Logger,
        C: Checksum,
    {
        if let Some(ref path) = self.path {
            logger.info(&format!("Diffing template: {}", path));
        } else {
            logger.info("Diffing all templates");
        }

        let project_templates_dir = project_path.join(".aiassisted").join("templates");

        if !fs.exists(&project_templates_dir) {
            logger.warn("No project templates to diff");
            return Ok(());
        }

        let global_dir = resolver.global_templates_dir();

        if !fs.exists(&global_dir) {
            logger.warn("No global templates to compare");
            return Ok(());
        }

        // If specific path provided, diff just that file
        if let Some(ref path_str) = self.path {
            let project_file = project_templates_dir.join(path_str);
            let global_file = global_dir.join(path_str);

            if !fs.exists(&project_file) {
                logger.warn(&format!("Project template not found: {}", path_str));
                return Ok(());
            }

            if !fs.exists(&global_file) {
                logger.warn(&format!("Global template not found: {}", path_str));
                return Ok(());
            }

            // Compare using checksums
            let project_hash = checksum.sha256_file(&project_file)?;
            let global_hash = checksum.sha256_file(&global_file)?;

            if project_hash == global_hash {
                logger.info("No differences");
            } else {
                logger.info(&format!("Modified: {}", path_str));
            }

            return Ok(());
        }

        // Diff all templates
        let project_files = collect_files(fs, &project_templates_dir, &project_templates_dir).await?;
        let global_files = collect_files(fs, &global_dir, &global_dir).await?;

        let mut modified = Vec::new();
        let mut added = Vec::new();
        let mut removed = Vec::new();

        // Check for modified files (in both project and global)
        for project_rel in &project_files {
            let project_full = project_templates_dir.join(project_rel);
            let global_full = global_dir.join(project_rel);

            if global_files.contains(project_rel) {
                // File exists in both - compare checksums
                let project_hash = checksum.sha256_file(&project_full)?;
                let global_hash = checksum.sha256_file(&global_full)?;

                if project_hash != global_hash {
                    modified.push(project_rel.display().to_string());
                }
            } else {
                // File exists in project but not in global
                added.push(project_rel.display().to_string());
            }
        }

        // Check for removed files (in global but not in project)
        for global_rel in &global_files {
            if !project_files.contains(global_rel) {
                removed.push(global_rel.display().to_string());
            }
        }

        // Display results
        if modified.is_empty() && added.is_empty() && removed.is_empty() {
            logger.info("No differences found");
            return Ok(());
        }

        if !modified.is_empty() {
            logger.info("\nModified files:");
            for file in modified {
                logger.info(&format!("  {}", file));
            }
        }

        if !added.is_empty() {
            logger.info("\nAdded in project (not in global):");
            for file in added {
                logger.info(&format!("  {}", file));
            }
        }

        if !removed.is_empty() {
            logger.info("\nRemoved from project (exists in global):");
            for file in removed {
                logger.info(&format!("  {}", file));
            }
        }

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::{mock, predicate::*};
    use std::collections::HashMap;

    // Mock definitions
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

    mock! {
        pub Checksum {}

        impl crate::core::infra::Checksum for Checksum {
            fn sha256(&self, data: &[u8]) -> String;
            fn sha256_file(&self, path: &Path) -> Result<String>;
        }
    }

    // SetupSkillsCommand tests

    #[tokio::test]
    async fn test_setup_skills_success() {
        let mut mock_fs = MockFileSystem::new();
        let mut mock_engine = MockTemplateEngine::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");
        let template = PathBuf::from("/templates/skill.SKILL.md.template");

        mock_resolver
            .expect_list_templates()
            .returning(move |_| Ok(vec![template.clone()]));
        mock_fs.expect_read().returning(|_| Ok("content".to_string()));
        mock_fs.expect_exists().returning(|_| false);
        mock_engine.expect_render().returning(|_, _| Ok("rendered".to_string()));
        mock_fs.expect_create_dir_all().returning(|_| Ok(()));
        mock_fs.expect_write().returning(|_, _| Ok(()));

        mock_logger.expect_info().returning(|_| ());
        mock_logger.expect_success().returning(|_| ());

        let cmd = SetupSkillsCommand {
            tool: ToolType::Claude,
            dry_run: false,
        };

        let result = cmd
            .execute(&mock_fs, &mock_engine, &mock_resolver, &mock_logger, &project_path)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_setup_skills_dry_run() {
        let mock_fs = MockFileSystem::new();
        let mock_engine = MockTemplateEngine::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");
        let template = PathBuf::from("/templates/skill.SKILL.md.template");

        mock_resolver
            .expect_list_templates()
            .returning(move |_| Ok(vec![template.clone()]));

        mock_logger.expect_info().returning(|_| ());
        mock_logger.expect_success().returning(|_| ());

        let cmd = SetupSkillsCommand {
            tool: ToolType::Claude,
            dry_run: true,
        };

        let result = cmd
            .execute(&mock_fs, &mock_engine, &mock_resolver, &mock_logger, &project_path)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_setup_skills_auto_detect() {
        let mut mock_fs = MockFileSystem::new();
        let mock_engine = MockTemplateEngine::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");

        mock_fs.expect_exists().returning(|_| false);
        mock_resolver
            .expect_list_templates()
            .returning(|_| Ok(vec![]));

        mock_logger.expect_info().returning(|_| ());
        mock_logger.expect_warn().returning(|_| ());

        let cmd = SetupSkillsCommand {
            tool: ToolType::Auto,
            dry_run: false,
        };

        let result = cmd
            .execute(&mock_fs, &mock_engine, &mock_resolver, &mock_logger, &project_path)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_setup_skills_no_templates() {
        let mock_fs = MockFileSystem::new();
        let mock_engine = MockTemplateEngine::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");

        mock_resolver
            .expect_list_templates()
            .returning(|_| Ok(vec![]));

        mock_logger.expect_info().returning(|_| ());
        mock_logger.expect_warn().returning(|_| ());

        let cmd = SetupSkillsCommand {
            tool: ToolType::Claude,
            dry_run: false,
        };

        let result = cmd
            .execute(&mock_fs, &mock_engine, &mock_resolver, &mock_logger, &project_path)
            .await;

        assert!(result.is_ok());
    }

    // SetupAgentsCommand tests

    #[tokio::test]
    async fn test_setup_agents_success() {
        let mut mock_fs = MockFileSystem::new();
        let mut mock_engine = MockTemplateEngine::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");
        let template = PathBuf::from("/templates/agent.md.template");

        mock_resolver
            .expect_list_templates()
            .returning(move |_| Ok(vec![template.clone()]));
        mock_fs.expect_read().returning(|_| Ok("content".to_string()));
        mock_fs.expect_exists().returning(|_| false);
        mock_engine.expect_render().returning(|_, _| Ok("rendered".to_string()));
        mock_fs.expect_create_dir_all().returning(|_| Ok(()));
        mock_fs.expect_write().returning(|_, _| Ok(()));

        mock_logger.expect_info().returning(|_| ());
        mock_logger.expect_success().returning(|_| ());

        let cmd = SetupAgentsCommand {
            tool: ToolType::Claude,
            dry_run: false,
        };

        let result = cmd
            .execute(&mock_fs, &mock_engine, &mock_resolver, &mock_logger, &project_path)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_setup_agents_dry_run() {
        let mock_fs = MockFileSystem::new();
        let mock_engine = MockTemplateEngine::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");
        let template = PathBuf::from("/templates/agent.md.template");

        mock_resolver
            .expect_list_templates()
            .returning(move |_| Ok(vec![template.clone()]));

        mock_logger.expect_info().returning(|_| ());
        mock_logger.expect_success().returning(|_| ());

        let cmd = SetupAgentsCommand {
            tool: ToolType::Claude,
            dry_run: true,
        };

        let result = cmd
            .execute(&mock_fs, &mock_engine, &mock_resolver, &mock_logger, &project_path)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_setup_agents_auto_detect() {
        let mut mock_fs = MockFileSystem::new();
        let mock_engine = MockTemplateEngine::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");

        mock_fs.expect_exists().returning(|_| false);
        mock_resolver
            .expect_list_templates()
            .returning(|_| Ok(vec![]));

        mock_logger.expect_info().returning(|_| ());
        mock_logger.expect_warn().returning(|_| ());

        let cmd = SetupAgentsCommand {
            tool: ToolType::Auto,
            dry_run: false,
        };

        let result = cmd
            .execute(&mock_fs, &mock_engine, &mock_resolver, &mock_logger, &project_path)
            .await;

        assert!(result.is_ok());
    }

    // ListTemplatesCommand tests

    #[tokio::test]
    async fn test_list_templates_with_templates() {
        let mut mock_fs = MockFileSystem::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");
        let skill = PathBuf::from("skill.SKILL.md.template");
        let agent = PathBuf::from("agent.md.template");

        mock_fs.expect_exists().returning(|_| false);
        mock_resolver
            .expect_list_templates()
            .returning(move |_| Ok(vec![skill.clone(), agent.clone()]));

        mock_logger.expect_info().returning(|_| ());

        let cmd = ListTemplatesCommand {
            tool: ToolType::Claude,
        };

        let result = cmd
            .execute(&mock_fs, &mock_resolver, &mock_logger, &project_path)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_templates_empty() {
        let mut mock_fs = MockFileSystem::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");

        mock_fs.expect_exists().returning(|_| false);
        mock_resolver
            .expect_list_templates()
            .returning(|_| Ok(vec![]));

        mock_logger.expect_info().returning(|_| ());
        mock_logger.expect_warn().returning(|_| ());

        let cmd = ListTemplatesCommand {
            tool: ToolType::Claude,
        };

        let result = cmd
            .execute(&mock_fs, &mock_resolver, &mock_logger, &project_path)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_templates_auto_detect() {
        let mut mock_fs = MockFileSystem::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");

        mock_fs.expect_exists().returning(|_| false);
        mock_resolver
            .expect_list_templates()
            .returning(|_| Ok(vec![]));

        mock_logger.expect_info().returning(|_| ());
        mock_logger.expect_warn().returning(|_| ());

        let cmd = ListTemplatesCommand {
            tool: ToolType::Auto,
        };

        let result = cmd
            .execute(&mock_fs, &mock_resolver, &mock_logger, &project_path)
            .await;

        assert!(result.is_ok());
    }

    // ShowTemplateCommand tests

    #[tokio::test]
    async fn test_show_template_success() {
        let mut mock_fs = MockFileSystem::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let template_path = PathBuf::from("/templates/test.SKILL.md.template");
        let template_path_clone = template_path.clone();

        mock_resolver
            .expect_resolve()
            .with(eq("test.SKILL.md"), eq(ToolType::Claude))
            .returning(move |_, _| Ok(template_path.clone()));

        mock_fs
            .expect_read()
            .with(eq(template_path_clone))
            .returning(|_| Ok("Template content".to_string()));

        mock_logger.expect_info().returning(|_| ());

        let cmd = ShowTemplateCommand {
            path: "test.SKILL.md".to_string(),
        };

        let result = cmd
            .execute(&mock_fs, &mock_resolver, &mock_logger, ToolType::Claude)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_show_template_not_found() {
        let mock_fs = MockFileSystem::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        mock_resolver
            .expect_resolve()
            .returning(|_, _| Err(crate::core::types::Error::NotFound("Template not found".to_string())));

        mock_logger.expect_info().returning(|_| ());

        let cmd = ShowTemplateCommand {
            path: "nonexistent.SKILL.md".to_string(),
        };

        let result = cmd
            .execute(&mock_fs, &mock_resolver, &mock_logger, ToolType::Claude)
            .await;

        assert!(result.is_err());
    }

    // TemplatesInitCommand tests

    #[tokio::test]
    async fn test_templates_init_success() {
        let mut mock_fs = MockFileSystem::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");
        let global_dir = PathBuf::from("/global/templates");

        mock_fs.expect_exists().returning(|_| false);
        mock_fs.expect_create_dir_all().returning(|_| Ok(()));
        mock_fs.expect_list_dir().returning(|_| Ok(vec![]));
        mock_resolver
            .expect_global_templates_dir()
            .returning(move || global_dir.clone());

        mock_logger.expect_info().returning(|_| ());
        mock_logger.expect_warn().returning(|_| ());
        mock_logger.expect_success().returning(|_| ());

        let cmd = TemplatesInitCommand { force: false };

        let result = cmd
            .execute(&mock_fs, &mock_resolver, &mock_logger, &project_path)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_templates_init_already_exists() {
        let mut mock_fs = MockFileSystem::new();
        let mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");

        mock_fs.expect_exists().returning(|_| true);
        mock_logger.expect_info().returning(|_| ());
        mock_logger.expect_warn().returning(|_| ());

        let cmd = TemplatesInitCommand { force: false };

        let result = cmd
            .execute(&mock_fs, &mock_resolver, &mock_logger, &project_path)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_templates_init_force_overwrite() {
        let mut mock_fs = MockFileSystem::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");
        let global_dir = PathBuf::from("/global/templates");

        mock_fs.expect_exists().returning(|_| true);
        mock_fs.expect_is_dir().returning(|_| true);
        mock_fs.expect_create_dir_all().returning(|_| Ok(()));
        mock_fs.expect_list_dir().returning(|_| Ok(vec![]));
        mock_resolver
            .expect_global_templates_dir()
            .returning(move || global_dir.clone());

        mock_logger.expect_info().returning(|_| ());
        mock_logger.expect_success().returning(|_| ());

        let cmd = TemplatesInitCommand { force: true };

        let result = cmd
            .execute(&mock_fs, &mock_resolver, &mock_logger, &project_path)
            .await;

        assert!(result.is_ok());
    }

    // TemplatesSyncCommand tests

    #[tokio::test]
    async fn test_templates_sync_success() {
        let mut mock_fs = MockFileSystem::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");
        let global_dir = PathBuf::from("/global/templates");

        mock_fs.expect_exists().returning(|_| true);
        mock_fs.expect_is_dir().returning(|_| true);
        mock_fs.expect_is_file().returning(|_| true);
        mock_fs.expect_create_dir_all().returning(|_| Ok(()));
        mock_fs.expect_list_dir().returning(|_| Ok(vec![]));
        mock_resolver
            .expect_global_templates_dir()
            .returning(move || global_dir.clone());

        mock_logger.expect_info().returning(|_| ());

        let cmd = TemplatesSyncCommand { force: false };

        let result = cmd
            .execute(&mock_fs, &mock_resolver, &mock_logger, &project_path)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_templates_sync_not_initialized() {
        let mut mock_fs = MockFileSystem::new();
        let mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");

        mock_fs.expect_exists().returning(|_| false);
        mock_logger.expect_info().returning(|_| ());
        mock_logger.expect_warn().returning(|_| ());

        let cmd = TemplatesSyncCommand { force: false };

        let result = cmd
            .execute(&mock_fs, &mock_resolver, &mock_logger, &project_path)
            .await;

        assert!(result.is_ok());
    }

    // TemplatesPathCommand tests

    #[tokio::test]
    async fn test_templates_path_shows_paths() {
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");
        let project_templates = PathBuf::from("/test/project/.aiassisted/templates");
        let global_dir = PathBuf::from("/global/templates");

        mock_resolver
            .expect_project_templates_dir()
            .returning(move || Some(project_templates.clone()));
        mock_resolver
            .expect_global_templates_dir()
            .returning(move || global_dir.clone());

        mock_logger.expect_info().returning(|_| ());

        let cmd = TemplatesPathCommand;

        let result = cmd
            .execute(&mock_resolver, &mock_logger, &project_path)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_templates_path_project_not_found() {
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();

        let project_path = PathBuf::from("/test/project");
        let global_dir = PathBuf::from("/global/templates");

        mock_resolver
            .expect_project_templates_dir()
            .returning(|| None);
        mock_resolver
            .expect_global_templates_dir()
            .returning(move || global_dir.clone());

        mock_logger.expect_info().returning(|_| ());

        let cmd = TemplatesPathCommand;

        let result = cmd
            .execute(&mock_resolver, &mock_logger, &project_path)
            .await;

        assert!(result.is_ok());
    }

    // TemplatesDiffCommand tests

    #[tokio::test]
    async fn test_templates_diff_basic() {
        let mut mock_fs = MockFileSystem::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();
        let mock_checksum = MockChecksum::new();

        let project_path = PathBuf::from("/test/project");
        let global_dir = PathBuf::from("/global/templates");

        mock_fs.expect_exists().returning(|_| true);
        mock_fs.expect_is_dir().returning(|_| true);
        mock_fs.expect_list_dir().returning(|_| Ok(vec![]));
        mock_resolver
            .expect_global_templates_dir()
            .returning(move || global_dir.clone());

        mock_logger.expect_info().returning(|_| ());

        let cmd = TemplatesDiffCommand { path: None };

        let result = cmd
            .execute(
                &mock_fs,
                &mock_resolver,
                &mock_logger,
                &mock_checksum,
                &project_path,
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_templates_diff_specific_path() {
        let mut mock_fs = MockFileSystem::new();
        let mut mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();
        let mut mock_checksum = MockChecksum::new();

        let project_path = PathBuf::from("/test/project");
        let global_dir = PathBuf::from("/global/templates");

        mock_fs.expect_exists().returning(|_| true);
        mock_checksum.expect_sha256_file().returning(|_| Ok("hash123".to_string()));
        mock_resolver
            .expect_global_templates_dir()
            .returning(move || global_dir.clone());

        mock_logger.expect_info().returning(|_| ());

        let cmd = TemplatesDiffCommand {
            path: Some("test.SKILL.md".to_string()),
        };

        let result = cmd
            .execute(
                &mock_fs,
                &mock_resolver,
                &mock_logger,
                &mock_checksum,
                &project_path,
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_templates_diff_no_project_templates() {
        let mut mock_fs = MockFileSystem::new();
        let mock_resolver = MockTemplateResolver::new();
        let mut mock_logger = MockLogger::new();
        let mock_checksum = MockChecksum::new();

        let project_path = PathBuf::from("/test/project");

        mock_fs.expect_exists().returning(|_| false);
        mock_logger.expect_info().returning(|_| ());
        mock_logger.expect_warn().returning(|_| ());

        let cmd = TemplatesDiffCommand { path: None };

        let result = cmd
            .execute(
                &mock_fs,
                &mock_resolver,
                &mock_logger,
                &mock_checksum,
                &project_path,
            )
            .await;

        assert!(result.is_ok());
    }
}
