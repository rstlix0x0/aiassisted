//! Core types shared across all domains.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type alias for the application.
pub type Result<T> = std::result::Result<T, Error>;

/// Common error type for the application.
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Template error: {0}")]
    Template(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Supported AI tools for template generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    /// Auto-detect the tool based on project structure.
    #[default]
    Auto,
    /// OpenCode AI tool.
    OpenCode,
    /// Claude Code AI tool.
    Claude,
}

impl std::fmt::Display for ToolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolType::Auto => write!(f, "auto"),
            ToolType::OpenCode => write!(f, "opencode"),
            ToolType::Claude => write!(f, "claude"),
        }
    }
}

impl std::str::FromStr for ToolType {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(ToolType::Auto),
            "opencode" => Ok(ToolType::OpenCode),
            "claude" => Ok(ToolType::Claude),
            _ => Err(Error::Parse(format!("Unknown tool type: {}", s))),
        }
    }
}

/// A manifest entry representing a file with its checksum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    /// Relative path to the file.
    pub path: PathBuf,
    /// SHA256 checksum of the file content.
    pub checksum: String,
}

/// Application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Default AI tool to use.
    #[serde(default)]
    pub default_tool: ToolType,
    /// Verbosity level (0-3).
    #[serde(default = "default_verbosity")]
    pub verbosity: u8,
    /// Whether to auto-update on check.
    #[serde(default = "default_true")]
    pub auto_update: bool,
    /// Whether to prefer project templates over global.
    #[serde(default = "default_true")]
    pub prefer_project: bool,
}

fn default_verbosity() -> u8 {
    1
}

fn default_true() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_tool: ToolType::Auto,
            verbosity: 1,
            auto_update: true,
            prefer_project: true,
        }
    }
}

/// Release information for self-update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    /// Version string (e.g., "1.0.0").
    pub version: String,
    /// Download URL for the binary.
    pub download_url: String,
    /// Optional checksum for verification.
    pub checksum: Option<String>,
}

