# Architecture

This document describes the architecture of `aiassisted` - a CLI tool for embedding AI assistant guidelines into projects.

## Overview

`aiassisted` is built as a **domain-based modular monolith** in Rust, organized around business domains rather than technical layers. The architecture follows clean architecture principles with dependency inversion and static dispatch.

## Migration from Shell to Rust

### Why Rust?

The project was originally implemented as a POSIX shell script but was rewritten in Rust for several key reasons:

1. **Type Safety**: Compile-time guarantees prevent entire classes of bugs
2. **Performance**: Native binary with minimal startup time (<50ms)
3. **Cross-Platform**: Single binary for Linux, macOS, and Windows
4. **Maintainability**: Better code organization and testing infrastructure
5. **Distribution**: cargo-dist provides automated binary releases
6. **Error Handling**: Comprehensive Result<T> types with helpful error messages

### Migration Strategy

The migration followed a phased approach documented in the original `plans/` directory:

- **Phase 1**: Project setup and core types
- **Phase 2**: CLI structure with clap
- **Phase 3**: Content domain (install, update, check)
- **Phase 4**: Templates domain (setup-skills, setup-agents)
- **Phase 5**: Config domain
- **Phase 6**: Self-update domain
- **Phase 7**: Distribution with cargo-dist
- **Phase 8**: Polish and testing

All phases are now complete, resulting in v0.2.0 - the first stable Rust release.

## Architecture Principles

### 1. Domain-Based Modular Monolith

Code is organized by **business domains**, not technical layers:

```
src/
├── content/       # Content domain (install, update, check)
├── templates/     # Templates domain (setup-skills, setup-agents)
├── config/        # Config domain (show, get, edit)
└── selfupdate/    # Self-update domain
```

Each domain is a module that handles a specific business capability.

### 2. Dependency Inversion

All domains depend on **abstractions** (traits) defined in `core/`, not concrete implementations:

```rust
// Core defines traits
pub trait FileSystem {
    async fn read(&self, path: &Path) -> Result<String>;
    async fn write(&self, path: &Path, content: &str) -> Result<()>;
}

// Domains depend on traits
pub async fn install<F: FileSystem>(fs: &F, ...) -> Result<()> {
    let content = fs.read(path).await?;
    // ...
}

// Main composes with concrete implementations
let fs = StdFileSystem::new();
install(&fs, ...).await?;
```

**Benefits:**
- Testable with mock implementations
- No circular dependencies
- Clear boundaries between layers

### 3. Static Dispatch Over Dynamic Dispatch

We prefer generics over `dyn` traits for better performance:

```rust
// ✅ Preferred: Static dispatch with generics
pub async fn process<F: FileSystem>(fs: &F) { }

// ❌ Avoided: Dynamic dispatch
pub async fn process(fs: &dyn FileSystem) { }
```

**Benefits:**
- Zero-cost abstractions
- Compiler optimizations (inlining)
- No vtable overhead

### 4. Minimal Arc Usage

We avoid `Arc` unless absolutely necessary. For this CLI tool:
- Commands run sequentially
- No shared state between threads
- Use owned values or references

**Why:** `Arc` adds complexity and runtime overhead that's unnecessary for a CLI tool.

### 5. Zero Warnings Policy

All code must compile with zero warnings:

```bash
cargo check 2>&1 | grep -c warning  # Must be 0
```

Warnings are treated as errors during development.

## Project Structure

