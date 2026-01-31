# Rust Rewrite Overview

## Executive Summary

Rewriting the `aiassisted` CLI from POSIX shell to Rust for:
- Single binary distribution (no runtime dependencies)
- Cross-platform compilation (macOS, Linux, Windows)
- Better error handling and type safety
- Modern CLI experience with shell completions

## Rust Development Policies

### 1. Zero Warning Policy

**All code must compile with zero warnings.** This includes:

- No unused imports, variables, or functions
- No dead code
- No deprecated API usage
- No clippy warnings

**Enforcement:**
```bash
# Development builds
cargo check 2>&1 | grep -c warning  # Must be 0

# CI/CD
RUSTFLAGS="-D warnings" cargo build
```

**Handling pre-implemented abstractions:**
- Use `#[allow(dead_code)]` sparingly and only with a comment explaining when it will be used
- Prefer implementing features incrementally rather than defining unused abstractions upfront
- Remove unused code rather than suppressing warnings

### 2. Static Dispatch Over Dynamic Dispatch

**Prefer generics over `dyn` traits.**

❌ **Avoid:**
```rust
struct AppContext {
    fs: Arc<dyn FileSystem>,
    http: Arc<dyn HttpClient>,
}
```

✅ **Prefer:**
```rust
struct AppContext<F: FileSystem, H: HttpClient> {
    fs: F,
    http: H,
}
```

**Rationale:**
- Zero-cost abstraction - no vtable lookups
- Better compiler optimizations (inlining, monomorphization)
- Compile-time type checking
- This is a CLI tool - we know concrete types at compile time

**When `dyn` is acceptable:**
- Heterogeneous collections (different types implementing same trait)
- Plugin systems with runtime-loaded code
- Reducing binary size when many concrete types would cause bloat

### 3. Minimal Arc Usage

**Only use `Arc` when concurrent shared ownership is actually needed.**

❌ **Avoid (for CLI tools):**
```rust
struct AppContext {
    fs: Arc<dyn FileSystem>,  // Unnecessary Arc
}
```

✅ **Prefer:**
```rust
// Option 1: Owned values
struct AppContext<F: FileSystem> {
    fs: F,
}

// Option 2: Simple references for short-lived contexts
fn run_command<F: FileSystem>(fs: &F, args: &Args) { }
```

**Rationale:**
- `Arc` adds atomic reference counting overhead
- CLI commands run sequentially, not concurrently
- No shared state between threads in this application
- Simpler code without wrapper types

**When `Arc` is appropriate:**
- Sharing state across spawned async tasks
- True multi-threaded concurrent access
- Long-lived shared caches

## Architecture: Domain-Based Modular Monolith

### Identified Domains

| Domain | Responsibility | CLI Commands |
|--------|---------------|--------------|
| `content` | Managing .aiassisted content | `install`, `update`, `check` |
| `templates` | Template processing & management | `setup-skills`, `setup-agents`, `templates *` |
| `config` | Application configuration | `config *` |
| `selfupdate` | CLI binary updates | `self-update` |

### Project Structure

```
src/
├── main.rs          # Entry point, composition root
├── cli.rs           # Clap CLI definitions
├── core/            # Shared abstractions (traits, types)
│   ├── mod.rs
│   ├── types.rs     # Error, ToolType, Result, DTOs
│   └── infra.rs     # FileSystem, HttpClient, Checksum, Logger
├── infra/           # Shared infrastructure implementations
│   ├── mod.rs
│   ├── fs.rs        # StdFileSystem
│   ├── http.rs      # ReqwestClient
│   ├── checksum.rs  # Sha2Checksum
│   └── logger.rs    # ColoredLogger
├── content/         # Content domain
├── templates/       # Templates domain
├── config/          # Config domain
└── selfupdate/      # Self-update domain
```

### Dependency Flow

```
         main.rs (composition root)
              │
    ┌─────────┼─────────┐
    ▼         ▼         ▼
content  templates  config  selfupdate
    │         │         │         │
    └─────────┴─────────┴─────────┘
              │
       ┌──────┴──────┐
       ▼             ▼
    core/         infra/
   (traits)    (implementations)
```

**Key principles:**
- Domains depend on `core/` for traits and types
- Domains receive `infra/` implementations via dependency injection
- Domains are independent - they don't depend on each other

## Target Platforms

| Platform | Architecture | Binary Name |
|----------|-------------|-------------|
| Linux | x86_64 | `aiassisted-linux-x86_64` |
| Linux | aarch64 | `aiassisted-linux-aarch64` |
| macOS | x86_64 | `aiassisted-darwin-x86_64` |
| macOS | aarch64 | `aiassisted-darwin-aarch64` |
| Windows | x86_64 | `aiassisted-windows-x86_64.exe` |

## Decisions Made

- **Content handling**: Downloaded from GitHub (not embedded)
- **Platform support**: macOS, Linux, and Windows
- **Project structure**: Replace shell code entirely
- **Release tooling**: cargo-dist for automated cross-platform releases
