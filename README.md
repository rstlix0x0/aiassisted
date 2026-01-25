# aiassisted

**Embed AI engineering guidelines directly into your projects.**

`aiassisted` is a CLI tool that embeds a standardized `.aiassisted/` directory into your projects. This directory contains curated guidelines, instructions, and prompts that you can explicitly reference when working with AI assistants—ensuring consistent, context-aware AI assistance across your team.

## What is `.aiassisted/`?

The `.aiassisted/` directory serves as a **knowledge base** that you can reference when working with AI assistants. By embedding this directory in your project, you provide:

- **Guidelines** - Architecture patterns, documentation standards, and language-specific best practices
- **Instructions** - Detailed AI agent behavior rules and prompt engineering guidelines  
- **Prompts** - Reusable templates for common tasks like commit messages and code reviews
- **Templates** - Customizable skills and agents for AI coding assistants

With `.aiassisted/` in your project, you can:
- Reference guidelines explicitly when asking questions or requesting code
- Use skills (like `/review-rust`) that are configured to load specific guidelines
- Point AI agents to relevant instruction files for specialized tasks
- Mention files with `@.aiassisted/guidelines/...` to provide context

**Key point:** AI assistants don't automatically read these files. You (or your configured skills/agents) explicitly reference them when needed.

## Why Use This?

**Problem:** AI assistants lack context about your team's coding standards, architectural decisions, and best practices. This leads to inconsistent code suggestions, repetitive explanations, and AI-generated code that doesn't match your team's conventions.

**Solution:** Embed a `.aiassisted/` directory containing your team's guidelines, instructions, and standards. When you need AI assistance, explicitly reference these files or use pre-configured skills/agents that load them automatically.

### Use Cases

#### 1. **Consistent Code Reviews**
Create a `/review-rust` skill that automatically loads your Rust guidelines when invoked:

```bash
aiassisted setup-skills
# Creates /review-rust skill configured to reference .aiassisted/guidelines/rust/
```

**Usage:**
```
You: /review-rust
AI: [Skill loads configured guidelines, then reviews code]
    - ✓ Error handling follows dispatch patterns
    - ⚠ Consider ADT instead of Option<Box<dyn Trait>>
```

#### 2. **Standardized Commit Messages**
Set up a `/git-commit` skill that references your commit conventions:

```bash
aiassisted setup-skills
# Creates /git-commit skill that loads commit guidelines
```

**Usage:**
```
You: /git-commit
AI: [Reads .aiassisted/prompts/git.commit.prompt.md]
    feat(auth): add OAuth2 login support
    (following your conventional commits standard)
```

#### 3. **Architecture-Aware Development**
Reference architecture guidelines when designing new features:

```
You: @.aiassisted/guidelines/architecture/modular-monolith.md
     How should I structure a payments module?
     
AI: Based on your modular monolith pattern...
    [provides architecture matching your guidelines]
```

#### 4. **Documentation Standards**
Point AI to your documentation guidelines:

```
You: @.aiassisted/guidelines/documentation/diataxis-guidelines.md
     Generate API docs for this module
     
AI: Following your Diátaxis framework...
    [creates properly structured documentation]
```

#### 5. **Multi-Project Consistency**
Install the same `.aiassisted/` across multiple projects:

```bash
for project in ~/projects/*; do
  aiassisted install --path="$project"
done
```

Now all projects share the same reference guidelines. Use `@.aiassisted/...` or skills like `/review-rust` consistently across all projects.

#### 6. **Team Onboarding**
New team members can reference `.aiassisted/` to understand conventions:

```
New Dev: @.aiassisted/guidelines/rust/rust-adt-implementation-guide.md
         How should I implement error types?
         
AI: According to your team's ADT guide...
```

### How It Works

1. **Install**: The CLI embeds `.aiassisted/` into your project directory
2. **Reference**: You (or AI skills/agents) explicitly reference `.aiassisted/` files when needed
3. **Context**: AI models read these files to understand your team's standards and patterns
4. **Apply**: AI uses the guidelines to generate better suggestions and code

