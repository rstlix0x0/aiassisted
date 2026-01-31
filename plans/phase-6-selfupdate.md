# Phase 6: Self-Update Domain

**Status:** Complete

## Objectives

- Implement the self-update domain for CLI binary updates
- Implement `self-update` command
- Use GitHub Releases API

## Tasks

- [x] Create `src/selfupdate/` domain structure
- [x] Implement version comparison logic
- [x] Implement platform detection
- [x] Implement GitHub Releases API client
- [x] Implement binary download and replacement
- [x] Implement `self-update` command
- [x] Add domain-specific tests

## Domain Structure

```
src/selfupdate/
├── mod.rs               # Public API exports
├── commands.rs          # SelfUpdateCommand
├── version.rs           # Version comparison (semver)
├── platform.rs          # Platform detection
└── github_releases.rs   # GitHub Releases API
```

## Implementation Details

### Version Comparison

Use semantic versioning (semver) for comparison:
- Current version from `CARGO_PKG_VERSION`
- Compare with latest GitHub release tag

### Platform Detection

Detect current platform at runtime:

```rust
fn detect_platform() -> Platform {
    let os = std::env::consts::OS;      // "linux", "macos", "windows"
    let arch = std::env::consts::ARCH;  // "x86_64", "aarch64"
    // Map to binary name
}
```

### GitHub Releases API

```
GET https://api.github.com/repos/rstlix0x0/aiassisted/releases/latest
```

Response includes:
- `tag_name`: Version tag (e.g., "v1.0.0")
- `assets`: List of downloadable binaries

### Binary Replacement

1. Download new binary to temp location
2. Verify checksum (if provided)
3. Make executable (`chmod +x`)
4. Replace current binary (atomic rename)
5. Print success message

### Self-Update Flow

```
1. Check current version
2. Fetch latest release from GitHub
3. Compare versions
4. If newer available:
   a. Download appropriate binary for platform
   b. Verify integrity
   c. Replace current binary
   d. Report success
5. If already latest:
   a. Report "Already up to date"
```

## Platform Binary Names

| Platform | Binary Asset Name |
|----------|------------------|
| Linux x86_64 | `aiassisted-x86_64-unknown-linux-gnu.tar.gz` |
| Linux aarch64 | `aiassisted-aarch64-unknown-linux-gnu.tar.gz` |
| macOS x86_64 | `aiassisted-x86_64-apple-darwin.tar.gz` |
| macOS aarch64 | `aiassisted-aarch64-apple-darwin.tar.gz` |
| Windows x86_64 | `aiassisted-x86_64-pc-windows-msvc.zip` |

## Testing

```bash
# Unit tests
cargo test selfupdate::

# Integration test (manual - use with caution)
cargo run -- self-update
```
