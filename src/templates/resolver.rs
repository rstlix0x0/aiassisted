//! Template resolution with cascading priority.

use std::path::{Path, PathBuf};

use crate::core::templates::TemplateResolver;
use crate::core::types::{Error, Result, ToolType};

/// Resolves templates with cascading priority:
/// 1. Project-specific templates (`./.aiassisted/templates/`)
/// 2. Global templates (`~/.aiassisted/templates/`)
pub struct CascadingResolver {
    project_root: PathBuf,
    global_root: PathBuf,
}

impl CascadingResolver {
    /// Create a new resolver with project and global roots.
    pub fn new(project_root: PathBuf, global_root: PathBuf) -> Self {
        Self {
            project_root,
            global_root,
        }
    }

    /// Get the template subdirectory for a given category and tool.
    fn template_subdir(&self, category: &str, tool: ToolType) -> String {
        let tool_name = match tool {
            ToolType::OpenCode => "opencode",
            ToolType::Claude => "claude",
            ToolType::Auto => "claude", // Default to Claude
        };
        format!("{}/{}", category, tool_name)
    }

    /// Try to resolve a template in a specific base directory.
    fn try_resolve(
        &self,
        base_dir: &Path,
        category: &str,
        name: &str,
        tool: ToolType,
    ) -> Option<PathBuf> {
        let subdir = self.template_subdir(category, tool);
        let template_path = base_dir
            .join(".aiassisted")
            .join("templates")
            .join(subdir)
            .join(format!("{}.template", name));

        if template_path.exists() && template_path.is_file() {
            Some(template_path)
        } else {
            None
        }
    }

    /// List templates in a specific directory (sync wrapper).
    fn list_in_dir(
        &self,
        base_dir: &Path,
        category: &str,
        tool: ToolType,
    ) -> Vec<PathBuf> {
        let subdir = self.template_subdir(category, tool);
        let dir_path = base_dir.join(".aiassisted").join("templates").join(subdir);

        // Use sync filesystem for directory listing
        if !dir_path.exists() || !dir_path.is_dir() {
            return Vec::new();
        }

        match std::fs::read_dir(&dir_path) {
            Ok(entries) => entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    p.extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext == "template")
                        .unwrap_or(false)
                })
                .collect(),
            Err(_) => Vec::new(),
        }
    }
}

impl TemplateResolver for CascadingResolver {
    fn resolve(&self, name: &str, tool: ToolType) -> Result<PathBuf> {
        // Determine category from name suffix
        let (category, clean_name) = if name.ends_with(".SKILL.md") {
            ("skills", name)
        } else if name.ends_with(".md") && !name.ends_with(".SKILL.md") {
            ("agents", name)
        } else {
            // Try both categories if no clear suffix
            ("skills", name)
        };

        // Try project templates first
        if let Some(path) =
            self.try_resolve(&self.project_root, category, clean_name, tool)
        {
            return Ok(path);
        }

        // Try global templates
        if let Some(path) =
            self.try_resolve(&self.global_root, category, clean_name, tool)
        {
            return Ok(path);
        }

        // If first category didn't work and name doesn't have clear suffix, try agents
        if category == "skills" && !name.ends_with(".SKILL.md") {
            if let Some(path) =
                self.try_resolve(&self.project_root, "agents", clean_name, tool)
            {
                return Ok(path);
            }

            if let Some(path) =
                self.try_resolve(&self.global_root, "agents", clean_name, tool)
            {
                return Ok(path);
            }
        }

        Err(Error::NotFound(format!(
            "Template '{}' not found for tool '{}'",
            name, tool
        )))
    }

    fn list_templates(&self, tool: ToolType) -> Result<Vec<PathBuf>> {
        let mut templates = Vec::new();

        // List skills
        templates.extend(self.list_in_dir(&self.project_root, "skills", tool));
        templates.extend(self.list_in_dir(&self.global_root, "skills", tool));

        // List agents
        templates.extend(self.list_in_dir(&self.project_root, "agents", tool));
        templates.extend(self.list_in_dir(&self.global_root, "agents", tool));

        // Remove duplicates (prefer project over global)
        templates.sort();
        templates.dedup_by(|a, b| {
            a.file_name()
                .and_then(|n| n.to_str())
                == b.file_name().and_then(|n| n.to_str())
        });

        Ok(templates)
    }

