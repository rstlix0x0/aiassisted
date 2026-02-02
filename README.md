# aiassisted

**Embed AI engineering guidelines directly into your projects.**

A Rust CLI tool that embeds a standardized `.aiassisted/` directory containing curated guidelines, instructions, and prompts that AI assistants can reference for consistent, context-aware assistance.

[![Release](https://img.shields.io/github/v/release/rstlix0x0/aiassisted)](https://github.com/rstlix0x0/aiassisted/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

## What is This?

`aiassisted` is a CLI tool that helps you maintain consistency when working with AI assistants by embedding a knowledge base directly into your projects.

**The Problem:**
- AI assistants lack context about your team's coding standards
- Repetitive explanations of architectural decisions
- Inconsistent AI suggestions that don't match your conventions
- No shared knowledge across team members using AI

**The Solution:**
- Embed a `.aiassisted/` directory with your team's guidelines and standards
- Reference these files when working with AI (via `@` mentions or pre-configured skills)
- AI reads your guidelines and provides context-aware assistance
- All team members share the same knowledge base

## Quick Start

### Installation

Download and install the latest release:

**macOS / Linux:**
```bash
curl -sSL https://github.com/rstlix0x0/aiassisted/releases/latest/download/aiassisted-installer.sh | sh
```

**Manual Installation:**

1. Download the binary for your platform from [Releases](https://github.com/rstlix0x0/aiassisted/releases)
2. Extract and move to your PATH:
   ```bash
   tar -xzf aiassisted-*.tar.xz
   sudo mv aiassisted /usr/local/bin/
   ```

### Usage

```bash
# Install .aiassisted/ directory to your project
cd my-project
aiassisted install

# Check what's inside
ls .aiassisted/
# guidelines/  instructions/  prompts/  templates/  manifest.json

# Setup AI skills (optional but recommended)
aiassisted skills setup   # Creates /git-commit, /review-rust, /memorybank-setup skills

# List available skills
aiassisted skills list    # Shows installed skills

# Update skills when source changes
aiassisted skills update  # Syncs only changed files

# Setup AI agents (compile to platform-specific format)
aiassisted agents                              # List available agents
aiassisted agents setup --platform claude-code # Compile for Claude Code
aiassisted agents setup --platform opencode    # Compile for OpenCode

# Check for updates
aiassisted check

# Update to latest guidelines
aiassisted update
```

## What's Inside `.aiassisted/`?

The `.aiassisted/` directory contains curated knowledge organized into:

### 📚 Guidelines (`guidelines/`)

**Architecture Patterns:**
- `architecture/algebraic-data-types.md` - ADT design patterns
- `architecture/builder-pattern.md` - Builder pattern guide
- `architecture/factory-pattern.md` - Factory pattern guide
- `architecture/modular-monolith.md` - Modular monolith architecture
- `architecture/dependency-management.md` - Dependency injection patterns

**Rust-Specific:**
- `rust/rust-adt-implementation-guide.md` - Rust ADT patterns
- `rust/rust-builder-pattern-guide.md` - Rust builder implementations
- `rust/rust-dispatch-guide.md` - Static vs dynamic dispatch
- `rust/rust-factory-pattern-guide.md` - Rust factory patterns
- `rust/rust-smart-pointers-guide.md` - Box, Rc, Arc usage
- `rust/rust-typestate-pattern-guide.md` - Type-state pattern
- `rust/microsoft-rust-guidelines.md` - Microsoft's Rust guidelines

**Documentation Standards:**
- `documentation/diataxis-guidelines.md` - Diátaxis framework
- `documentation/documentation-quality-standards.md` - Quality criteria
- `documentation/task-documentation-standards.md` - Task documentation

**AI Tool Guides:**
- `ai/agents/claude-code/` - Claude Code usage guides
- `ai/agents/opencode/` - OpenCode usage guides

### 📝 Instructions (`instructions/`)

AI behavior rules and constraints:
- `conventional-commits.instructions.md` - Commit message standards
- `rust.instructions.md` - Rust development rules
- `ai-prompt-engineering-safety-best-practices.instructions.md` - Prompt engineering
- `multi-project-memory-bank.instructions.md` - Cross-project knowledge
- `setup-agents-context.instructions.md` - Agent configuration

### 🎯 Prompts (`prompts/`)

Reusable templates:
- `git.commit.prompt.md` - Conventional commit template

### 🛠️ Skills (`skills/`)

Pre-built AI skills (slash commands):
- `git-commit/` - Conventional commit message generator
- `review-rust/` - Rust code review
- `review-codes/` - General code review
- `doc-code/` - Code documentation
- `doc-project/` - Project documentation
- `policy-rust/` - Rust coding policies
- `memorybank-setup/` - Memory bank initialization

### 🤖 Agents (`agents/`)

Platform-agnostic agent definitions that compile to tool-specific formats:
- `code-explorer/` - Fast codebase exploration (read-only, fast model)
- `code-reviewer/` - Code quality and security review (read-only, balanced model)
- `memorybank-planner/` - Creates task plans for memory bank sub-projects (read-write, capable model)
- `memorybank-implementer/` - Executes planned tasks with progress tracking (read-write, capable model)
- `memorybank-verifier/` - Validates planner and implementer work results (read-only, capable model)

Agents are defined with YAML frontmatter and compiled to:
- **Claude Code**: TOML config with `disallowedTools` and `model` fields
- **OpenCode**: JSON config with `tools` restrictions and full model IDs

## How to Use `.aiassisted/` with AI

The `.aiassisted/` directory provides reference documentation that you explicitly provide to AI assistants. Here are the main approaches:

### 1. Direct File References

Reference specific guidelines when asking questions:

```
You: @.aiassisted/guidelines/rust/rust-adt-implementation-guide.md
     Review this code for ADT best practices

AI: [Reads the referenced file]
    Based on the ADT implementation guide:
    - ✓ Proper enum usage
    - ⚠ Consider making Error non-exhaustive (section 3.2)
```

### 2. Using Skills (Slash Commands)

Create pre-configured skills that automatically load guidelines:

```bash
aiassisted skills setup
# Creates .claude/skills/git-commit/SKILL.md
# Creates .claude/skills/review-rust/SKILL.md
# Creates .claude/skills/memorybank-setup/SKILL.md
```

**Usage:**
```
You: /review-rust

AI: [Skill automatically loads configured Rust guidelines]
    Reviewing against project guidelines...
    - ✓ Error handling follows dispatch patterns
    - ⚠ Consider ADT instead of Option<Box<dyn Trait>>
```

### 3. Memory Bank Setup

Initialize a memory bank for maintaining context across sessions:

```
You: /memorybank-setup

AI: [Skill loads memory bank setup instructions]
    Creating .memory-bank/ structure...
    - workspace/ for shared context
    - sub-projects/ for project-specific context
    - templates/ for documentation templates
```

**Usage:**
```
You: /memorybank-setup

AI: [Creates memory bank directory structure]
    Setting up multi-project memory bank...
    Created .memory-bank/README.md
    Created .memory-bank/current-context.md
    Created .memory-bank/workspace/...
```

### 4. Team Consistency

Install the same `.aiassisted/` across all team projects:

```bash
# All team members run in their projects
aiassisted install
aiassisted skills setup
```

Now everyone references the same guidelines, ensuring consistent AI assistance across the team.

## Use Cases

### Code Reviews

```
You: /review-rust

AI: Reviewing against .aiassisted/guidelines/rust/:
    1. ✓ ADT pattern correctly used
    2. ⚠ Error handling could follow rust-dispatch-guide.md
    3. ✓ Builder pattern matches rust-builder-pattern-guide.md
```

### Commit Messages

```
You: /git-commit

AI: [Loads .aiassisted/prompts/git.commit.prompt.md]

    feat(auth): add OAuth2 login support

    - Implement authorization code flow
    - Add token refresh mechanism
```

### Architecture Questions

```
You: @ai-knowledge-architecture
     How should I structure a new payment module?

AI: [References .aiassisted/guidelines/architecture/modular-monolith.md]

    Based on your modular monolith guidelines:
    - Create src/modules/payments/
    - Separate domain, service, repository layers
    - Use factory pattern for DI
```

### Documentation

```
You: @.aiassisted/guidelines/documentation/diataxis-guidelines.md
     Generate API docs for this module

AI: Following your Diátaxis framework...
    [Creates properly structured documentation]
```

## Commands

### Content Management

```bash
# Install .aiassisted/ directory
aiassisted install [--path=DIR]

# Check for updates
aiassisted check [--path=DIR]

# Update to latest version
aiassisted update [--path=DIR] [--force]

# Update CLI binary itself
aiassisted self-update
```

### AI Skills

```bash
# Setup skills (slash commands)
aiassisted skills setup [--tool=auto|claude|opencode] [--dry-run] [--force]

# List available skills
aiassisted skills list [--tool=auto|claude|opencode]

# Update installed skills (sync changes)
aiassisted skills update [--tool=auto|claude|opencode] [--dry-run] [--force]
```

**Note:** `setup-skills` is deprecated. Use `skills setup` instead.

### AI Agents

```bash
# List available agents
aiassisted agents

# Compile and install agents for a platform
aiassisted agents setup --platform claude-code [--dry-run] [--force]
aiassisted agents setup --platform opencode [--dry-run] [--force]

# Update installed agents (sync changes)
aiassisted agents update --platform claude-code [--dry-run] [--force]
aiassisted agents update --platform opencode [--dry-run] [--force]
```

**Agent compilation:**
- Agents are defined in `.aiassisted/agents/{name}/AGENT.md` with YAML frontmatter
- `capabilities: read-only` → restricts write/edit tools
- `model-tier: fast|balanced|capable` → maps to platform-specific models
- `skills: [...]` → attaches skills (Claude Code only)

### Configuration

```bash
# Show configuration
aiassisted config show

# Get specific value
aiassisted config get <key>

# Edit in $EDITOR
aiassisted config edit

# Reset to defaults
aiassisted config reset

# Show config path
aiassisted config path
```

### Utility

```bash
# Show version
aiassisted version

# Show help
aiassisted help
```

## Configuration

Configuration is stored in `~/.aiassisted/config.toml`:

```toml
default_tool = "auto"     # auto, claude, opencode
verbosity = 1             # 0=quiet, 1=normal, 2=debug
auto_update = true        # Check for updates on install
prefer_project = true     # Use project templates over global
```

**Edit configuration:**
```bash
aiassisted config edit
```

## Skills Customization

Skills are copied from `.aiassisted/skills/` to your tool's skills directory:
- Claude Code: `.claude/skills/`
- OpenCode: `.opencode/skills/`

**Customize skills:**
```bash
# Edit a skill directly
vim .aiassisted/skills/git-commit/SKILL.md

# Update installed skills with your changes
aiassisted skills update

# Commit custom skills for your team
git add .aiassisted/skills/
git commit -m "feat: customize AI skills"
```

**Sync workflow:**
```bash
# Check what would be updated
aiassisted skills update --dry-run

# Apply updates
aiassisted skills update

# Force update all files
aiassisted skills update --force
```

## Examples

### Example 1: New Rust Project

```bash
cd ~/my-rust-project

# Install guidelines
aiassisted install

# Setup AI skills
aiassisted skills setup

# Now you can use:
# - /git-commit for standardized commits
# - /review-rust for Rust code reviews
# - /memorybank-setup for initializing memory bank
# - @.aiassisted/guidelines/rust/* for Rust questions
```

### Example 2: Keeping Guidelines Updated

```bash
# Check for updates
aiassisted check

# Update if available
aiassisted update

# Your skills and agents automatically use the updated guidelines
```

### Example 3: Team Onboarding

```bash
# New team member clones project
git clone https://github.com/company/project
cd project

# Install CLI and setup (one command)
curl -sSL https://github.com/rstlix0x0/aiassisted/releases/latest/download/aiassisted-installer.sh | sh

# .aiassisted/ already in the project
aiassisted skills setup

# Ready to use with team's standards
```

## Architecture

Built as a domain-based modular monolith in Rust:

```
src/
├── agents/        # Agent compilation (setup, update, list)
├── content/       # Install, update, check
├── skills/        # Skills management (setup, list, update)
├── config/        # Configuration
└── selfupdate/    # Binary updates
```

**Key principles:**
- Dependency inversion (traits)
- Static dispatch (generics)
- Zero warnings policy
- Comprehensive testing (176 unit tests + 27 integration tests)

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed architecture documentation.

## Features

✅ **Complete Feature Set:**
- Content management (install, update, check)
- Skills system with unified commands (setup, list, update)
- Agents system with platform-specific compilation
- Incremental skill/agent updates with SHA256 diffing
- Configuration management
- Self-update capability
- Cross-platform binaries (Linux, macOS, Windows)
- SHA256 checksum verification
- Smart incremental updates
- Tool auto-detection (Claude Code, OpenCode)

**Available Skills:**
- `git-commit` - Conventional commit messages
- `review-rust` - Rust code review
- `review-codes` - General code review
- `doc-code` - Code documentation
- `doc-project` - Project documentation
- `policy-rust` - Rust coding policies
- `memorybank-setup` - Initialize memory bank structure

**Available Agents:**
- `code-explorer` - Fast codebase exploration (read-only, fast model)
- `code-reviewer` - Code quality and security review (read-only, balanced model)
- `memorybank-planner` - Creates task plans for memory bank workflows (read-write, capable model)
- `memorybank-implementer` - Executes planned tasks with progress tracking (read-write, capable model)
- `memorybank-verifier` - Validates planner and implementer work (read-only, capable model)

See [FEATURES.md](FEATURES.md) for comprehensive feature list.

## Platform Support

Pre-built binaries available for:
- Linux (x86_64, aarch64)
- macOS (x86_64, aarch64)
- Windows (x86_64)

Binary size: ~5MB (release build)
Startup time: <50ms

## Requirements

**Required:**
- None (static binary)

**Optional:**
- `git` - For cloning/updating the repository during development
- AI assistant (Claude Code, OpenCode, or any AI tool that can read files)

## Troubleshooting

### Command not found

```bash
# Add to PATH
export PATH="$PATH:$HOME/.local/bin"

# Or restart terminal
```

### Update shows 404

Ensure you're running the latest CLI:
```bash
aiassisted self-update
```

### Templates not found

Ensure .aiassisted/ is installed:
```bash
aiassisted install
```

## Migration from v0.1.x (Shell Version)

If you have the old shell-based version:

```bash
# The new Rust version auto-detects and migrates
aiassisted migrate

# This will:
# - Detect old installation at ~/.aiassisted/source/
# - Migrate configuration to new format
# - Create backup at ~/.aiassisted/source.backup.{timestamp}
# - Remove old files
```

## Development

**Build from source:**
```bash
git clone https://github.com/rstlix0x0/aiassisted
cd aiassisted
cargo build --release

# Binary at: target/release/aiassisted
```

**Run tests:**
```bash
cargo test                    # All tests
cargo test --test integration # Integration tests only
./scripts/smoke-test.sh       # End-to-end smoke tests
```

**See also:**
- [ARCHITECTURE.md](ARCHITECTURE.md) - Architecture documentation
- [FEATURES.md](FEATURES.md) - Complete feature list
- [CLAUDE.md](CLAUDE.md) - Development guidelines for Claude Code

## Contributing

Contributions welcome! Please:

1. Follow the architecture (see ARCHITECTURE.md)
2. Write tests (unit + integration)
3. Ensure zero warnings (`cargo check`)
4. Use conventional commits
5. Update documentation

See [CLAUDE.md](CLAUDE.md) for development guidelines.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Links

- **Repository:** https://github.com/rstlix0x0/aiassisted
- **Releases:** https://github.com/rstlix0x0/aiassisted/releases
- **Issues:** https://github.com/rstlix0x0/aiassisted/issues

---

**Built for AI-assisted development workflows.**

Embed knowledge • Maintain consistency • Empower AI assistants
