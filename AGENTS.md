# AGENTS.md - Coding Agent Guidelines

This document provides guidelines for AI coding agents working in the `aiassisted` repository.

## Project Overview

A POSIX-compliant shell script installer for AI-Assisted Engineering Guidelines. The project installs and manages `.aiassisted` directories with guidelines, instructions, and prompts for AI development workflows.

**Key Components:**
- `install.sh` - One-command installer script (pipe to sh)
- `bin/aiassisted` - CLI tool for managing `.aiassisted` installations
- `.aiassisted/` - Guidelines, instructions, and prompts directory

## Build, Lint, and Test Commands

### No Build Required
This is a pure shell script project with no compilation step.

### Using Make (Recommended)

The project includes a Makefile for common tasks:

```bash
# Show all available commands
make help

# Update version after editing .aiassisted/
make update-version

# Run all tests
make test

# Lint shell scripts (requires shellcheck)
make lint

# Show project status
make status

# Clean temporary files
make clean
```

### Testing
Currently no automated test suite exists.

**Manual Testing:**
```bash
# Test the installer
./install.sh

# Test CLI commands
./bin/aiassisted help
./bin/aiassisted install --path=/tmp/test-project
./bin/aiassisted check --path=/tmp/test-project
./bin/aiassisted update --path=/tmp/test-project

# Test with different shells
sh ./bin/aiassisted help
bash ./bin/aiassisted help
zsh ./bin/aiassisted help
```

### Linting
```bash
# Check POSIX compliance with shellcheck (if available)
shellcheck install.sh bin/aiassisted

# Verify script executes without errors
sh -n install.sh
sh -n bin/aiassisted
```

### Running Single Test
Since there are no automated tests, validate specific functionality:
```bash
# Test a specific command
./bin/aiassisted version

# Test with verbose output
./bin/aiassisted install --verbose --path=/tmp/test
```

## Code Style Guidelines

### Shell Scripting Standards

#### POSIX Compliance
- **MUST** be POSIX-compliant (sh, dash, ash, bash, zsh compatible)
- **NO** bash-specific features (arrays, `[[`, `source`, etc.)
- Use `command -v` instead of `which`
- Use `[ ]` instead of `[[ ]]`
- Use `printf` instead of `echo` for formatted output

#### Script Structure
```bash
#!/bin/sh
#
# Script description
# Usage examples
#

set -e  # Exit on error

# Constants (UPPERCASE)
VERSION="1.0.0"
GITHUB_REPO="rstlix0x0/aiassisted"

# Functions (snake_case with underscores)
function_name() {
    _local_var="value"  # Prefix local vars with underscore
    
    # Function body
}

# Main entry point
main() {
    # Main logic
}

# Execute main
main "$@"
```

#### Naming Conventions
- **Constants:** UPPERCASE_WITH_UNDERSCORES (e.g., `GITHUB_REPO`, `COLOR_RED`)
- **Functions:** lowercase_with_underscores (e.g., `log_error`, `download_file`)
- **Local variables:** prefix with underscore (e.g., `_temp_dir`, `_local_hash`)
- **Global variables:** UPPERCASE (e.g., `VERBOSITY`)

#### Variables
- Always quote variables: `"$var"` not `$var`
- Use parameter expansion: `${var}` for clarity
- Local vars in functions start with underscore: `_temp_file=$(mktemp)`
- Check variable existence: `[ -n "$var" ]` or `[ -z "$var" ]`

#### Error Handling
```bash
# Check command success
if ! command_that_might_fail; then
    log_error "Command failed"
    return 1
fi

# Cleanup temp files
_temp=$(mktemp)
trap 'rm -f "$_temp"' EXIT

# Validate prerequisites
if ! command -v curl >/dev/null 2>&1; then
    log_error "curl not found"
    exit 1
fi
```

#### Logging Functions
Use consistent logging with color support:
```bash
log_error "Error message"   # Red, always shown, to stderr
log_success "Success"        # Green, respects VERBOSITY
log_info "Information"       # Blue, respects VERBOSITY
log_warn "Warning"           # Yellow, respects VERBOSITY
log_debug "Debug detail"     # Blue, only if VERBOSITY >= 2
```

#### Color Support
- Detect terminal color capability with `tput`
- Provide fallbacks for non-color terminals
- Use variables: `COLOR_RED`, `COLOR_GREEN`, `COLOR_YELLOW`, `COLOR_BLUE`, `COLOR_RESET`

#### Path Handling
```bash
# Resolve to absolute path
_target_path=$(cd "$_target_path" 2>/dev/null && pwd || echo "$_target_path")

# Check directory/file existence
if [ -d "$_path" ]; then
if [ -f "$_file" ]; then
if [ -w "$_dir" ]; then  # writable
```

