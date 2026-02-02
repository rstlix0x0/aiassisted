//! Agent diff computation using SHA256 checksums

use crate::agents::compiler::{compile_agent, CompiledAgent, Platform};
use crate::agents::parser::parse_agent_md;
use crate::core::infra::{Checksum, FileSystem};
use crate::core::types::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Status of an agent
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentStatus {
    /// Agent doesn't exist in target
    New,
    /// Agent content has changed
    Modified,
    /// Agent content is identical
    Unchanged,
    /// Agent exists in target but not source
    Removed,
}

/// Diff information for a single agent
#[derive(Debug, Clone)]
pub struct AgentDiff {
    /// Agent name
    pub name: String,
    /// Overall agent status
    pub status: AgentStatus,
    /// Source path (if exists)
    pub source_path: Option<PathBuf>,
    /// Target path
    pub target_path: PathBuf,
    /// Config file changed
    pub config_changed: bool,
    /// Prompt file changed
    pub prompt_changed: bool,
}

/// Complete diff between source and target agents
#[derive(Debug, Clone)]
pub struct AgentsUpdateDiff {
    /// Diffs for each agent
    pub agents: Vec<AgentDiff>,
}

impl AgentsUpdateDiff {
    /// Count of new agents
    pub fn new_agents_count(&self) -> usize {
        self.agents.iter().filter(|a| a.status == AgentStatus::New).count()
    }

    /// Count of modified agents
    pub fn modified_agents_count(&self) -> usize {
        self.agents.iter().filter(|a| a.status == AgentStatus::Modified).count()
    }

    /// Count of unchanged agents
    pub fn unchanged_agents_count(&self) -> usize {
        self.agents.iter().filter(|a| a.status == AgentStatus::Unchanged).count()
    }

    /// Count of removed agents
    pub fn removed_agents_count(&self) -> usize {
        self.agents.iter().filter(|a| a.status == AgentStatus::Removed).count()
    }

    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        self.agents.iter().any(|a| a.status != AgentStatus::Unchanged)
    }

    /// Get agents that need to be installed/updated
    pub fn agents_to_update(&self) -> Vec<&AgentDiff> {
        self.agents
            .iter()
            .filter(|a| matches!(a.status, AgentStatus::New | AgentStatus::Modified))
            .collect()
    }
}

/// Computes diffs between source agents and installed compiled agents
pub struct AgentDiffer<'a, F: FileSystem, C: Checksum> {
    fs: &'a F,
    checksum: &'a C,
}

impl<'a, F: FileSystem, C: Checksum> AgentDiffer<'a, F, C> {
    pub fn new(fs: &'a F, checksum: &'a C) -> Self {
        Self { fs, checksum }
    }

    /// Compute diff between source agents and installed agents
    pub async fn compute_diff(
        &self,
        source_dir: &Path,
        target_dir: &Path,
        platform: Platform,
    ) -> Result<AgentsUpdateDiff> {
        let mut agent_diffs = Vec::new();

        // Get source agents
        let source_agents = self.discover_agent_dirs(source_dir).await?;
        let source_names: HashMap<String, PathBuf> = source_agents
            .into_iter()
            .filter_map(|p| {
                let name = p.file_name().and_then(|n| n.to_str()).map(|n| n.to_string());
                name.map(|n| (n, p))
            })
            .collect();

        // Get target agents
        let target_names: HashMap<String, PathBuf> = if self.fs.exists(target_dir) {
            let target_agents = self.discover_installed_dirs(target_dir).await.unwrap_or_default();
            target_agents
                .into_iter()
                .filter_map(|p| {
                    let name = p.file_name().and_then(|n| n.to_str()).map(|n| n.to_string());
                    name.map(|n| (n, p))
                })
                .collect()
        } else {
            HashMap::new()
        };

        // Process source agents
        for (name, source_path) in &source_names {
            let target_path = target_dir.join(name);

            if let Some(existing_target) = target_names.get(name) {
                // Agent exists in both - compare content
                let (config_changed, prompt_changed) = self
                    .compare_agent_content(source_path, existing_target, platform)
                    .await?;

                let status = if config_changed || prompt_changed {
                    AgentStatus::Modified
                } else {
                    AgentStatus::Unchanged
                };

                agent_diffs.push(AgentDiff {
                    name: name.clone(),
                    status,
                    source_path: Some(source_path.clone()),
                    target_path,
                    config_changed,
                    prompt_changed,
                });
            } else {
                // New agent
                agent_diffs.push(AgentDiff {
                    name: name.clone(),
                    status: AgentStatus::New,
                    source_path: Some(source_path.clone()),
                    target_path,
                    config_changed: true,
                    prompt_changed: true,
                });
            }
        }

        // Process removed agents (exist in target but not source)
        for name in target_names.keys() {
            if !source_names.contains_key(name) {
                agent_diffs.push(AgentDiff {
                    name: name.clone(),
                    status: AgentStatus::Removed,
                    source_path: None,
                    target_path: target_dir.join(name),
                    config_changed: false,
                    prompt_changed: false,
                });
            }
        }

        // Sort by name for consistent output
        agent_diffs.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(AgentsUpdateDiff {
            agents: agent_diffs,
        })
    }

