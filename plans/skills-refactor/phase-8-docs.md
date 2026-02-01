# Phase 8: Update Documentation

**Status**: Pending

## Objective

Update all documentation to reflect the simplified skills approach and removal of templates/agents features.

## Files to Update

### 1. FEATURES.md

#### Remove Sections
- "Templates Domain" section (entire section)
- `setup-agents` command documentation
- `templates list/show/init/sync/diff/path` command documentation

#### Update Sections

**Quick Stats** - Update counts:
```markdown
**Quick Stats:**
- **25+ Commands** - Full CLI interface (was 30+)
```

**setup-skills section** - Rewrite:
```markdown
### setup-skills
**Set up AI skills (copy to tool directory)**

```bash
aiassisted setup-skills
aiassisted setup-skills --tool=claude
aiassisted setup-skills --tool=opencode
aiassisted setup-skills --dry-run
aiassisted setup-skills --force
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
[INFO] Found 6 skill(s)
[OK] Copied: git-commit
[OK] Copied: review-rust
[OK] Copied: doc-code
[OK] Copied: doc-project
[OK] Copied: review-codes
[OK] Copied: policy-rust
[OK] Setup complete: 6 copied, 0 skipped
```
```

**Add new section for `skills list`:**
```markdown
### skills list
**List available skills**

```bash
aiassisted skills list
aiassisted skills list --tool=claude
```

**What it does:**
1. Lists skills available in `.aiassisted/skills/`
2. Shows installation status for each skill

**Output:**
```
[INFO] Skills source: .aiassisted/skills
[INFO] Target directory: .claude/skills
[INFO]
[INFO] Available skills (6):
[INFO]
[INFO]   - doc-code
[INFO]   - doc-project
[INFO]   - git-commit [installed]
[INFO]   - policy-rust
[INFO]   - review-codes
[INFO]   - review-rust [installed]
```
```

#### Update Feature Categories

Remove from "Template System" category:
```markdown
### Template System
- ✅ Auto-detect AI tool (Claude Code, OpenCode)
- ✅ Manual tool selection
- ~~Template variable substitution~~ (removed)
- ~~Cascading resolution (project overrides global)~~ (removed)
- ~~List/show/init/sync/diff templates~~ (removed)
- ✅ Dry-run mode
```

Replace with "Skills System":
```markdown
### Skills System
- ✅ Auto-detect AI tool (Claude Code, OpenCode)
- ✅ Manual tool selection
- ✅ Copy skills to tool directory
- ✅ Preserve skill directory structure
- ✅ Force overwrite option
- ✅ Dry-run mode
- ✅ List available skills
```

#### Update Summary Table

```markdown
| Category | Features | Status |
|----------|----------|--------|
| Core commands | 4 | ✅ Complete |
| Content domain | 3 | ✅ Complete |
| Skills domain | 2 | ✅ Complete |
| Config domain | 5 | ✅ Complete |
| Self-update | 1 | ✅ Complete |
| Migration | 1 | ✅ Complete |
| **Total** | **16 commands** | **✅ Production Ready** |
```

### 2. CLAUDE.md

#### Update Architecture Section

```markdown
### Source Code Structure

```
src/
├── main.rs          # Entry point, composition root
├── cli.rs           # Clap CLI definitions
├── core/            # All abstractions (traits, types)
│   ├── types.rs     # Error, ToolType, Result, DTOs
│   ├── infra.rs     # FileSystem, HttpClient, Checksum, Logger
│   ├── content.rs   # ManifestStore, ContentDownloader
│   └── config.rs    # ConfigStore
├── infra/           # Shared infrastructure implementations
├── content/         # Content domain (install, update, check)
├── skills/          # Skills domain (setup-skills, skills list)
├── config/          # Config domain
├── selfupdate/      # Self-update domain
└── migration/       # Migration domain
```
```

#### Update Common Commands

Remove templates commands, add skills commands:
```markdown
### Skills

```bash
# Set up skills for detected tool
cargo run -- setup-skills

# Set up skills for specific tool
cargo run -- setup-skills --tool=claude

# Preview what would be copied
cargo run -- setup-skills --dry-run

# Overwrite existing skills
cargo run -- setup-skills --force

# List available skills
cargo run -- skills list
```
```

### 3. README.md (if exists)

Update any references to:
- `setup-agents` command (remove)
- `templates` commands (remove)
- Add `skills list` command

### 4. Architecture Documentation

If `ARCHITECTURE.md` exists, update domain descriptions:
- Remove "Templates Domain" description
- Add "Skills Domain" description

## Content Updates

### Skills README

Create `.aiassisted/skills/README.md`:

```markdown
# AI Skills

This directory contains skills that can be installed to AI assistant tool directories.

## Available Skills

| Skill | Description |
|-------|-------------|
| doc-code | Document code with comprehensive documentation |
| doc-project | Document project structure and architecture |
| git-commit | Generate conventional commit messages |
| policy-rust | Rust coding policy expert |
| review-codes | General code review skill |
| review-rust | Rust-specific code review |

## Installation

```bash
# Install skills to Claude Code
aiassisted setup-skills --tool=claude

# Install skills to OpenCode
aiassisted setup-skills --tool=opencode

# List available skills
aiassisted skills list
```

## Skill Structure

Each skill follows the Agent Skills standard:

```
skill-name/
├── SKILL.md           # Main skill definition (required)
└── references/        # Supporting documentation (optional)
    └── *.md
```

## Creating Custom Skills

See the [Claude Code Skills Guide](guidelines/ai/agents/claude-code/claude-code-skills-guide.md) or [OpenCode Skills Guide](guidelines/ai/agents/opencode/opencode-skills-guide.md) for creating custom skills.
```

## Verification

```bash
# Verify documentation links work
# Check all referenced files exist

# Test documented commands
cargo run -- setup-skills --help
cargo run -- skills list --help
```

## Dependencies

- All previous phases completed
- Code changes merged

## Completion Checklist

- [ ] FEATURES.md updated
- [ ] CLAUDE.md updated
- [ ] README.md updated (if exists)
- [ ] Skills README created
- [ ] All code examples tested
- [ ] No broken links

## Post-Completion

After all phases are complete:
1. Run full test suite: `cargo test`
2. Verify zero warnings: `cargo check`
3. Update CHANGELOG.md with breaking changes
4. Create release notes highlighting:
   - Simplified skills installation
   - Removed `setup-agents` command
   - Removed templates system
   - New `skills list` command
