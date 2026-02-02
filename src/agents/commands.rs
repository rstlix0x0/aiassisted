//! Agents domain commands

use crate::agents::compiler::{compile_agent, Platform};
use crate::agents::diff::{AgentDiffer, AgentStatus};
use crate::agents::discovery::AgentDiscovery;
use crate::agents::parser::parse_agent_md;
use crate::agents::validator::validate_agent;
use crate::core::infra::{Checksum, FileSystem, Logger};
use crate::core::types::Result;
use std::path::Path;

/// Command to list available agents
pub struct AgentsListCommand;

impl AgentsListCommand {
    pub async fn execute<F: FileSystem, L: Logger>(
        &self,
        fs: &F,
        logger: &L,
        project_path: &Path,
    ) -> Result<()> {
        let discovery = AgentDiscovery::new(fs, project_path);
        let source_dir = discovery.agents_source_dir();

        logger.info(&format!("Agents source: {}", source_dir.display()));
        logger.info("");

        // Discover agents
        match discovery.discover_agents().await {
            Ok(agents) => {
                if agents.is_empty() {
                    logger.warn("No agents found in .aiassisted/agents/");
                    logger.info("Run 'aiassisted install' to install content first");
                } else {
                    logger.info(&format!("Available agents ({}):", agents.len()));
                    logger.info("");

                    for agent_info in &agents {
                        // Read and parse agent to get description
                        if let Ok(content) = fs.read(&agent_info.agent_md_path).await {
                            if let Ok(parsed) =
                                parse_agent_md(&content, agent_info.agent_md_path.clone())
                            {
                                logger.info(&format!("  {} - {}", agent_info.name, parsed.spec.description));
                            } else {
                                logger.info(&format!("  {} (parse error)", agent_info.name));
                            }
                        } else {
                            logger.info(&format!("  {}", agent_info.name));
                        }
                    }
                }
            }
            Err(e) => {
                logger.warn(&format!("Could not list agents: {}", e));
                logger.info("Run 'aiassisted install' to install content first");
            }
        }

        Ok(())
    }
}

/// Command to set up (compile and install) agents for a platform
pub struct AgentsSetupCommand {
    pub platform: Platform,
    pub dry_run: bool,
    pub force: bool,
}

impl AgentsSetupCommand {
    pub async fn execute<F: FileSystem, L: Logger>(
        &self,
        fs: &F,
        logger: &L,
        project_path: &Path,
    ) -> Result<()> {
        let discovery = AgentDiscovery::new(fs, project_path);

        logger.info(&format!("Setting up agents for {}", self.platform));

        let source_dir = discovery.agents_source_dir();
        let skills_dir = discovery.skills_source_dir();
        let target_dir = discovery.agents_target_dir(self.platform);

        logger.info(&format!("Source: {}", source_dir.display()));
        logger.info(&format!("Target: {}", target_dir.display()));

        // Discover agents
        let agents = discovery.discover_agents().await?;

        if agents.is_empty() {
            logger.warn("No agents found in .aiassisted/agents/");
            logger.info("Run 'aiassisted install' to install content first");
            return Ok(());
        }

        logger.info(&format!("Found {} agent(s)", agents.len()));

        // Create target directory if needed
        if !self.dry_run {
            fs.create_dir_all(&target_dir).await?;
        }

        // Process each agent
        let mut compiled_count = 0;
        let mut skipped_count = 0;
        let mut error_count = 0;

        for agent_info in &agents {
            // Read and parse agent
            let content = match fs.read(&agent_info.agent_md_path).await {
                Ok(c) => c,
                Err(e) => {
                    logger.error(&format!("Failed to read {}: {}", agent_info.name, e));
                    error_count += 1;
                    continue;
                }
            };

            let parsed = match parse_agent_md(&content, agent_info.agent_md_path.clone()) {
                Ok(p) => p,
                Err(e) => {
                    logger.error(&format!("Failed to parse {}: {}", agent_info.name, e));
                    error_count += 1;
                    continue;
                }
            };

            // Validate agent
            if let Err(e) = validate_agent(&parsed.spec, &agent_info.agent_md_path, &skills_dir, fs).await {
                logger.error(&format!("Validation failed for {}: {}", agent_info.name, e));
                error_count += 1;
                continue;
            }

            // Compile agent
            let compiled = compile_agent(&parsed, self.platform);

            // Check if already exists
            let agent_target_file = target_dir.join(&compiled.filename);
            if fs.exists(&agent_target_file) && !self.force {
                if self.dry_run {
                    logger.info(&format!("Would skip (exists): {}", agent_info.name));
                } else {
                    logger.warn(&format!("Skipped (exists): {}", agent_info.name));
                }
                skipped_count += 1;
                continue;
            }

            if self.dry_run {
                logger.info(&format!(
                    "Would compile: {} -> {}",
                    agent_info.name,
                    agent_target_file.display()
                ));
                compiled_count += 1;
            } else {
                // Write single markdown file
                fs.write(&agent_target_file, &compiled.content).await?;

                logger.success(&format!("Compiled: {}", agent_info.name));
                compiled_count += 1;
            }
        }

        // Summary
        logger.info("");
        if self.dry_run {
            logger.info(&format!(
                "Dry run: {} agent(s) would be compiled, {} skipped, {} errors",
                compiled_count, skipped_count, error_count
            ));
        } else {
            logger.success(&format!(
                "Setup complete: {} compiled, {} skipped, {} errors",
                compiled_count, skipped_count, error_count
            ));

            if skipped_count > 0 {
                logger.info("Use --force to overwrite existing agents");
            }
        }

        Ok(())
    }
}

