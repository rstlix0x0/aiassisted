//! Agent compilation to platform-specific formats

use crate::agents::parser::{Capabilities, ModelTier, ParsedAgent};

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

/// Compiled agent output - a single markdown file with YAML frontmatter
#[derive(Debug, Clone)]
pub struct CompiledAgent {
    /// Agent name
    pub name: String,
    /// Full content (YAML frontmatter + system prompt)
    pub content: String,
    /// Output filename (e.g., "code-reviewer.md")
    pub filename: String,
}


/// Compile an agent to a platform-specific format
pub fn compile_agent(agent: &ParsedAgent, platform: Platform) -> CompiledAgent {
    match platform {
        Platform::ClaudeCode => compile_for_claude_code(agent),
        Platform::OpenCode => compile_for_opencode(agent),
    }
}

/// Compile agent for Claude Code format
/// Output: Single markdown file with YAML frontmatter
/// Format per agent-spec.guideline.md:
/// ```markdown
/// ---
/// name: code-reviewer
/// description: Reviews code for quality and best practices.
/// disallowedTools: Write, Edit
/// model: sonnet
/// skills:
///   - review-codes
/// ---
///
/// [System prompt body]
/// ```
fn compile_for_claude_code(agent: &ParsedAgent) -> CompiledAgent {
    let mut frontmatter_lines = Vec::new();

    // Required fields
    frontmatter_lines.push(format!("name: {}", agent.spec.name));
    frontmatter_lines.push(format!("description: {}", agent.spec.description));

    // Tool restrictions (before model per guideline example)
    match agent.spec.capabilities {
        Capabilities::ReadOnly => {
            frontmatter_lines.push("disallowedTools: Write, Edit".to_string());
        }
        Capabilities::ReadWrite => {} // Default, omit
    }

    // Model mapping per agent-spec.guideline.md
    let model = match agent.spec.model_tier {
        ModelTier::Fast => "haiku",
        ModelTier::Balanced => "sonnet",
        ModelTier::Capable => "opus",
    };
    frontmatter_lines.push(format!("model: {}", model));

    // Skills (if any)
    if !agent.spec.skills.is_empty() {
        frontmatter_lines.push("skills:".to_string());
        for skill in &agent.spec.skills {
            frontmatter_lines.push(format!("  - {}", skill));
        }
    }

    let frontmatter = frontmatter_lines.join("\n");
    let content = format!("---\n{}\n---\n\n{}", frontmatter, agent.system_prompt);

    CompiledAgent {
        name: agent.spec.name.clone(),
        content,
        filename: format!("{}.md", agent.spec.name),
    }
}

