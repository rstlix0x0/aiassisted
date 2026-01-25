# aiassisted

A simple, safe installer for AI-Assisted Engineering Guidelines. This tool helps you maintain consistent AI development practices across your projects by providing a standardized `.aiassisted` directory with guidelines, instructions, and prompts.

## Features

- **One-command installation** via `curl` pipe
- **Multi-runtime support** - Choose between Shell, Python (UV), or Bun backends
- **Smart version tracking** using git commit hashes
- **Automatic update detection** with diff preview
- **Safe installation** to user directory (no sudo required)
- **POSIX-compliant** shell scripts for maximum portability
- **Colored terminal output** with automatic capability detection
- **Parallel downloads** with Python and Bun runtimes for faster installs

## Quick Start

Install the CLI tool and `.aiassisted` directory in one command:

```bash
curl -fsSL https://raw.githubusercontent.com/rstlix0x0/aiassisted/main/install.sh | sh
```

This will:
1. Install the `aiassisted` CLI to `~/.local/bin/`
2. Add `~/.local/bin` to your PATH (if needed)
3. Install `.aiassisted/` directory to your current directory
4. Show quick usage tips

After installation, restart your terminal or run:
```bash
source ~/.bashrc  # or ~/.zshrc, depending on your shell
```

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

Update the `aiassisted` CLI tool itself:
```bash
aiassisted self-update
```

### View Help

Show all available commands:
```bash
aiassisted help
```

## Commands Reference

| Command | Description | Options |
|---------|-------------|---------|
| `install` | Install .aiassisted to directory | `--path=DIR`, `--verbose`, `--quiet`, `--runtime=<shell\|python\|bun>` |
| `update` | Update existing installation | `--force`, `--path=DIR`, `--verbose`, `--quiet`, `--runtime=<shell\|python\|bun>` |
| `check` | Check if updates available | `--path=DIR`, `--runtime=<shell\|python\|bun>` |
| `version` | Show CLI version | `--runtime=<shell\|python\|bun>` |
| `self-update` | Update the CLI tool | - |
| `runtime list` | Show available runtimes | - |
| `runtime set <name>` | Set preferred runtime | - |
| `runtime info` | Show current runtime details | - |
| `help` | Show help message | - |

## Runtime Selection

`aiassisted` supports three different runtime backends:

### Available Runtimes

1. **Shell (Default)** - Pure POSIX sh, zero dependencies
   - Always available
   - Sequential downloads
   - ~2s install time

2. **Python + UV** - Modern Python with fast dependency management
   - Requires: [UV](https://docs.astral.sh/uv/getting-started/installation/)
   - Parallel downloads
   - Rich terminal output
   - ~500ms install time

3. **Bun** - Fast JavaScript runtime with native TypeScript
   - Requires: [Bun](https://bun.sh/docs/installation)
   - Parallel downloads
   - Native TypeScript support
   - ~400ms install time

### Check Available Runtimes

```bash
aiassisted runtime list
```

Output:
```
Available Runtimes:

  ✓ shell (default)
  ✓ python (detected: uv 0.9.16, active)
  ✓ bun (detected: 1.3.0)
```

### Set Preferred Runtime

Set your preferred runtime (saved to `~/.config/aiassisted/config`):

```bash
# Use Python runtime by default
aiassisted runtime set python

# Use Bun runtime by default
aiassisted runtime set bun

# Use Shell runtime by default
aiassisted runtime set shell
```

### Explicit Runtime Selection

Override the default runtime for a single command:

```bash
# Use Python runtime for this install
aiassisted install --runtime=python

# Use Bun runtime for this update
aiassisted update --runtime=bun

# Use Shell runtime for this check
aiassisted check --runtime=shell
```

### Runtime Auto-Detection

If no runtime is configured, `aiassisted` auto-detects in this order:

1. Python (if `uv` is available)
2. Bun (if `bun` is available)
3. Shell (always available)

## Global Options

- `--verbose` - Show detailed debug output
- `--quiet` - Show only errors
- `--path=DIR` - Specify target directory (default: current directory)
- `--runtime=<shell|python|bun>` - Select runtime backend explicitly
- `--force` - Skip confirmation prompts (update command only)

## What's Inside `.aiassisted`?

The `.aiassisted` directory contains:

```
.aiassisted/
├── .version                    # Version tracking (git commit hash)
├── FILES.txt                   # File manifest with SHA256 checksums
├── guidelines/
│   ├── architecture/          # Architecture patterns and best practices
│   ├── documentation/         # Documentation standards (Diátaxis, etc.)
│   └── rust/                  # Rust-specific guidelines
├── instructions/              # AI agent instructions and prompts
└── prompts/                   # Reusable prompt templates
```

These files provide:
- **Guidelines**: Best practices for architecture, documentation, and Rust development
- **Instructions**: Comprehensive AI agent behavior instructions
- **Prompts**: Ready-to-use prompt templates for common tasks

## Version Tracking

Each `.aiassisted` installation includes a `.version` file that tracks:
- **COMMIT_HASH**: Git commit hash from this repository
- **UPDATED_AT**: Last update timestamp

The CLI compares local and remote commit hashes to detect updates.

## Update Workflow

When you run `aiassisted update`:

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

**Benefits:**
- Only downloads files that actually changed
- Verifies file integrity with SHA256 checksums
- Saves bandwidth on partial updates
- Secure and efficient

## Examples

### Install in a New Project

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

**Made with care for AI-assisted development workflows**
