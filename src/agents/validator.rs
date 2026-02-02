//! Agent validation rules

use crate::agents::parser::AgentSpec;
use crate::core::infra::FileSystem;
use crate::core::types::{Error, Result};
use std::path::Path;

/// Maximum allowed name length
const MAX_NAME_LENGTH: usize = 64;
/// Maximum allowed description length
const MAX_DESCRIPTION_LENGTH: usize = 1024;

/// Validation error details
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Validation result with all errors
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self { errors: vec![] }
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn add_error(&mut self, field: &str, message: &str) {
        self.errors.push(ValidationError {
            field: field.to_string(),
            message: message.to_string(),
        });
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate agent name format
/// - 1-64 characters
/// - Lowercase alphanumeric and hyphens only
/// - No leading/trailing/consecutive hyphens
pub fn validate_name(name: &str) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Check length
    if name.is_empty() {
        result.add_error("name", "Name cannot be empty");
        return result;
    }

    if name.len() > MAX_NAME_LENGTH {
        result.add_error(
            "name",
            &format!("Name exceeds maximum length of {} characters", MAX_NAME_LENGTH),
        );
    }

    // Check characters
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        result.add_error(
            "name",
            "Name must contain only lowercase letters, digits, and hyphens",
        );
    }

    // Check leading hyphen
    if name.starts_with('-') {
        result.add_error("name", "Name cannot start with a hyphen");
    }

    // Check trailing hyphen
    if name.ends_with('-') {
        result.add_error("name", "Name cannot end with a hyphen");
    }

    // Check consecutive hyphens
    if name.contains("--") {
        result.add_error("name", "Name cannot contain consecutive hyphens");
    }

    result
}

/// Validate that the agent name matches the directory name
pub fn validate_name_matches_directory(name: &str, source_path: &Path) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Get the directory name (parent of AGENT.md)
    if let Some(dir) = source_path.parent()
        && let Some(dir_name) = dir.file_name().and_then(|n| n.to_str())
        && name != dir_name
    {
        result.add_error(
            "name",
            &format!(
                "Agent name '{}' does not match directory name '{}'",
                name, dir_name
            ),
        );
    }

    result
}

/// Validate agent description
pub fn validate_description(description: &str) -> ValidationResult {
    let mut result = ValidationResult::new();

    if description.is_empty() {
        result.add_error("description", "Description cannot be empty");
    }

    if description.len() > MAX_DESCRIPTION_LENGTH {
        result.add_error(
            "description",
            &format!(
                "Description exceeds maximum length of {} characters",
                MAX_DESCRIPTION_LENGTH
            ),
        );
    }

    result
}

/// Validate that referenced skills exist
pub async fn validate_skills<F: FileSystem>(
    skills: &[String],
    skills_dir: &Path,
    fs: &F,
) -> ValidationResult {
    let mut result = ValidationResult::new();

    for skill in skills {
        let skill_path = skills_dir.join(skill).join("SKILL.md");
        if !fs.exists(&skill_path) {
            result.add_error(
                "skills",
                &format!("Referenced skill '{}' not found at {}", skill, skill_path.display()),
            );
        }
    }

    result
}

/// Validate a complete agent specification
pub async fn validate_agent<F: FileSystem>(
    spec: &AgentSpec,
    source_path: &Path,
    skills_dir: &Path,
    fs: &F,
) -> Result<()> {
    let mut all_errors = Vec::new();

    // Validate name
    let name_result = validate_name(&spec.name);
    all_errors.extend(name_result.errors);

    // Validate name matches directory
    let dir_result = validate_name_matches_directory(&spec.name, source_path);
    all_errors.extend(dir_result.errors);

    // Validate description
    let desc_result = validate_description(&spec.description);
    all_errors.extend(desc_result.errors);

    // Validate skills
    let skills_result = validate_skills(&spec.skills, skills_dir, fs).await;
    all_errors.extend(skills_result.errors);

    if all_errors.is_empty() {
        Ok(())
    } else {
        let error_messages: Vec<String> = all_errors.iter().map(|e| e.to_string()).collect();
        Err(Error::Parse(format!(
            "Agent validation failed:\n  - {}",
            error_messages.join("\n  - ")
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name_valid() {
        let result = validate_name("my-agent");
        assert!(result.is_valid());

        let result = validate_name("agent123");
        assert!(result.is_valid());

        let result = validate_name("a");
        assert!(result.is_valid());

        let result = validate_name("my-test-agent-v2");
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_name_empty() {
        let result = validate_name("");
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.message.contains("empty")));
    }

    #[test]
    fn test_validate_name_too_long() {
        let long_name = "a".repeat(65);
        let result = validate_name(&long_name);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.message.contains("maximum length")));
    }

    #[test]
    fn test_validate_name_invalid_characters() {
        let result = validate_name("MyAgent");
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.message.contains("lowercase")));

        let result = validate_name("my_agent");
        assert!(!result.is_valid());

        let result = validate_name("my agent");
        assert!(!result.is_valid());
    }

    #[test]
    fn test_validate_name_leading_hyphen() {
        let result = validate_name("-agent");
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.message.contains("start with")));
    }

    #[test]
    fn test_validate_name_trailing_hyphen() {
        let result = validate_name("agent-");
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.message.contains("end with")));
    }

    #[test]
    fn test_validate_name_consecutive_hyphens() {
        let result = validate_name("my--agent");
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.message.contains("consecutive")));
    }

    #[test]
    fn test_validate_name_matches_directory() {
        let path = Path::new("/project/.aiassisted/agents/my-agent/AGENT.md");
        let result = validate_name_matches_directory("my-agent", path);
        assert!(result.is_valid());

        let result = validate_name_matches_directory("different-name", path);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.message.contains("does not match")));
    }

    #[test]
    fn test_validate_description_valid() {
        let result = validate_description("A helpful agent for code review.");
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_description_empty() {
        let result = validate_description("");
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.message.contains("empty")));
    }

    #[test]
    fn test_validate_description_too_long() {
        let long_desc = "a".repeat(1025);
        let result = validate_description(&long_desc);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.message.contains("maximum length")));
    }

    #[test]
    fn test_validation_result_default() {
        let result = ValidationResult::default();
        assert!(result.is_valid());
    }
}