    /// Discover agent directories (directories containing AGENT.md)
    async fn discover_agent_dirs(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        if !self.fs.exists(dir) {
            return Ok(Vec::new());
        }

        let mut agents = Vec::new();
        let entries = self.fs.list_dir(dir).await?;

        for entry in entries {
            let agent_md = entry.join("AGENT.md");
            if self.fs.is_dir(&entry) && self.fs.exists(&agent_md) {
                agents.push(entry);
            }
        }

        Ok(agents)
    }

    /// Discover installed agent directories
    async fn discover_installed_dirs(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        if !self.fs.exists(dir) {
            return Ok(Vec::new());
        }

        let mut agents = Vec::new();
        let entries = self.fs.list_dir(dir).await?;

        for entry in entries {
            if self.fs.is_dir(&entry) {
                agents.push(entry);
            }
        }

        Ok(agents)
    }

    /// Compare agent content between source and target
    async fn compare_agent_content(
        &self,
        source_path: &Path,
        target_path: &Path,
        platform: Platform,
    ) -> Result<(bool, bool)> {
        // Read and parse source agent
        let agent_md_path = source_path.join("AGENT.md");
        let content = self.fs.read(&agent_md_path).await?;
        let parsed = parse_agent_md(&content, agent_md_path.clone())?;

        // Compile to get expected content
        let compiled = compile_agent(&parsed, platform);

        // Compare config file
        let target_config = target_path.join(&compiled.config_filename);
        let config_changed = if self.fs.exists(&target_config) {
            let source_hash = self.checksum.sha256(compiled.config_content.as_bytes());
            let target_hash = self.checksum.sha256_file(&target_config)?;
            source_hash != target_hash
        } else {
            true
        };

        // Compare prompt file
        let target_prompt = target_path.join(&compiled.prompt_filename);
        let prompt_changed = if self.fs.exists(&target_prompt) {
            let source_hash = self.checksum.sha256(compiled.prompt_content.as_bytes());
            let target_hash = self.checksum.sha256_file(&target_prompt)?;
            source_hash != target_hash
        } else {
            true
        };

        Ok((config_changed, prompt_changed))
    }

    /// Compile an agent from source
    pub async fn compile_from_source(
        &self,
        source_path: &Path,
        platform: Platform,
    ) -> Result<CompiledAgent> {
        let agent_md_path = source_path.join("AGENT.md");
        let content = self.fs.read(&agent_md_path).await?;
        let parsed = parse_agent_md(&content, agent_md_path)?;
        Ok(compile_agent(&parsed, platform))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agents_update_diff_counts() {
        let diff = AgentsUpdateDiff {
            agents: vec![
                AgentDiff {
                    name: "new-agent".to_string(),
                    status: AgentStatus::New,
                    source_path: Some(PathBuf::from("/src")),
                    target_path: PathBuf::from("/tgt"),
                    config_changed: true,
                    prompt_changed: true,
                },
                AgentDiff {
                    name: "modified-agent".to_string(),
                    status: AgentStatus::Modified,
                    source_path: Some(PathBuf::from("/src")),
                    target_path: PathBuf::from("/tgt"),
                    config_changed: true,
                    prompt_changed: false,
                },
                AgentDiff {
                    name: "unchanged-agent".to_string(),
                    status: AgentStatus::Unchanged,
                    source_path: Some(PathBuf::from("/src")),
                    target_path: PathBuf::from("/tgt"),
                    config_changed: false,
                    prompt_changed: false,
                },
                AgentDiff {
                    name: "removed-agent".to_string(),
                    status: AgentStatus::Removed,
                    source_path: None,
                    target_path: PathBuf::from("/tgt"),
                    config_changed: false,
                    prompt_changed: false,
                },
            ],
        };

        assert_eq!(diff.new_agents_count(), 1);
        assert_eq!(diff.modified_agents_count(), 1);
        assert_eq!(diff.unchanged_agents_count(), 1);
        assert_eq!(diff.removed_agents_count(), 1);
        assert!(diff.has_changes());
    }

    #[test]
    fn test_agents_update_diff_no_changes() {
        let diff = AgentsUpdateDiff {
            agents: vec![AgentDiff {
                name: "unchanged".to_string(),
                status: AgentStatus::Unchanged,
                source_path: Some(PathBuf::from("/src")),
                target_path: PathBuf::from("/tgt"),
                config_changed: false,
                prompt_changed: false,
            }],
        };

        assert!(!diff.has_changes());
    }

    #[test]
    fn test_agents_to_update() {
        let diff = AgentsUpdateDiff {
            agents: vec![
                AgentDiff {
                    name: "new".to_string(),
                    status: AgentStatus::New,
                    source_path: Some(PathBuf::from("/src")),
                    target_path: PathBuf::from("/tgt"),
                    config_changed: true,
                    prompt_changed: true,
                },
                AgentDiff {
                    name: "modified".to_string(),
                    status: AgentStatus::Modified,
                    source_path: Some(PathBuf::from("/src")),
                    target_path: PathBuf::from("/tgt"),
                    config_changed: true,
                    prompt_changed: false,
                },
                AgentDiff {
                    name: "unchanged".to_string(),
                    status: AgentStatus::Unchanged,
                    source_path: Some(PathBuf::from("/src")),
                    target_path: PathBuf::from("/tgt"),
                    config_changed: false,
                    prompt_changed: false,
                },
            ],
        };

        let to_update = diff.agents_to_update();
        assert_eq!(to_update.len(), 2);
        assert!(to_update.iter().any(|a| a.name == "new"));
        assert!(to_update.iter().any(|a| a.name == "modified"));
    }
}
