# Phase 1: Update CLI Definitions

**Status**: Pending

## Objective

Update `src/cli.rs` to remove `setup-agents` command and restructure skills-related commands.

## Changes

### 1. Remove SetupAgentsArgs

Delete the struct:
```rust
// DELETE THIS
#[derive(Debug, Args)]
pub struct SetupAgentsArgs {
    #[arg(long, default_value = "auto")]
    pub tool: CliToolType,
    #[arg(long, default_value = "false")]
    pub dry_run: bool,
}
```

### 2. Remove Commands::SetupAgents Variant

Delete from the `Commands` enum:
```rust
// DELETE THIS
#[command(name = "setup-agents")]
SetupAgents(SetupAgentsArgs),
```

### 3. Update SetupSkillsArgs

Add `--force` flag:
```rust
#[derive(Debug, Args)]
pub struct SetupSkillsArgs {
    /// Tool type: auto, opencode, claude
    #[arg(long, default_value = "auto")]
    pub tool: CliToolType,

    /// Show what would be copied without making changes
    #[arg(long, default_value = "false")]
    pub dry_run: bool,

    /// Overwrite existing skills
    #[arg(long, default_value = "false")]
    pub force: bool,
}
```

### 4. Add Skills Subcommand Group

Add new subcommand for `skills list`:
```rust
#[derive(Debug, Subcommand)]
pub enum SkillsCommands {
    /// List available skills
    #[command(name = "list")]
    List(SkillsListArgs),
}

#[derive(Debug, Args)]
pub struct SkillsListArgs {
    /// Tool type filter: auto, opencode, claude
    #[arg(long, default_value = "auto")]
    pub tool: CliToolType,
}
```

### 5. Update Commands Enum

```rust
pub enum Commands {
    // ... existing commands ...

    /// Set up AI skills (copy to tool directory)
    #[command(name = "setup-skills")]
    SetupSkills(SetupSkillsArgs),

    /// Manage skills
    #[command(name = "skills")]
    Skills(SkillsCommands),

    // REMOVE: SetupAgents
    // REMOVE: Templates (entire subcommand group)
}
```

### 6. Remove Templates Subcommand Group

Delete the entire `TemplatesCommands` enum and related args:
- `TemplatesCommands`
- `TemplatesListArgs`
- `TemplatesShowArgs`
- `TemplatesInitArgs`
- `TemplatesSyncArgs`
- `TemplatesPathArgs`
- `TemplatesDiffArgs`

## Verification

After changes:
```bash
cargo check
cargo run -- --help
cargo run -- setup-skills --help
cargo run -- skills --help
cargo run -- skills list --help
```

Expected help output for `setup-skills`:
```
Set up AI skills (copy to tool directory)

Usage: aiassisted setup-skills [OPTIONS]

Options:
      --tool <TOOL>  Tool type: auto, opencode, claude [default: auto]
      --dry-run      Show what would be copied without making changes
      --force        Overwrite existing skills
  -h, --help         Print help
```

## Dependencies

None - this is the first phase.

## Next Phase

[Phase 2: Simplify Core Traits](phase-2-core.md)