/// Compile agent for OpenCode format
/// Output: Single markdown file with YAML frontmatter
/// Format per agent-spec.guideline.md:
/// ```markdown
/// ---
/// description: Reviews code for quality and best practices.
/// mode: subagent
/// model: anthropic/claude-sonnet-4-20250514
/// tools:
///   write: false
///   edit: false
/// ---
///
/// [System prompt body]
/// ```
fn compile_for_opencode(agent: &ParsedAgent) -> CompiledAgent {
    let mut frontmatter_lines = Vec::new();

    // Required fields
    frontmatter_lines.push(format!("description: {}", agent.spec.description));
    frontmatter_lines.push("mode: subagent".to_string());

    // Model (full provider/model-id format per agent-spec.guideline.md)
    let model = match agent.spec.model_tier {
        ModelTier::Fast => "anthropic/claude-haiku-4-20250514",
        ModelTier::Balanced => "anthropic/claude-sonnet-4-20250514",
        ModelTier::Capable => "anthropic/claude-opus-4-20250514",
    };
    frontmatter_lines.push(format!("model: {}", model));

    // Tool restrictions (nested YAML for read-only)
    match agent.spec.capabilities {
        Capabilities::ReadOnly => {
            frontmatter_lines.push("tools:".to_string());
            frontmatter_lines.push("  write: false".to_string());
            frontmatter_lines.push("  edit: false".to_string());
        }
        Capabilities::ReadWrite => {} // Default, omit
    }

    let frontmatter = frontmatter_lines.join("\n");
    let content = format!("---\n{}\n---\n\n{}", frontmatter, agent.system_prompt);

    CompiledAgent {
        name: agent.spec.name.clone(),
        content,
        filename: format!("{}.md", agent.spec.name),
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
        assert_eq!(compiled.filename, "test-agent.md");
        assert!(compiled.content.contains("model: haiku"));
        assert!(compiled.content.contains("disallowedTools: Write, Edit"));
        assert!(compiled.content.contains("You are a test agent."));
    }

    #[test]
    fn test_compile_claude_code_read_write_balanced() {
        let agent = make_test_agent(Capabilities::ReadWrite, ModelTier::Balanced, vec![]);
        let compiled = compile_agent(&agent, Platform::ClaudeCode);

        // Balanced model should output "model: sonnet"
        assert!(compiled.content.contains("model: sonnet"));
        // ReadWrite should not include disallowedTools
        assert!(!compiled.content.contains("disallowedTools"));
    }

    #[test]
    fn test_compile_claude_code_capable() {
        let agent = make_test_agent(Capabilities::ReadWrite, ModelTier::Capable, vec![]);
        let compiled = compile_agent(&agent, Platform::ClaudeCode);

        assert!(compiled.content.contains("model: opus"));
    }

    #[test]
    fn test_compile_claude_code_with_skills() {
        let agent = make_test_agent(
            Capabilities::ReadWrite,
            ModelTier::Balanced,
            vec!["review-codes".to_string(), "doc-code".to_string()],
        );
        let compiled = compile_agent(&agent, Platform::ClaudeCode);

        assert!(compiled.content.contains("skills:"));
        assert!(compiled.content.contains("  - review-codes"));
        assert!(compiled.content.contains("  - doc-code"));
    }

    #[test]
    fn test_compile_opencode_read_only_fast() {
        let agent = make_test_agent(Capabilities::ReadOnly, ModelTier::Fast, vec![]);
        let compiled = compile_agent(&agent, Platform::OpenCode);

        assert_eq!(compiled.name, "test-agent");
        assert_eq!(compiled.filename, "test-agent.md");
        assert!(compiled.content.contains("anthropic/claude-haiku-4"));
        assert!(compiled.content.contains("mode: subagent"));
        assert!(compiled.content.contains("tools:"));
        assert!(compiled.content.contains("  write: false"));
        assert!(compiled.content.contains("  edit: false"));
    }

    #[test]
    fn test_compile_opencode_read_write_balanced() {
        let agent = make_test_agent(Capabilities::ReadWrite, ModelTier::Balanced, vec![]);
        let compiled = compile_agent(&agent, Platform::OpenCode);

        assert!(compiled.content.contains("anthropic/claude-sonnet-4"));
        // ReadWrite should not include tools section
        assert!(!compiled.content.contains("tools:"));
    }

    #[test]
    fn test_compile_opencode_capable() {
        let agent = make_test_agent(Capabilities::ReadWrite, ModelTier::Capable, vec![]);
        let compiled = compile_agent(&agent, Platform::OpenCode);

        assert!(compiled.content.contains("anthropic/claude-opus-4"));
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
        assert!(!compiled.content.contains("skills"));
        assert!(!compiled.content.contains("some-skill"));
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

        assert!(claude_compiled.content.contains("Custom system prompt\n\nWith multiple lines."));
        assert!(opencode_compiled.content.contains("Custom system prompt\n\nWith multiple lines."));
    }

    #[test]
    fn test_platform_display() {
        assert_eq!(Platform::ClaudeCode.to_string(), "claude-code");
        assert_eq!(Platform::OpenCode.to_string(), "opencode");
    }

    #[test]
    fn test_claude_code_yaml_frontmatter_format() {
        let agent = make_test_agent(Capabilities::ReadOnly, ModelTier::Fast, vec!["skill1".to_string()]);
        let compiled = compile_agent(&agent, Platform::ClaudeCode);

        // Check YAML frontmatter structure
        assert!(compiled.content.starts_with("---\n"));
        assert!(compiled.content.contains("\n---\n\n"));
        assert!(compiled.content.contains("name: test-agent"));
        assert!(compiled.content.contains("description: A test agent"));
    }

    #[test]
    fn test_opencode_yaml_frontmatter_format() {
        let agent = make_test_agent(Capabilities::ReadOnly, ModelTier::Fast, vec![]);
        let compiled = compile_agent(&agent, Platform::OpenCode);

        // Check YAML frontmatter structure
        assert!(compiled.content.starts_with("---\n"));
        assert!(compiled.content.contains("\n---\n\n"));
        assert!(compiled.content.contains("description: A test agent"));
        assert!(compiled.content.contains("mode: subagent"));
    }
}
