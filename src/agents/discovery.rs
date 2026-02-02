//! Agent discovery - find agents in .aiassisted/agents/

use crate::agents::compiler::Platform;
use crate::core::infra::FileSystem;
use crate::core::types::Result;
use std::path::{Path, PathBuf};

/// Information about a discovered agent
#[derive(Debug, Clone)]
pub struct AgentInfo {
    /// Agent name (directory name)
    pub name: String,
    /// Path to the agent directory
    pub source_path: PathBuf,
    /// Path to the AGENT.md file
    pub agent_md_path: PathBuf,
}

/// Agent discovery and path resolution
pub struct AgentDiscovery<'a, F: FileSystem> {
    fs: &'a F,
    project_path: PathBuf,
}

impl<'a, F: FileSystem> AgentDiscovery<'a, F> {
    pub fn new(fs: &'a F, project_path: &Path) -> Self {
        Self {
            fs,
            project_path: project_path.to_path_buf(),
        }
    }

    /// Get the agents source directory
    pub fn agents_source_dir(&self) -> PathBuf {
        self.project_path.join(".aiassisted/agents")
    }

    /// Get the skills source directory (for validation)
    pub fn skills_source_dir(&self) -> PathBuf {
        self.project_path.join(".aiassisted/skills")
    }

    /// Get the target directory for compiled agents
    pub fn agents_target_dir(&self, platform: Platform) -> PathBuf {
        match platform {
            Platform::ClaudeCode => self.project_path.join(".claude/agents"),
            Platform::OpenCode => self.project_path.join(".opencode/agents"),
        }
    }

    /// Discover all agents in the source directory
    pub async fn discover_agents(&self) -> Result<Vec<AgentInfo>> {
        let source_dir = self.agents_source_dir();

        if !self.fs.exists(&source_dir) {
            return Ok(Vec::new());
        }

        let mut agents = Vec::new();
        let entries = self.fs.list_dir(&source_dir).await?;

        for entry in entries {
            let agent_md = entry.join("AGENT.md");

            if self.fs.is_dir(&entry) && self.fs.exists(&agent_md) {
                let name = entry
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                agents.push(AgentInfo {
                    name,
                    source_path: entry,
                    agent_md_path: agent_md,
                });
            }
        }

        // Sort by name for consistent output
        agents.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(agents)
    }

    /// Discover installed agents for a platform
    pub async fn discover_installed_agents(&self, platform: Platform) -> Result<Vec<AgentInfo>> {
        let target_dir = self.agents_target_dir(platform);

        if !self.fs.exists(&target_dir) {
            return Ok(Vec::new());
        }

        let mut agents = Vec::new();
        let entries = self.fs.list_dir(&target_dir).await?;

        for entry in entries {
            if self.fs.is_dir(&entry) {
                let name = entry
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // For installed agents, there's no AGENT.md (it's compiled)
                agents.push(AgentInfo {
                    name,
                    source_path: entry.clone(),
                    agent_md_path: entry, // Not applicable for installed agents
                });
            }
        }

        agents.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(agents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock filesystem for testing
    struct MockFs {
        dirs: Vec<PathBuf>,
        files: Vec<PathBuf>,
    }

    impl MockFs {
        fn new() -> Self {
            Self {
                dirs: vec![],
                files: vec![],
            }
        }

        fn with_agent(mut self, name: &str) -> Self {
            let base = PathBuf::from("/project/.aiassisted/agents").join(name);
            self.dirs.push(base.clone());
            self.files.push(base.join("AGENT.md"));
            self
        }
    }

    #[async_trait::async_trait]
    impl FileSystem for MockFs {
        async fn read(&self, _path: &Path) -> Result<String> {
            Ok(String::new())
        }

        async fn write(&self, _path: &Path, _content: &str) -> Result<()> {
            Ok(())
        }

        fn exists(&self, path: &Path) -> bool {
            self.dirs.iter().any(|d| d == path) || self.files.iter().any(|f| f == path)
        }

        fn is_dir(&self, path: &Path) -> bool {
            self.dirs.iter().any(|d| d == path)
        }

        fn is_file(&self, path: &Path) -> bool {
            self.files.iter().any(|f| f == path)
        }

        async fn create_dir_all(&self, _path: &Path) -> Result<()> {
            Ok(())
        }

        async fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
            Ok(self
                .dirs
                .iter()
                .filter(|d| d.parent() == Some(path))
                .cloned()
                .collect())
        }

        async fn copy(&self, _from: &Path, _to: &Path) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_agents_source_dir() {
        let fs = MockFs::new();
        let discovery = AgentDiscovery::new(&fs, Path::new("/project"));

        assert_eq!(
            discovery.agents_source_dir(),
            PathBuf::from("/project/.aiassisted/agents")
        );
    }

    #[test]
    fn test_skills_source_dir() {
        let fs = MockFs::new();
        let discovery = AgentDiscovery::new(&fs, Path::new("/project"));

        assert_eq!(
            discovery.skills_source_dir(),
            PathBuf::from("/project/.aiassisted/skills")
        );
    }

    #[test]
    fn test_agents_target_dir() {
        let fs = MockFs::new();
        let discovery = AgentDiscovery::new(&fs, Path::new("/project"));

        assert_eq!(
            discovery.agents_target_dir(Platform::ClaudeCode),
            PathBuf::from("/project/.claude/agents")
        );

        assert_eq!(
            discovery.agents_target_dir(Platform::OpenCode),
            PathBuf::from("/project/.opencode/agents")
        );
    }

    #[tokio::test]
    async fn test_discover_agents_empty() {
        let fs = MockFs::new();
        let discovery = AgentDiscovery::new(&fs, Path::new("/project"));

        let agents = discovery.discover_agents().await.unwrap();
        assert!(agents.is_empty());
    }

    #[tokio::test]
    async fn test_discover_agents_with_agents() {
        let mut fs = MockFs::new()
            .with_agent("code-explorer")
            .with_agent("code-reviewer");

        // Add the agents directory itself
        fs.dirs.push(PathBuf::from("/project/.aiassisted/agents"));

        let discovery = AgentDiscovery::new(&fs, Path::new("/project"));

        let agents = discovery.discover_agents().await.unwrap();
        assert_eq!(agents.len(), 2);
        assert_eq!(agents[0].name, "code-explorer");
        assert_eq!(agents[1].name, "code-reviewer");
    }
}
