//! Templates domain trait abstractions.
//!
//! These traits define the interfaces for template processing operations
//! like rendering templates and resolving template paths.

use std::collections::HashMap;
use std::path::PathBuf;

use super::types::{Result, ToolType};

/// Abstraction for template rendering.
#[allow(dead_code)] // Used in Phase 4 (templates domain)
pub trait TemplateEngine: Send + Sync {
    /// Render a template string with the given variables.
    ///
    /// Supported variables:
    /// - `{{PROJECT_ROOT}}` - The project root path
    /// - `{{RUST_GUIDELINES_LIST}}` - List of Rust guidelines
    /// - `{{ARCH_GUIDELINES_LIST}}` - List of architecture guidelines
    fn render(&self, template: &str, vars: &HashMap<String, String>) -> Result<String>;
}

/// Abstraction for resolving template paths.
///
/// Templates are resolved with cascading priority:
/// 1. Project-specific templates (`./.aiassisted/templates/`)
/// 2. Global templates (`~/.aiassisted/templates/`)
#[allow(dead_code)] // Used in Phase 4 (templates domain)
pub trait TemplateResolver: Send + Sync {
    /// Resolve a template by name for the given tool.
    ///
    /// Returns the path to the template file if found.
    fn resolve(&self, name: &str, tool: ToolType) -> Result<PathBuf>;

    /// List all available templates for the given tool.
    fn list_templates(&self, tool: ToolType) -> Result<Vec<PathBuf>>;

    /// Get the project templates directory.
    fn project_templates_dir(&self) -> Option<PathBuf>;

    /// Get the global templates directory.
    fn global_templates_dir(&self) -> PathBuf;
}
