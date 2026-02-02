# Features

Complete feature reference for `aiassisted` v0.5.0

## Overview

`aiassisted` is a Rust CLI tool that embeds AI engineering guidelines into your projects. All features are tested and production-ready.

**Quick Stats:**
- **20 Commands** - Full CLI interface
- **176 Unit Tests + 27 Integration Tests** - Comprehensive test coverage
- **5 Platforms** - Linux, macOS, Windows (x64 + ARM64)
- **Zero Warnings** - Strict code quality standards
- **<50ms Startup** - Native Rust performance

---

## Core Commands

### version
**Show version information**

```bash
aiassisted version
```

**Output:**
```
aiassisted 0.2.0
```

**Options:**
- `-V, --version` - Short form
- `-v` - Verbose output (shows commit hash)
- `-vv` - Debug output (shows build info)

---

### help
**Show help for all commands**

```bash
aiassisted help
aiassisted <command> --help
```

**Features:**
- Lists all available commands
- Shows global options
- Per-command detailed help

---

## Content Domain

Manage `.aiassisted/` directory installation and updates.

### install
**Install .aiassisted directory to a project**

```bash
aiassisted install
aiassisted install --path=/path/to/project
```

**What it does:**
1. Downloads `manifest.json` from GitHub
2. Parses list of 43 files with SHA256 checksums
3. Downloads all files from `https://raw.githubusercontent.com/rstlix0x0/aiassisted/main/.aiassisted/`
4. Verifies checksums for each file
5. Creates directory structure:
   - `guidelines/` - Architecture and language guides
   - `instructions/` - AI behavior rules
   - `prompts/` - Reusable templates
   - `templates/` - Skill and agent templates
   - `manifest.json` - Local copy for update checking

**Options:**
- `--path=DIR` - Target directory (default: current)
- `-v, --verbose` - Show detailed progress
- `-q, --quiet` - Only show errors

**Output:**
```
[INFO] Installing .aiassisted to .
[INFO] Downloading manifest...
[INFO] Manifest loaded: version 87d2583, 43 files
[INFO] Downloading files...
[OK] Successfully installed 43 files to ./.aiassisted
```

**Files installed:**
- 27 guideline files
- 5 instruction files
- 1 prompt file
- 10 template files
- 1 manifest

---

### check
**Check if updates are available**

```bash
aiassisted check
aiassisted check --path=/path/to/project
```

**What it does:**
1. Loads local `manifest.json` (shows current version)
2. Fetches remote `manifest.json` from GitHub
3. Compares version strings (git commit hashes)
4. Reports if update available

**Options:**
- `--path=DIR` - Project directory to check
- `-v, --verbose` - Show version details

**Output when up-to-date:**
```
[INFO] Checking for updates in .
[INFO] Checking for updates...
[INFO] Local: v87d2583, Remote: v87d2583
[OK] No updates available. You're up to date!
```

**Output when outdated:**
```
[INFO] Local: v87d2583, Remote: va1b2c3d
[WARN] Update available!
[INFO] Run 'aiassisted update' to update
```

---

### update
**Update existing .aiassisted installation**

```bash
aiassisted update
aiassisted update --force
aiassisted update --path=/path/to/project
```

**What it does:**
1. Checks if update available (same as `check`)
2. Downloads remote manifest
3. Calculates diff (new files, modified files)
4. Shows diff and prompts for confirmation
5. Downloads only changed files
6. Verifies checksums
7. Updates local manifest

**Options:**
- `--force` - Skip confirmation prompt
- `--path=DIR` - Target directory
- `-v, --verbose` - Show detailed progress

**Output:**
```
[INFO] Updating .aiassisted in .
[INFO] Checking for updates...
[INFO] Local: v87d2583, Remote: va1b2c3d
[INFO] Changes: 3 new, 5 modified
[INFO] Apply these changes? [y/N]:
```

**Efficiency:**
- Only downloads changed files (SHA256-based diffing)
- Saves bandwidth on partial updates
- Verifies integrity with checksums

---

## Skills Domain

Manage AI skills for Claude Code and OpenCode.

### skills setup
**Set up AI skills (copy to tool directory)**

```bash
aiassisted skills setup
aiassisted skills setup --tool=claude
aiassisted skills setup --tool=opencode
aiassisted skills setup --dry-run
aiassisted skills setup --force
```