    fn project_templates_dir(&self) -> Option<PathBuf> {
        let dir = self.project_root.join(".aiassisted").join("templates");
        if dir.exists() && dir.is_dir() {
            Some(dir)
        } else {
            None
        }
    }

    fn global_templates_dir(&self) -> PathBuf {
        self.global_root.join(".aiassisted").join("templates")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_template_file(base: &Path, category: &str, tool: &str, name: &str) {
        let template_dir = base
            .join(".aiassisted")
            .join("templates")
            .join(category)
            .join(tool);
        fs::create_dir_all(&template_dir).unwrap();
        let template_path = template_dir.join(format!("{}.template", name));
        fs::write(template_path, "test content").unwrap();
    }

    #[test]
    fn test_resolve_skill_from_project() {
        let project_dir = TempDir::new().unwrap();
        let global_dir = TempDir::new().unwrap();

        create_template_file(project_dir.path(), "skills", "claude", "test.SKILL.md");

        let resolver = CascadingResolver::new(
            project_dir.path().to_path_buf(),
            global_dir.path().to_path_buf(),
        );

        let result = resolver.resolve("test.SKILL.md", ToolType::Claude);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.starts_with(project_dir.path()));
    }

    #[test]
    fn test_resolve_skill_from_global() {
        let project_dir = TempDir::new().unwrap();
        let global_dir = TempDir::new().unwrap();

        create_template_file(global_dir.path(), "skills", "claude", "global.SKILL.md");

        let resolver = CascadingResolver::new(
            project_dir.path().to_path_buf(),
            global_dir.path().to_path_buf(),
        );

        let result = resolver.resolve("global.SKILL.md", ToolType::Claude);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.starts_with(global_dir.path()));
    }

    #[test]
    fn test_resolve_project_overrides_global() {
        let project_dir = TempDir::new().unwrap();
        let global_dir = TempDir::new().unwrap();

        create_template_file(project_dir.path(), "skills", "claude", "override.SKILL.md");
        create_template_file(global_dir.path(), "skills", "claude", "override.SKILL.md");

        let resolver = CascadingResolver::new(
            project_dir.path().to_path_buf(),
            global_dir.path().to_path_buf(),
        );

        let result = resolver.resolve("override.SKILL.md", ToolType::Claude);
        assert!(result.is_ok());
        let path = result.unwrap();
        // Should prefer project over global
        assert!(path.starts_with(project_dir.path()));
    }

    #[test]
    fn test_resolve_agent_from_project() {
        let project_dir = TempDir::new().unwrap();
        let global_dir = TempDir::new().unwrap();

        create_template_file(project_dir.path(), "agents", "claude", "agent.md");

        let resolver = CascadingResolver::new(
            project_dir.path().to_path_buf(),
            global_dir.path().to_path_buf(),
        );

        let result = resolver.resolve("agent.md", ToolType::Claude);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.starts_with(project_dir.path()));
    }

    #[test]
    fn test_resolve_not_found() {
        let project_dir = TempDir::new().unwrap();
        let global_dir = TempDir::new().unwrap();

        let resolver = CascadingResolver::new(
            project_dir.path().to_path_buf(),
            global_dir.path().to_path_buf(),
        );

        let result = resolver.resolve("nonexistent.SKILL.md", ToolType::Claude);
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::NotFound(_))));
    }

    #[test]
    fn test_resolve_opencode_tool() {
        let project_dir = TempDir::new().unwrap();
        let global_dir = TempDir::new().unwrap();

        create_template_file(project_dir.path(), "skills", "opencode", "test.SKILL.md");

        let resolver = CascadingResolver::new(
            project_dir.path().to_path_buf(),
            global_dir.path().to_path_buf(),
        );

        let result = resolver.resolve("test.SKILL.md", ToolType::OpenCode);
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_auto_tool_uses_claude() {
        let project_dir = TempDir::new().unwrap();
        let global_dir = TempDir::new().unwrap();

        create_template_file(project_dir.path(), "skills", "claude", "auto.SKILL.md");

        let resolver = CascadingResolver::new(
            project_dir.path().to_path_buf(),
            global_dir.path().to_path_buf(),
        );

        let result = resolver.resolve("auto.SKILL.md", ToolType::Auto);
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_fallback_to_agents_when_ambiguous() {
        let project_dir = TempDir::new().unwrap();
        let global_dir = TempDir::new().unwrap();

        // Create template without clear suffix (not .SKILL.md)
        create_template_file(project_dir.path(), "agents", "claude", "ambiguous");

        let resolver = CascadingResolver::new(
            project_dir.path().to_path_buf(),
            global_dir.path().to_path_buf(),
        );

        let result = resolver.resolve("ambiguous", ToolType::Claude);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_templates_empty() {
        let project_dir = TempDir::new().unwrap();
        let global_dir = TempDir::new().unwrap();

        let resolver = CascadingResolver::new(
            project_dir.path().to_path_buf(),
            global_dir.path().to_path_buf(),
        );

        let result = resolver.list_templates(ToolType::Claude);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_list_templates_project_and_global() {
        let project_dir = TempDir::new().unwrap();
        let global_dir = TempDir::new().unwrap();

        create_template_file(project_dir.path(), "skills", "claude", "project.SKILL.md");
        create_template_file(global_dir.path(), "skills", "claude", "global.SKILL.md");
        create_template_file(project_dir.path(), "agents", "claude", "agent.md");

        let resolver = CascadingResolver::new(
            project_dir.path().to_path_buf(),
            global_dir.path().to_path_buf(),
        );

        let result = resolver.list_templates(ToolType::Claude);
        assert!(result.is_ok());
        let templates = result.unwrap();
        assert_eq!(templates.len(), 3);
    }

    #[test]
    fn test_list_templates_deduplication() {
        let project_dir = TempDir::new().unwrap();
        let global_dir = TempDir::new().unwrap();

        // Create same template in both locations
        create_template_file(project_dir.path(), "skills", "claude", "duplicate.SKILL.md");
        create_template_file(global_dir.path(), "skills", "claude", "duplicate.SKILL.md");

        let resolver = CascadingResolver::new(
            project_dir.path().to_path_buf(),
            global_dir.path().to_path_buf(),
        );

        let result = resolver.list_templates(ToolType::Claude);
        assert!(result.is_ok());
        let templates = result.unwrap();
        // Should only have one entry after deduplication
        let count = templates
            .iter()
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n == "duplicate.SKILL.md.template")
                    .unwrap_or(false)
            })
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_project_templates_dir_exists() {
        let project_dir = TempDir::new().unwrap();
        let global_dir = TempDir::new().unwrap();

        // Create templates directory
        let templates_dir = project_dir.path().join(".aiassisted").join("templates");
        fs::create_dir_all(&templates_dir).unwrap();

        let resolver = CascadingResolver::new(
            project_dir.path().to_path_buf(),
            global_dir.path().to_path_buf(),
        );

        let result = resolver.project_templates_dir();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), templates_dir);
    }

    #[test]
    fn test_project_templates_dir_not_exists() {
        let project_dir = TempDir::new().unwrap();
        let global_dir = TempDir::new().unwrap();

        let resolver = CascadingResolver::new(
            project_dir.path().to_path_buf(),
            global_dir.path().to_path_buf(),
        );

        let result = resolver.project_templates_dir();
        assert!(result.is_none());
    }

    #[test]
    fn test_global_templates_dir() {
        let project_dir = TempDir::new().unwrap();
        let global_dir = TempDir::new().unwrap();

        let resolver = CascadingResolver::new(
            project_dir.path().to_path_buf(),
            global_dir.path().to_path_buf(),
        );

        let result = resolver.global_templates_dir();
        assert_eq!(
            result,
            global_dir.path().join(".aiassisted").join("templates")
        );
    }

    #[test]
    fn test_template_subdir_skills_claude() {
        let project_dir = TempDir::new().unwrap();
        let global_dir = TempDir::new().unwrap();

        let resolver = CascadingResolver::new(
            project_dir.path().to_path_buf(),
            global_dir.path().to_path_buf(),
        );

        let result = resolver.template_subdir("skills", ToolType::Claude);
        assert_eq!(result, "skills/claude");
    }

    #[test]
    fn test_template_subdir_agents_opencode() {
        let project_dir = TempDir::new().unwrap();
        let global_dir = TempDir::new().unwrap();

        let resolver = CascadingResolver::new(
            project_dir.path().to_path_buf(),
            global_dir.path().to_path_buf(),
        );

        let result = resolver.template_subdir("agents", ToolType::OpenCode);
        assert_eq!(result, "agents/opencode");
    }
}
