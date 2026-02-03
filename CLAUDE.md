# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`aiassisted` is a CLI tool that embeds a `.aiassisted/` directory into projects. This directory contains curated guidelines, instructions, prompts, and skills that AI assistants can reference for consistent, context-aware assistance.

**Note:** This project is being rewritten from POSIX shell to Rust. See `plans/README.md` for progress.

## Development Plans

See `plans/` directory:
- `plans/README.md` - Plan index and status
- `plans/overview.md` - Architecture and policies
- `plans/phase-*.md` - Individual phase plans

## Memory Bank

This project uses a **Multi-Project Memory Bank** for AI-assisted development workflows. The memory bank maintains context between sessions and tracks sub-project progress.

**Location:** `.memory-bank/` (not tracked in git)

**Full Instructions:** `.aiassisted/instructions/multi-project-memory-bank.instructions.md`

### Quick Reference

```
.memory-bank/
├── README.md              # Entry point
├── current-context.md     # Active sub-project tracker
├── workspace/             # Shared workspace context
│   ├── project-brief.md
│   ├── shared-patterns.md
│   ├── workspace-architecture.md
│   └── workspace-progress.md
├── templates/docs/        # Documentation templates
├── context-snapshots/     # Saved context states
└── sub-projects/          # Individual sub-project contexts
    └── <project-name>/
        ├── project-brief.md
        ├── product-context.md
        ├── active-context.md
        ├── system-patterns.md
        ├── tech-context.md
        ├── progress.md
        ├── tasks/
        └── docs/
```

### Key Commands

- **Start session:** Read `current-context.md` to identify active sub-project
- **Switch context:** Update `current-context.md` when changing sub-projects
- **Resume work:** Read `active-context.md` for current focus and next steps
- **Track progress:** Update `progress.md` and `tasks/_index.md`

### Active Sub-Projects

- `tui-integration` - Ratatui-based TUI for progress display

### Memory Bank Agents

Three specialized agents handle memory bank workflows. These agents are defined in `.aiassisted/agents/` following the [Agent Specification](.aiassisted/guidelines/ai/agents/agent-spec.guideline.md).

| Agent | Purpose | Capabilities |
|-------|---------|--------------|
| `memorybank-planner` | Creates task plans following memory bank format | read-write |
| `memorybank-implementer` | Executes planned tasks with progress tracking | read-write |
| `memorybank-verifier` | Validates planner and implementer work results | read-only |

#### Agent Workflow

```
┌─────────────────┐                    ┌─────────────────────┐
│ memorybank-     │──── completes ────▶│ USER                │
│ planner         │                    │ (decides next step) │
└─────────────────┘                    └─────────────────────┘
                                                │
                                                ▼ (optional)
                                       ┌─────────────────────┐
                                       │ memorybank-         │
                                       │ verifier            │
                                       └─────────────────────┘
                                                ▲
                                                │ (optional)
┌─────────────────┐                    ┌─────────────────────┐
│ memorybank-     │──── completes ────▶│ USER                │
│ implementer     │                    │ (decides next step) │
└─────────────────┘                    └─────────────────────┘
```

#### No Automatic Agent Chaining

**CRITICAL:** Agents do NOT automatically invoke other agents.

- Each agent completes its work and reports to the user
- The USER decides whether to run verification
- This prevents infinite recursion and gives user control

#### When to Use Each Agent

| Scenario | Agent to Use |
|----------|--------------|
| "Plan a new task for X" | `memorybank-planner` |
| "Create implementation plan for feature Y" | `memorybank-planner` |
| "Implement task TASK-001" | `memorybank-implementer` |
| "Execute the planned work" | `memorybank-implementer` |
| "Verify the plan is correct" | `memorybank-verifier` (user-initiated) |
| "Check if implementation meets standards" | `memorybank-verifier` (user-initiated) |
| "Audit the task files" | `memorybank-verifier` (user-initiated) |

#### Recommended Workflow

