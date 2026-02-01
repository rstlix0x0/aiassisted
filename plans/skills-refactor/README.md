# Skills Refactor Plan

## Overview

Refactor `setup-skills` command to use a simpler copy-based approach instead of template rendering, and remove `setup-agents` command.

**Status**: Planning

## Goals

1. Replace template-based skill generation with direct directory copying
2. Remove `setup-agents` command (not needed)
3. Create new `src/skills/` module (replacing `src/templates/`)
4. Simplify the codebase by removing template engine and resolver

## Key Decisions

| Decision | Choice |
|----------|--------|
| Module naming | `src/skills/` (new module) |
| Skills source | Project-based only (`.aiassisted/skills/`) |
| Overwrite behavior | Skip by default, `--force` flag to overwrite |
| List command | Keep `aiassisted skills list` |

## Current vs New Approach

### Current (Template-based)
```
.aiassisted/templates/skills/{claude,opencode}/*.SKILL.md.template
    ↓ (variable substitution: {{PROJECT_ROOT}}, etc.)
.claude/commands/ or .opencode/skills/
```

### New (Copy-based)
```
.aiassisted/skills/<skill-name>/SKILL.md
    ↓ (direct copy)
.claude/skills/<skill-name>/SKILL.md
.opencode/skills/<skill-name>/SKILL.md
```

## Phase Summary

| Phase | Description | Status |
|-------|-------------|--------|
| [Phase 1](phase-1-cli.md) | Update CLI definitions | Pending |
| [Phase 2](phase-2-core.md) | Simplify core traits | Pending |
| [Phase 3](phase-3-skills-module.md) | Create new skills module | Pending |
| [Phase 4](phase-4-main.md) | Update main entry point | Pending |
| [Phase 5](phase-5-cleanup.md) | Remove old templates code | Pending |
| [Phase 6](phase-6-content.md) | Update content files | Pending |
| [Phase 7](phase-7-tests.md) | Update tests | Pending |
| [Phase 8](phase-8-docs.md) | Update documentation | Pending |

## Files to Remove

### Source Files
- `src/templates/` (entire directory)
- `src/core/templates.rs`

### Content Files
- `.aiassisted/templates/` (entire directory)

## Files to Create

### Source Files
- `src/skills/mod.rs`
- `src/skills/commands.rs`
- `src/skills/discovery.rs`
- `src/skills/copier.rs`

## Files to Modify

### Source Files
- `src/cli.rs` - Remove setup-agents, add skills subcommands
- `src/main.rs` - Update command handlers
- `src/core/mod.rs` - Remove templates module

### Content Files
- `manifest.json` - Remove template entries

### Documentation
- `FEATURES.md` - Update feature list
- `CLAUDE.md` - Update if needed

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Breaking existing users | Release notes with migration guide |
| Tests fail | Update tests in Phase 7 |
| Missing skills | Verify `.aiassisted/skills/` included in install |

## Success Criteria

1. `aiassisted setup-skills` copies skills from `.aiassisted/skills/` to target
2. `aiassisted setup-skills --force` overwrites existing skills
3. `aiassisted setup-skills --dry-run` shows what would be copied
4. `aiassisted skills list` shows available skills
5. All tests pass
6. Zero warnings in `cargo check`
