# Architecture: Multi-Runtime Support

## Overview

The `aiassisted` CLI supports multiple runtime backends while maintaining a single, consistent interface. Users can choose between:

- **Shell** (default): Pure POSIX sh, zero dependencies
- **Python + UV**: Modern Python with fast dependency management
- **Bun + TypeScript**: Fast JavaScript runtime with native TypeScript

## Design Principles

1. **Shell orchestrator remains the entry point** - `bin/aiassisted` detects and delegates
2. **Feature parity** - All runtimes support identical commands and flags
3. **Graceful degradation** - Falls back to shell if preferred runtime unavailable
4. **Zero config by default** - Works out of the box with shell runtime
5. **Explicit choice** - Users can select runtime via flag or config

## Directory Structure

```
aiassisted/
├── bin/
│   └── aiassisted                  # Shell orchestrator (entry point)
├── src/
│   ├── shell/
│   │   └── core.sh                # Current shell implementation
│   ├── python/
│   │   ├── pyproject.toml         # UV project definition
│   │   └── aiassisted/
│   │       ├── __init__.py
│   │       ├── __main__.py        # CLI entry point
│   │       ├── cli.py             # Command routing
│   │       ├── installer.py       # Install/update logic
│   │       ├── manifest.py        # File manifest & checksums
│   │       └── downloader.py      # HTTP download utilities
│   └── bun/
│       ├── package.json           # Bun project definition
│       ├── tsconfig.json
│       └── src/
│           ├── index.ts           # CLI entry point
│           ├── cli.ts             # Command routing
│           ├── installer.ts       # Install/update logic
│           ├── manifest.ts        # File manifest & checksums
│           └── downloader.ts      # HTTP download utilities
├── install.sh                     # One-command installer (unchanged)
├── Makefile
├── README.md
├── AGENTS.md
└── ARCHITECTURE.md                # This file
```

## Runtime Detection & Selection

### Detection Order

1. Check for explicit `--runtime=<name>` flag
2. Check config file `~/.config/aiassisted/config`
3. Auto-detect available runtimes in order: python → bun → shell
4. Default to shell if none configured/detected

### Runtime Availability Detection

```sh
# In bin/aiassisted orchestrator

detect_runtime() {
    if command -v uv >/dev/null 2>&1; then
        echo "python"
        return 0
    fi
    
    if command -v bun >/dev/null 2>&1; then
        echo "bun"
        return 0
    fi
    
    echo "shell"
}
```

## Command Interface

All runtimes must support:

### Commands

- `install [--path=DIR] [--verbose] [--quiet]`
- `update [--force] [--path=DIR] [--verbose] [--quiet]`
- `check [--path=DIR]`
- `version`
- `self-update`
- `help`

### Runtime-specific Commands

- `runtime list` - Show available runtimes
- `runtime set <name>` - Set preferred runtime
- `runtime info` - Show current runtime details

### Exit Codes

- `0` - Success
- `1` - General error
- `2` - Command not found
- `3` - Runtime not available

## Implementation Details

### Shell Orchestrator (`bin/aiassisted`)

Responsibilities:
- Parse global flags (`--runtime`, `--verbose`, `--quiet`)
- Detect available runtimes
- Load user config from `~/.config/aiassisted/config`
- Delegate to appropriate runtime backend
- Handle runtime-not-found errors gracefully

### Python Backend (`src/python/`)

**Dependencies (via UV):**
- `httpx` - Modern HTTP client
- `rich` - Terminal formatting and colors
- `typer` - CLI framework

**Key Features:**
- Type hints throughout
- Async/await for parallel downloads
- Structured logging
- Comprehensive error handling

**Entry point:**
```sh
uv run python -m aiassisted install --path=/tmp/test
```

### Bun Backend (`src/bun/`)

**Dependencies:**
- Zero external dependencies (use Bun APIs)

**Key Features:**
- Native TypeScript support
- Fast startup time
- Built-in HTTP fetch
- Streaming downloads

**Entry point:**
```sh
bun run src/index.ts install --path=/tmp/test
```

### Shell Backend (`src/shell/core.sh`)

Current implementation extracted to separate file.

**Entry point:**
```sh
sh src/shell/core.sh install --path=/tmp/test
```

## Configuration File

Location: `~/.config/aiassisted/config`

Format (simple KEY=VALUE):
```sh
RUNTIME=python
VERBOSITY=1
```

## Example Workflows

### User wants default behavior (shell)
```sh
$ aiassisted install
# Uses shell runtime automatically
```

### User wants Python performance
```sh
$ aiassisted install --runtime=python
# Uses Python + UV if available, errors otherwise
```

### User sets permanent preference
```sh
$ aiassisted runtime set python
# Writes to ~/.config/aiassisted/config

$ aiassisted install
# Now uses Python by default
```

### Developer testing all runtimes
```sh
$ aiassisted install --runtime=shell --path=/tmp/test1
$ aiassisted install --runtime=python --path=/tmp/test2
$ aiassisted install --runtime=bun --path=/tmp/test3
```

## Migration Path

### Phase 1: Refactor (Current Sprint)
1. Extract current shell logic to `src/shell/core.sh`
2. Update `bin/aiassisted` to detect and delegate
3. Maintain backward compatibility

### Phase 2: Python Implementation
1. Create `src/python/` structure
2. Implement core commands
3. Add comprehensive tests
4. Document Python-specific setup

### Phase 3: Bun Implementation
1. Create `src/bun/` structure
2. Implement core commands
3. Add comprehensive tests
4. Document Bun-specific setup

### Phase 4: Documentation & Release
1. Update README with runtime selection guide
2. Update AGENTS.md with new structure
3. Create migration guide for existing users
4. Release as v2.0.0 (breaking: new structure)

## Testing Strategy

### Manual Testing Matrix

| Command | Shell | Python | Bun |
|---------|-------|--------|-----|
| `install` | ✓ | ✓ | ✓ |
| `update` | ✓ | ✓ | ✓ |
| `check` | ✓ | ✓ | ✓ |
| `version` | ✓ | ✓ | ✓ |
| `self-update` | ✓ | ✓ | ✓ |
| `runtime list` | ✓ | ✓ | ✓ |

### Automated Testing (Future)

- Shell: `shellspec` or `bats`
- Python: `pytest` with `uv run pytest`
- Bun: `bun test`

## Performance Expectations

| Operation | Shell | Python | Bun |
|-----------|-------|--------|-----|
| Startup | ~5ms | ~50ms | ~10ms |
| Download (10 files) | Sequential | Parallel | Parallel |
| Checksum (10 files) | Sequential | Parallel | Parallel |
| Install time | ~2s | ~500ms | ~400ms |

## Security Considerations

All runtimes must:
- Verify SHA256 checksums before applying updates
- Never require sudo/root access
- Show diffs before applying changes (unless `--force`)
- Clean up temporary files on error
- Use secure HTTPS for all downloads

## Backward Compatibility

Existing users running `aiassisted` commands will continue to work without changes:
- Current `bin/aiassisted` becomes orchestrator
- Shell runtime is default
- No breaking changes to command interface
- Old installations remain compatible

## Future Enhancements

- **Parallel downloads**: Python/Bun download multiple files simultaneously
- **Progress bars**: Rich progress indication in Python/Bun
- **Interactive TUI**: Select files to update with visual interface
- **Plugins**: Runtime-specific extensions
- **Auto-update**: Check for CLI updates on each run (opt-in)
