//! Configuration settings and validation.

use crate::core::types::{AppConfig, Error, Result};

/// Validates configuration values.
pub fn validate_config(config: &AppConfig) -> Result<()> {
    // Validate verbosity level (0-2)
    if config.verbosity > 2 {
        return Err(Error::Config(format!(
            "Invalid verbosity level: {}. Must be 0-2.",
            config.verbosity
        )));
    }

    Ok(())
}

/// Gets a configuration value by key using dot notation.
pub fn get_value(config: &AppConfig, key: &str) -> Option<String> {
    match key {
        "default_tool" => Some(config.default_tool.to_string()),
        "verbosity" => Some(config.verbosity.to_string()),
        "auto_update" => Some(config.auto_update.to_string()),
        "prefer_project" => Some(config.prefer_project.to_string()),
        "templates.prefer_project" => Some(config.prefer_project.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_config_valid() {
        let config = AppConfig::default();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_invalid_verbosity() {
        let config = AppConfig {
            verbosity: 3,
            ..Default::default()
        };
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_get_value_default_tool() {
        let config = AppConfig::default();
        assert_eq!(get_value(&config, "default_tool"), Some("auto".to_string()));
    }

    #[test]
    fn test_get_value_verbosity() {
        let config = AppConfig::default();
        assert_eq!(get_value(&config, "verbosity"), Some("1".to_string()));
    }

    #[test]
    fn test_get_value_auto_update() {
        let config = AppConfig::default();
        assert_eq!(get_value(&config, "auto_update"), Some("true".to_string()));
    }

    #[test]
    fn test_get_value_prefer_project() {
        let config = AppConfig::default();
        assert_eq!(
            get_value(&config, "prefer_project"),
            Some("true".to_string())
        );
    }

    #[test]
    fn test_get_value_templates_prefer_project() {
        let config = AppConfig::default();
        assert_eq!(
            get_value(&config, "templates.prefer_project"),
            Some("true".to_string())
        );
    }

    #[test]
    fn test_get_value_unknown_key() {
        let config = AppConfig::default();
        assert_eq!(get_value(&config, "unknown.key"), None);
    }
}
