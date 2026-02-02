//! Skill diff computation using SHA256 checksums

use crate::core::infra::{Checksum, FileSystem};
use crate::core::types::Result;
use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;

/// Status of a file within a skill
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStatus {
    /// Exists in source but not target
    New,
    /// Checksum differs between source and target
    Modified,
    /// Checksums match
    Unchanged,
    /// Exists in target but not source
    Removed,
}

/// Status of a skill directory
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillStatus {
    /// Directory doesn't exist in target
    New,
    /// Some files changed
    Updated,
    /// All files identical
    Unchanged,
    /// Exists in target but not source
    Removed,
}

/// Information about a file within a skill
#[derive(Debug, Clone)]
pub struct SkillFileInfo {
    /// Relative path within the skill directory
    pub relative_path: PathBuf,
    /// Absolute source path
    pub source_path: PathBuf,
    /// Absolute target path
    pub target_path: PathBuf,
    /// File status
    pub status: FileStatus,
}

/// Diff information for a skill
#[derive(Debug, Clone)]
pub struct SkillDiff {
    /// Skill name
    pub name: String,
    /// Overall skill status
    pub status: SkillStatus,
    /// Files within the skill
    pub files: Vec<SkillFileInfo>,
}

impl SkillDiff {
    /// Count of new files
    pub fn new_count(&self) -> usize {
        self.files.iter().filter(|f| f.status == FileStatus::New).count()
    }

    /// Count of modified files
    pub fn modified_count(&self) -> usize {
        self.files.iter().filter(|f| f.status == FileStatus::Modified).count()
    }

    /// Count of unchanged files
    pub fn unchanged_count(&self) -> usize {
        self.files.iter().filter(|f| f.status == FileStatus::Unchanged).count()
    }

    /// Count of removed files
    pub fn removed_count(&self) -> usize {
        self.files.iter().filter(|f| f.status == FileStatus::Removed).count()
    }
}

/// Complete diff between source and target skills directories
#[derive(Debug, Clone)]
pub struct SkillsUpdateDiff {
    /// Diffs for each skill
    pub skills: Vec<SkillDiff>,
}

impl SkillsUpdateDiff {
    /// Count of new skills
    pub fn new_skills_count(&self) -> usize {
        self.skills.iter().filter(|s| s.status == SkillStatus::New).count()
    }

    /// Count of updated skills
    pub fn updated_skills_count(&self) -> usize {
        self.skills.iter().filter(|s| s.status == SkillStatus::Updated).count()
    }

    /// Count of unchanged skills
    pub fn unchanged_skills_count(&self) -> usize {
        self.skills.iter().filter(|s| s.status == SkillStatus::Unchanged).count()
    }

    /// Count of removed skills
    pub fn removed_skills_count(&self) -> usize {
        self.skills.iter().filter(|s| s.status == SkillStatus::Removed).count()
    }

    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        self.skills.iter().any(|s| s.status != SkillStatus::Unchanged)
    }

    /// Get all files that need to be copied (new or modified)
    pub fn files_to_update(&self) -> Vec<&SkillFileInfo> {
        self.skills
            .iter()
            .flat_map(|s| s.files.iter())
            .filter(|f| matches!(f.status, FileStatus::New | FileStatus::Modified))
            .collect()
    }
}

/// Computes diffs between source and target skills using SHA256 checksums
pub struct SkillDiffer<'a, F: FileSystem, C: Checksum> {
    fs: &'a F,
    checksum: &'a C,
}

impl<'a, F: FileSystem, C: Checksum> SkillDiffer<'a, F, C> {
    /// Create a new SkillDiffer
    pub fn new(fs: &'a F, checksum: &'a C) -> Self {
        Self { fs, checksum }
    }

