# Migration Guide: Shell to Rust Version

This document describes the migration process for users upgrading from the old shell-based version of `aiassisted` to the new Rust version.

## Overview

The `aiassisted migrate` command automates the migration process by:

1. Detecting old shell-based installations
2. Converting old configuration format to new format
3. Backing up the old installation
4. Cleaning up obsolete files

## Migration Process

### Automatic Migration

Simply run:

```bash
aiassisted migrate
```

This will:
- Check for old installation at `~/.aiassisted/source/`
- Read old config from `~/.aiassisted/config.toml`
- Convert and save new config format
- Create a timestamped backup at `~/.aiassisted/source.backup.{timestamp}`
- Attempt to remove old git repository

### Verbose Mode

For detailed output during migration:

```bash
aiassisted migrate -v
```

## Configuration Mapping

The old shell-based config used a different structure. The migration automatically maps:

| Old Config | New Config | Notes |
|------------|------------|-------|
| `[general].default_runtime = "shell"` | `default_tool = "auto"` | Shell runtime → Auto tool |
| `[general].default_runtime = "auto"` | `default_tool = "auto"` | Direct mapping |
| `[general].default_runtime = "claude"` | `default_tool = "claude"` | Direct mapping |
| `[general].default_runtime = "opencode"` | `default_tool = "opencode"` | Direct mapping |
| `[general].default_runtime = "python"` | `default_tool = "auto"` | Unsupported → Auto |
| `[general].default_runtime = "bun"` | `default_tool = "auto"` | Unsupported → Auto |
| `[general].verbosity` | `verbosity` | Direct mapping |
| `[install].auto_update` | `auto_update` | Direct mapping |
| `[templates].prefer_project` | `prefer_project` | Direct mapping |

### Old Config Example

```toml
[general]
default_runtime = "shell"
verbosity = 1

[install]
auto_update = true
confirm_before_install = false
install_path = ".aiassisted"

[templates]
prefer_project = true
auto_init_templates = false

[skills]
tools = []

[update]
check_on_startup = false

[github]
repo = "rstlix0x0/aiassisted"
```

### New Config Example

```toml
default_tool = "auto"
verbosity = 1
auto_update = true
prefer_project = true
```

## What Gets Migrated

### Configuration
- ✅ Default tool/runtime preference
- ✅ Verbosity level
- ✅ Auto-update setting
- ✅ Template preference

### Not Migrated
- ❌ `confirm_before_install` - No longer needed
- ❌ `install_path` - Always `.aiassisted` now
- ❌ `auto_init_templates` - Removed in Rust version
- ❌ `auto_sync_templates` - Removed in Rust version
- ❌ Skills configuration - Handled differently in Rust version
- ❌ Update check settings - Simplified in Rust version
- ❌ GitHub settings - Built into Rust version

## Backup and Safety

The migration is designed to be safe:

1. **Config is converted, not moved** - Original config is preserved
2. **Source directory is backed up** - Full backup created at `~/.aiassisted/source.backup.{timestamp}`
3. **Backup happens before deletion** - Data is safe even if cleanup fails
4. **Non-destructive on failure** - If migration fails, old installation remains intact

### Backup Location

Backups are created at:
```
~/.aiassisted/source.backup.YYYYMMDD_HHMMSS/
```

Example:
```
~/.aiassisted/source.backup.20260131_170526/
```

### Manual Cleanup

If the automatic cleanup fails (e.g., due to permissions), you can manually delete:
```bash
rm -rf ~/.aiassisted/source/
```

Your data is safely backed up, so this is safe to do.

## Troubleshooting

### No Old Installation Found

If you see:
```
No old installation found. Nothing to migrate.
```

This means:
- You don't have an old shell-based installation, OR
- You've already migrated

This is normal for new users.

### Permission Denied During Cleanup

If you see:
```
Could not fully remove old installation: IO error: Permission denied
```

This is not critical. The important steps (config migration and backup) succeeded. You can:

1. Manually delete the old installation:
   ```bash
   rm -rf ~/.aiassisted/source/
   ```

2. Or leave it - the new Rust version doesn't use it.

### Config Parse Error

If you see:
```
Error: Parse error: Failed to parse shell config
```

Your old config file may be corrupted. You can:

1. Check the old config at `~/.aiassisted/config.toml`
2. Fix any TOML syntax errors
3. Run migration again

Or start fresh:
```bash
mv ~/.aiassisted/config.toml ~/.aiassisted/config.toml.old
aiassisted config reset
```

## Post-Migration

After migration:

1. **Verify new config**:
   ```bash
   aiassisted config show
   ```

2. **Check your backup**:
   ```bash
   ls -la ~/.aiassisted/source.backup.*/
   ```

3. **Test the new version**:
   ```bash
   aiassisted install
   aiassisted check
   ```

4. **Optional: Clean up old backup** (after verifying everything works):
   ```bash
   rm -rf ~/.aiassisted/source.backup.*
   ```

## Getting Help

If you encounter issues:

1. Check the backup exists and is complete
2. Try running with verbose mode: `aiassisted migrate -v`
3. Report issues at: https://github.com/rstlix0x0/aiassisted/issues

## Technical Details

### Architecture

The migration implementation consists of:

- `src/migration/mod.rs` - Module root
- `src/migration/shell_config.rs` - Old TOML format parser
- `src/migration/commands.rs` - Migration command logic
- `tests/migration_integration.rs` - Integration tests

### Testing

Comprehensive test coverage:
- ✅ 21 unit tests (shell_config + commands)
- ✅ 5 integration tests
- ✅ Config conversion for all runtime types
- ✅ Empty/minimal/complete config parsing
- ✅ Backup creation and directory operations
- ✅ Error handling (invalid TOML, missing files, etc.)

Run tests:
```bash
cargo test migration
```
