//! Configuration settings and validation.

use crate::core::types::{AppConfig, Error, Result, ToolType};

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

/// Sets a configuration value by key using dot notation.
#[allow(dead_code)] // Used by TomlConfigStore
pub fn set_value(config: &mut AppConfig, key: &str, value: &str) -> Result<()> {
    match key {
        "default_tool" => {
            config.default_tool = value
                .parse::<ToolType>()
                .map_err(|e| Error::Config(format!("Invalid tool type: {}", e)))?;
        }
        "verbosity" => {
            config.verbosity = value
                .parse::<u8>()
                .map_err(|_| Error::Config(format!("Invalid verbosity value: {}", value)))?;
            if config.verbosity > 2 {
                return Err(Error::Config(
                    "Verbosity must be 0-2".to_string(),
                ));
            }
        }
        "auto_update" => {
            config.auto_update = value
                .parse::<bool>()
                .map_err(|_| Error::Config(format!("Invalid boolean value: {}", value)))?;
        }
        "prefer_project" | "templates.prefer_project" => {
            config.prefer_project = value
                .parse::<bool>()
                .map_err(|_| Error::Config(format!("Invalid boolean value: {}", value)))?;
        }
        _ => {
            return Err(Error::Config(format!("Unknown configuration key: {}", key)));
        }
    }

    validate_config(config)?;
    Ok(())
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

    #[test]
    fn test_set_value_default_tool() {
        let mut config = AppConfig::default();
        assert!(set_value(&mut config, "default_tool", "claude").is_ok());
        assert_eq!(config.default_tool, ToolType::Claude);
    }

    #[test]
    fn test_set_value_verbosity() {
        let mut config = AppConfig::default();
        assert!(set_value(&mut config, "verbosity", "2").is_ok());
        assert_eq!(config.verbosity, 2);
    }

    #[test]
    fn test_set_value_verbosity_invalid() {
        let mut config = AppConfig::default();
        assert!(set_value(&mut config, "verbosity", "3").is_err());
    }

    #[test]
    fn test_set_value_auto_update() {
        let mut config = AppConfig::default();
        assert!(set_value(&mut config, "auto_update", "false").is_ok());
        assert!(!config.auto_update);
    }

    #[test]
    fn test_set_value_prefer_project() {
        let mut config = AppConfig::default();
        assert!(set_value(&mut config, "prefer_project", "false").is_ok());
        assert!(!config.prefer_project);
    }

    #[test]
    fn test_set_value_templates_prefer_project() {
        let mut config = AppConfig::default();
        assert!(set_value(&mut config, "templates.prefer_project", "false").is_ok());
        assert!(!config.prefer_project);
    }

    #[test]
    fn test_set_value_unknown_key() {
        let mut config = AppConfig::default();
        let result = set_value(&mut config, "unknown.key", "value");
        assert!(result.is_err());
    }

    #[test]
    fn test_set_value_invalid_tool_type() {
        let mut config = AppConfig::default();
        let result = set_value(&mut config, "default_tool", "invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_set_value_invalid_boolean() {
        let mut config = AppConfig::default();
        let result = set_value(&mut config, "auto_update", "not_a_bool");
        assert!(result.is_err());
    }
}
