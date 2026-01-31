.PHONY: help update-version lint test check build run clean status check-uncommitted version

# Default target
help:
	@echo "aiassisted - Maintainer Makefile"
	@echo ""
	@echo "Development targets:"
	@echo "  make check             - Check code compiles (cargo check)"
	@echo "  make test              - Run all tests (cargo test)"
	@echo "  make build             - Build release binary"
	@echo "  make run               - Run the CLI (cargo run -- --help)"
	@echo "  make lint              - Run clippy linter"
	@echo ""
	@echo "Content management:"
	@echo "  make update-version    - Update .version and FILES.txt with checksums"
	@echo "  make verify-manifest   - Verify FILES.txt is up-to-date"
	@echo "  make commit-guidelines - Quick commit workflow for guidelines"
	@echo ""
	@echo "Utility targets:"
	@echo "  make status            - Show git status and file counts"
	@echo "  make version           - Show version information"
	@echo "  make clean             - Clean build artifacts and temp files"
	@echo "  make check-uncommitted - Check for uncommitted changes"
	@echo ""
	@echo "Workflow for updating guidelines:"
	@echo "  1. Edit files in .aiassisted/"
	@echo "  2. make update-version    # Regenerate manifest and version"
	@echo "  3. make test              # Verify everything works"
	@echo "  4. git add .aiassisted/"
	@echo "  5. git commit -m 'docs: update guidelines'"
	@echo "  6. git push origin main"

# Check code compiles
check:
	@echo "Checking code..."
	@cargo check
	@echo ""
	@echo "✓ Code compiles successfully"

# Run all tests
test:
	@echo "Running tests..."
	@cargo test
	@echo ""
	@echo "✓ All tests passed"

# Build release binary
build:
	@echo "Building release binary..."
	@cargo build --release
	@echo ""
	@echo "✓ Binary built: target/release/aiassisted"

# Run the CLI
run:
	@cargo run -- --help

# Run clippy linter
lint:
	@echo "Running clippy..."
	@cargo clippy -- -D warnings
	@echo ""
	@echo "✓ Linting passed"

# Format code
fmt:
	@echo "Formatting code..."
	@cargo fmt
	@echo "✓ Code formatted"

# Check code formatting
fmt-check:
	@echo "Checking code formatting..."
	@cargo fmt -- --check
	@echo "✓ Code formatting is correct"

# Update version and regenerate FILES.txt manifest with checksums
update-version:
	@echo "Updating version and manifest..."
	@./scripts/update-version.sh
	@echo ""
	@echo "✓ Version updated successfully"
	@echo ""
	@echo "Next steps:"
	@echo "  git add .aiassisted/"
	@echo "  git commit -m 'docs: update .aiassisted content'"

# Verify FILES.txt is up-to-date
verify-manifest:
	@echo "Verifying FILES.txt manifest..."
	@./scripts/update-version.sh > /dev/null
	@if git diff --quiet .aiassisted/FILES.txt; then \
		echo "✓ FILES.txt is up-to-date"; \
	else \
		echo "⚠ FILES.txt is out of date"; \
		echo "  Run: make update-version"; \
		exit 1; \
	fi

# Show project status
status:
	@echo "Project Status:"
	@echo ""
	@echo "Git Status:"
	@git status --short
	@echo ""
	@echo "File Counts:"
	@echo "  Guidelines:   $$(find .aiassisted/guidelines -type f 2>/dev/null | wc -l | tr -d ' ') files"
	@echo "  Instructions: $$(find .aiassisted/instructions -type f 2>/dev/null | wc -l | tr -d ' ') files"
	@echo "  Prompts:      $$(find .aiassisted/prompts -type f 2>/dev/null | wc -l | tr -d ' ') files"
	@echo "  Templates:    $$(find .aiassisted/templates -type f 2>/dev/null | wc -l | tr -d ' ') files"
	@echo "  Total:        $$(find .aiassisted -type f ! -name '.version' ! -name 'FILES.txt' 2>/dev/null | wc -l | tr -d ' ') files"
	@echo ""
	@if [ -f .aiassisted/.version ]; then \
		echo "Content Version:"; \
		grep "COMMIT_HASH" .aiassisted/.version | sed 's/^/  /'; \
		grep "UPDATED_AT" .aiassisted/.version | sed 's/^/  /'; \
	fi
	@echo ""
	@echo "Rust:"
	@echo "  Cargo version:  $$(cargo --version)"
	@echo "  Binary version: $$(cargo run -q -- version 2>/dev/null || echo 'Not built')"

# Check for uncommitted changes
check-uncommitted:
	@echo "Checking for uncommitted changes..."
	@if [ -n "$$(git status --porcelain)" ]; then \
		echo "⚠ Uncommitted changes detected:"; \
		git status --short; \
		exit 1; \
	else \
		echo "✓ No uncommitted changes"; \
	fi

# Show version information
version:
	@echo "Version Information:"
	@echo ""
	@echo "CLI Version:"
	@cargo run -q -- version
	@echo ""
	@if [ -f .aiassisted/.version ]; then \
		echo "Content Version:"; \
		cat .aiassisted/.version; \
	fi

# Quick commit workflow for guidelines updates
commit-guidelines:
	@echo "Committing guidelines updates..."
	@if [ -z "$$(git status --porcelain .aiassisted/)" ]; then \
		echo "⚠ No changes in .aiassisted/ directory"; \
		exit 1; \
	fi
	@./scripts/update-version.sh
	@git add .aiassisted/
	@echo ""
	@echo "Ready to commit. Suggested commit message:"
	@echo "  git commit -m 'docs: update guidelines and instructions'"
	@echo ""
	@echo "Or run: git commit"

# Show what files changed in .aiassisted
diff-aiassisted:
	@echo "Changes in .aiassisted/:"
	@echo ""
	@git diff --stat .aiassisted/
	@echo ""
	@git diff .aiassisted/

# Clean build artifacts and temporary files
clean:
	@echo "Cleaning build artifacts and temporary files..."
	@cargo clean
	@find . -name "*.tmp" -delete
	@find . -name ".DS_Store" -delete
	@echo "✓ Cleaned"

# Pre-commit checks
pre-commit: fmt-check check test verify-manifest
	@echo ""
	@echo "✓ Pre-commit checks passed"

# Install the binary locally
install:
	@echo "Installing binary to ~/.local/bin/..."
	@cargo install --path .
	@echo "✓ Installed to ~/.local/bin/aiassisted"

# Watch for changes and run tests
watch:
	@echo "Watching for changes..."
	@cargo watch -x test

# Generate documentation
docs:
	@echo "Generating documentation..."
	@cargo doc --no-deps --open

# CI workflow
ci: fmt-check check test
	@echo ""
	@echo "✓ CI checks passed"

# Release workflow
release: pre-commit
	@echo ""
	@echo "Ready for release!"
	@echo ""
	@echo "Next steps:"
	@echo "  1. Update version in Cargo.toml"
	@echo "  2. git add Cargo.toml Cargo.lock"
	@echo "  3. git commit -m 'chore: bump version to vX.Y.Z'"
	@echo "  4. git tag vX.Y.Z"
	@echo "  5. git push origin main --tags"
