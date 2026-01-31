//! Platform detection for binary selection.

use std::env;

/// Platform information for binary selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Platform {
    pub os: String,
    pub arch: String,
}

impl Platform {
    /// Detect the current platform at runtime.
    pub fn detect() -> Self {
        Self {
            os: env::consts::OS.to_string(),
            arch: env::consts::ARCH.to_string(),
        }
    }

    /// Get the binary asset name for this platform.
    ///
    /// Maps to cargo-dist generated binary names:
    /// - Linux x86_64: aiassisted-x86_64-unknown-linux-gnu.tar.gz
    /// - Linux aarch64: aiassisted-aarch64-unknown-linux-gnu.tar.gz
    /// - macOS x86_64: aiassisted-x86_64-apple-darwin.tar.gz
    /// - macOS aarch64: aiassisted-aarch64-apple-darwin.tar.gz
    /// - Windows x86_64: aiassisted-x86_64-pc-windows-msvc.zip
    pub fn asset_name(&self) -> String {
        let target_triple = self.target_triple();
        let extension = if self.os == "windows" { "zip" } else { "tar.gz" };
        format!("aiassisted-{}.{}", target_triple, extension)
    }

    /// Get the target triple for this platform.
    fn target_triple(&self) -> String {
        match (self.os.as_str(), self.arch.as_str()) {
            ("linux", "x86_64") => "x86_64-unknown-linux-gnu".to_string(),
            ("linux", "aarch64") => "aarch64-unknown-linux-gnu".to_string(),
            ("macos", "x86_64") => "x86_64-apple-darwin".to_string(),
            ("macos", "aarch64") => "aarch64-apple-darwin".to_string(),
            ("windows", "x86_64") => "x86_64-pc-windows-msvc".to_string(),
            _ => format!("{}-{}", self.arch, self.os), // Fallback
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_returns_valid_platform() {
        let platform = Platform::detect();
        assert!(!platform.os.is_empty());
        assert!(!platform.arch.is_empty());
    }

    #[test]
    fn test_asset_name_linux_x86_64() {
        let platform = Platform {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
        };
        assert_eq!(
            platform.asset_name(),
            "aiassisted-x86_64-unknown-linux-gnu.tar.gz"
        );
    }

    #[test]
    fn test_asset_name_linux_aarch64() {
        let platform = Platform {
            os: "linux".to_string(),
            arch: "aarch64".to_string(),
        };
        assert_eq!(
            platform.asset_name(),
            "aiassisted-aarch64-unknown-linux-gnu.tar.gz"
        );
    }

    #[test]
    fn test_asset_name_macos_x86_64() {
        let platform = Platform {
            os: "macos".to_string(),
            arch: "x86_64".to_string(),
        };
        assert_eq!(
            platform.asset_name(),
            "aiassisted-x86_64-apple-darwin.tar.gz"
        );
    }

    #[test]
    fn test_asset_name_macos_aarch64() {
        let platform = Platform {
            os: "macos".to_string(),
            arch: "aarch64".to_string(),
        };
        assert_eq!(
            platform.asset_name(),
            "aiassisted-aarch64-apple-darwin.tar.gz"
        );
    }

    #[test]
    fn test_asset_name_windows_x86_64() {
        let platform = Platform {
            os: "windows".to_string(),
            arch: "x86_64".to_string(),
        };
        assert_eq!(
            platform.asset_name(),
            "aiassisted-x86_64-pc-windows-msvc.zip"
        );
    }

    #[test]
    fn test_asset_name_unsupported_platform() {
        let platform = Platform {
            os: "freebsd".to_string(),
            arch: "riscv64".to_string(),
        };
        // Fallback format
        assert_eq!(platform.asset_name(), "aiassisted-riscv64-freebsd.tar.gz");
    }

    #[test]
    fn test_target_triple_linux_x86_64() {
        let platform = Platform {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
        };
        assert_eq!(platform.target_triple(), "x86_64-unknown-linux-gnu");
    }

    #[test]
    fn test_target_triple_macos_aarch64() {
        let platform = Platform {
            os: "macos".to_string(),
            arch: "aarch64".to_string(),
        };
        assert_eq!(platform.target_triple(), "aarch64-apple-darwin");
    }

    #[test]
    fn test_target_triple_windows() {
        let platform = Platform {
            os: "windows".to_string(),
            arch: "x86_64".to_string(),
        };
        assert_eq!(platform.target_triple(), "x86_64-pc-windows-msvc");
    }

    #[test]
    fn test_platform_equality() {
        let p1 = Platform {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
        };
        let p2 = Platform {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
        };
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_platform_clone() {
        let p1 = Platform {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
        };
        let p2 = p1.clone();
        assert_eq!(p1, p2);
    }
}
