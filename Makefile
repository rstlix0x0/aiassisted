.PHONY: help update-version lint test test-cli test-installer clean status check-uncommitted

# Default target
help:
	@echo "aiassisted - Maintainer Makefile"
	@echo ""
	@echo "Common targets:"
	@echo "  make help              - Show this help message"
	@echo "  make update-version    - Update .version and FILES.txt with checksums"
	@echo "  make lint              - Lint shell scripts with shellcheck"
	@echo "  make test              - Run all tests"
	@echo "  make test-cli          - Test CLI commands"
	@echo "  make test-installer    - Test installer script"
	@echo "  make status            - Show git status and file counts"
	@echo "  make check-uncommitted - Check for uncommitted changes"
	@echo "  make clean             - Clean temporary files"
	@echo ""
	@echo "Workflow for updating guidelines:"
	@echo "  1. Edit files in .aiassisted/"
	@echo "  2. make update-version    # Regenerate manifest and version"
	@echo "  3. make test              # Verify everything works"
	@echo "  4. git add .aiassisted/"
	@echo "  5. git commit -m 'docs: update guidelines'"
	@echo "  6. git push origin main"

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

# Lint shell scripts
lint:
	@echo "Linting shell scripts..."
	@if command -v shellcheck >/dev/null 2>&1; then \
		echo "Checking install.sh..."; \
		shellcheck install.sh || exit 1; \
		echo "Checking bin/aiassisted..."; \
		shellcheck bin/aiassisted || exit 1; \
		echo "Checking scripts/update-version.sh..."; \
		shellcheck scripts/update-version.sh || exit 1; \
		echo "✓ All scripts passed shellcheck"; \
	else \
		echo "⚠ shellcheck not found, skipping lint"; \
		echo "  Install: brew install shellcheck (macOS)"; \
		echo "           apt-get install shellcheck (Debian/Ubuntu)"; \
	fi

# Run all tests
test: test-syntax test-cli test-installer
	@echo ""
	@echo "✓ All tests passed"

# Test shell script syntax
test-syntax:
	@echo "Testing shell script syntax..."
	@sh -n install.sh && echo "  ✓ install.sh syntax OK" || exit 1
	@sh -n bin/aiassisted && echo "  ✓ bin/aiassisted syntax OK" || exit 1
	@sh -n scripts/update-version.sh && echo "  ✓ scripts/update-version.sh syntax OK" || exit 1

# Test CLI commands
test-cli:
	@echo "Testing CLI commands..."
	@./bin/aiassisted version >/dev/null && echo "  ✓ aiassisted version" || exit 1
	@./bin/aiassisted help >/dev/null && echo "  ✓ aiassisted help" || exit 1
	@echo "  ✓ CLI commands working"

# Test installer script syntax and structure
test-installer:
	@echo "Testing installer..."
	@sh -n install.sh && echo "  ✓ Installer syntax OK" || exit 1
	@grep -q "GITHUB_REPO=" install.sh && echo "  ✓ GITHUB_REPO defined" || exit 1
	@grep -q "download_file" install.sh && echo "  ✓ download_file function exists" || exit 1

# Show project status
status:
	@echo "Project Status:"
	@echo ""
	@echo "Git Status:"
	@git status --short
	@echo ""
	@echo "File Counts:"
	@echo "  Guidelines:  $$(find .aiassisted/guidelines -type f | wc -l | tr -d ' ') files"
	@echo "  Instructions: $$(find .aiassisted/instructions -type f | wc -l | tr -d ' ') files"
	@echo "  Prompts:      $$(find .aiassisted/prompts -type f | wc -l | tr -d ' ') files"
	@echo "  Total:        $$(find .aiassisted -type f ! -name '.version' ! -name 'FILES.txt' | wc -l | tr -d ' ') files"
	@echo ""
	@if [ -f .aiassisted/.version ]; then \
		echo "Current Version:"; \
		grep "COMMIT_HASH" .aiassisted/.version | sed 's/^/  /'; \
		grep "UPDATED_AT" .aiassisted/.version | sed 's/^/  /'; \
	fi

# Check for uncommitted changes (useful in CI)
check-uncommitted:
	@echo "Checking for uncommitted changes..."
	@if [ -n "$$(git status --porcelain)" ]; then \
		echo "⚠ Uncommitted changes detected:"; \
		git status --short; \
		exit 1; \
	else \
		echo "✓ No uncommitted changes"; \
	fi

# Clean temporary files
clean:
	@echo "Cleaning temporary files..."
	@find . -name "*.tmp" -delete
	@find . -name ".DS_Store" -delete
	@echo "✓ Cleaned"

# Verify FILES.txt is up-to-date
verify-manifest:
	@echo "Verifying FILES.txt manifest..."
	@scripts/update-version.sh > /dev/null
	@if git diff --quiet .aiassisted/FILES.txt; then \
		echo "✓ FILES.txt is up-to-date"; \
	else \
		echo "⚠ FILES.txt is out of date"; \
		echo "  Run: make update-version"; \
		exit 1; \
	fi

# Show version information
version:
	@echo "Version Information:"
	@echo ""
	@echo "CLI Version:"
	@./bin/aiassisted version
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

# Pre-commit checks (useful as git pre-commit hook)
pre-commit: lint test-syntax verify-manifest
	@echo ""
	@echo "✓ Pre-commit checks passed"