**Supported AI Tools:**
- **OpenCode** - Skills and agents can reference `.aiassisted/` files
- **Claude Code** - Agents can load `.aiassisted/` as context
- **Any AI assistant** - You can point AI to specific files using `@` mentions or file paths

## Features

- **📚 AI Knowledge Base** - Embed curated guidelines that AI assistants reference automatically
- **🎯 Context-Aware AI** - AI suggestions follow your team's patterns and conventions
- **⚡ One-Command Installation** - Install via `curl` pipe to shell
- **🔄 Smart Updates** - Selective file updates using SHA256 checksums
- **🛡️ Safe Installation** - No sudo required, user directory only
- **📦 Zero Dependencies** - Pure POSIX shell (except git)
- **🎨 Skills & Agents** - Create custom AI skills and agents that reference `.aiassisted/`
- **🔍 Version Tracking** - Git commit-based versioning with automatic update detection
- **🌈 Colored Output** - Beautiful terminal output with automatic capability detection

## Requirements

- **git** - Required for installation and updates
  - macOS: `xcode-select --install`
  - Ubuntu/Debian: `sudo apt install git`
  - Fedora: `sudo dnf install git`
  - Arch: `sudo pacman -S git`

That's it! Pure POSIX shell - works everywhere.

## Quick Start

### 1. Install the CLI and Embed `.aiassisted/`

Install in one command:

```bash
curl -fsSL https://raw.githubusercontent.com/rstlix0x0/aiassisted/main/install.sh | sh
```

This will:
1. ✓ Check that `git` is installed
2. ✓ Clone the repository to `~/.aiassisted/source/aiassisted/`
3. ✓ Create a symlink at `~/.local/bin/aiassisted`
4. ✓ Add `~/.local/bin` to your PATH (if needed)
5. ✓ Set up global config at `~/.aiassisted/config.toml`
6. ✓ Install `.aiassisted/` directory to your current directory
7. ✓ Show quick usage tips

After installation, restart your terminal or run:
```bash
source ~/.bashrc  # or ~/.zshrc, depending on your shell
```

### 2. Set Up AI Skills and Agents (Optional but Recommended)

Create AI skills (slash commands) and agents that reference your `.aiassisted/` guidelines:

```bash
# Auto-detect and setup for available AI tools (OpenCode, Claude Code)
aiassisted setup-skills
aiassisted setup-agents
```

Now you can use skills like:
- `/git-commit` - Generate commit messages following conventional commits
- `/review-rust` - Review Rust code against your guidelines

And specialized agents:
- `ai-knowledge-rust` - Rust expert with access to your Rust guidelines
- `ai-knowledge-architecture` - Architecture expert familiar with your patterns

### 3. Start Using AI with Embedded Context

Your `.aiassisted/` directory is now available for AI reference. You can:

**Use skills (recommended):**
- `/git-commit` - Skill loads commit guidelines automatically
- `/review-rust` - Skill loads Rust guidelines automatically

**Reference files directly:**
- `@.aiassisted/guidelines/rust/rust-adt-implementation-guide.md` when asking about ADTs
- `@.aiassisted/guidelines/architecture/modular-monolith.md` for architecture questions
- `@.aiassisted/prompts/git.commit.prompt.md` for commit message help

**Use specialized agents:**
- `@ai-knowledge-rust` - Agent configured to reference Rust guidelines
- `@ai-knowledge-architecture` - Agent configured to reference architecture patterns

**What's available:**
- ✓ Architecture patterns (ADT, Builder, Factory, Modular Monolith)
- ✓ Rust guidelines (error handling, type system, patterns)
- ✓ Documentation standards (Diátaxis framework)
- ✓ Commit conventions (conventional commits)
- ✓ Code review standards

