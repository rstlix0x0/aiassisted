//! Agent compilation to platform-specific formats

use crate::agents::parser::{Capabilities, ModelTier, ParsedAgent};
use serde::Serialize;

/// Target platform for agent compilation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// Claude Code format
    ClaudeCode,
    /// OpenCode format
    OpenCode,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::ClaudeCode => write!(f, "claude-code"),
            Platform::OpenCode => write!(f, "opencode"),
        }
    }
}

/// Compiled agent output
#[derive(Debug, Clone)]
pub struct CompiledAgent {
    /// Agent name
    pub name: String,
    /// Compiled TOML/JSON content
    pub config_content: String,
    /// System prompt markdown content
    pub prompt_content: String,
    /// Config file name (e.g., "agent.toml" or "agent.json")
    pub config_filename: String,
    /// Prompt file name
    pub prompt_filename: String,
}

/// Claude Code agent configuration structure
#[derive(Debug, Clone, Serialize)]
struct ClaudeCodeConfig {
    name: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(rename = "disallowedTools", skip_serializing_if = "Vec::is_empty")]
    disallowed_tools: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    skills: Vec<String>,
}

/// OpenCode agent configuration structure
#[derive(Debug, Clone, Serialize)]
struct OpenCodeConfig {
    name: String,
    description: String,
    model: String,
    mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<OpenCodeTools>,
}

/// OpenCode tools configuration
#[derive(Debug, Clone, Serialize)]
struct OpenCodeTools {
    write: bool,
    edit: bool,
}

/// Compile an agent to a platform-specific format
pub fn compile_agent(agent: &ParsedAgent, platform: Platform) -> CompiledAgent {
    match platform {
        Platform::ClaudeCode => compile_for_claude_code(agent),
        Platform::OpenCode => compile_for_opencode(agent),
    }
}

fn compile_for_claude_code(agent: &ParsedAgent) -> CompiledAgent {
    let model = match agent.spec.model_tier {
        ModelTier::Fast => Some("haiku".to_string()),
        ModelTier::Balanced => None, // Default, don't include
        ModelTier::Capable => Some("opus".to_string()),
    };

    let disallowed_tools = match agent.spec.capabilities {
        Capabilities::ReadOnly => vec!["Write".to_string(), "Edit".to_string()],
        Capabilities::ReadWrite => vec![],
    };

    let config = ClaudeCodeConfig {
        name: agent.spec.name.clone(),
        description: agent.spec.description.clone(),
        model,
        disallowed_tools,
        skills: agent.spec.skills.clone(),
    };

    let config_content =
        toml::to_string_pretty(&config).unwrap_or_else(|_| "# Error generating config".to_string());

    CompiledAgent {
        name: agent.spec.name.clone(),
        config_content,
        prompt_content: agent.system_prompt.clone(),
        config_filename: "agent.toml".to_string(),
        prompt_filename: "prompt.md".to_string(),
    }
}