/// Command to update installed agents (sync changes from source)
pub struct AgentsUpdateCommand {
    pub platform: Platform,
    pub dry_run: bool,
    pub force: bool,
}

impl AgentsUpdateCommand {
    pub async fn execute<F: FileSystem, C: Checksum, L: Logger>(
        &self,
        fs: &F,
        checksum: &C,
        logger: &L,
        project_path: &Path,
    ) -> Result<()> {
        let discovery = AgentDiscovery::new(fs, project_path);
        let differ = AgentDiffer::new(fs, checksum);

        let source_dir = discovery.agents_source_dir();
        let target_dir = discovery.agents_target_dir(self.platform);

        logger.info(&format!("Updating agents for {}", self.platform));
        logger.info(&format!("Source: {}", source_dir.display()));
        logger.info(&format!("Target: {}", target_dir.display()));

        // Check source exists
        if !fs.exists(&source_dir) {
            logger.warn("No agents found in .aiassisted/agents/");
            logger.info("Run 'aiassisted install' to install content first");
            return Ok(());
        }

        // Check target exists
        if !fs.exists(&target_dir) {
            logger.warn("No agents installed yet");
            logger.info(&format!(
                "Run 'aiassisted agents setup --platform {}' to install agents first",
                self.platform
            ));
            return Ok(());
        }

        logger.info("Analyzing agents...");

        // Compute diff
        let diff = differ.compute_diff(&source_dir, &target_dir, self.platform).await?;

        // Summary
        logger.info(&format!(
            "Summary: {} new, {} modified, {} unchanged, {} removed",
            diff.new_agents_count(),
            diff.modified_agents_count(),
            diff.unchanged_agents_count(),
            diff.removed_agents_count()
        ));
        logger.info("");

        // Show agent status
        logger.info("Agents status:");
        for agent in &diff.agents {
            let indicator = match agent.status {
                AgentStatus::New => "+",
                AgentStatus::Modified => "~",
                AgentStatus::Unchanged => "=",
                AgentStatus::Removed => "-",
            };

            let details = match agent.status {
                AgentStatus::New => "(new)",
                AgentStatus::Modified => "(modified)",
                AgentStatus::Unchanged => "(unchanged)",
                AgentStatus::Removed => "(removed from source)",
            };

            logger.info(&format!("  {} {} {}", indicator, agent.name, details));
        }

        // Check if there are changes
        if !diff.has_changes() {
            logger.success("All agents are up to date!");
            return Ok(());
        }

        // Get agents to update
        let agents_to_update = if self.force {
            // Force mode: update all non-removed agents
            diff.agents
                .iter()
                .filter(|a| a.status != AgentStatus::Removed)
                .collect::<Vec<_>>()
        } else {
            diff.agents_to_update()
        };

        if agents_to_update.is_empty() {
            logger.info("No agents to update");
            return Ok(());
        }

        logger.info("");
        logger.info("Agents to update:");
        for agent in &agents_to_update {
            let indicator = match agent.status {
                AgentStatus::New => "+",
                AgentStatus::Modified => "~",
                _ => " ",
            };
            logger.info(&format!("  {} {}", indicator, agent.name));
        }

        // Perform update
        if self.dry_run {
            logger.info("");
            logger.info(&format!(
                "Dry run: {} agent(s) would be updated",
                agents_to_update.len()
            ));
        } else {
            logger.info("");

            let mut updated = 0;
            for agent in &agents_to_update {
                if let Some(source_path) = &agent.source_path {
                    // Compile and write
                    let compiled = differ.compile_from_source(source_path, self.platform).await?;

                    // Write single markdown file
                    fs.write(&agent.target_path, &compiled.content).await?;

                    updated += 1;
                }
            }

            logger.success(&format!("Updated {} agent(s)", updated));

            // Note about removed agents
            if diff.removed_agents_count() > 0 {
                logger.info(&format!(
                    "Note: {} agent(s) removed from source but still installed",
                    diff.removed_agents_count()
                ));
            }
        }

        Ok(())
    }
}
