# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`aiassisted` is a POSIX-compliant shell CLI tool that embeds a `.aiassisted/` directory into projects. This directory contains curated guidelines, instructions, prompts, and templates that AI assistants can reference for consistent, context-aware assistance.

## Common Commands

### Development

```bash
# Run all tests (syntax, CLI, installer, setup)
make test

# Lint shell scripts (requires shellcheck)
make lint

# Update version and FILES.txt manifest after modifying .aiassisted/ content
make update-version

# Show project status and file counts
make status

# Clean temporary files
make clean
```

### Testing Individual Components

```bash
# Test CLI commands only
make test-cli

# Test installer syntax
make test-installer

# Test setup-skills and setup-agents
make test-setup

# Verify FILES.txt is current
make verify-manifest
```

### CLI Usage (for testing)

```bash
# Run CLI directly from source
./bin/aiassisted help
./bin/aiassisted version
./bin/aiassisted setup-skills --dry-run
./bin/aiassisted setup-agents --dry-run
```

## Architecture

### Source Code Structure

- `bin/aiassisted` - Entry point script, resolves symlinks and executes `src/shell/core.sh`
- `src/shell/core.sh` - Main CLI implementation (all commands, template processing, config management)
- `install.sh` - Standalone installer script for curl-pipe installation
- `scripts/update-version.sh` - Regenerates `.aiassisted/.version` and `FILES.txt` manifest

### Key Design Decisions

1. **Pure POSIX shell** - No bash-isms, works with sh/dash/ash. Only external dependency is `git`.

2. **Symlink-based installation** - CLI installs to `~/.aiassisted/source/aiassisted/` with symlink at `~/.local/bin/aiassisted`.

3. **Template system** - Templates in `.aiassisted/templates/` are processed to generate skills/agents in `.opencode/` or `.claude/` directories.

4. **Cascading template resolution** - Project templates (`./.aiassisted/templates/`) override global templates (`~/.aiassisted/templates/`).

5. **Checksum-based updates** - `FILES.txt` contains SHA256 hashes for efficient partial updates.

## Content Organization

The `.aiassisted/` directory contains:

- `guidelines/` - Architecture patterns, documentation standards, language-specific guides
- `instructions/` - AI agent behavior rules and constraints
- `prompts/` - Reusable prompt templates (e.g., commit messages)
- `templates/` - Skill and agent templates for OpenCode and Claude Code
- `config/` - Configuration documentation

## Workflow for Updating Guidelines

1. Edit files in `.aiassisted/`
2. Run `make update-version` to regenerate manifest
3. Run `make test` to verify
4. Commit changes