**You're all set!** Use `@` mentions or skills to reference `.aiassisted/` when working with AI.

## How AI Assistants Use `.aiassisted/`

The `.aiassisted/` directory provides **reference documentation** that you can explicitly provide to AI assistants. Here's how it works in practice:

### 1. Direct File References

You can reference specific guidelines when working with AI:

```
You: @.aiassisted/guidelines/rust/rust-adt-implementation-guide.md 
     Review this code for ADT best practices

AI: [Reads the referenced file]
    Based on the ADT implementation guide:
    - ✓ Proper enum usage
    - ⚠ Consider making Error non-exhaustive
    - See section "Error Handling Patterns" in the guide
```

### 2. Skills That Load Guidelines

Skills (slash commands) are pre-configured to load specific guidelines:

```markdown
<!-- In .opencode/skills/review-rust/skill.md -->
When invoked, read these files:
- .aiassisted/guidelines/rust/rust-adt-implementation-guide.md
- .aiassisted/guidelines/rust/rust-builder-pattern-guide.md
- .aiassisted/instructions/rust.instructions.md

Then review the code against these standards.
```

**Usage:**
```
You: /review-rust
AI: [Skill automatically loads the configured guidelines]
    Reviewing against project Rust guidelines...
```

### 3. Agents With Project Knowledge

Agents are configured to reference `.aiassisted/` when answering questions:

```markdown
<!-- In .opencode/agents/ai-knowledge-rust.md -->
You are a Rust expert. When answering questions, reference:
- .aiassisted/guidelines/rust/**/*.md
- .aiassisted/instructions/rust.instructions.md
```

**Usage:**
```
You: @ai-knowledge-rust How should I implement error handling?
AI: [Agent reads rust guidelines]
    According to your project's Rust guidelines...
```

### Real-World Examples

**Example 1: Explicit File Reference**
```
You: I need to add error handling. 
     @.aiassisted/guidelines/rust/rust-dispatch-guide.md
     
AI: Based on your dispatch guide, I recommend using Result<T, E> 
    with a custom error type following the pattern in section 3...
```

**Example 2: Using Skills**
```
You: /review-rust
     
AI: [Loads configured guidelines automatically]
    
    Reviewing against .aiassisted/guidelines/rust/:
    1. ✓ ADT pattern correctly used
    2. ⚠ Error handling could follow rust-dispatch-guide.md
    3. ✓ Builder pattern matches rust-builder-pattern-guide.md
```

**Example 3: Architecture Question**
```
You: @ai-knowledge-architecture 
     How should I structure a new payment module?
     
AI: [Agent references .aiassisted/guidelines/architecture/]
    
    Based on your modular-monolith.md guidelines:
    - Create src/modules/payments/
    - Separate domain, service, repository layers
    - Use factory pattern for DI (see factory-pattern.md)
```

**Example 4: Commit Message**
```
You: /git-commit
     
AI: [Loads .aiassisted/prompts/git.commit.prompt.md and 
     .aiassisted/instructions/conventional-commits.instructions.md]
     
    Based on your changes and commit conventions:
    
    feat(auth): add OAuth2 login support
    
    - Implement authorization code flow
    - Add token refresh mechanism
```

### Key Points

- **You control when guidelines are used** - via `@` mentions, skills, or agents
- **Skills automate the referencing** - pre-configured to load specific files
- **Agents are context-aware** - know which guidelines to reference for their domain
- **Files are explicit** - AI reads what you point it to, not automatically
- **Living documentation** - Update `.aiassisted/`, and all references use the latest version

## Usage

### Install to a Project

Install `.aiassisted` directory to current directory:
```bash
aiassisted install
```

Install to a specific directory:
```bash
aiassisted install --path=/path/to/project
```

### Check for Updates

Check if a new version is available:
```bash
aiassisted check
```

### Update Existing Installation

Update with confirmation prompt (shows diff):
```bash
aiassisted update
```