**What it does:**
1. Auto-detects AI tool (Claude Code or OpenCode) or uses `--tool`
2. Finds skills in `.aiassisted/skills/`
3. Copies skill directories to tool's skills folder:
   - Claude Code: `.claude/skills/`
   - OpenCode: `.opencode/skills/`
4. Preserves directory structure (including `references/` subdirectories)

**Options:**
- `--tool=TYPE` - Specify tool: `auto` (default), `claude`, `opencode`
- `--dry-run` - Preview what would be copied
- `--force` - Overwrite existing skills

**Output:**
```
[INFO] Auto-detected tool: claude
[INFO] Setting up skills for claude
[INFO] Found 7 skill(s)
[OK] Copied: git-commit
[OK] Copied: review-rust
[OK] Copied: doc-code
[OK] Copied: doc-project
[OK] Copied: review-codes
[OK] Copied: policy-rust
[OK] Copied: memorybank-setup
[OK] Setup complete: 7 copied, 0 skipped
```

**Available skills:**
- `git-commit` - Conventional commit messages
- `review-rust` - Rust code review
- `doc-code` - Code documentation
- `doc-project` - Project documentation
- `review-codes` - General code review
- `policy-rust` - Rust coding policies
- `memorybank-setup` - Initialize memory bank structure

**Deprecation notice:** The `setup-skills` command is deprecated. Use `skills setup` instead.

---

### skills list
**List available skills**

```bash
aiassisted skills list
aiassisted skills list --tool=claude
```

**What it does:**
1. Lists skills available in `.aiassisted/skills/`
2. Shows installation status for each skill

**Options:**
- `--tool=TYPE` - Specify tool to check installation status

**Output:**
```
[INFO] Skills source: .aiassisted/skills
[INFO] Target directory: .claude/skills
[INFO]
[INFO] Available skills (7):
[INFO]
[INFO]   - doc-code
[INFO]   - doc-project
[INFO]   - git-commit [installed]
[INFO]   - memorybank-setup
[INFO]   - policy-rust
[INFO]   - review-codes
[INFO]   - review-rust [installed]
```

---

### skills update
**Update installed skills (sync changes from source)**

```bash
aiassisted skills update
aiassisted skills update --tool=claude
aiassisted skills update --dry-run
aiassisted skills update --force
```

**What it does:**
1. Auto-detects AI tool or uses `--tool`
2. Compares source and target skill files using SHA256 checksums
3. Identifies new, modified, unchanged, and removed skills
4. Copies only changed files (incremental update)

**Options:**
- `--tool=TYPE` - Specify tool: `auto` (default), `claude`, `opencode`
- `--dry-run` - Preview what would be updated without making changes
- `--force` - Force update all files regardless of checksum

**Output (changes detected):**
```
[INFO] Auto-detected tool: claude
[INFO] Source: .aiassisted/skills
[INFO] Target: .claude/skills
[INFO] Analyzing skills...
[INFO] Summary: 1 new, 2 updated, 3 unchanged, 1 removed
[INFO]
[INFO] Skills status:
[INFO]   + memorybank-setup (new, 1 file)
[INFO]   ~ git-commit (0 new, 1 modified)
[INFO]   = doc-code (unchanged)
[INFO]   = doc-project (unchanged)
[INFO]   = review-codes (unchanged)
[INFO]   ~ policy-rust (1 new, 0 modified)
[INFO]   - old-deprecated-skill (removed from source)
[INFO]
[INFO] Files to update:
[INFO]   + .claude/skills/memorybank-setup/SKILL.md
[INFO]   ~ .claude/skills/git-commit/SKILL.md
[INFO]   + .claude/skills/policy-rust/references/clippy-lints.md
[OK] Updated 3 files across 3 skills
[INFO] Note: 1 skill(s) removed from source but still installed
```

**Output (no changes):**
```
[INFO] Summary: 0 new, 0 updated, 7 unchanged, 0 removed
[OK] All skills are up to date!
```

**Removed skills handling:**
- Removed skills are reported but NOT automatically deleted
- User must manually remove if desired
- This prevents accidental deletion of customized skills

---

## Agents Domain

Compile and manage platform-agnostic AI agents.

### agents
**List available agents**

```bash
aiassisted agents
```

**What it does:**
1. Scans `.aiassisted/agents/` for directories containing `AGENT.md`
2. Parses YAML frontmatter to extract agent descriptions
3. Lists all available agents with their descriptions

