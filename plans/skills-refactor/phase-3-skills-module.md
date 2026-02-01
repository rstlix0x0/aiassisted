# Phase 3: Create New Skills Module

**Status**: Pending

## Objective

Create `src/skills/` module with simplified implementation for copying skills.

## Directory Structure

```
src/skills/
├── mod.rs           # Module exports
├── commands.rs      # SetupSkillsCommand, SkillsListCommand
├── discovery.rs     # ToolDetector (moved from templates)
└── copier.rs        # SkillCopier implementation
```

## Files to Create

### 1. `src/skills/mod.rs`

```rust
//! Skills domain - install and manage AI skills

mod commands;
mod copier;
mod discovery;

pub use commands::{SetupSkillsCommand, SkillsListCommand};
pub use discovery::ToolDetector;
```

### 2. `src/skills/discovery.rs`

Move and modify `ToolDetector` from `src/templates/discovery.rs`:

```rust
//! Tool detection for AI assistants

use crate::core::{Result, ToolType};
use crate::core::infra::FileSystem;
use std::path::{Path, PathBuf};

/// Detects which AI tool a project uses and provides output paths
pub struct ToolDetector<'a, F: FileSystem> {
    fs: &'a F,
    project_path: PathBuf,
}

impl<'a, F: FileSystem> ToolDetector<'a, F> {
    pub fn new(fs: &'a F, project_path: &Path) -> Self {
        Self {
            fs,
            project_path: project_path.to_path_buf(),
        }
    }

    /// Detect which tool the project uses
    pub fn detect(&self) -> ToolType {
        // Check for OpenCode config
        if self.fs.exists(&self.project_path.join(".opencode.json")) {
            return ToolType::OpenCode;
        }

        // Check for Claude Code indicators
        if self.fs.exists(&self.project_path.join(".claude"))
            || self.fs.exists(&self.project_path.join("CLAUDE.md"))
        {
            return ToolType::Claude;
        }

        // Default to Claude
        ToolType::Claude
    }

    /// Get the skills output directory for a tool
    pub fn skills_dir(&self, tool: ToolType) -> PathBuf {
        let tool = if tool == ToolType::Auto {
            self.detect()
        } else {
            tool
        };

        match tool {
            ToolType::OpenCode => self.project_path.join(".opencode/skills"),
            ToolType::Claude | ToolType::Auto => self.project_path.join(".claude/skills"),
        }
    }

    /// Get the skills source directory
    pub fn skills_source_dir(&self) -> PathBuf {
        self.project_path.join(".aiassisted/skills")
    }
}
```

**Key changes from old implementation:**
- Claude skills now go to `.claude/skills` (not `.claude/commands`)
- Removed `agents_dir()` method
- Added `skills_source_dir()` method

### 3. `src/skills/copier.rs`

```rust
//! Skill directory copying

use crate::core::{Error, Result};
use crate::core::infra::FileSystem;
use std::path::Path;

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
    pub fn discover_skills(&self, source_dir: &Path) -> Result<Vec<SkillInfo>> {
        if !self.fs.exists(source_dir) {
            return Err(Error::NotFound(format!(
                "Skills source directory not found: {}",
                source_dir.display()
            )));
        }

        let mut skills = Vec::new();
        let entries = self.fs.list_dir(source_dir)?;

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
    pub fn copy_skill(
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
        self.fs.create_dir_all(&target_skill_dir)?;

        // Copy all files recursively
        self.copy_dir_recursive(&skill.source_path, &target_skill_dir)?;

        Ok(true)
    }

    /// Recursively copy directory contents
    fn copy_dir_recursive(&self, source: &Path, target: &Path) -> Result<()> {
        let entries = self.fs.list_dir(source)?;

        for entry in entries {
            let file_name = entry
                .file_name()
                .ok_or_else(|| Error::InvalidInput("Invalid file name".to_string()))?;
            let target_path = target.join(file_name);

            if self.fs.is_dir(&entry) {
                self.fs.create_dir_all(&target_path)?;
                self.copy_dir_recursive(&entry, &target_path)?;
            } else {
                self.fs.copy(&entry, &target_path)?;
            }
        }

        Ok(())
    }
}
```

### 4. `src/skills/commands.rs`

```rust
//! Skills domain commands

use crate::core::{Result, ToolType};
use crate::core::infra::{FileSystem, Logger};
use crate::skills::copier::{SkillCopier, SkillInfo};
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
        let skills = copier.discover_skills(&source_dir)?;

        if skills.is_empty() {
            logger.warn("No skills found in .aiassisted/skills/");
            logger.info("Run 'aiassisted install' to install skills first");
            return Ok(());
        }

        logger.info(&format!("Found {} skill(s)", skills.len()));

        // Create target directory if needed
        if !self.dry_run {
            fs.create_dir_all(&target_dir)?;
        }

        // Copy each skill
        let mut copied = 0;
        let mut skipped = 0;

        for skill in &skills {
            if self.dry_run {
                logger.info(&format!("Would copy: {} -> {}/{}",
                    skill.name,
                    target_dir.display(),
                    skill.name
                ));
                copied += 1;
            } else {
                match copier.copy_skill(skill, &target_dir, self.force)? {
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
            logger.info(&format!("Dry run: {} skill(s) would be copied to {}",
                copied, target_dir.display()));
        } else {
            logger.success(&format!("Setup complete: {} copied, {} skipped",
                copied, skipped));

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
        match copier.discover_skills(&source_dir) {
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
```

## Verification

```bash
cargo check
cargo test --lib skills
```

## Dependencies

- Phase 1 (CLI definitions)
- Phase 2 (core trait cleanup - can be done in parallel)

## Next Phase

[Phase 4: Update Main Entry Point](phase-4-main.md)