    /// Compute diff between source and target skills directories
    pub async fn compute_diff(
        &self,
        source_dir: &Path,
        target_dir: &Path,
    ) -> Result<SkillsUpdateDiff> {
        let mut skill_diffs = Vec::new();

        // Get source skills
        let source_skills = self.discover_skill_dirs(source_dir).await?;
        let source_names: HashMap<String, PathBuf> = source_skills
            .into_iter()
            .filter_map(|p| {
                let name = p.file_name().and_then(|n| n.to_str()).map(|n| n.to_string());
                name.map(|n| (n, p))
            })
            .collect();

        // Get target skills (if target exists)
        let target_names: HashMap<String, PathBuf> = if self.fs.exists(target_dir) {
            let target_skills = self.discover_skill_dirs(target_dir).await.unwrap_or_default();
            target_skills
                .into_iter()
                .filter_map(|p| {
                    let name = p.file_name().and_then(|n| n.to_str()).map(|n| n.to_string());
                    name.map(|n| (n, p))
                })
                .collect()
        } else {
            HashMap::new()
        };

        // Process source skills
        for (name, source_path) in &source_names {
            let target_path = target_dir.join(name);

            if let Some(existing_target) = target_names.get(name) {
                // Skill exists in both - compute file diffs
                let files = self
                    .compute_skill_files_diff(source_path, existing_target)
                    .await?;

                let status = if files.iter().all(|f| f.status == FileStatus::Unchanged) {
                    SkillStatus::Unchanged
                } else {
                    SkillStatus::Updated
                };

                skill_diffs.push(SkillDiff {
                    name: name.clone(),
                    status,
                    files,
                });
            } else {
                // New skill - all files are new
                let files = self.collect_all_files_as_new(source_path, &target_path).await?;

                skill_diffs.push(SkillDiff {
                    name: name.clone(),
                    status: SkillStatus::New,
                    files,
                });
            }
        }

        // Process removed skills (exist in target but not source)
        for name in target_names.keys() {
            if !source_names.contains_key(name) {
                skill_diffs.push(SkillDiff {
                    name: name.clone(),
                    status: SkillStatus::Removed,
                    files: Vec::new(), // We don't need file details for removed skills
                });
            }
        }

        // Sort by name for consistent output
        skill_diffs.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(SkillsUpdateDiff {
            skills: skill_diffs,
        })
    }

    /// Discover skill directories (directories containing SKILL.md)
    async fn discover_skill_dirs(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        if !self.fs.exists(dir) {
            return Ok(Vec::new());
        }

        let mut skills = Vec::new();
        let entries = self.fs.list_dir(dir).await?;

        for entry in entries {
            let skill_md = entry.join("SKILL.md");
            if self.fs.is_dir(&entry) && self.fs.exists(&skill_md) {
                skills.push(entry);
            }
        }

        Ok(skills)
    }

    /// Compute file-level diff between source and target skill directories
    async fn compute_skill_files_diff(
        &self,
        source_skill: &Path,
        target_skill: &Path,
    ) -> Result<Vec<SkillFileInfo>> {
        let mut files = Vec::new();

        // Get all files in source
        let source_files = self.collect_files_recursive(source_skill).await?;
        let source_map: HashMap<PathBuf, PathBuf> = source_files
            .into_iter()
            .filter_map(|p| {
                let rel = p.strip_prefix(source_skill).ok().map(|r| r.to_path_buf());
                rel.map(|r| (r, p))
            })
            .collect();

        // Get all files in target
        let target_files = self.collect_files_recursive(target_skill).await?;
        let target_map: HashMap<PathBuf, PathBuf> = target_files
            .into_iter()
            .filter_map(|p| {
                let rel = p.strip_prefix(target_skill).ok().map(|r| r.to_path_buf());
                rel.map(|r| (r, p))
            })
            .collect();

        // Check source files
        for (rel_path, source_path) in &source_map {
            let target_path = target_skill.join(rel_path);

            let status = if let Some(existing_target) = target_map.get(rel_path) {
                // File exists in both - compare checksums
                let source_hash = self.checksum.sha256_file(source_path)?;
                let target_hash = self.checksum.sha256_file(existing_target)?;

                if source_hash == target_hash {
                    FileStatus::Unchanged
                } else {
                    FileStatus::Modified
                }
            } else {
                FileStatus::New
            };

            files.push(SkillFileInfo {
                relative_path: rel_path.clone(),
                source_path: source_path.clone(),
                target_path,
                status,
            });
        }

        // Check for removed files (in target but not source)
        for (rel_path, target_path) in &target_map {
            if !source_map.contains_key(rel_path) {
                files.push(SkillFileInfo {
                    relative_path: rel_path.clone(),
                    source_path: PathBuf::new(), // No source for removed files
                    target_path: target_path.clone(),
                    status: FileStatus::Removed,
                });
            }
        }

        // Sort by relative path
        files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

        Ok(files)
    }

    /// Collect all files for a new skill (mark all as New)
    async fn collect_all_files_as_new(
        &self,
        source_skill: &Path,
        target_skill: &Path,
    ) -> Result<Vec<SkillFileInfo>> {
        let source_files = self.collect_files_recursive(source_skill).await?;

        let files: Vec<SkillFileInfo> = source_files
            .into_iter()
            .filter_map(|source_path| {
                let rel = source_path
                    .strip_prefix(source_skill)
                    .ok()
                    .map(|r| r.to_path_buf());
                rel.map(|relative_path| SkillFileInfo {
                    target_path: target_skill.join(&relative_path),
                    relative_path,
                    source_path,
                    status: FileStatus::New,
                })
            })
            .collect();

        Ok(files)
    }

