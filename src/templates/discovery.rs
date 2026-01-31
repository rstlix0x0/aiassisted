//! AI tool detection from project files.

use std::path::Path;

use crate::core::infra::FileSystem;
use crate::core::types::ToolType;

/// Detects which AI tool a project is using.
pub struct ToolDetector;

impl ToolDetector {
    pub fn new() -> Self {
        Self
    }

    /// Detect the AI tool from project structure.
    ///
    /// Detection rules:
    /// - If `.opencode.json` exists → OpenCode
    /// - If `.claude/` directory or `CLAUDE.md` file exists → Claude
    /// - Default → Claude Code
    pub fn detect<F: FileSystem>(&self, fs: &F, project_path: &Path) -> ToolType {
        // Check for OpenCode
        let opencode_json = project_path.join(".opencode.json");
        if fs.exists(&opencode_json) && fs.is_file(&opencode_json) {
            return ToolType::OpenCode;
        }

        // Check for Claude Code
        let claude_dir = project_path.join(".claude");
        if fs.exists(&claude_dir) && fs.is_dir(&claude_dir) {
            return ToolType::Claude;
        }

        let claude_md = project_path.join("CLAUDE.md");
        if fs.exists(&claude_md) && fs.is_file(&claude_md) {
            return ToolType::Claude;
        }

        // Default to Claude Code
        ToolType::Claude
    }

    /// Get output directory for skills based on tool type.
    pub fn skills_dir(&self, tool: ToolType) -> &'static str {
        match tool {
            ToolType::OpenCode => ".opencode/skills",
            ToolType::Claude => ".claude/commands",
            ToolType::Auto => ".claude/commands", // Default to Claude
        }
    }

    /// Get output directory for agents based on tool type.
    pub fn agents_dir(&self, tool: ToolType) -> &'static str {
        match tool {
            ToolType::OpenCode => ".opencode/agents",
            ToolType::Claude => ".claude/agents",
            ToolType::Auto => ".claude/agents", // Default to Claude
        }
    }
}

impl Default for ToolDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use std::path::PathBuf;

    // Mock FileSystem for testing
    mock! {
        pub FileSystem {}

        #[async_trait::async_trait]
        impl crate::core::infra::FileSystem for FileSystem {
            async fn read(&self, path: &Path) -> crate::core::types::Result<String>;
            async fn write(&self, path: &Path, content: &str) -> crate::core::types::Result<()>;
            fn exists(&self, path: &Path) -> bool;
            fn is_dir(&self, path: &Path) -> bool;
            fn is_file(&self, path: &Path) -> bool;
            async fn create_dir_all(&self, path: &Path) -> crate::core::types::Result<()>;
            async fn list_dir(&self, path: &Path) -> crate::core::types::Result<Vec<PathBuf>>;
            async fn copy(&self, from: &Path, to: &Path) -> crate::core::types::Result<()>;
        }
    }

    #[test]
    fn test_detect_opencode_with_json() {
        let mut mock_fs = MockFileSystem::new();
        let project_path = PathBuf::from("/test/project");

        mock_fs
            .expect_exists()
            .returning(|_| true);
        mock_fs
            .expect_is_file()
            .returning(|_| true);

        let detector = ToolDetector::new();
        let result = detector.detect(&mock_fs, &project_path);
        assert_eq!(result, ToolType::OpenCode);
    }

    #[test]
    fn test_detect_claude_with_directory() {
        let mut mock_fs = MockFileSystem::new();
        let project_path = PathBuf::from("/test/project");

        // First: check .opencode.json exists -> false
        // Second: check .opencode.json is_file -> skip since exists is false
        // Third: check .claude exists -> true
        // Fourth: check .claude is_dir -> true
        mock_fs
            .expect_exists()
            .times(2)
            .returning(|_| false)
            .returning(|_| true);
        mock_fs
            .expect_is_file()
            .times(1)
            .returning(|_| false);
        mock_fs
            .expect_is_dir()
            .returning(|_| true);

        let detector = ToolDetector::new();
        let result = detector.detect(&mock_fs, &project_path);
        assert_eq!(result, ToolType::Claude);
    }

    #[test]
    fn test_detect_claude_with_md_file() {
        let mut mock_fs = MockFileSystem::new();
        let project_path = PathBuf::from("/test/project");

        // Check .opencode.json: exists=false, is_file skipped
        // Check .claude dir: exists=false, is_dir skipped
        // Check CLAUDE.md: exists=true, is_file=true
        mock_fs.expect_exists().times(3).returning(|p| {
            p.to_str().unwrap().contains("CLAUDE.md")
        });
        mock_fs.expect_is_file().times(1).returning(|_| true);

        let detector = ToolDetector::new();
        let result = detector.detect(&mock_fs, &project_path);
        assert_eq!(result, ToolType::Claude);
    }

    #[test]
    fn test_detect_defaults_to_claude() {
        let mut mock_fs = MockFileSystem::new();
        let project_path = PathBuf::from("/test/project");

        mock_fs.expect_exists().returning(|_| false);

        let detector = ToolDetector::new();
        let result = detector.detect(&mock_fs, &project_path);
        assert_eq!(result, ToolType::Claude);
    }

    #[test]
    fn test_detect_opencode_takes_precedence() {
        let mut mock_fs = MockFileSystem::new();
        let project_path = PathBuf::from("/test/project");

        // Both OpenCode and Claude markers exist, OpenCode should win
        mock_fs
            .expect_exists()
            .returning(|_| true);
        mock_fs
            .expect_is_file()
            .returning(|_| true);

        let detector = ToolDetector::new();
        let result = detector.detect(&mock_fs, &project_path);
        assert_eq!(result, ToolType::OpenCode);
    }

    #[test]
    fn test_skills_dir_opencode() {
        let detector = ToolDetector::new();
        let result = detector.skills_dir(ToolType::OpenCode);
        assert_eq!(result, ".opencode/skills");
    }

    #[test]
    fn test_skills_dir_claude() {
        let detector = ToolDetector::new();
        let result = detector.skills_dir(ToolType::Claude);
        assert_eq!(result, ".claude/commands");
    }

    #[test]
    fn test_skills_dir_auto_defaults_to_claude() {
        let detector = ToolDetector::new();
        let result = detector.skills_dir(ToolType::Auto);
        assert_eq!(result, ".claude/commands");
    }

    #[test]
    fn test_agents_dir_opencode() {
        let detector = ToolDetector::new();
        let result = detector.agents_dir(ToolType::OpenCode);
        assert_eq!(result, ".opencode/agents");
    }

    #[test]
    fn test_agents_dir_claude() {
        let detector = ToolDetector::new();
        let result = detector.agents_dir(ToolType::Claude);
        assert_eq!(result, ".claude/agents");
    }

    #[test]
    fn test_agents_dir_auto_defaults_to_claude() {
        let detector = ToolDetector::new();
        let result = detector.agents_dir(ToolType::Auto);
        assert_eq!(result, ".claude/agents");
    }

    #[test]
    fn test_default_creates_detector() {
        let detector = ToolDetector::default();
        let result = detector.skills_dir(ToolType::Claude);
        assert_eq!(result, ".claude/commands");
    }
}
