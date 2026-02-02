//! AGENT.md parsing - YAML frontmatter and markdown body extraction

use crate::core::types::{Error, Result};
use serde::Deserialize;
use std::path::PathBuf;

/// Agent capabilities - determines which tools are available
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Capabilities {
    /// Agent can only read, not modify
    ReadOnly,
    /// Agent can read and write (default)
    #[default]
    ReadWrite,
}

impl<'de> Deserialize<'de> for Capabilities {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "read-only" | "readonly" => Ok(Capabilities::ReadOnly),
            "read-write" | "readwrite" => Ok(Capabilities::ReadWrite),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid capabilities: {}. Expected 'read-only' or 'read-write'",
                s
            ))),
        }
    }
}

/// Model tier - determines which model to use
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModelTier {
    /// Fast, cheaper model (e.g., Haiku)
    Fast,
    /// Balanced model (default, e.g., Sonnet)
    #[default]
    Balanced,
    /// Most capable model (e.g., Opus)
    Capable,
}

impl<'de> Deserialize<'de> for ModelTier {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "fast" => Ok(ModelTier::Fast),
            "balanced" => Ok(ModelTier::Balanced),
            "capable" => Ok(ModelTier::Capable),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid model-tier: {}. Expected 'fast', 'balanced', or 'capable'",
                s
            ))),
        }
    }
}

/// Raw YAML frontmatter structure
#[derive(Debug, Clone, Deserialize)]
struct RawFrontmatter {
    name: String,
    description: String,
    #[serde(default)]
    capabilities: Option<Capabilities>,
    #[serde(rename = "model-tier", default)]
    model_tier: Option<ModelTier>,
    #[serde(default)]
    skills: Option<Vec<String>>,
}

/// Parsed agent specification from YAML frontmatter
#[derive(Debug, Clone)]
pub struct AgentSpec {
    /// Agent name (must match directory name)
    pub name: String,
    /// Short description of the agent
    pub description: String,
    /// Agent capabilities (read-only or read-write)
    pub capabilities: Capabilities,
    /// Model tier to use
    pub model_tier: ModelTier,
    /// Skills the agent can use
    pub skills: Vec<String>,
}

/// Complete parsed agent with spec and system prompt
#[derive(Debug, Clone)]
pub struct ParsedAgent {
    /// Agent specification from frontmatter
    pub spec: AgentSpec,
    /// System prompt from markdown body
    pub system_prompt: String,
    /// Source path of the AGENT.md file
    pub source_path: PathBuf,
}

/// Parse AGENT.md content into structured data
pub fn parse_agent_md(content: &str, source_path: PathBuf) -> Result<ParsedAgent> {
    // Split content by --- delimiters
    let parts: Vec<&str> = content.splitn(3, "---").collect();

    if parts.len() < 3 {
        return Err(Error::Parse(
            "AGENT.md must have YAML frontmatter delimited by ---".to_string(),
        ));
    }

    // parts[0] is empty (before first ---)
    // parts[1] is the YAML frontmatter
    // parts[2] is the markdown body

    let yaml_content = parts[1].trim();
    let markdown_body = parts[2].trim();

    // Parse YAML frontmatter
    let raw: RawFrontmatter =
        serde_yaml::from_str(yaml_content).map_err(|e| Error::Parse(format!("YAML parse error: {}", e)))?;

    let spec = AgentSpec {
        name: raw.name,
        description: raw.description,
        capabilities: raw.capabilities.unwrap_or_default(),
        model_tier: raw.model_tier.unwrap_or_default(),
        skills: raw.skills.unwrap_or_default(),
    };

    Ok(ParsedAgent {
        spec,
        system_prompt: markdown_body.to_string(),
        source_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_agent_md() {
        let content = r#"---
name: test-agent
description: A test agent for testing
capabilities: read-only
model-tier: fast
skills:
  - review-codes
  - doc-code
---

You are a test agent.

## Instructions

Do testing things.
"#;

        let result = parse_agent_md(content, PathBuf::from("/test/AGENT.md")).unwrap();

        assert_eq!(result.spec.name, "test-agent");
        assert_eq!(result.spec.description, "A test agent for testing");
        assert_eq!(result.spec.capabilities, Capabilities::ReadOnly);
        assert_eq!(result.spec.model_tier, ModelTier::Fast);
        assert_eq!(result.spec.skills, vec!["review-codes", "doc-code"]);
        assert!(result.system_prompt.contains("You are a test agent"));
    }

    #[test]
    fn test_parse_with_defaults() {
        let content = r#"---
name: minimal-agent
description: Minimal agent
---

System prompt here.
"#;

        let result = parse_agent_md(content, PathBuf::from("/test/AGENT.md")).unwrap();

        assert_eq!(result.spec.name, "minimal-agent");
        assert_eq!(result.spec.capabilities, Capabilities::ReadWrite);
        assert_eq!(result.spec.model_tier, ModelTier::Balanced);
        assert!(result.spec.skills.is_empty());
    }

    #[test]
    fn test_parse_missing_frontmatter() {
        let content = "Just markdown without frontmatter";

        let result = parse_agent_md(content, PathBuf::from("/test/AGENT.md"));

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let content = r#"---
name: [invalid yaml
description: test
---

Body
"#;

        let result = parse_agent_md(content, PathBuf::from("/test/AGENT.md"));

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_capabilities() {
        let content = r#"---
name: test
description: test
capabilities: invalid
---

Body
"#;

        let result = parse_agent_md(content, PathBuf::from("/test/AGENT.md"));

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_model_tier() {
        let content = r#"---
name: test
description: test
model-tier: ultra
---

Body
"#;

        let result = parse_agent_md(content, PathBuf::from("/test/AGENT.md"));

        assert!(result.is_err());
    }

    #[test]
    fn test_capabilities_variants() {
        // Test read-only
        let content = r#"---
name: test
description: test
capabilities: read-only
---
Body
"#;
        let result = parse_agent_md(content, PathBuf::from("/test/AGENT.md")).unwrap();
        assert_eq!(result.spec.capabilities, Capabilities::ReadOnly);

        // Test readonly (alternative)
        let content = r#"---
name: test
description: test
capabilities: readonly
---
Body
"#;
        let result = parse_agent_md(content, PathBuf::from("/test/AGENT.md")).unwrap();
        assert_eq!(result.spec.capabilities, Capabilities::ReadOnly);

        // Test read-write
        let content = r#"---
name: test
description: test
capabilities: read-write
---
Body
"#;
        let result = parse_agent_md(content, PathBuf::from("/test/AGENT.md")).unwrap();
        assert_eq!(result.spec.capabilities, Capabilities::ReadWrite);
    }

    #[test]
    fn test_model_tier_variants() {
        for (tier_str, expected) in [
            ("fast", ModelTier::Fast),
            ("balanced", ModelTier::Balanced),
            ("capable", ModelTier::Capable),
        ] {
            let content = format!(
                r#"---
name: test
description: test
model-tier: {}
---
Body
"#,
                tier_str
            );
            let result = parse_agent_md(&content, PathBuf::from("/test/AGENT.md")).unwrap();
            assert_eq!(result.spec.model_tier, expected);
        }
    }
}