#### Downloads
```bash
# Detect curl or wget
_tool=$(detect_download_tool)

# Download with proper error handling
if ! download_file "$url" "$output"; then
    log_error "Download failed"
    return 1
fi
```

#### User Input
```bash
# Prompt for confirmation
printf "%s%s [y/N]:%s " "$COLOR_YELLOW" "$_prompt" "$COLOR_RESET"
read -r _response

case "$_response" in
    [yY]|[yY][eE][sS])
        return 0
        ;;
    *)
        return 1
        ;;
esac
```

### Documentation

#### Script Headers
```bash
#!/bin/sh
#
# Script Name - Brief description
# 
# Longer description explaining what the script does,
# its purpose, and any important context.
#
# Usage:
#   script_name <command> [options]
#   script_name --help
#
```

#### Function Documentation
```bash
# Download file from URL to output path
# Usage: download_file <url> <output_file>
# Returns: 0 on success, 1 on failure
download_file() {
    _url="$1"
    _output="$2"
    # ...
}
```

#### Comments
- Explain **why** not **what** (code shows what)
- Section dividers for organization:
```bash
###########################################
# Section Name
###########################################
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

**Format:** `<type>[optional scope]: <description>`

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `refactor`: Code restructuring
- `test`: Test additions/changes
- `chore`: Maintenance tasks
- `ci`: CI/CD changes

**Examples:**
```
feat(cli): add self-update command
fix: handle missing version file gracefully
docs: update README with installation steps
refactor: extract version comparison logic
```

**Rules:**
- Use imperative mood ("add" not "added")
- No period at end
- Keep under 72 characters
- Breaking changes: append `!` or add `BREAKING CHANGE:` footer

## Security Considerations

- **Never** require sudo/root access
- Only write to user directories (`~/.local/bin`, current dir)
- Validate downloads before execution
- Show diffs before applying updates
- Use `set -e` to exit on errors
- Clean up temporary files with `trap`

## File Operations

```bash
# Create directory safely
if ! mkdir -p "$_dir"; then
    log_error "Failed to create directory"
    exit 1
fi

# Copy recursively
cp -r "$_source" "$_target"

# Remove safely
rm -rf "$_temp_dir"

# Create temp file/directory
_temp_file=$(mktemp)
_temp_dir=$(mktemp -d)
```

## Development Workflow

### Using Make (Recommended)
1. **Edit scripts** in `install.sh` or `bin/aiassisted`
2. **Run tests**: `make test`
3. **Lint code**: `make lint`
4. **Update README.md** if user-facing changes
5. **Commit** with conventional commit message

### When Updating `.aiassisted/` Content
1. **Edit files** in `.aiassisted/` directory
2. **Update version**: `make update-version`
3. **Test**: `make test`
4. **Commit**: `git add .aiassisted/ && git commit -m 'docs: update guidelines'`

### Manual Testing

## Repository Structure

```
aiassisted/
├── bin/
│   └── aiassisted          # Main CLI tool
├── scripts/
│   └── update-version.sh   # Helper to update version and manifest
├── .aiassisted/            # Guidelines and instructions
│   ├── .version           # Version tracking
│   ├── FILES.txt          # File manifest with SHA256 checksums
│   ├── guidelines/        # Best practices
│   ├── instructions/      # AI agent instructions
│   └── prompts/          # Reusable prompts
├── install.sh             # One-command installer
├── Makefile               # Maintainer tasks
├── README.md             # User documentation
├── AGENTS.md             # This file
└── LICENSE               # MIT License
```

## Tips for AI Agents

- Maintain POSIX compatibility - test with `sh` not just `bash`
- Preserve existing logging and color patterns
- Always handle errors explicitly
- Clean up temporary files
- Use existing utility functions (download_file, log_*, etc.)
- Keep functions focused and single-purpose
- Test with `--verbose` flag for debugging
- Respect the `VERBOSITY` level in output

## File Manifest System

The CLI uses `FILES.txt` manifest with SHA256 checksums for efficient updates:

**Format:** `filepath:sha256hash`

**Example:**
```
guidelines/rust/rust-adt-implementation-guide.md:558b09b2c97f47042210800905dfaa24cba366f43ed997da411952fe57f8f47b
instructions/rust.instructions.md:7d18292fa0f3b01b90410cb0bc684c7a1b2247d16f73bc5c958d60fbf0013ba8
```

**Features:**
- Selective updates: Only downloads files with changed checksums
- Integrity verification: Verifies SHA256 after download
- Bandwidth efficient: Skips unchanged files
- POSIX-compliant: Works with sha256sum, shasum, or openssl

**When adding/removing files in `.aiassisted/`:**
1. Use `scripts/update-version.sh` to regenerate `FILES.txt` with checksums
2. The script automatically scans, calculates checksums, and updates manifest
3. Commit both content changes and updated `FILES.txt`

**Never hardcode file lists** in shell scripts - always use the manifest approach.
