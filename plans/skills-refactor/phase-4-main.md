# Phase 4: Update Main Entry Point

**Status**: Pending

## Objective

Update `src/main.rs` to use the new skills module and remove old template handlers.

## Changes

### 1. Update Module Imports

```rust
// ADD
mod skills;

// REMOVE (after Phase 5)
// mod templates;
```

### 2. Update Use Statements

```rust
// ADD
use skills::{SetupSkillsCommand, SkillsListCommand};

// REMOVE
// use templates::{
//     SetupSkillsCommand, SetupAgentsCommand,
//     ListTemplatesCommand, ShowTemplateCommand,
//     TemplatesInitCommand, TemplatesSyncCommand,
//     TemplatesPathCommand, TemplatesDiffCommand,
//     SimpleTemplateEngine, CascadingResolver,
// };
```

### 3. Update SetupSkills Handler

Replace the old handler:

```rust
Commands::SetupSkills(args) => {
    let tool: ToolType = args.tool.into();
    let cmd = SetupSkillsCommand {
        tool,
        dry_run: args.dry_run,
        force: args.force,
    };
    let project_path = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."));

    cmd.execute(&ctx.fs, &ctx.logger, &project_path).await
}
```

**Note**: No more `engine`, `resolver` dependencies.

### 4. Add Skills Subcommand Handler

```rust
Commands::Skills(subcmd) => {
    match subcmd {
        SkillsCommands::List(args) => {
            let tool: ToolType = args.tool.into();
            let cmd = SkillsListCommand { tool };
            let project_path = std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));

            cmd.execute(&ctx.fs, &ctx.logger, &project_path).await
        }
    }
}
```

### 5. Remove Old Handlers

Delete these command handlers:
- `Commands::SetupAgents` handler
- `Commands::Templates` handler (entire match arm)

## Full Command Match After Changes

```rust
match cli.command {
    Commands::Version => { /* unchanged */ }
    Commands::Install(args) => { /* unchanged */ }
    Commands::Check(args) => { /* unchanged */ }
    Commands::Update(args) => { /* unchanged */ }
    Commands::Config(subcmd) => { /* unchanged */ }
    Commands::SelfUpdate => { /* unchanged */ }
    Commands::Migrate => { /* unchanged */ }

    // NEW: Simplified setup-skills
    Commands::SetupSkills(args) => {
        let tool: ToolType = args.tool.into();
        let cmd = SetupSkillsCommand {
            tool,
            dry_run: args.dry_run,
            force: args.force,
        };
        let project_path = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."));
        cmd.execute(&ctx.fs, &ctx.logger, &project_path).await
    }

    // NEW: Skills subcommands
    Commands::Skills(subcmd) => {
        match subcmd {
            SkillsCommands::List(args) => {
                let tool: ToolType = args.tool.into();
                let cmd = SkillsListCommand { tool };
                let project_path = std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("."));
                cmd.execute(&ctx.fs, &ctx.logger, &project_path).await
            }
        }
    }

    // REMOVED: SetupAgents
    // REMOVED: Templates
}
```

## Verification

```bash
cargo check
cargo run -- setup-skills --help
cargo run -- setup-skills --dry-run
cargo run -- skills list
```

## Dependencies

- Phase 1 (CLI definitions)
- Phase 3 (skills module implementation)

## Next Phase

[Phase 5: Remove Old Templates Code](phase-5-cleanup.md)
