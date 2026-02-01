//! Tool detection for AI assistants

use crate::core::infra::FileSystem;
use crate::core::types::ToolType;
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