```
aiassisted/
├── src/
│   ├── main.rs              # Entry point, composition root
│   ├── lib.rs               # Library crate root
│   ├── cli.rs               # Clap CLI definitions
│   │
│   ├── core/                # All abstractions (traits, types)
│   │   ├── mod.rs
│   │   ├── types.rs         # Error, ToolType, Result, DTOs
│   │   ├── infra.rs         # FileSystem, HttpClient, Checksum, Logger
│   │   ├── content.rs       # ManifestStore, ContentDownloader
│   │   ├── templates.rs     # TemplateEngine, TemplateResolver
│   │   ├── config.rs        # ConfigStore
│   │   └── selfupdate.rs    # ReleaseProvider
│   │
│   ├── infra/               # Shared infrastructure implementations
│   │   ├── mod.rs
│   │   ├── fs.rs            # StdFileSystem
│   │   ├── http.rs          # ReqwestClient
│   │   ├── checksum.rs      # Sha2Checksum
│   │   └── logger.rs        # ColoredLogger
│   │
│   ├── content/             # Content domain
│   │   ├── mod.rs
│   │   ├── github.rs        # GitHub API utilities
│   │   ├── manifest.rs      # Manifest parsing
│   │   ├── sync.rs          # Install/update/check operations
│   │   └── migration.rs     # Shell → Rust migration
│   │
│   ├── templates/           # Templates domain
│   │   ├── mod.rs
│   │   ├── resolver.rs      # Cascading template resolution
│   │   ├── engine.rs        # Template variable substitution
│   │   ├── detector.rs      # Tool detection (Claude/OpenCode)
│   │   ├── setup.rs         # Setup skills/agents
│   │   ├── manager.rs       # List/show/init/sync/diff
│   │   └── types.rs         # Template types
│   │
│   ├── config/              # Config domain
│   │   ├── mod.rs
│   │   ├── store.rs         # TOML config management
│   │   ├── defaults.rs      # Default values
│   │   └── commands.rs      # Config commands
│   │
│   └── selfupdate/          # Self-update domain
│       ├── mod.rs
│       ├── release.rs       # GitHub releases API
│       ├── platform.rs      # Platform detection
│       └── update.rs        # Binary update logic
│
├── tests/                   # Integration tests
│   ├── content_integration.rs
│   ├── templates_integration.rs
│   └── config_integration.rs
│
├── scripts/                 # Helper scripts
│   ├── update-version.sh    # Regenerate manifest.json
│   ├── smoke-test.sh        # End-to-end testing
│   └── quick-test.sh        # Rapid validation
│
├── .aiassisted/             # Embedded guidelines
│   ├── manifest.json        # File manifest with checksums
│   ├── guidelines/          # Architecture & language guides
│   ├── instructions/        # AI behavior rules
│   ├── prompts/             # Reusable templates
│   └── templates/           # Skill & agent templates
│
├── Cargo.toml               # Dependencies and metadata
├── dist-workspace.toml      # cargo-dist configuration
└── README.md                # User documentation
```

## Core Abstractions

### Types (`src/core/types.rs`)

```rust
// Error type with variants for all domains
pub enum Error {
    Io(String),
    Network(String),
    Serialization(String),
    ChecksumMismatch { expected: String, actual: String },
    NotFound(String),
    // ...
}

// Tool type for AI assistants
pub enum ToolType {
    Auto,      // Auto-detect
    Claude,    // Claude Code
    OpenCode,  // OpenCode
}

// Manifest entry for content files
pub struct ManifestEntry {
    pub path: PathBuf,
    pub checksum: String,
}
```

### Infrastructure Traits (`src/core/infra.rs`)

```rust
// File system operations
#[async_trait]
pub trait FileSystem {
    async fn read(&self, path: &Path) -> Result<String>;
    async fn write(&self, path: &Path, content: &str) -> Result<()>;
    async fn exists(&self, path: &Path) -> bool;
    async fn create_dir_all(&self, path: &Path) -> Result<()>;
    // ...
}

// HTTP client
#[async_trait]
pub trait HttpClient {
    async fn get(&self, url: &str) -> Result<String>;
    async fn download(&self, url: &str, dest: &Path) -> Result<()>;
}

// SHA256 checksumming
pub trait Checksum {
    fn sha256(&self, data: &[u8]) -> String;
    fn sha256_file(&self, path: &Path) -> Result<String>;
}

// Logging
pub trait Logger {
    fn info(&self, msg: &str);
    fn success(&self, msg: &str);
    fn warn(&self, msg: &str);
    fn error(&self, msg: &str);
}
```

## Domain Architecture

### Content Domain (`src/content/`)

Handles downloading and syncing `.aiassisted/` directory from GitHub.

**Responsibilities:**
- Download manifest.json from GitHub
- Install .aiassisted directory with all files
- Check for updates by comparing versions
- Update only changed files (SHA256-based)
- Migrate from old shell-based installations