Update without confirmation (force):
```bash
aiassisted update --force
```

Update in a specific directory:
```bash
aiassisted update --path=/path/to/project
```

### Update the CLI Tool

Update the `aiassisted` CLI tool itself (uses `git pull`):
```bash
aiassisted self-update
```

This will pull the latest changes from the repository and show you what was updated. You can view the changelog with:
```bash
cd ~/.aiassisted/source/aiassisted
git log --oneline -10
```

### Setup AI Skills and Agents

Create customized AI skills (slash commands) and agents (custom subagents) for OpenCode and Claude Code.

#### Setup Skills (Slash Commands)

Create reusable slash commands like `/git-commit` and `/review-rust`:

```bash
# Auto-detect and setup for available tools
aiassisted setup-skills

# Setup for specific tool
aiassisted setup-skills --tool=opencode
aiassisted setup-skills --tool=claude

# Preview what would be created
aiassisted setup-skills --dry-run
```

This creates:
- **OpenCode**: `.opencode/skills/git-commit/`, `.opencode/skills/review-rust/`
- **Claude Code**: `.claude/skills/git-commit/`, `.claude/skills/review-rust/`

#### Setup Agents (Custom Subagents)

Create specialized AI agents with project-specific knowledge:

```bash
# Auto-detect and setup for available tools
aiassisted setup-agents

# Setup for specific tool
aiassisted setup-agents --tool=opencode
aiassisted setup-agents --tool=claude

# Preview what would be created
aiassisted setup-agents --dry-run
```

This creates:
- **OpenCode**: `.opencode/agents/ai-knowledge-rust/`, `.opencode/agents/ai-knowledge-architecture/`
- **Claude Code**: `.claude/agents/ai-knowledge-rust/`, `.claude/agents/ai-knowledge-architecture/`

**Note:** All skills and agents reference `.aiassisted/` files directly, so updates propagate automatically without needing to sync.

### Configuration Management

The CLI stores user preferences in `~/.aiassisted/config.toml`:

```bash
# View current configuration
aiassisted config show

# Get specific value
aiassisted config get verbosity

# Edit configuration in $EDITOR
aiassisted config edit

# Reset to defaults
aiassisted config reset

# Show config file path
aiassisted config path
```

#### Configuration Options

The config file (`~/.aiassisted/config.toml`) supports these settings:

```toml
[general]
verbosity = 1               # 0=quiet, 1=normal, 2=verbose

[install]
auto_update = true          # Check for updates on install
confirm_before_install = false

[templates]
prefer_project = true       # Use ./.aiassisted/templates/ over global

[skills]
tools = []                  # AI tools to setup (empty = auto-detect)
auto_setup_skills = false   # Auto-run setup-skills after install
auto_setup_agents = false   # Auto-run setup-agents after install

[github]
repo = "rstlix0x0/aiassisted"  # Source repository
ref = ""                       # Branch/tag (empty = latest)
```

See `.aiassisted/config/README.md` for detailed documentation.

#### Template Customization

Templates are resolved with cascading priority:

1. **Project templates** (`./.aiassisted/templates/`) - Custom per-project
2. **Global templates** (`~/.aiassisted/templates/`) - Default templates

**Easy way - Use the templates command:**

```bash
# Initialize templates in your project (copies from global)
aiassisted templates init

# List all available templates
aiassisted templates list

# Edit templates
vim .aiassisted/templates/skills/opencode/git-commit.SKILL.md.template

# Regenerate skills and agents with custom templates
aiassisted setup-skills
aiassisted setup-agents

# Commit custom templates to share with team
git add .aiassisted/templates/
git commit -m "feat: add custom AI skill templates"

# Later, sync templates from updated global templates
aiassisted templates sync
```

**Templates command reference:**
- `aiassisted templates list` - List all templates (global and project)
- `aiassisted templates init` - Copy global templates to project for customization
- `aiassisted templates show <path>` - Display a specific template
- `aiassisted templates sync` - Update project templates from global
- `aiassisted templates diff` - Show differences between project and global
- `aiassisted templates path` - Show template directory paths