fn compile_for_opencode(agent: &ParsedAgent) -> CompiledAgent {
    let model = match agent.spec.model_tier {
        ModelTier::Fast => "anthropic/claude-3-5-haiku-20241022".to_string(),
        ModelTier::Balanced => "anthropic/claude-sonnet-4-20250514".to_string(),
        ModelTier::Capable => "anthropic/claude-opus-4-20250514".to_string(),
    };

    let tools = match agent.spec.capabilities {
        Capabilities::ReadOnly => Some(OpenCodeTools {
            write: false,
            edit: false,
        }),
        Capabilities::ReadWrite => None, // Default, don't include
    };

    let config = OpenCodeConfig {
        name: agent.spec.name.clone(),
        description: agent.spec.description.clone(),
        model,
        mode: "subagent".to_string(),
        tools,
    };

    let config_content =
        serde_json::to_string_pretty(&config).unwrap_or_else(|_| "{}".to_string());

    CompiledAgent {
        name: agent.spec.name.clone(),
        config_content,
        prompt_content: agent.system_prompt.clone(),
        config_filename: "agent.json".to_string(),
        prompt_filename: "prompt.md".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::parser::{AgentSpec, ParsedAgent};
    use std::path::PathBuf;

    fn make_test_agent(capabilities: Capabilities, model_tier: ModelTier, skills: Vec<String>) -> ParsedAgent {
        ParsedAgent {
            spec: AgentSpec {
                name: "test-agent".to_string(),
                description: "A test agent".to_string(),
                capabilities,
                model_tier,
                skills,
            },
            system_prompt: "You are a test agent.".to_string(),
            source_path: PathBuf::from("/test/AGENT.md"),
        }
    }

    #[test]
    fn test_compile_claude_code_read_only_fast() {
        let agent = make_test_agent(Capabilities::ReadOnly, ModelTier::Fast, vec![]);
        let compiled = compile_agent(&agent, Platform::ClaudeCode);

        assert_eq!(compiled.name, "test-agent");
        assert_eq!(compiled.config_filename, "agent.toml");
        assert!(compiled.config_content.contains("model = \"haiku\""));
        assert!(compiled.config_content.contains("disallowedTools"));
        assert!(compiled.config_content.contains("Write"));
        assert!(compiled.config_content.contains("Edit"));
    }

    #[test]
    fn test_compile_claude_code_read_write_balanced() {
        let agent = make_test_agent(Capabilities::ReadWrite, ModelTier::Balanced, vec![]);
        let compiled = compile_agent(&agent, Platform::ClaudeCode);

        // Balanced model should not include model field (it's the default)
        assert!(!compiled.config_content.contains("model ="));
        // ReadWrite should not include disallowedTools
        assert!(!compiled.config_content.contains("disallowedTools"));
    }

    #[test]
    fn test_compile_claude_code_capable() {
        let agent = make_test_agent(Capabilities::ReadWrite, ModelTier::Capable, vec![]);
        let compiled = compile_agent(&agent, Platform::ClaudeCode);

        assert!(compiled.config_content.contains("model = \"opus\""));
    }

    #[test]
    fn test_compile_claude_code_with_skills() {
        let agent = make_test_agent(
            Capabilities::ReadWrite,
            ModelTier::Balanced,
            vec!["review-codes".to_string(), "doc-code".to_string()],
        );
        let compiled = compile_agent(&agent, Platform::ClaudeCode);

        assert!(compiled.config_content.contains("skills"));
        assert!(compiled.config_content.contains("review-codes"));
        assert!(compiled.config_content.contains("doc-code"));
    }

    #[test]
    fn test_compile_opencode_read_only_fast() {
        let agent = make_test_agent(Capabilities::ReadOnly, ModelTier::Fast, vec![]);
        let compiled = compile_agent(&agent, Platform::OpenCode);

        assert_eq!(compiled.name, "test-agent");
        assert_eq!(compiled.config_filename, "agent.json");
        assert!(compiled.config_content.contains("claude-3-5-haiku"));
        assert!(compiled.config_content.contains("\"mode\": \"subagent\""));
        assert!(compiled.config_content.contains("\"write\": false"));
        assert!(compiled.config_content.contains("\"edit\": false"));
    }

    #[test]
    fn test_compile_opencode_read_write_balanced() {
        let agent = make_test_agent(Capabilities::ReadWrite, ModelTier::Balanced, vec![]);
        let compiled = compile_agent(&agent, Platform::OpenCode);

        assert!(compiled.config_content.contains("claude-sonnet-4"));
        // ReadWrite should not include tools section
        assert!(!compiled.config_content.contains("\"tools\""));
    }

    #[test]
    fn test_compile_opencode_capable() {
        let agent = make_test_agent(Capabilities::ReadWrite, ModelTier::Capable, vec![]);
        let compiled = compile_agent(&agent, Platform::OpenCode);

        assert!(compiled.config_content.contains("claude-opus-4"));
    }

    #[test]
    fn test_compile_opencode_ignores_skills() {
        // OpenCode doesn't support skills field
        let agent = make_test_agent(
            Capabilities::ReadWrite,
            ModelTier::Balanced,
            vec!["some-skill".to_string()],
        );
        let compiled = compile_agent(&agent, Platform::OpenCode);

        // Skills should not appear in OpenCode output
        assert!(!compiled.config_content.contains("skills"));
        assert!(!compiled.config_content.contains("some-skill"));
    }

    #[test]
    fn test_prompt_content_preserved() {
        let agent = ParsedAgent {
            spec: AgentSpec {
                name: "test".to_string(),
                description: "test".to_string(),
                capabilities: Capabilities::ReadWrite,
                model_tier: ModelTier::Balanced,
                skills: vec![],
            },
            system_prompt: "Custom system prompt\n\nWith multiple lines.".to_string(),
            source_path: PathBuf::from("/test/AGENT.md"),
        };

        let claude_compiled = compile_agent(&agent, Platform::ClaudeCode);
        let opencode_compiled = compile_agent(&agent, Platform::OpenCode);

        assert_eq!(claude_compiled.prompt_content, agent.system_prompt);
        assert_eq!(opencode_compiled.prompt_content, agent.system_prompt);
    }

    #[test]
    fn test_platform_display() {
        assert_eq!(Platform::ClaudeCode.to_string(), "claude-code");
        assert_eq!(Platform::OpenCode.to_string(), "opencode");
    }
}
