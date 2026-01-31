# Phase 7: Distribution with cargo-dist

**Status:** Partially Complete

## Objectives

- Configure cargo-dist for automated releases
- Update install.sh for cargo-dist URLs
- Generate shell completions
- Test release workflow

## Completed Tasks

- [x] Configured `dist-workspace.toml`
- [x] Generated `.github/workflows/release.yml`

## Remaining Tasks

- [ ] Update `install.sh` for cargo-dist URLs
- [ ] Test release workflow with a pre-release tag
- [ ] Generate shell completions (bash, zsh, fish, PowerShell)

## cargo-dist Configuration

File: `dist-workspace.toml`

```toml
[workspace]
members = ["."]

[dist]
cargo-dist-version = "0.x.x"
targets = [
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
]
installers = ["shell", "powershell"]
```

## Release Process

1. Update version in `Cargo.toml`
2. Commit and push changes
3. Create and push a tag:
   ```bash
   git tag "v0.1.0"
   git push --tags
   ```
4. GitHub Actions automatically:
   - Builds binaries for all platforms
   - Creates GitHub Release
   - Uploads platform-specific binaries
   - Generates install scripts

## Install Script Update

Current: `install.sh` clones git repository
New: `install.sh` downloads pre-built binary

```bash
#!/bin/bash
set -euo pipefail

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Map architecture
case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    arm64|aarch64) ARCH="aarch64" ;;
esac

# Download URL
VERSION="latest"
BASE_URL="https://github.com/rstlix0x0/aiassisted/releases/${VERSION}/download"
BINARY="aiassisted-${ARCH}-${OS}"

# Download and install
curl -fsSL "${BASE_URL}/${BINARY}.tar.gz" | tar xz
chmod +x aiassisted
mv aiassisted ~/.local/bin/
```

## Shell Completions

Generate completions during build:

```rust
// build.rs
use clap_complete::{generate_to, shells::*};

fn main() {
    let mut cmd = build_cli();
    let outdir = "completions";
    generate_to(Bash, &mut cmd, "aiassisted", outdir);
    generate_to(Zsh, &mut cmd, "aiassisted", outdir);
    generate_to(Fish, &mut cmd, "aiassisted", outdir);
    generate_to(PowerShell, &mut cmd, "aiassisted", outdir);
}
```

Include in release artifacts.

## Testing

```bash
# Test local build
cargo dist build

# Test release workflow (dry run)
cargo dist plan

# Test with pre-release tag
git tag "v0.1.0-rc1"
git push --tags
```