### View Help

Show all available commands:
```bash
aiassisted help
```

## Commands Reference

| Command | Description | Options |
|---------|-------------|---------|
| `install` | Install .aiassisted to directory | `--path=DIR`, `--verbose`, `--quiet` |
| `update` | Update existing installation | `--force`, `--path=DIR`, `--verbose`, `--quiet` |
| `check` | Check if updates available | `--path=DIR` |
| `setup-skills` | Setup AI skills (slash commands) | `--tool=<opencode\|claude\|auto>`, `--dry-run` |
| `setup-agents` | Setup AI agents (custom subagents) | `--tool=<opencode\|claude\|auto>`, `--dry-run` |
| `templates <subcommand>` | Manage templates | `list`, `show <path>`, `init`, `sync`, `diff`, `path` |
| `config <subcommand>` | Manage configuration | `show`, `get <key>`, `edit`, `reset`, `path` |
| `version` | Show CLI version | - |
| `self-update` | Update the CLI tool | - |
| `help` | Show help message | - |

## Global Options

- `--verbose` - Show detailed debug output
- `--quiet` - Show only errors
- `--path=DIR` - Specify target directory (default: current directory)
- `--force` - Skip confirmation prompts (update command only)

## Installation Structure

The installer creates the following structure:

```
~/.aiassisted/                      # Main directory (everything lives here)
├── config.toml                    # Global configuration
├── templates/                     # User-customizable templates
├── cache/                        # Temporary cache files
├── state/                        # State files
└── source/                       # Source code
    └── aiassisted/               # Git clone of the repository
        ├── .git/                 # Git metadata (for updates)
        ├── bin/aiassisted        # CLI script (POSIX shell)
        ├── src/shell/            # Core implementation
        ├── .aiassisted/          # Guidelines and instructions
        └── README.md

~/.local/bin/aiassisted            # Symlink → ~/.aiassisted/source/aiassisted/bin/aiassisted
```

**Benefits of this structure:**
- **Single source of truth**: Everything in `~/.aiassisted/`
- **Easy updates**: `aiassisted self-update` runs `git pull`
- **Easy uninstall**: `rm -rf ~/.aiassisted ~/.local/bin/aiassisted`
- **Inspect source**: `cd ~/.aiassisted/source/aiassisted`
- **View changes**: `cd ~/.aiassisted/source/aiassisted && git log`

## What's Inside `.aiassisted`?

The `.aiassisted` directory contains curated knowledge for AI assistants:

```
.aiassisted/
├── .version                    # Version tracking (git commit hash)
├── FILES.txt                   # File manifest with SHA256 checksums
├── config/
│   └── README.md              # Configuration documentation
├── guidelines/
│   ├── ai/                    # AI tool usage guides
│   │   └── agents/           # OpenCode and Claude Code guides
│   ├── architecture/          # Architecture patterns and best practices
│   │   ├── algebraic-data-types.md
│   │   ├── builder-pattern.md
│   │   ├── factory-pattern.md
│   │   ├── dependency-management.md
│   │   └── modular-monolith.md
│   ├── documentation/         # Documentation standards
│   │   ├── diataxis-guidelines.md           # Diátaxis framework
│   │   ├── documentation-quality-standards.md
│   │   └── task-documentation-standards.md
│   └── rust/                  # Rust-specific guidelines
│       ├── microsoft-rust-guidelines.md
│       ├── rust-adt-implementation-guide.md
│       ├── rust-builder-pattern-guide.md
│       ├── rust-dispatch-guide.md
│       ├── rust-factory-pattern-guide.md
│       └── rust-typestate-pattern-guide.md
├── instructions/              # AI agent behavior instructions
│   ├── conventional-commits.instructions.md
│   ├── rust.instructions.md
│   ├── ai-prompt-engineering-safety-best-practices.instructions.md
│   ├── multi-project-memory-bank.instructions.md
│   └── setup-agents-context.instructions.md
├── prompts/                   # Reusable prompt templates
│   └── git.commit.prompt.md  # Conventional commit template
└── templates/                 # Skill and agent templates
    ├── agents/               # Custom agent templates
    │   ├── claude/          # Claude Code agents
    │   └── opencode/        # OpenCode agents
    └── skills/              # Slash command templates
        ├── claude/          # Claude Code skills
        └── opencode/        # OpenCode skills
```