1. Run `memorybank-planner` to create a task plan
2. (Optional) Run `memorybank-verifier` to validate the plan
3. Run `memorybank-implementer` to execute the plan
4. (Optional) Run `memorybank-verifier` to validate the implementation

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
│   ├── config.rs    # ConfigStore
│   └── selfupdate.rs# ReleaseProvider
├── infra/           # Shared infrastructure implementations
│   ├── fs.rs        # StdFileSystem
│   ├── http.rs      # ReqwestClient
│   ├── checksum.rs  # Sha2Checksum
│   └── logger.rs    # ColoredLogger
├── content/         # Content domain (install, update, check)
├── skills/          # Skills domain (setup-skills, skills list)
├── config/          # Config domain
├── selfupdate/      # Self-update domain
└── migration/       # Migration domain
```

### Key Design Decisions

1. **Domain-based modular monolith** - Organized by business domains, not technical layers.

2. **Dependency inversion** - Domains depend on `core/` traits, receive implementations via DI.

3. **Flat domain structure** - No nested api/domain/infrastructure inside domains.

4. **cargo-dist for releases** - Automated cross-platform binary builds and GitHub Releases.

## Rust Development Policies

See `.aiassisted/guidelines/rust/rust-policy-guide.md` for detailed Rust policies including:

- **Zero Warning Policy** - All code must compile with zero warnings
- **Static Dispatch** - Prefer generics over `dyn` traits
- **Minimal Arc Usage** - Use Arc only for actual concurrent sharing
- **Comprehensive Testing** - Unit tests for all modules (positive + negative)
- **Integration Testing** - Workflow tests in `tests/` directory

Quick verification:
```bash
cargo check 2>&1 | grep -c warning  # Must be 0
cargo test                           # All tests must pass
```

## Content Organization

The `.aiassisted/` directory contains:

- `guidelines/` - Architecture patterns, documentation standards, language-specific guides
- `instructions/` - AI agent behavior rules and constraints
- `prompts/` - Reusable prompt templates (e.g., commit messages)
- `skills/` - Pre-built skills for OpenCode and Claude Code
- `config/` - Configuration documentation

## Workflow for Updating Guidelines

1. Edit files in `.aiassisted/`
2. Run `make update-version` to regenerate manifest
3. Run `make test` to verify
4. Commit changes

## Git Commit Policy

This project follows [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/) specification for all commit messages.

### Commit Message Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Allowed Types

- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that do not affect the meaning of the code (white-space, formatting, etc)
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing tests or correcting existing tests
- **build**: Changes that affect the build system or external dependencies
- **ci**: Changes to CI configuration files and scripts
- **chore**: Other changes that don't modify src or test files
- **revert**: Reverts a previous commit

### Commit Rules

1. **Subject Line**:
   - Use the imperative mood ("add" not "added", "change" not "changed")
   - No period at the end
   - Keep it short (preferably under 50 chars, max 72)

2. **Body** (Optional):
   - Use the imperative mood
   - Wrap lines at 72 characters
   - Explain *what* and *why* vs. *how*

3. **Footer** (Optional):
   - Reference issues (e.g., `Closes #123`)
   - Mention breaking changes

4. **Breaking Changes**:
   - Append a `!` after the type/scope, e.g., `feat!: ...` or `feat(api)!: ...`
   - OR include a footer with `BREAKING CHANGE: <description>`

### Commit Examples

**Feature:**
```
feat(templates): add recursive directory copying for init command
```

**Bug Fix:**
```
fix(config): prevent panic when home directory is not found
```

**Breaking Change:**
```
feat(api)!: remove deprecated v1 endpoints

BREAKING CHANGE: The /v1/* endpoints have been removed. Use /v2/* instead.
```

**Documentation:**
```
docs: update CLAUDE.md with git commit policy
```

**Tests:**
```
test(templates): add 81 comprehensive unit tests for templates domain
```

**Multiple Changes:**
```
feat(templates): complete placeholder implementations

- Implement recursive copy for templates init
- Add smart sync with modification time comparison
- Implement SHA256-based diffing

Closes #8
```

### When Committing

Always include the co-authored-by tag when commits are made with AI assistance:

```
feat(domain): implement new feature

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

For detailed instructions, see `.aiassisted/instructions/conventional-commits.instructions.md`.
