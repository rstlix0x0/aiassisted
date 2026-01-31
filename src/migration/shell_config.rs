//! Shell config parser for reading old TOML format.
//!
//! The old shell-based version used a different TOML structure at
//! `~/.aiassisted/config.toml` with multiple sections.

use serde::Deserialize;

use crate::core::types::{AppConfig, Error, Result, ToolType};

/// Old configuration structure from shell-based version.
#[derive(Debug, Deserialize)]
pub struct ShellConfig {
    #[serde(default)]
    pub general: GeneralSection,
    #[serde(default)]
    pub install: InstallSection,
    #[serde(default)]
    pub templates: TemplatesSection,
}

/// [general] section from old config.
#[derive(Debug, Default, Deserialize)]
pub struct GeneralSection {
    /// default_runtime = "auto" | "shell" | "python" | "bun"
    #[serde(default = "default_runtime")]
    pub default_runtime: String,
    #[serde(default = "default_verbosity")]
    pub verbosity: u8,
}

/// [install] section from old config.
#[derive(Debug, Default, Deserialize)]
pub struct InstallSection {
    #[serde(default = "default_true")]
    pub auto_update: bool,
}

/// [templates] section from old config.
#[derive(Debug, Default, Deserialize)]
pub struct TemplatesSection {
    #[serde(default = "default_true")]
    pub prefer_project: bool,
}

fn default_runtime() -> String {
    "auto".to_string()
}

fn default_verbosity() -> u8 {
    1
}

fn default_true() -> bool {
    true
}

impl ShellConfig {
    /// Parse old TOML config from string.
    pub fn from_str(contents: &str) -> Result<Self> {
        toml::from_str(contents)
            .map_err(|e| Error::Parse(format!("Failed to parse shell config: {}", e)))
    }

    /// Convert old config to new AppConfig.
    ///
    /// Mapping:
    /// - general.default_runtime → default_tool (shell→auto, keep auto)
    /// - general.verbosity → verbosity
    /// - install.auto_update → auto_update
    /// - templates.prefer_project → prefer_project
    pub fn to_app_config(&self) -> AppConfig {
        let default_tool = match self.general.default_runtime.as_str() {
            "shell" => ToolType::Auto,
            "auto" => ToolType::Auto,
            "opencode" => ToolType::OpenCode,
            "claude" => ToolType::Claude,
            _ => ToolType::Auto, // Default to Auto for unknown values
        };

        AppConfig {
            default_tool,
            verbosity: self.general.verbosity,
            auto_update: self.install.auto_update,
            prefer_project: self.templates.prefer_project,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_complete_shell_config() {
        let toml = r#"
[general]
default_runtime = "shell"
verbosity = 2

[install]
auto_update = false
confirm_before_install = false
install_path = ".aiassisted"

[templates]
prefer_project = false
auto_init_templates = false
auto_sync_templates = false

[skills]
tools = []
auto_setup = false

[update]
check_on_startup = false
channel = "stable"

[github]
repo = "rstlix0x0/aiassisted"
ref = ""
"#;

        let config = ShellConfig::from_str(toml).unwrap();
        assert_eq!(config.general.default_runtime, "shell");
        assert_eq!(config.general.verbosity, 2);
        assert!(!config.install.auto_update);
        assert!(!config.templates.prefer_project);
    }

    #[test]
    fn test_parse_minimal_shell_config() {
        let toml = r#"
[general]
default_runtime = "auto"

[install]
auto_update = true

[templates]
prefer_project = true
"#;

        let config = ShellConfig::from_str(toml).unwrap();
        assert_eq!(config.general.default_runtime, "auto");
        assert_eq!(config.general.verbosity, 1); // default
        assert!(config.install.auto_update);
        assert!(config.templates.prefer_project);
    }

    #[test]
    fn test_parse_empty_config() {
        let toml = r#"
[general]

[install]

[templates]
"#;
        let config = ShellConfig::from_str(toml).unwrap();
        assert_eq!(config.general.default_runtime, "auto"); // default
        assert_eq!(config.general.verbosity, 1);
        assert!(config.install.auto_update);
        assert!(config.templates.prefer_project);
    }

    #[test]
    fn test_parse_invalid_toml() {
        let toml = "invalid { toml";
        let result = ShellConfig::from_str(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_shell_to_auto() {
        let config = ShellConfig {
            general: GeneralSection {
                default_runtime: "shell".to_string(),
                verbosity: 1,
            },
            install: InstallSection { auto_update: true },
            templates: TemplatesSection {
                prefer_project: true,
            },
        };

        let app_config = config.to_app_config();
        assert_eq!(app_config.default_tool, ToolType::Auto);
        assert_eq!(app_config.verbosity, 1);
        assert!(app_config.auto_update);
        assert!(app_config.prefer_project);
    }

    #[test]
    fn test_convert_auto_to_auto() {
        let config = ShellConfig {
            general: GeneralSection {
                default_runtime: "auto".to_string(),
                verbosity: 2,
            },
            install: InstallSection { auto_update: false },
            templates: TemplatesSection {
                prefer_project: false,
            },
        };

        let app_config = config.to_app_config();
        assert_eq!(app_config.default_tool, ToolType::Auto);
        assert_eq!(app_config.verbosity, 2);
        assert!(!app_config.auto_update);
        assert!(!app_config.prefer_project);
    }

    #[test]
    fn test_convert_claude_to_claude() {
        let config = ShellConfig {
            general: GeneralSection {
                default_runtime: "claude".to_string(),
                verbosity: 1,
            },
            install: InstallSection { auto_update: true },
            templates: TemplatesSection {
                prefer_project: true,
            },
        };

        let app_config = config.to_app_config();
        assert_eq!(app_config.default_tool, ToolType::Claude);
    }

    #[test]
    fn test_convert_opencode_to_opencode() {
        let config = ShellConfig {
            general: GeneralSection {
                default_runtime: "opencode".to_string(),
                verbosity: 1,
            },
            install: InstallSection { auto_update: true },
            templates: TemplatesSection {
                prefer_project: true,
            },
        };

        let app_config = config.to_app_config();
        assert_eq!(app_config.default_tool, ToolType::OpenCode);
    }

    #[test]
    fn test_convert_unknown_runtime_to_auto() {
        let config = ShellConfig {
            general: GeneralSection {
                default_runtime: "python".to_string(),
                verbosity: 1,
            },
            install: InstallSection { auto_update: true },
            templates: TemplatesSection {
                prefer_project: true,
            },
        };

        let app_config = config.to_app_config();
        assert_eq!(app_config.default_tool, ToolType::Auto);
    }

    #[test]
    fn test_all_verbosity_levels() {
        for verbosity in 0..=2 {
            let config = ShellConfig {
                general: GeneralSection {
                    default_runtime: "auto".to_string(),
                    verbosity,
                },
                install: InstallSection { auto_update: true },
                templates: TemplatesSection {
                    prefer_project: true,
                },
            };

            let app_config = config.to_app_config();
            assert_eq!(app_config.verbosity, verbosity);
        }
    }

    #[test]
    fn test_all_boolean_combinations() {
        let combinations = [
            (true, true),
            (true, false),
            (false, true),
            (false, false),
        ];

        for (auto_update, prefer_project) in combinations {
            let config = ShellConfig {
                general: GeneralSection {
                    default_runtime: "auto".to_string(),
                    verbosity: 1,
                },
                install: InstallSection { auto_update },
                templates: TemplatesSection { prefer_project },
            };

            let app_config = config.to_app_config();
            assert_eq!(app_config.auto_update, auto_update);
            assert_eq!(app_config.prefer_project, prefer_project);
        }
    }
}
