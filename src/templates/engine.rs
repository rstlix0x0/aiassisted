//! Template rendering engine implementation.

use std::collections::HashMap;

use crate::core::templates::TemplateEngine;
use crate::core::types::{Error, Result};

/// Simple template engine that replaces {{variable}} patterns.
pub struct SimpleTemplateEngine;

impl SimpleTemplateEngine {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleTemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateEngine for SimpleTemplateEngine {
    fn render(&self, template: &str, vars: &HashMap<String, String>) -> Result<String> {
        let mut result = template.to_string();

        // Replace all {{variable}} patterns
        for (key, value) in vars {
            let pattern = format!("{{{{{}}}}}", key);
            result = result.replace(&pattern, value);
        }

        // Check for unresolved variables
        if result.contains("{{") && result.contains("}}") {
            // Extract unresolved variable names for better error messages
            let mut unresolved = Vec::new();
            let mut start = 0;
            while let Some(pos) = result[start..].find("{{") {
                let abs_pos = start + pos;
                if let Some(end_pos) = result[abs_pos..].find("}}") {
                    let var_name = &result[abs_pos + 2..abs_pos + end_pos];
                    unresolved.push(var_name.to_string());
                    start = abs_pos + end_pos + 2;
                } else {
                    break;
                }
            }

            return Err(Error::Template(format!(
                "Unresolved template variables: {}",
                unresolved.join(", ")
            )));
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_single_variable() {
        let engine = SimpleTemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "World".to_string());

        let result = engine.render("Hello {{name}}!", &vars).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_render_multiple_variables() {
        let engine = SimpleTemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("greeting".to_string(), "Hello".to_string());
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("punctuation".to_string(), "!".to_string());

        let result = engine
            .render("{{greeting}} {{name}}{{punctuation}}", &vars)
            .unwrap();
        assert_eq!(result, "Hello Alice!");
    }

    #[test]
    fn test_render_same_variable_multiple_times() {
        let engine = SimpleTemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("word".to_string(), "test".to_string());

        let result = engine
            .render("{{word}} {{word}} {{word}}", &vars)
            .unwrap();
        assert_eq!(result, "test test test");
    }

    #[test]
    fn test_render_unresolved_variable() {
        let engine = SimpleTemplateEngine::new();
        let vars = HashMap::new();

        let result = engine.render("Hello {{name}}!", &vars);
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::Template(_))));
        if let Err(Error::Template(msg)) = result {
            assert!(msg.contains("Unresolved template variables"));
            assert!(msg.contains("name"));
        }
    }

    #[test]
    fn test_render_partially_resolved() {
        let engine = SimpleTemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("greeting".to_string(), "Hello".to_string());

        let result = engine.render("{{greeting}} {{name}}!", &vars);
        assert!(result.is_err());
        if let Err(Error::Template(msg)) = result {
            assert!(msg.contains("name"));
        }
    }

    #[test]
    fn test_render_empty_template() {
        let engine = SimpleTemplateEngine::new();
        let vars = HashMap::new();

        let result = engine.render("", &vars).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_render_no_variables() {
        let engine = SimpleTemplateEngine::new();
        let vars = HashMap::new();

        let result = engine
            .render("This template has no variables.", &vars)
            .unwrap();
        assert_eq!(result, "This template has no variables.");
    }

    #[test]
    fn test_render_special_characters_in_value() {
        let engine = SimpleTemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("path".to_string(), "/usr/local/bin".to_string());
        vars.insert("email".to_string(), "user@example.com".to_string());
        vars.insert("symbols".to_string(), "!@#$%^&*()".to_string());

        let result = engine
            .render(
                "Path: {{path}}, Email: {{email}}, Symbols: {{symbols}}",
                &vars,
            )
            .unwrap();
        assert_eq!(
            result,
            "Path: /usr/local/bin, Email: user@example.com, Symbols: !@#$%^&*()"
        );
    }

    #[test]
    fn test_render_unicode_in_value() {
        let engine = SimpleTemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("unicode".to_string(), "Hello 世界 🌍".to_string());

        let result = engine.render("{{unicode}}", &vars).unwrap();
        assert_eq!(result, "Hello 世界 🌍");
    }

    #[test]
    fn test_render_multiline_template() {
        let engine = SimpleTemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("title".to_string(), "My Document".to_string());
        vars.insert("author".to_string(), "John Doe".to_string());

        let template = "# {{title}}\n\nAuthor: {{author}}";
        let result = engine.render(template, &vars).unwrap();
        assert_eq!(result, "# My Document\n\nAuthor: John Doe");
    }

    #[test]
    fn test_render_malformed_pattern_single_brace() {
        let engine = SimpleTemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Test".to_string());

        // Single braces should pass through
        let result = engine.render("{name}", &vars).unwrap();
        assert_eq!(result, "{name}");
    }

    #[test]
    fn test_render_nested_braces() {
        let engine = SimpleTemplateEngine::new();
        let vars = HashMap::new();

        // This should fail as it has unresolved variables
        let result = engine.render("{{{{nested}}}}", &vars);
        assert!(result.is_err());
    }

    #[test]
    fn test_render_empty_variable_name() {
        let engine = SimpleTemplateEngine::new();
        let vars = HashMap::new();

        let result = engine.render("Hello {{}}", &vars);
        assert!(result.is_err());
        if let Err(Error::Template(msg)) = result {
            assert!(msg.contains("Unresolved template variables"));
        }
    }

    #[test]
    fn test_default_creates_engine() {
        let engine = SimpleTemplateEngine::default();
        let mut vars = HashMap::new();
        vars.insert("test".to_string(), "value".to_string());

        let result = engine.render("{{test}}", &vars).unwrap();
        assert_eq!(result, "value");
    }

    #[test]
    fn test_render_whitespace_in_variable_name() {
        let engine = SimpleTemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("var name".to_string(), "value".to_string());

        let result = engine.render("{{var name}}", &vars).unwrap();
        assert_eq!(result, "value");
    }
}
