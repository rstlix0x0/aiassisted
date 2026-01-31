# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`aiassisted` is a CLI tool that embeds a `.aiassisted/` directory into projects. This directory contains curated guidelines, instructions, prompts, and templates that AI assistants can reference for consistent, context-aware assistance.

**Note:** This project is being rewritten from POSIX shell to Rust. See `plans/README.md` for progress.

## Development Plans

See `plans/` directory:
- `plans/README.md` - Plan index and status
- `plans/overview.md` - Architecture and policies
- `plans/phase-*.md` - Individual phase plans

## Common Commands (Rust)

### Development

```bash
# Check code compiles
cargo check

# Run all tests
cargo test

# Run the CLI
cargo run -- --help
cargo run -- version
cargo run -- install

# Build release binary
cargo build --release

# Lint with clippy
cargo clippy

# Format code
cargo fmt
```

### Release (with cargo-dist)

```bash
# Tag a version and push to trigger release
git tag "v0.1.0"
git push --tags
```

## Architecture (Rust)

### Source Code Structure

```
src/
├── main.rs          # Entry point, composition root
├── cli.rs           # Clap CLI definitions
├── core/            # All abstractions (traits, types)
│   ├── types.rs     # Error, ToolType, Result, DTOs
│   ├── infra.rs     # FileSystem, HttpClient, Checksum, Logger
│   ├── content.rs   # ManifestStore, ContentDownloader
│   ├── templates.rs # TemplateEngine, TemplateResolver
│   ├── config.rs    # ConfigStore
│   └── selfupdate.rs# ReleaseProvider
├── infra/           # Shared infrastructure implementations
│   ├── fs.rs        # StdFileSystem
│   ├── http.rs      # ReqwestClient
│   ├── checksum.rs  # Sha2Checksum
│   └── logger.rs    # ColoredLogger
├── content/         # Content domain (install, update, check)
├── templates/       # Templates domain (setup-skills, setup-agents)
├── config/          # Config domain
└── selfupdate/      # Self-update domain
```

### Key Design Decisions

1. **Domain-based modular monolith** - Organized by business domains, not technical layers.

2. **Dependency inversion** - Domains depend on `core/` traits, receive implementations via DI.

3. **Flat domain structure** - No nested api/domain/infrastructure inside domains.

4. **cargo-dist for releases** - Automated cross-platform binary builds and GitHub Releases.

## Rust Development Policies

### 1. Zero Warning Policy

All code must compile with zero warnings. Run:
```bash
cargo check 2>&1 | grep -c warning  # Must be 0
```

### 2. Static Dispatch Over Dynamic Dispatch

Prefer generics over `dyn` traits:

```rust
// ❌ Avoid
fn process(handler: &dyn Handler) { }

// ✅ Prefer
fn process<H: Handler>(handler: &H) { }
```

### 3. Minimal Arc Usage

Only use `Arc` when concurrent shared ownership is required. For this CLI tool:
- Commands run sequentially
- No shared state between threads
- Use owned values or references instead

## Content Organization

The `.aiassisted/` directory contains:

- `guidelines/` - Architecture patterns, documentation standards, language-specific guides
- `instructions/` - AI agent behavior rules and constraints
- `prompts/` - Reusable prompt templates (e.g., commit messages)
- `templates/` - Skill and agent templates for OpenCode and Claude Code
- `config/` - Configuration documentation

## Workflow for Updating Guidelines

1. Edit files in `.aiassisted/`
2. Run `make update-version` to regenerate manifest
3. Run `make test` to verify
4. Commit changes