**Output:**
```
[INFO] Agents source: .aiassisted/agents
[INFO]
[INFO] Available agents (2):
[INFO]
[INFO]   code-explorer - Fast codebase exploration for finding files, searching code, and understanding project structure.
[INFO]   code-reviewer - Reviews code for quality, security, and best practices.
```

---

### agents setup
**Compile and install agents for a platform**

```bash
aiassisted agents setup --platform claude-code
aiassisted agents setup --platform opencode
aiassisted agents setup --platform claude-code --dry-run
aiassisted agents setup --platform claude-code --force
```

**What it does:**
1. Discovers agents in `.aiassisted/agents/`
2. Parses each `AGENT.md` file (YAML frontmatter + markdown body)
3. Validates agent specification (name, description, skill references)
4. Compiles to platform-specific format
5. Writes compiled files to target directory:
   - Claude Code: `.claude/agents/{name}/agent.toml` + `prompt.md`
   - OpenCode: `.opencode/agents/{name}/agent.json` + `prompt.md`

**Options:**
- `--platform=TYPE` - Target platform: `claude-code` or `opencode` (required)
- `--dry-run` - Preview what would be compiled
- `--force` - Overwrite existing agents

**Output:**
```
[INFO] Setting up agents for claude-code
[INFO] Source: .aiassisted/agents
[INFO] Target: .claude/agents
[INFO] Found 2 agent(s)
[OK] Compiled: code-explorer
[OK] Compiled: code-reviewer
[INFO]
[OK] Setup complete: 2 compiled, 0 skipped, 0 errors
```

**AGENT.md format:**
```markdown
---
name: code-explorer
description: Fast codebase exploration
capabilities: read-only
model-tier: fast
skills:
  - review-codes
---

You are a codebase exploration specialist...
```

**Compilation mapping:**

| Field | Claude Code | OpenCode |
|-------|-------------|----------|
| `capabilities: read-only` | `disallowedTools = ["Write", "Edit"]` | `"tools": {"write": false, "edit": false}` |
| `model-tier: fast` | `model = "haiku"` | `"model": "anthropic/claude-3-5-haiku-20241022"` |
| `model-tier: balanced` | (default, omitted) | `"model": "anthropic/claude-sonnet-4-20250514"` |
| `model-tier: capable` | `model = "opus"` | `"model": "anthropic/claude-opus-4-20250514"` |
| `skills: [...]` | `skills = [...]` | (ignored) |

---

### agents update
**Update installed agents (sync changes from source)**

```bash
aiassisted agents update --platform claude-code
aiassisted agents update --platform opencode
aiassisted agents update --platform claude-code --dry-run
aiassisted agents update --platform claude-code --force
```

**What it does:**
1. Compares source agents with installed compiled agents using SHA256 checksums
2. Identifies new, modified, unchanged, and removed agents
3. Recompiles only changed agents (incremental update)

**Options:**
- `--platform=TYPE` - Target platform: `claude-code` or `opencode` (required)
- `--dry-run` - Preview what would be updated without making changes
- `--force` - Force update all agents regardless of checksum

**Output (changes detected):**
```
[INFO] Updating agents for claude-code
[INFO] Source: .aiassisted/agents
[INFO] Target: .claude/agents
[INFO] Analyzing agents...
[INFO] Summary: 0 new, 1 modified, 1 unchanged, 0 removed
[INFO]
[INFO] Agents status:
[INFO]   ~ code-explorer (modified: config + prompt)
[INFO]   = code-reviewer (unchanged)
[INFO]
[INFO] Agents to update:
[INFO]   ~ code-explorer
[OK] Updated 1 agent(s)
```

**Output (no changes):**
```
[INFO] Summary: 0 new, 0 modified, 2 unchanged, 0 removed
[OK] All agents are up to date!
```

---

## Config Domain

Manage user configuration.

**Config file:** `~/.aiassisted/config.toml`

### config show
**Show current configuration**

```bash
aiassisted config show
```

**Output:**
```
[INFO] Current configuration:

  default_tool      = auto
  verbosity         = 1
  auto_update       = true
  prefer_project    = true

[INFO] Configuration file: /Users/user/.aiassisted/config.toml
```

---

### config get
**Get specific configuration value**

```bash
aiassisted config get default_tool
aiassisted config get verbosity
```

**Output:**
```
auto
```