### Content Categories

**Guidelines** (`guidelines/`)
- **Architecture**: Design patterns (ADT, Builder, Factory), modular monolith, dependency management
- **Documentation**: Diátaxis framework, quality standards, task documentation
- **Rust**: Language-specific patterns, error handling, type system usage
- **AI Tools**: Guides for OpenCode and Claude Code usage

**Instructions** (`instructions/`)
- AI agent behavior rules and constraints
- Conventional commits specification
- Rust development instructions
- Prompt engineering best practices
- Multi-project memory management

**Prompts** (`prompts/`)
- Ready-to-use templates for common tasks
- Git commit message templates
- Code review prompts

**Templates** (`templates/`)
- Customizable skills (slash commands) for AI assistants
- Custom agent definitions for specialized tasks
- Supports both OpenCode and Claude Code

These files provide AI models with:
- **Context** - Understanding of your project's architecture and patterns
- **Standards** - Coding conventions and documentation style
- **Constraints** - Rules and best practices to follow
- **Templates** - Reusable prompts for consistent outputs

## Version Tracking

Each `.aiassisted` installation includes a `.version` file that tracks:
- **COMMIT_HASH**: Git commit hash from this repository
- **UPDATED_AT**: Last update timestamp

The CLI compares local and remote commit hashes to detect updates.

## Update Workflow

When you run `aiassisted update`, the CLI synchronizes your local `.aiassisted/` with the latest guidelines:

1. Checks local version (commit hash)
2. Fetches latest version from GitHub
3. Compares versions
4. If outdated:
   - Downloads remote `FILES.txt` manifest
   - Compares file checksums to identify changed files
   - Downloads only changed files (efficient!)
   - Verifies SHA256 checksums for integrity
   - Shows diff between current and new
   - Asks for confirmation (unless `--force`)
   - Applies selective update (only changed files)
5. If up-to-date: Shows success message

**Why keep `.aiassisted/` updated?**
- Get new guidelines and best practices as they're added
- Stay current with evolving architectural patterns
- Receive bug fixes in instructions and prompts
- Ensure AI assistants have the latest knowledge

**Benefits:**
- Only downloads files that actually changed
- Verifies file integrity with SHA256 checksums
- Saves bandwidth on partial updates
- AI assistants automatically use updated guidelines

## Examples

### Example 1: Install in a New Rust Project

```bash
cd ~/my-rust-project
aiassisted install
aiassisted setup-skills
aiassisted setup-agents
```

**What happens:**
1. `.aiassisted/` directory is created with all guidelines and instructions
2. Skills like `/git-commit` and `/review-rust` are created in `.opencode/skills/`
3. Agents like `ai-knowledge-rust` are created in `.opencode/agents/`

**AI assistants can now:**
- Reference Rust guidelines when suggesting code
- Use conventional commits format for commit messages
- Apply Rust-specific patterns (ADT, Builder, Factory)
- Review code against your architectural standards

### Example 2: Using AI Skills with Embedded Context

**Without `.aiassisted/`:**
```
You: Review this Rust code
AI: The code looks fine, but you might want to add error handling.
```