    /// Recursively collect all files in a directory
    fn collect_files_recursive<'b>(
        &'b self,
        dir: &'b Path,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<PathBuf>>> + Send + 'b>> {
        Box::pin(async move {
            if !self.fs.exists(dir) {
                return Ok(Vec::new());
            }

            let mut files = Vec::new();
            let entries = self.fs.list_dir(dir).await?;

            for entry in entries {
                if self.fs.is_dir(&entry) {
                    let sub_files = self.collect_files_recursive(&entry).await?;
                    files.extend(sub_files);
                } else {
                    files.push(entry);
                }
            }

            Ok(files)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_diff_counts() {
        let diff = SkillDiff {
            name: "test".to_string(),
            status: SkillStatus::Updated,
            files: vec![
                SkillFileInfo {
                    relative_path: PathBuf::from("new.md"),
                    source_path: PathBuf::from("/src/new.md"),
                    target_path: PathBuf::from("/tgt/new.md"),
                    status: FileStatus::New,
                },
                SkillFileInfo {
                    relative_path: PathBuf::from("modified.md"),
                    source_path: PathBuf::from("/src/modified.md"),
                    target_path: PathBuf::from("/tgt/modified.md"),
                    status: FileStatus::Modified,
                },
                SkillFileInfo {
                    relative_path: PathBuf::from("unchanged.md"),
                    source_path: PathBuf::from("/src/unchanged.md"),
                    target_path: PathBuf::from("/tgt/unchanged.md"),
                    status: FileStatus::Unchanged,
                },
            ],
        };

        assert_eq!(diff.new_count(), 1);
        assert_eq!(diff.modified_count(), 1);
        assert_eq!(diff.unchanged_count(), 1);
    }

    #[test]
    fn test_skills_update_diff_counts() {
        let diff = SkillsUpdateDiff {
            skills: vec![
                SkillDiff {
                    name: "new-skill".to_string(),
                    status: SkillStatus::New,
                    files: vec![],
                },
                SkillDiff {
                    name: "updated-skill".to_string(),
                    status: SkillStatus::Updated,
                    files: vec![],
                },
                SkillDiff {
                    name: "unchanged-skill".to_string(),
                    status: SkillStatus::Unchanged,
                    files: vec![],
                },
                SkillDiff {
                    name: "removed-skill".to_string(),
                    status: SkillStatus::Removed,
                    files: vec![],
                },
            ],
        };

        assert_eq!(diff.new_skills_count(), 1);
        assert_eq!(diff.updated_skills_count(), 1);
        assert_eq!(diff.unchanged_skills_count(), 1);
        assert_eq!(diff.removed_skills_count(), 1);
        assert!(diff.has_changes());
    }

    #[test]
    fn test_skills_update_diff_no_changes() {
        let diff = SkillsUpdateDiff {
            skills: vec![SkillDiff {
                name: "unchanged-skill".to_string(),
                status: SkillStatus::Unchanged,
                files: vec![],
            }],
        };

        assert!(!diff.has_changes());
    }

    #[test]
    fn test_files_to_update() {
        let diff = SkillsUpdateDiff {
            skills: vec![SkillDiff {
                name: "skill".to_string(),
                status: SkillStatus::Updated,
                files: vec![
                    SkillFileInfo {
                        relative_path: PathBuf::from("new.md"),
                        source_path: PathBuf::from("/src/new.md"),
                        target_path: PathBuf::from("/tgt/new.md"),
                        status: FileStatus::New,
                    },
                    SkillFileInfo {
                        relative_path: PathBuf::from("modified.md"),
                        source_path: PathBuf::from("/src/modified.md"),
                        target_path: PathBuf::from("/tgt/modified.md"),
                        status: FileStatus::Modified,
                    },
                    SkillFileInfo {
                        relative_path: PathBuf::from("unchanged.md"),
                        source_path: PathBuf::from("/src/unchanged.md"),
                        target_path: PathBuf::from("/tgt/unchanged.md"),
                        status: FileStatus::Unchanged,
                    },
                ],
            }],
        };

        let files_to_update = diff.files_to_update();
        assert_eq!(files_to_update.len(), 2);
    }
}
