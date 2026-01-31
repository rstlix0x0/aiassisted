# Phase 4: Templates Domain

**Status:** Pending

## Objectives

- Implement the templates domain for AI skill/agent file generation
- Implement `setup-skills`, `setup-agents` commands
- Implement `templates` subcommands (list, show, init, sync, path, diff)

## Tasks

- [ ] Create `src/templates/` domain structure
- [ ] Implement template variable substitution engine
- [ ] Implement cascading resolution (project → global)
- [ ] Implement `setup-skills` command
- [ ] Implement `setup-agents` command
- [ ] Implement `templates list` subcommand
- [ ] Implement `templates show` subcommand
- [ ] Implement `templates init` subcommand
- [ ] Implement `templates sync` subcommand
- [ ] Implement `templates path` subcommand
- [ ] Implement `templates diff` subcommand
- [ ] Add domain-specific tests

## Domain Structure

```
src/templates/
├── mod.rs           # Public API exports
├── commands.rs      # SetupSkillsCommand, SetupAgentsCommand, etc.
├── engine.rs        # Template variable substitution
├── resolver.rs      # Cascading resolution (project → global)
├── generator.rs     # Skill/Agent file generation
└── discovery.rs     # Template file discovery
```

## Implementation Details

### Template Resolution Order

1. Project templates: `.aiassisted/templates/`
2. Global templates: `~/.aiassisted/templates/`
3. Embedded defaults (compiled into binary)

### Template Variables

Templates use `{{variable}}` syntax:

- `{{tool}}` - Target AI tool (opencode, claude)
- `{{project_name}}` - Project directory name
- `{{timestamp}}` - Current timestamp

### AI Tool Detection

Auto-detection based on project files:
- `.opencode.json` → OpenCode
- `.claude/` or `CLAUDE.md` → Claude Code
- Default: Claude Code

### Skill/Agent Output Locations

| Tool | Skills Output | Agents Output |
|------|--------------|---------------|
| OpenCode | `.opencode/skills/` | `.opencode/agents/` |
| Claude | `.claude/commands/` | `.claude/agents/` |

## Testing

```bash
# Unit tests
cargo test templates::

# Integration test (manual)
cargo run -- setup-skills --tool=claude --dry-run
cargo run -- setup-agents --tool=opencode --dry-run
cargo run -- templates list
```
