//! Skill directory copying

use crate::core::infra::FileSystem;
use crate::core::types::{Error, Result};
use std::future::Future;
use std::path::Path;
use std::pin::Pin;

/// Information about a skill to be copied
#[derive(Debug, Clone)]
pub struct SkillInfo {
    pub name: String,
    pub source_path: std::path::PathBuf,
}

/// Copies skill directories from source to target
pub struct SkillCopier<'a, F: FileSystem> {
    fs: &'a F,
}

impl<'a, F: FileSystem> SkillCopier<'a, F> {
    pub fn new(fs: &'a F) -> Self {
        Self { fs }
    }

    /// Discover all skills in the source directory
    pub async fn discover_skills(&self, source_dir: &Path) -> Result<Vec<SkillInfo>> {
        if !self.fs.exists(source_dir) {
            return Err(Error::NotFound(format!(
                "Skills source directory not found: {}",
                source_dir.display()
            )));
        }

        let mut skills = Vec::new();
        let entries = self.fs.list_dir(source_dir).await?;

        for entry in entries {
            let skill_md = entry.join("SKILL.md");
            if self.fs.is_dir(&entry) && self.fs.exists(&skill_md) {
                let name = entry
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                skills.push(SkillInfo {
                    name,
                    source_path: entry,
                });
            }
        }

        skills.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(skills)
    }

    /// Copy a single skill to the target directory
    /// Returns Ok(true) if copied, Ok(false) if skipped
    pub async fn copy_skill(
        &self,
        skill: &SkillInfo,
        target_dir: &Path,
        force: bool,
    ) -> Result<bool> {
        let target_skill_dir = target_dir.join(&skill.name);

        // Check if already exists
        if self.fs.exists(&target_skill_dir) && !force {
            return Ok(false); // Skipped
        }

        // Create target directory
        self.fs.create_dir_all(&target_skill_dir).await?;

        // Copy all files recursively
        self.copy_dir_recursive(&skill.source_path, &target_skill_dir).await?;

        Ok(true)
    }

    /// Recursively copy directory contents
    fn copy_dir_recursive<'b>(
        &'b self,
        source: &'b Path,
        target: &'b Path,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'b>> {
        Box::pin(async move {
            let entries = self.fs.list_dir(source).await?;

            for entry in entries {
                let file_name = entry
                    .file_name()
                    .ok_or_else(|| Error::Parse("Invalid file name".to_string()))?;
                let target_path = target.join(file_name);

                if self.fs.is_dir(&entry) {
                    self.fs.create_dir_all(&target_path).await?;
                    self.copy_dir_recursive(&entry, &target_path).await?;
                } else {
                    self.fs.copy(&entry, &target_path).await?;
                }
            }

            Ok(())
        })
    }
}

