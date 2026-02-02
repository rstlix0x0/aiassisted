//! Skills domain commands

use crate::core::infra::{Checksum, FileSystem, Logger};
use crate::core::types::{Result, ToolType};
use crate::skills::copier::SkillCopier;
use crate::skills::diff::{FileStatus, SkillDiffer, SkillStatus};
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

/// Command to update installed skills (sync changes from source)
pub struct SkillsUpdateCommand {
    pub tool: ToolType,
    pub dry_run: bool,
    pub force: bool,
}

impl SkillsUpdateCommand {
    pub async fn execute<F: FileSystem, C: Checksum, L: Logger>(
        &self,
        fs: &F,
        checksum: &C,
        logger: &L,
        project_path: &Path,
    ) -> Result<()> {
        let detector = ToolDetector::new(fs, project_path);
        let differ = SkillDiffer::new(fs, checksum);

        // Resolve tool type
        let tool = if self.tool == ToolType::Auto {
            let detected = detector.detect();
            logger.info(&format!("Auto-detected tool: {}", detected));
            detected
        } else {
            self.tool
        };

        // Get source and target directories
        let source_dir = detector.skills_source_dir();
        let target_dir = detector.skills_dir(tool);

        logger.info(&format!("Source: {}", source_dir.display()));
        logger.info(&format!("Target: {}", target_dir.display()));

        // Check source exists
        if !fs.exists(&source_dir) {
            logger.warn("No skills found in .aiassisted/skills/");
            logger.info("Run 'aiassisted install' to install skills first");
            return Ok(());
        }

        // Check target exists
        if !fs.exists(&target_dir) {
            logger.warn("No skills installed yet");
            logger.info("Run 'aiassisted skills setup' to install skills first");
            return Ok(());
        }

        logger.info("Analyzing skills...");

        // Compute diff
        let diff = differ.compute_diff(&source_dir, &target_dir).await?;

        // Summary
        logger.info(&format!(
            "Summary: {} new, {} updated, {} unchanged, {} removed",
            diff.new_skills_count(),
            diff.updated_skills_count(),
            diff.unchanged_skills_count(),
            diff.removed_skills_count()
        ));
        logger.info("");

        // Show skill status
        logger.info("Skills status:");
        for skill in &diff.skills {
            let indicator = match skill.status {
                SkillStatus::New => "+",
                SkillStatus::Updated => "~",
                SkillStatus::Unchanged => "=",
                SkillStatus::Removed => "-",
            };

            let details = match skill.status {
                SkillStatus::New => format!("(new, {} file(s))", skill.files.len()),
                SkillStatus::Updated => format!(
                    "({} new, {} modified)",
                    skill.new_count(),
                    skill.modified_count()
                ),
                SkillStatus::Unchanged => "(unchanged)".to_string(),
                SkillStatus::Removed => "(removed from source)".to_string(),
            };

            logger.info(&format!("  {} {} {}", indicator, skill.name, details));
        }

        // Check if there are changes
        if !diff.has_changes() {
            logger.success("All skills are up to date!");
            return Ok(());
        }

        // Get files to update
        let files_to_update = if self.force {
            // Force mode: update all files from non-removed skills
            diff.skills
                .iter()
                .filter(|s| s.status != SkillStatus::Removed)
                .flat_map(|s| s.files.iter())
                .filter(|f| f.status != FileStatus::Removed)
                .collect::<Vec<_>>()
        } else {
            diff.files_to_update()
        };

        if files_to_update.is_empty() {
            logger.info("No files to update");
            return Ok(());
        }

        logger.info("");
        logger.info("Files to update:");
        for file in &files_to_update {
            let indicator = match file.status {
                FileStatus::New => "+",
                FileStatus::Modified => "~",
                _ => " ",
            };
            logger.info(&format!("  {} {}", indicator, file.target_path.display()));
        }

        // Perform update
        if self.dry_run {
            logger.info("");
            logger.info(&format!(
                "Dry run: {} file(s) would be updated",
                files_to_update.len()
            ));
        } else {
            logger.info("");

            let mut updated = 0;
            for file in &files_to_update {
                // Ensure parent directory exists
                if let Some(parent) = file.target_path.parent() {
                    fs.create_dir_all(parent).await?;
                }

                // Copy file
                fs.copy(&file.source_path, &file.target_path).await?;
                updated += 1;
            }

            logger.success(&format!(
                "Updated {} file(s) across {} skill(s)",
                updated,
                diff.skills
                    .iter()
                    .filter(|s| matches!(s.status, SkillStatus::New | SkillStatus::Updated))
                    .count()
            ));

            // Note about removed skills
            if diff.removed_skills_count() > 0 {
                logger.info(&format!(
                    "Note: {} skill(s) removed from source but still installed",
                    diff.removed_skills_count()
                ));
            }
        }

        Ok(())
    }
}