**Key Types:**
```rust
pub struct Manifest {
    pub version: String,
    pub files: Vec<ManifestEntry>,
}

pub struct ManifestDiff {
    pub new_files: Vec<ManifestEntry>,
    pub modified_files: Vec<ManifestEntry>,
}
```

**Entry Points:**
- `install()` - Download and install .aiassisted
- `check()` - Check if updates available
- `update()` - Update changed files
- `migrate()` - Migrate from shell version

### Templates Domain (`src/templates/`)

Manages AI skill and agent templates with cascading resolution.

**Responsibilities:**
- Detect AI tool type (Claude Code vs OpenCode)
- Resolve templates (project overrides global)
- Substitute variables in templates
- Generate skills and agents from templates
- List/show/init/sync/diff templates

**Cascading Resolution:**
1. Project templates (`./.aiassisted/templates/`)
2. Global templates (`~/.aiassisted/templates/`)

**Template Variables:**
- `{{PROJECT_ROOT}}` - Absolute path to project
- `{{GUIDELINES_LIST}}` - List of guideline files
- `{{TOOL_TYPE}}` - claude or opencode

**Entry Points:**
- `setup_skills()` - Generate slash commands
- `setup_agents()` - Generate custom agents
- `list_templates()` - List available templates
- `show_template()` - Display template content
- `init_templates()` - Copy global to project
- `sync_templates()` - Update project from global
- `diff_templates()` - Show differences

### Config Domain (`src/config/`)

Manages user configuration in TOML format.

**Configuration File:** `~/.aiassisted/config.toml`

**Settings:**
```toml
default_tool = "auto"    # auto, claude, opencode
verbosity = 1            # 0=quiet, 1=normal, 2=debug
auto_update = true       # Check updates on install
prefer_project = true    # Use project templates first
```

**Entry Points:**
- `show()` - Display all settings
- `get()` - Get specific value
- `edit()` - Open in $EDITOR
- `reset()` - Reset to defaults
- `path()` - Show config file location

### Self-Update Domain (`src/selfupdate/`)

Updates the CLI binary itself from GitHub releases.

**Responsibilities:**
- Query GitHub releases API
- Detect platform (OS + architecture)
- Download correct binary for platform
- Extract and replace current binary
- Verify version after update

**Platform Support:**
- Linux: x86_64, aarch64
- macOS: x86_64, aarch64
- Windows: x86_64

**Entry Points:**
- `self_update()` - Update CLI binary

## Data Flow

### Install Flow

```
User runs: aiassisted install

1. CLI (main.rs)
   ↓ Parse args with clap
2. install() in content/sync.rs
   ↓ Call manifest_url() from github.rs
3. HttpClient.get(url)
   ↓ Download manifest.json
4. Manifest::load_remote()
   ↓ Parse JSON to Manifest struct
5. download_batch() in github.rs
   ↓ For each file in manifest
6. HttpClient.get(file_url)
   ↓ Download file content
7. Checksum.sha256()
   ↓ Verify checksum matches
8. FileSystem.write()
   ↓ Save file to disk
9. Manifest.save()
   ↓ Save manifest locally
```

### Update Flow

```
User runs: aiassisted update

1. check() determines if update needed
   ↓ Compare local vs remote versions
2. If outdated:
   ↓ Load remote manifest
3. manifest.diff(local_manifest)
   ↓ Calculate new/modified files
4. Show diff to user
   ↓ Wait for confirmation (unless --force)
5. download_batch(changed_files)
   ↓ Download only changed files
6. Verify checksums
   ↓ Ensure integrity
7. Save new manifest
   ↓ Update local version
```

### Setup Skills Flow

```
User runs: aiassisted setup-skills

1. Detect AI tool
   ↓ Auto-detect or use --tool flag
2. Resolve templates
   ↓ Project templates override global
3. Find skill templates
   ↓ Filter by *.SKILL.md.template
4. For each template:
   ↓ Load template content
5. Substitute variables
   ↓ {{PROJECT_ROOT}}, {{GUIDELINES_LIST}}
6. Write to output directory
   ↓ .claude/commands/ or .opencode/skills/
```

## Testing Strategy

### Unit Tests