**Available keys:**
- `default_tool` - Tool type: auto, claude, opencode
- `verbosity` - Logging level: 0 (quiet), 1 (normal), 2 (debug)
- `auto_update` - Check updates on install: true/false
- `prefer_project` - Use project templates first: true/false

---

### config edit
**Edit configuration in $EDITOR**

```bash
aiassisted config edit
```

**What it does:**
Opens `~/.aiassisted/config.toml` in `$EDITOR` (or `vim` fallback).

---

### config reset
**Reset configuration to defaults**

```bash
aiassisted config reset
aiassisted config reset --force
```

**What it does:**
Resets all settings to default values.

**Options:**
- `--force` - Skip confirmation prompt

**Defaults:**
```toml
default_tool = "auto"
verbosity = 1
auto_update = true
prefer_project = true
```

---

### config path
**Show configuration file path**

```bash
aiassisted config path
```

**Output:**
```
/Users/user/.aiassisted/config.toml
```

---

## Self-Update Domain

Update the CLI binary itself.

### self-update
**Update the CLI binary**

```bash
aiassisted self-update
```

**What it does:**
1. Queries GitHub releases API for latest version
2. Compares with current version
3. Detects platform (OS + architecture)
4. Downloads correct binary for platform
5. Extracts and replaces current binary
6. Verifies new version

**Platform detection:**
- OS: Linux, macOS, Windows
- Architecture: x86_64, aarch64

**Output:**
```
[INFO] Checking for updates...
[INFO] Current: v0.2.0
[INFO] Latest: v0.2.1
[INFO] Downloading aiassisted-aarch64-apple-darwin...
[INFO] Extracting binary...
[OK] Updated to v0.2.1
```

---

## Migration Domain

Migrate from old shell-based version.

### migrate
**Migrate from shell-based v0.1.x**

```bash
aiassisted migrate
```

**What it does:**
1. Detects old installation at `~/.aiassisted/source/`
2. Reads old TOML config format
3. Maps old settings to new format:
   - `[general].default_runtime` → `default_tool`
   - `[general].verbosity` → `verbosity`
   - `[install].auto_update` → `auto_update`
   - `[templates].prefer_project` → `prefer_project`
4. Creates backup at `~/.aiassisted/source.backup.{timestamp}`
5. Removes old installation files
6. Writes new config

**Output:**
```
[INFO] Checking for old shell-based installation...
[INFO] Old installation detected:
[INFO]   - Config: ~/.aiassisted/config.toml
[INFO]   - Source: ~/.aiassisted/source
[INFO] Migrating configuration...
[INFO] Backup created: ~/.aiassisted/source.backup.20260201_120000
[OK] Migration complete!
```

**Safe:**
- Non-destructive (creates backup)
- Graceful error handling
- Idempotent (can run multiple times)

---

## Global Options

Available for all commands:

- `-v, --verbose` - Verbose output (info level)
- `-vv` - Debug output (debug level)
- `-q, --quiet` - Quiet mode (errors only)
- `-h, --help` - Show help
- `-V, --version` - Show version

---

## Feature Categories

### Content Management
- ✅ Install .aiassisted directory
- ✅ Check for updates
- ✅ Update changed files only
- ✅ SHA256 checksum verification
- ✅ Smart incremental updates

### Skills System
- ✅ Auto-detect AI tool (Claude Code, OpenCode)
- ✅ Manual tool selection
- ✅ Copy skills to tool directory
- ✅ Preserve skill directory structure
- ✅ Force overwrite option
- ✅ Dry-run mode
- ✅ List available skills
- ✅ Incremental update (SHA256-based diffing)
- ✅ Unified command structure (`skills setup/list/update`)

### Agents System
- ✅ Platform-agnostic agent definitions (YAML frontmatter)
- ✅ Compile to Claude Code format (TOML)
- ✅ Compile to OpenCode format (JSON)
- ✅ Capability mapping (read-only → tool restrictions)
- ✅ Model tier mapping (fast/balanced/capable)
- ✅ Skill attachment (Claude Code only)
- ✅ Agent validation (name, description, skill references)
- ✅ Incremental update (SHA256-based diffing)
- ✅ Force overwrite option
- ✅ Dry-run mode

### Configuration
- ✅ TOML-based configuration
- ✅ Show/get/edit/reset commands
- ✅ Validation of values
- ✅ Default fallbacks

### Self-Update
- ✅ GitHub releases integration
- ✅ Platform detection
- ✅ Binary download and replacement
- ✅ Version comparison

