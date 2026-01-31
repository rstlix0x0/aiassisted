# Development Plans

This directory contains the implementation plans for the `aiassisted` CLI rewrite from POSIX shell to Rust.

## Plan Structure

| File | Description | Status |
|------|-------------|--------|
| [overview.md](./overview.md) | Architecture decisions, project structure, and Rust policies | Reference |
| [phase-1-setup.md](./phase-1-setup.md) | Project setup & core abstractions | ✅ Complete |
| [phase-2-cli.md](./phase-2-cli.md) | CLI structure & composition root | ✅ Complete |
| [phase-3-content.md](./phase-3-content.md) | Content domain (install, update, check) | ✅ Complete |
| [phase-4-templates.md](./phase-4-templates.md) | Templates domain (setup-skills, setup-agents) | 🔄 Next |
| [phase-5-config.md](./phase-5-config.md) | Config domain | Pending |
| [phase-6-selfupdate.md](./phase-6-selfupdate.md) | Self-update domain | Pending |
| [phase-7-distribution.md](./phase-7-distribution.md) | cargo-dist & release workflow | Pending |
| [phase-8-polish.md](./phase-8-polish.md) | Documentation & integration tests | Pending |

## Quick Start

```bash
# Verify current state
cargo check
cargo test
cargo run -- --help

# Next phase: Templates Domain
# See phase-4-templates.md
```

## Rust Development Policies

See [overview.md](./overview.md) for full details. Key policies:

1. **Zero Warning Policy** - Code must compile with no warnings
2. **Static Dispatch** - Prefer generics over `dyn` traits
3. **Minimal `Arc`** - Only use when concurrent shared ownership is actually needed
