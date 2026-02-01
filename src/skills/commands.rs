//! Skills domain commands

use crate::core::infra::{FileSystem, Logger};
use crate::core::types::{Result, ToolType};
use crate::skills::copier::SkillCopier;
use crate::skills::discovery::ToolDetector;
use std::path::Path;

/// Command to set up skills by copying from .aiassisted/skills/
pub struct SetupSkillsCommand {
    pub tool: ToolType,
    pub dry_run: bool,
    pub force: bool,
}

impl SetupSkillsCommand {
    pub async fn execute<F: FileSystem, L: Logger>(
        &self,
        fs: &F,
        logger: &L,
        project_path: &Path,
    ) -> Result<()> {
        let detector = ToolDetector::new(fs, project_path);
        let copier = SkillCopier::new(fs);

        // Resolve tool type
        let tool = if self.tool == ToolType::Auto {
            let detected = detector.detect();
            logger.info(&format!("Auto-detected tool: {}", detected));
            detected
        } else {
            self.tool
        };

        logger.info(&format!("Setting up skills for {}", tool));

        // Get source and target directories
        let source_dir = detector.skills_source_dir();
        let target_dir = detector.skills_dir(tool);

        // Discover available skills
        let skills = copier.discover_skills(&source_dir).await?;

        if skills.is_empty() {
            logger.warn("No skills found in .aiassisted/skills/");
            logger.info("Run 'aiassisted install' to install skills first");
            return Ok(());
        }

        logger.info(&format!("Found {} skill(s)", skills.len()));

        // Create target directory if needed
        if !self.dry_run {
            fs.create_dir_all(&target_dir).await?;
        }

        // Copy each skill
        let mut copied = 0;
        let mut skipped = 0;

        for skill in &skills {
            if self.dry_run {
                logger.info(&format!(
                    "Would copy: {} -> {}/{}",
                    skill.name,
                    target_dir.display(),
                    skill.name
                ));
                copied += 1;
            } else {
                match copier.copy_skill(skill, &target_dir, self.force).await? {
                    true => {
                        logger.success(&format!("Copied: {}", skill.name));
                        copied += 1;
                    }
                    false => {
                        logger.warn(&format!("Skipped (exists): {}", skill.name));
                        skipped += 1;
                    }
                }
            }
        }

        // Summary
        if self.dry_run {
            logger.info(&format!(
                "Dry run: {} skill(s) would be copied to {}",
                copied,
                target_dir.display()
            ));
        } else {
            logger.success(&format!(
                "Setup complete: {} copied, {} skipped",
                copied, skipped
            ));

            if skipped > 0 {
                logger.info("Use --force to overwrite existing skills");
            }
        }

        Ok(())
    }
}

/// Command to list available skills
pub struct SkillsListCommand {
    pub tool: ToolType,
}

impl SkillsListCommand {
    pub async fn execute<F: FileSystem, L: Logger>(
        &self,
        fs: &F,
        logger: &L,
        project_path: &Path,
    ) -> Result<()> {
        let detector = ToolDetector::new(fs, project_path);
        let copier = SkillCopier::new(fs);

        // Resolve tool type
        let tool = if self.tool == ToolType::Auto {
            detector.detect()
        } else {
            self.tool
        };

        let source_dir = detector.skills_source_dir();
        let target_dir = detector.skills_dir(tool);

        logger.info(&format!("Skills source: {}", source_dir.display()));
        logger.info(&format!("Target directory: {}", target_dir.display()));
        logger.info("");

        // Discover skills
        match copier.discover_skills(&source_dir).await {
            Ok(skills) => {
                if skills.is_empty() {
                    logger.warn("No skills found");
                    logger.info("Run 'aiassisted install' to install skills first");
                } else {
                    logger.info(&format!("Available skills ({}):", skills.len()));
                    logger.info("");
                    for skill in &skills {
                        let installed = fs.exists(&target_dir.join(&skill.name));
                        let status = if installed { "[installed]" } else { "" };
                        logger.info(&format!("  - {} {}", skill.name, status));
                    }
                }
            }
            Err(e) => {
                logger.warn(&format!("Could not list skills: {}", e));
                logger.info("Run 'aiassisted install' to install skills first");
            }
        }

        Ok(())
    }
}

