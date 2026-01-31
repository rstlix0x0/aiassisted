# Phase 3: Content Domain

**Status:** ✅ Complete

## Objectives

- Implement the content domain for managing .aiassisted content
- Implement `install`, `update`, and `check` commands
- Download content from GitHub repository

## Prerequisites

Before starting Phase 3, refactor the codebase to comply with Rust policies:

1. **Remove `Arc<dyn Trait>` pattern** - Use generics for static dispatch
2. **Fix all compiler warnings** - Zero warning policy
3. **Simplify AppContext** - Remove unnecessary abstractions

## Tasks

- [x] Refactor main.rs to use static dispatch (generics)
- [x] Fix all compiler warnings (zero warnings policy maintained)
- [x] Create `src/content/` domain structure
- [x] Implement manifest parsing and checksum verification
- [x] Implement `install` command
- [x] Implement `update` command (checksum-based selective updates)
- [x] Implement `check` command
- [x] Add domain-specific tests (53 unit + 10 integration tests)

## Domain Structure

```
src/content/
├── mod.rs           # Public API exports
├── commands.rs      # InstallCommand, UpdateCommand, CheckCommand
├── manifest.rs      # Manifest parsing, checksum verification
├── sync.rs          # Install/update logic
└── github.rs        # GitHub API for downloading content
```

## Implementation Details

### Install Command

1. Check if `.aiassisted/` already exists
2. Download manifest from GitHub
3. Download all files listed in manifest
4. Verify checksums
5. Create `.aiassisted/` directory structure

### Update Command

1. Read local manifest
2. Fetch remote manifest
3. Compare checksums
4. Download only changed files
5. If `--force`, redownload all files

### Check Command

1. Read local manifest
2. Fetch remote manifest
3. Compare checksums
4. Report which files have updates available

### Manifest Format

```json
{
  "version": "1.0.0",
  "files": [
    {
      "path": "guidelines/architecture/modular-monolith.md",
      "checksum": "sha256:abc123..."
    }
  ]
}
```

## GitHub API

Content is hosted at:
- Base URL: `https://raw.githubusercontent.com/rstlix0x0/aiassisted/main/`
- Manifest: `.aiassisted/manifest.json`

## Testing

```bash
# Unit tests
cargo test content::

# Integration test (manual)
cargo run -- install --path=/tmp/test-project
cargo run -- check --path=/tmp/test-project
cargo run -- update --path=/tmp/test-project
```