### Migration
- ✅ Auto-detect old installation
- ✅ Config migration
- ✅ Backup creation
- ✅ Safe cleanup

---

## Performance

| Operation | Time | Notes |
|-----------|------|-------|
| Startup | <50ms | Cold start |
| version | <10ms | Instant |
| install (42 files) | ~3-5s | Network dependent |
| check | ~1-2s | HTTP request |
| update | ~2-4s | Only changed files |
| skills setup | <100ms | Local copy operation |
| skills list | <50ms | Directory scan |
| skills update | <100ms | SHA256 comparison + copy |
| agents | <50ms | Directory scan + parse |
| agents setup | <100ms | YAML parse + compile |
| agents update | <100ms | SHA256 comparison + compile |

**Memory usage:** <20MB peak (during install)

---

## Platform Support

Pre-built binaries for all major platforms:

| Platform | Architecture | Binary | Status |
|----------|--------------|--------|--------|
| Linux | x86_64 | aiassisted-x86_64-unknown-linux-gnu.tar.xz | ✅ |
| Linux | aarch64 | aiassisted-aarch64-unknown-linux-gnu.tar.xz | ✅ |
| macOS | x86_64 | aiassisted-x86_64-apple-darwin.tar.xz | ✅ |
| macOS | aarch64 | aiassisted-aarch64-apple-darwin.tar.xz | ✅ |
| Windows | x86_64 | aiassisted-x86_64-pc-windows-msvc.zip | ✅ |

All binaries include:
- Main executable
- LICENSE
- README.md

Binary size: ~5MB (release build, stripped)

---

## Testing

Comprehensive test coverage across all domains:

### Test Statistics

| Type | Count | Coverage |
|------|-------|----------|
| Unit tests | 176 | Core logic, error handling |
| Integration tests | 27 | Multi-module workflows |
| **Total** | **203** | **Complete coverage** |

### Test Breakdown by Domain

| Domain | Unit | Integration | Total |
|--------|------|-------------|-------|
| agents | 38 | 0 | 38 |
| config | 15 | 14 | 29 |
| content | 40 | 8 | 48 |
| core/infra | 20 | 0 | 20 |
| migration | 21 | 5 | 26 |
| selfupdate | 30 | 0 | 30 |
| skills | 12 | 0 | 12 |
| **Total** | **176** | **27** | **203** |

### Smoke Tests

End-to-end validation via `scripts/smoke-test.sh`:
- Tests actual binary (not unit tests)
- Uses real GitHub API
- Tests complete workflows
- 19 smoke tests covering all commands

**Run smoke tests:**
```bash
./scripts/smoke-test.sh --binary ./target/release/aiassisted
```

---

## Security

- ✅ **No sudo required** - User directory only
- ✅ **SHA256 verification** - All downloads checksummed
- ✅ **HTTPS only** - All GitHub downloads use TLS
- ✅ **No code execution** - No eval or shell injection
- ✅ **Sandboxed** - Only touches target directory
- ✅ **Safe updates** - Backup before modifications

---

## Code Quality

- ✅ **Zero warnings** - `cargo check` produces 0 warnings
- ✅ **Static dispatch** - Generics over `dyn` traits
- ✅ **Minimal Arc** - No Arc in codebase
- ✅ **Dependency inversion** - All domains depend on traits
- ✅ **Error handling** - Comprehensive Result<T> usage
- ✅ **Domain separation** - Clear module boundaries

---

## Summary

**v0.5.0 Feature Completion:**

| Category | Features | Status |
|----------|----------|--------|
| Core commands | 4 | ✅ Complete |
| Content domain | 3 | ✅ Complete |
| Skills domain | 3 | ✅ Complete |
| Agents domain | 3 | ✅ Complete |
| Config domain | 5 | ✅ Complete |
| Self-update | 1 | ✅ Complete |
| Migration | 1 | ✅ Complete |
| **Total** | **20 commands** | **✅ Production Ready** |

**Additional features:**
- 176 unit tests + 27 integration tests
- 5 platform binaries
- cargo-dist automated releases
- Migration from shell version
- 7 built-in skills
- 2 built-in agents
- Unified skills command structure (`skills setup/list/update`)
- Unified agents command structure (`agents setup/update`)

---

**All features tested and production-ready.**

For architecture details, see [ARCHITECTURE.md](ARCHITECTURE.md)

For usage examples, see [README.md](README.md)