Located in `#[cfg(test)] mod tests` within each module.

**Coverage:**
- Core business logic
- Error handling
- Edge cases
- Mock dependencies with `mockall`

**Example:**
```rust
#[cfg(test)]
mod tests {
    use mockall::mock;

    mock! {
        pub FileSystem {}

        #[async_trait::async_trait]
        impl crate::core::infra::FileSystem for FileSystem {
            async fn read(&self, path: &Path) -> Result<String>;
        }
    }

    #[tokio::test]
    async fn test_install() {
        let mut mock_fs = MockFileSystem::new();
        mock_fs.expect_read()
            .returning(|_| Ok("content".to_string()));

        let result = install(&mock_fs).await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests

Located in `tests/` directory.

**Coverage:**
- Multi-module workflows
- Real implementations (no mocks for infra)
- HTTP mocking with `wiremock`
- File operations with `tempfile`

**Example:**
```rust
#[tokio::test]
async fn test_full_install_workflow() {
    let mock_server = MockServer::start().await;
    let fs = StdFileSystem::new();
    let http = ReqwestClient::new();

    // Setup mock HTTP responses
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    // Test with real implementations
    let result = install(&fs, &http, &url).await;
    assert!(result.is_ok());
}
```

### Smoke Tests

Shell script (`scripts/smoke-test.sh`) that tests the actual binary:

```bash
./scripts/smoke-test.sh --binary ./target/release/aiassisted
```

Tests all commands end-to-end with real GitHub API.

## Build and Release

### Build Profiles

```toml
# Development
cargo build

# Release (optimized)
cargo build --release

# Distribution (via cargo-dist)
[profile.dist]
inherits = "release"
lto = "thin"
```

### Release Process

Automated via cargo-dist and GitHub Actions:

```bash
# 1. Update version
# Edit Cargo.toml: version = "0.2.0"

# 2. Update FEATURES.md and CHANGELOG.md

# 3. Commit and tag
git commit -m "chore: bump version to 0.2.0"
git tag v0.2.0
git push origin main --tags

# 4. GitHub Actions triggers
# - Builds binaries for all platforms
# - Creates GitHub release
# - Uploads binaries and checksums
```

### Platforms

cargo-dist builds for:
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

## Performance Characteristics

| Metric | Value | Notes |
|--------|-------|-------|
| Binary size | ~5MB | Release build, stripped |
| Startup time | <50ms | Cold start |
| Install (43 files) | ~3-5s | Network dependent |
| Memory usage | <20MB | Peak during install |

## Security

- **No sudo required** - User directory only
- **SHA256 verification** - All downloads checksummed
- **HTTPS only** - All GitHub downloads
- **No code execution** - No eval or shell injection
- **Sandboxed** - Only touches target directory

## Future Enhancements

Potential improvements for future versions:

1. **Offline Mode** - Cache downloads for offline use
2. **Proxy Support** - HTTP proxy configuration
3. **Custom Sources** - Support non-GitHub sources
4. **Shell Completions** - bash/zsh/fish completions
5. **Parallel Downloads** - Faster installs with concurrent downloads
6. **Delta Updates** - Binary diff for smaller updates

## Design Decisions

### Why No Workspace?

Single binary crate (not workspace) because:
- Simple project structure
- All code ships together
- No need for separate libraries
- Easier to maintain

### Why Async?

HTTP and file I/O are naturally async:
- Better resource utilization
- Non-blocking downloads
- Future-proof for parallel downloads

### Why Not a Library?

Primarily a CLI tool, not a library:
- `src/lib.rs` exists for testing
- Not published to crates.io
- Focus on binary distribution

### Why TOML for Config?

- Human-readable
- Well-supported in Rust
- Better than JSON for config files
- Supports comments

## Contributing

See the main README.md for contribution guidelines.

When working on the codebase:

1. **Follow architecture** - Keep domains separate
2. **Use dependency inversion** - Depend on traits
3. **Write tests** - Unit + integration coverage
4. **Zero warnings** - Fix all compiler warnings
5. **Document** - Add doc comments for public APIs

---

**Architecture Version:** 0.2.0
**Last Updated:** 2026-02-01
**Status:** Stable
