//! Semantic version comparison for self-update checks.

/// Parse a version string and compare it with another version.
///
/// Supports semantic versioning (e.g., "v1.2.3", "1.2.3").
/// Returns true if `latest` is newer than `current`.
pub fn is_newer_version(current: &str, latest: &str) -> bool {
    let current_ver = parse_version(current);
    let latest_ver = parse_version(latest);

    match (current_ver, latest_ver) {
        (Some(c), Some(l)) => l > c,
        _ => false, // If parsing fails, assume no update
    }
}

/// Parse a version string into (major, minor, patch) tuple.
///
/// Handles both "v1.2.3" and "1.2.3" formats.
fn parse_version(version: &str) -> Option<(u32, u32, u32)> {
    let version = version.trim_start_matches('v');
    let parts: Vec<&str> = version.split('.').collect();

    if parts.len() != 3 {
        return None;
    }

    let major = parts[0].parse::<u32>().ok()?;
    let minor = parts[1].parse::<u32>().ok()?;
    let patch = parts[2].parse::<u32>().ok()?;

    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_with_v_prefix() {
        assert_eq!(parse_version("v1.2.3"), Some((1, 2, 3)));
    }

    #[test]
    fn test_parse_version_without_v_prefix() {
        assert_eq!(parse_version("1.2.3"), Some((1, 2, 3)));
    }

    #[test]
    fn test_parse_version_invalid_format() {
        assert_eq!(parse_version("1.2"), None);
        assert_eq!(parse_version("1.2.3.4"), None);
        assert_eq!(parse_version("abc"), None);
    }

    #[test]
    fn test_parse_version_non_numeric() {
        assert_eq!(parse_version("1.2.x"), None);
    }

    #[test]
    fn test_is_newer_version_major_upgrade() {
        assert!(is_newer_version("v1.0.0", "v2.0.0"));
        assert!(!is_newer_version("v2.0.0", "v1.0.0"));
    }

    #[test]
    fn test_is_newer_version_minor_upgrade() {
        assert!(is_newer_version("v1.0.0", "v1.1.0"));
        assert!(!is_newer_version("v1.1.0", "v1.0.0"));
    }

    #[test]
    fn test_is_newer_version_patch_upgrade() {
        assert!(is_newer_version("v1.0.0", "v1.0.1"));
        assert!(!is_newer_version("v1.0.1", "v1.0.0"));
    }

    #[test]
    fn test_is_newer_version_same_version() {
        assert!(!is_newer_version("v1.2.3", "v1.2.3"));
    }

    #[test]
    fn test_is_newer_version_without_v_prefix() {
        assert!(is_newer_version("1.0.0", "1.1.0"));
    }

    #[test]
    fn test_is_newer_version_mixed_formats() {
        assert!(is_newer_version("v1.0.0", "1.1.0"));
        assert!(is_newer_version("1.0.0", "v1.1.0"));
    }

    #[test]
    fn test_is_newer_version_invalid_current() {
        assert!(!is_newer_version("invalid", "v1.0.0"));
    }

    #[test]
    fn test_is_newer_version_invalid_latest() {
        assert!(!is_newer_version("v1.0.0", "invalid"));
    }

    #[test]
    fn test_is_newer_version_complex_comparison() {
        // Test various complex scenarios
        assert!(is_newer_version("v0.9.9", "v1.0.0"));
        assert!(is_newer_version("v1.9.9", "v2.0.0"));
        assert!(is_newer_version("v1.0.9", "v1.1.0"));
        assert!(!is_newer_version("v1.1.0", "v1.0.9"));
    }

    #[test]
    fn test_is_newer_version_large_numbers() {
        assert!(is_newer_version("v99.99.99", "v100.0.0"));
        assert!(is_newer_version("v1.999.0", "v1.1000.0"));
    }
}