**With `.aiassisted/` and skills:**
```
You: /review-rust

AI: [Skill loads .aiassisted/guidelines/rust/ files]
    
    Reviewing against project Rust guidelines:
    - ✓ Error handling follows dispatch patterns (rust-dispatch-guide.md)
    - ⚠ Consider using ADT instead of Option<Box<dyn Trait>>
      See: rust-adt-implementation-guide.md, section "Trait Objects"
    - ✓ Builder pattern correctly implemented
    - ⚠ Missing public API documentation
      See: ../documentation/documentation-quality-standards.md
```

The skill automatically references your specific guidelines.

### Example 3: Direct File References

**You can explicitly mention files:**
```
You: @.aiassisted/guidelines/rust/rust-builder-pattern-guide.md
     Review my builder implementation

AI: [Reads the specified guide]
    Comparing your implementation to the builder pattern guide:
    - ✓ Follows the recommended fluent interface pattern
    - ⚠ Missing `build()` validation mentioned in section 4.2
    - ⚠ Consider adding convenience methods (section 5)
```

### Example 4: Consistent Commit Messages Across Team

**Setup once:**
```bash
aiassisted install
aiassisted setup-skills --tool=opencode
```

**Everyone on the team uses:**
```
You: /git-commit

AI: [Skill loads .aiassisted/prompts/git.commit.prompt.md and 
     .aiassisted/instructions/conventional-commits.instructions.md]

Based on your changes and commit conventions:

feat(auth): add OAuth2 login support

- Implement OAuth2 authorization code flow
- Add token refresh mechanism
- Update user session management
```

The skill ensures consistent format across all team commits.

### Example 5: Architecture-Aware Code Generation

**Prompt:**
```
You: @.aiassisted/guidelines/architecture/modular-monolith.md
     @.aiassisted/guidelines/rust/rust-factory-pattern-guide.md
     Create a new service for handling payments
```

**AI Response:**
```
AI: [Reads the referenced architecture and pattern guides]

Based on your modular monolith and factory pattern guidelines:

src/modules/payments/
├── mod.rs              # Module interface
├── domain.rs           # Payment domain types (ADTs)
├── service.rs          # PaymentService implementation
├── repository.rs       # PaymentRepository trait
└── factory.rs          # Factory for dependency injection

[Provides code following your specific patterns]
```

### Example 6: Keep Guidelines in Sync Across Projects

Check if updates are available:

```bash
cd ~/my-rust-project
aiassisted install
```

### Check for Updates

```bash
cd ~/my-rust-project
aiassisted check
```

Output:
```
[INFO] Checking version in /Users/username/my-rust-project

Current version: 16869e3304f924b8d00135482742f203b90e83aa
Latest version:  a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0

[WARN] An update is available
[INFO] Run 'aiassisted update' to update to the latest version
```

### Update with Preview

```bash
cd ~/my-rust-project
aiassisted update
```

You'll see a diff and confirmation prompt:
```
[INFO] Update available!
[INFO] Current version: 16869e3...
[INFO] Latest version:  a1b2c3d...

[Changes to be applied:]

--- .aiassisted/instructions/rust.instructions.md
+++ .aiassisted/instructions/rust.instructions.md
@@ -10,3 +10,4 @@
 ...

Apply these changes? [y/N]:
```

### Force Update

```bash
aiassisted update --force
```

Skips the confirmation and applies updates immediately.

## Troubleshooting

### `aiassisted: command not found`

The installation directory isn't in your PATH. Either:

1. Restart your terminal, or
2. Run: `source ~/.bashrc` (or `~/.zshrc` for zsh)
3. Or manually add to PATH:
   ```bash
   export PATH="$PATH:$HOME/.local/bin"
   ```

### Permission Denied

The installer only writes to your home directory (`~/.local/bin`) and doesn't require sudo. If you get permission errors:

1. Check if `~/.local/bin` is writable: `ls -la ~/.local/bin`
2. Ensure you own the directory: `chown -R $USER ~/.local/bin`

### Download Failures

If downloads fail:

1. Check your internet connection
2. Verify GitHub is accessible: `curl -I https://github.com`
3. Try with verbose mode: `aiassisted install --verbose`

### Update Shows No Changes

If `aiassisted check` says you're outdated but `aiassisted update` shows no diff:
- This is normal for metadata-only updates (version file changes)
- The update will still refresh version tracking

## For Maintainers

### Updating `.aiassisted` Content

The easiest way to update content is using the Makefile:

```bash
# After making changes to .aiassisted/ files:
make update-version    # Regenerates manifest and version
make test              # Verifies everything works
git add .aiassisted/
git commit -m 'docs: update guidelines'
git push origin main
```

**Using the script directly:**

The `scripts/update-version.sh` helper script can also be used directly:

```bash
# After making changes to .aiassisted/ files:
./scripts/update-version.sh
```

This script will:
1. Regenerate `FILES.txt` manifest with all files in `.aiassisted/`
2. Update `.version` file with the current git commit hash
3. Display summary and next steps

**What it does:**
- Scans `.aiassisted/` for all files (excluding `.version` and `FILES.txt`)
- Calculates SHA256 checksum for each file
- Generates sorted file list in `FILES.txt` (format: `filepath:sha256hash`)
- Updates `COMMIT_HASH` from git history
- Updates `UPDATED_AT` timestamp

**Manual workflow:**
```bash
# 1. Make changes to .aiassisted/ files
# 2. Run update script
./scripts/update-version.sh

# 3. Commit and push
git add .aiassisted/
git commit -m "docs: update guidelines"
git push origin main
```

**Available Makefile targets:**
- `make help` - Show all available commands
- `make update-version` - Update version and manifest
- `make test` - Run all tests (syntax, CLI, installer)
- `make lint` - Lint scripts with shellcheck
- `make status` - Show project status and file counts
- `make clean` - Clean temporary files

See `make help` for the complete list.

### Release Workflow

1. **Make changes** to `.aiassisted/` content
2. **Update version** using the script above
3. **Commit and push** to main branch
4. **Tag release** (optional but recommended):
   ```bash
   git tag -a v1.1.0 -m "Release v1.1.0: Add new Rust guidelines"
   git push origin v1.1.0
   ```

Users will receive the update on their next `aiassisted check` or `aiassisted update`.

## Technical Details

### Shell Compatibility

The scripts are POSIX-compliant and work with:
- sh (POSIX shell)
- bash
- zsh
- dash
- ash (Alpine Linux)

### Dependencies

Required:
- `curl` or `wget` (at least one)

Optional:
- `diff` (for showing update previews)
- `tput` (for colored output)

### Security

- No sudo required
- Only writes to user directories (`~/.local/bin`, current directory)
- Downloads validated before installation
- Shows changes before applying updates

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Here's how you can help:

### Improving the Installer

To suggest improvements to the CLI tool or installer scripts:
1. Fork this repository
2. Create a feature branch: `git checkout -b feat/your-feature`
3. Make your changes to `install.sh` or `bin/aiassisted`
4. Test thoroughly with different shells (sh, bash, zsh)
5. Verify POSIX compliance: `shellcheck install.sh bin/aiassisted`
6. Commit using [Conventional Commits](https://www.conventionalcommits.org/)
7. Submit a pull request

### Improving Guidelines and Instructions

To improve the `.aiassisted` content (guidelines, instructions, prompts):
1. Fork this repository
2. Create a feature branch: `git checkout -b docs/your-improvement`
3. Edit files in `.aiassisted/`
4. Update the version file (see "For Maintainers" section)
5. Commit your changes
6. Submit a pull request

### Reporting Issues

Found a bug or have a feature request?
- Open an issue on [GitHub Issues](https://github.com/rstlix0x0/aiassisted/issues)
- Provide clear reproduction steps for bugs
- Describe your environment (OS, shell, version)

---

**Built for AI-assisted development workflows**

Embed knowledge, maintain consistency, empower your AI assistants.
