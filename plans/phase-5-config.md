# Phase 5: Config Domain

**Status:** Complete

## Objectives

- Implement the config domain for application settings
- Implement `config` subcommands (show, get, edit, reset, path)

## Tasks

- [x] Create `src/config/` domain structure
- [x] Implement TOML config file persistence
- [x] Implement config validation and defaults
- [x] Implement `config show` subcommand
- [x] Implement `config get` subcommand
- [x] Implement `config edit` subcommand
- [x] Implement `config reset` subcommand
- [x] Implement `config path` subcommand
- [x] Add domain-specific tests

## Domain Structure

```
src/config/
├── mod.rs           # Public API exports
├── commands.rs      # ShowCommand, GetCommand, EditCommand, etc.
├── settings.rs      # Config validation, defaults
└── toml_store.rs    # TOML file persistence
```

## Configuration File

Location: `~/.aiassisted/config.toml`

```toml
# Default AI tool
default_tool = "auto"

# Verbosity level (0-2)
verbosity = 1

# Auto-update check
auto_check_updates = true

# Template preferences
[templates]
prefer_project = true
```

## Configuration Keys

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `default_tool` | string | `"auto"` | Default AI tool (auto, opencode, claude) |
| `verbosity` | integer | `1` | Output verbosity (0=quiet, 1=normal, 2=debug) |
| `auto_check_updates` | boolean | `true` | Check for updates automatically |
| `templates.prefer_project` | boolean | `true` | Prefer project templates over global |

## Implementation Details

### Config Show

Display all current configuration values in a formatted table.

### Config Get

```bash
aiassisted config get default_tool
# Output: auto
```

### Config Edit

Open config file in `$EDITOR` (fallback to `vim` or `nano`).

### Config Reset

Reset all values to defaults, with confirmation prompt unless `--force`.

### Config Path

Print the config file path:
```bash
aiassisted config path
# Output: /home/user/.aiassisted/config.toml
```

## Testing

```bash
# Unit tests
cargo test config::

# Integration test (manual)
cargo run -- config show
cargo run -- config get default_tool
cargo run -- config path
```
