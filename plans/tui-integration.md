# TUI Integration Plan

**Status:** Draft
**Created:** 2026-02-02

## Overview

Replace sequential log output with a clean Ratatui-based TUI for better visual feedback.

## Current State

The tool outputs sequential log messages:
```
[INFO] Installing .aiassisted to .
[INFO] Downloading manifest...
[INFO] Downloaded: guidelines/rust/rust-policy-guide.md
[INFO] Downloaded: guidelines/rust/rust-dispatch-guide.md
... (many more lines)
[OK] Installation complete
```

## Goals

1. Replace log spam with clean progress indicators
2. Show clear status for multi-file operations
3. Provide better visual feedback for long-running operations
4. Maintain scriptability (non-interactive by default)

## UI Designs

### 1. Minimal Progress UI (Recommended for Phase 1)

Progress display for install/update operations:

```
┌─ aiassisted install ─────────────────────────────────────────┐
│                                                              │
│  Installing .aiassisted                                      │
│                                                              │
│  ████████████████████████░░░░░░░░░░  54/72 files  (75%)     │
│                                                              │
│  ↓ guidelines/ratatui/ratatui-widgets-guide.md              │
│                                                              │
│  ✓ 54 downloaded  ○ 18 remaining                            │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

Completion state:

```
┌─ aiassisted install ─────────────────────────────────────────┐
│                                                              │
│  ✓ Installation Complete                                     │
│                                                              │
│  72 files installed to .aiassisted/                          │
│                                                              │
│  Guidelines:  32 files                                       │
│  Skills:      24 files                                       │
│  Instructions: 8 files                                       │
│  Config:       8 files                                       │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### 2. Dashboard Style (for check/update)

For commands that show diffs:

```
┌─ aiassisted check ───────────────────────────────────────────┐
│                                                              │
│  Comparing local vs remote                                   │
│                                                              │
│  Local:  v1.2.0 (2024-01-15)                                │
│  Remote: v1.3.0 (2024-02-01)                                │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  Changes Available                                           │
│                                                              │
│  + 8 new files                                               │
│    └ guidelines/ratatui/ (8 files)                          │
│                                                              │
│  ~ 3 modified files                                          │
│    └ guidelines/rust/rust-policy-guide.md                   │
│    └ skills/git-commit/SKILL.md                             │
│    └ manifest.json                                          │
│                                                              │
│  Press [u] to update · [q] to quit                          │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### 3. Skills/Agents List View

For `skills list` and `agents list`:

```
┌─ aiassisted skills ──────────────────────────────────────────┐
│                                                              │
│  Skills for Claude Code                     12 available     │
│                                                              │
│  Installed                                                   │
│  ├─ ✓ git-commit         Conventional commit messages       │
│  ├─ ✓ review-rust        Rust code review                   │
│  ├─ ✓ doc-code           Code documentation                 │
│  └─ ✓ policy-rust        Rust policy enforcement            │
│                                                              │
│  Available                                                   │
│  ├─ ○ review-codes       General code review                │
│  ├─ ○ doc-project        Project documentation              │
│  └─ ○ memorybank-setup   Memory bank initialization         │
│                                                              │
│  [↑↓] Navigate · [Enter] Details · [i] Install · [q] Quit   │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### 4. Self-Update Progress

For `self-update`:

```
┌─ aiassisted self-update ─────────────────────────────────────┐
│                                                              │
│  Updating aiassisted                                         │
│                                                              │
│  Current: v0.3.0                                             │
│  Latest:  v0.4.0                                             │
│                                                              │
│  Downloading...                                              │
│  ████████████████████░░░░░░░░░░  2.1 MB / 3.2 MB   65%      │
│                                                              │
│  Changelog:                                                  │
│  • feat(agents): add agents command                          │
│  • fix(selfupdate): handle Unix archives                     │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

## Implementation Phases

### Phase 1: Enhanced Progress (Minimal)

- Replace log spam with single-line progress bars
- Show summary on completion
- Keep it non-interactive for scriptability
- No breaking changes to existing behavior

**Scope:**
- `install` command: progress bar
- `update` command: progress bar
- `self-update` command: download progress

### Phase 2: Interactive Mode (Optional)

- Add `--interactive` or `-i` flag
- Full TUI for browsing skills/agents
- Confirmation dialogs for destructive operations
- Keyboard navigation

**Scope:**
- `skills list`: interactive browsing
- `agents list`: interactive browsing
- `check`: interactive diff view with update option

### Phase 3: Full Dashboard

- Unified dashboard view
- Real-time status updates
- Configuration management UI

## Architecture

### New Module Structure

```
src/
├── ui/                    # New TUI module
│   ├── mod.rs
│   ├── progress.rs        # Progress bar widget
│   ├── summary.rs         # Completion summary
│   ├── diff.rs            # Diff visualization
│   └── list.rs            # List/tree views
├── core/
│   └── infra.rs           # Add TuiRenderer trait
└── infra/
    └── tui.rs             # Ratatui-based renderer
```

### Integration Pattern

Replace `ColoredLogger` calls with a `TuiRenderer` that renders to a Ratatui terminal:

```rust
// Current
logger.info("Downloading manifest...");
logger.info(&format!("Downloaded: {}", path));
logger.success("Installation complete");

// New
renderer.set_status("Downloading manifest...");
renderer.update_progress(current, total);
renderer.set_current_file(&path);
renderer.complete("Installation complete", stats);
```

### Dependencies

```toml
[dependencies]
ratatui = "0.30.0"
crossterm = "0.29.0"
```

## Design Decisions

### 1. Non-Interactive by Default

Keep the tool scriptable. TUI should enhance output, not require interaction.

### 2. Graceful Fallback

Detect non-TTY environments and fall back to simple log output:

```rust
if std::io::stdout().is_terminal() {
    run_with_tui()?;
} else {
    run_with_logger()?;
}
```

### 3. Consistent Widget Style

Use Block with rounded borders, consistent colors:
- Blue: info/progress
- Green: success
- Yellow: warnings
- Red: errors

### 4. Static Dispatch

Following project policy, use generics over trait objects:

```rust
fn run<R: Renderer>(renderer: &R) { ... }
```

## Operations Analysis

| Command | Current Output | TUI Enhancement |
|---------|----------------|-----------------|
| `install` | Sequential logs | Multi-file progress bar |
| `update` | Version comparison | Diff visualization |
| `check` | Text list | Colored diff, summary |
| `skills setup` | Per-skill status | Skill tree, copy progress |
| `skills update` | Status indicators | Visual diff tree |
| `skills list` | Plain text list | Interactive browsing |
| `agents setup` | Per-agent status | Compilation progress |
| `agents update` | Status indicators | Diff tree |
| `agents list` | Plain text list | Interactive browsing |
| `self-update` | Sequential logs | Download progress bar |
| `config show` | Plain text | Formatted table |

## Questions to Resolve

1. Should TUI be opt-in (`--tui`) or opt-out (`--no-tui`)?
2. Minimum terminal size requirements?
3. Color theme customization?
4. Animation preferences (spinners, etc.)?

## Resources

- Ratatui guidelines: `.aiassisted/guidelines/ratatui/`
- Project policies: `.aiassisted/guidelines/rust/rust-policy-guide.md`
- Existing logger: `src/infra/logger.rs`
