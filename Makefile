.PHONY: help update-version lint lint-strict install-shellcheck test test-cli test-installer test-runtimes clean status check-uncommitted

# Default target
help:
	@echo "aiassisted - Maintainer Makefile"
	@echo ""
	@echo "Common targets:"
	@echo "  make help              - Show this help message"
	@echo "  make update-version    - Update .version and FILES.txt with checksums"
	@echo "  make lint              - Lint all source code (shell, python, typescript)"
	@echo "  make lint-strict       - Lint all source code (fails if linters not available)"
	@echo "  make install-shellcheck - Install shellcheck via Homebrew (macOS only)"
	@echo "  make test              - Run all tests"
	@echo "  make test-cli          - Test CLI commands"
	@echo "  make test-installer    - Test installer script"
	@echo "  make test-runtimes     - Test all runtime backends"
	@echo "  make status            - Show git status and file counts"
	@echo "  make check-uncommitted - Check for uncommitted changes"
	@echo "  make clean             - Clean temporary files and build artifacts"
	@echo ""
	@echo "Runtime-specific targets:"
	@echo "  make lint-shell        - Lint shell scripts"
	@echo "  make lint-python       - Lint Python code (ruff/mypy)"
	@echo "  make lint-bun          - Lint TypeScript code"
	@echo "  make test-shell        - Test shell runtime"
	@echo "  make test-python       - Test Python runtime"
	@echo "  make test-bun          - Test Bun runtime"
	@echo "  make deps-python       - Install Python dependencies"
	@echo "  make deps-bun          - Install Bun dependencies"
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

# Lint shell scripts (graceful skip if shellcheck not installed)
lint: lint-shell lint-python lint-bun
	@echo ""
	@echo "✓ All linting passed"

# Lint shell scripts only
lint-shell:
	@echo "Linting shell scripts..."
	@if command -v shellcheck >/dev/null 2>&1; then \
		echo "  Checking install.sh..."; \
		shellcheck install.sh || exit 1; \
		echo "  Checking bin/aiassisted..."; \
		shellcheck bin/aiassisted || exit 1; \
		echo "  Checking src/shell/core.sh..."; \
		shellcheck src/shell/core.sh || exit 1; \
		echo "  Checking scripts/update-version.sh..."; \
		shellcheck scripts/update-version.sh || exit 1; \
		echo "  ✓ Shell scripts passed shellcheck"; \
	else \
		echo "  ⚠  shellcheck not found - skipping shell lint"; \
		echo "     Install: make install-shellcheck"; \
	fi

# Lint Python code
lint-python:
	@echo "Linting Python code..."
	@if command -v uv >/dev/null 2>&1; then \
		cd src/python && uv run ruff check aiassisted/ 2>/dev/null && echo "  ✓ Python code passed ruff" || echo "  ⚠  ruff not configured (optional)"; \
	else \
		echo "  ⚠  uv not found - skipping Python lint"; \
		echo "     Install: https://docs.astral.sh/uv/getting-started/installation/"; \
	fi

# Lint TypeScript code
lint-bun:
	@echo "Linting TypeScript code..."
	@if command -v bun >/dev/null 2>&1; then \
		cd src/bun && bun run --bun tsc --noEmit 2>/dev/null && echo "  ✓ TypeScript code passed type check" || echo "  ⚠  TypeScript type check skipped"; \
	else \
		echo "  ⚠  bun not found - skipping TypeScript lint"; \
		echo "     Install: https://bun.sh/docs/installation"; \
	fi

# Lint shell scripts (fails if shellcheck not installed - for CI)
lint-strict: lint-shell-strict lint-python lint-bun
	@echo ""
	@echo "✓ All linting passed (strict mode)"

# Strict shell linting
lint-shell-strict:
	@echo "Linting shell scripts (strict mode)..."
	@if ! command -v shellcheck >/dev/null 2>&1; then \
		echo "✗ Error: shellcheck is required but not installed"; \
		echo "  Install: brew install shellcheck (macOS)"; \
		exit 1; \
	fi
	@echo "  Checking install.sh..."
	@shellcheck install.sh || exit 1
	@echo "  Checking bin/aiassisted..."
	@shellcheck bin/aiassisted || exit 1
	@echo "  Checking src/shell/core.sh..."
	@shellcheck src/shell/core.sh || exit 1
	@echo "  Checking scripts/update-version.sh..."
	@shellcheck scripts/update-version.sh || exit 1
	@echo "  ✓ All shell scripts passed shellcheck"

# Install shellcheck (macOS only)
install-shellcheck:
	@echo "Installing shellcheck..."
	@if [ "$$(uname -s)" = "Darwin" ]; then \
		if command -v brew >/dev/null 2>&1; then \
			brew install shellcheck && echo "✓ shellcheck installed"; \
		else \
			echo "✗ Error: Homebrew not found"; \
			echo "  Install Homebrew from: https://brew.sh"; \
			exit 1; \
		fi; \
	else \
		echo "✗ Error: This target only works on macOS"; \
		echo "  On Linux, use your package manager:"; \
		echo "    Ubuntu/Debian: sudo apt-get install shellcheck"; \
		echo "    Fedora:        sudo dnf install shellcheck"; \
		exit 1; \
	fi

# Run all tests
test: test-syntax test-cli test-installer test-runtimes
	@echo ""
	@echo "✓ All tests passed"

# Test shell script syntax
test-syntax:
	@echo "Testing shell script syntax..."
	@sh -n install.sh && echo "  ✓ install.sh syntax OK" || exit 1
	@sh -n bin/aiassisted && echo "  ✓ bin/aiassisted syntax OK" || exit 1
	@sh -n src/shell/core.sh && echo "  ✓ src/shell/core.sh syntax OK" || exit 1
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

# Test all runtime backends
test-runtimes: test-shell test-python test-bun
	@echo "  ✓ All runtimes tested"

# Test shell runtime
test-shell:
	@echo "Testing shell runtime..."
	@./bin/aiassisted version --runtime=shell >/dev/null && echo "  ✓ Shell runtime works" || exit 1

# Test Python runtime
test-python:
	@echo "Testing Python runtime..."
	@if command -v uv >/dev/null 2>&1; then \
		./bin/aiassisted version --runtime=python 2>/dev/null >/dev/null && echo "  ✓ Python runtime works" || exit 1; \
	else \
		echo "  ⚠  UV not installed - skipping Python runtime test"; \
	fi

# Test Bun runtime
test-bun:
	@echo "Testing Bun runtime..."
	@if command -v bun >/dev/null 2>&1; then \
		./bin/aiassisted version --runtime=bun >/dev/null && echo "  ✓ Bun runtime works" || exit 1; \
	else \
		echo "  ⚠  Bun not installed - skipping Bun runtime test"; \
	fi

# Show project status
status:
	@echo "Project Status:"
	@echo ""
	@echo "Git Status:"
	@git status --short
	@echo ""
	@echo "Runtime Availability:"
	@echo "  Shell:  ✓ (always available)"
	@command -v uv >/dev/null 2>&1 && echo "  Python: ✓ (uv $$(uv --version | cut -d' ' -f2))" || echo "  Python: ✗ (uv not installed)"
	@command -v bun >/dev/null 2>&1 && echo "  Bun:    ✓ (bun $$(bun --version))" || echo "  Bun:    ✗ (bun not installed)"
	@echo ""
	@echo "File Counts:"
	@echo "  Guidelines:   $$(find .aiassisted/guidelines -type f 2>/dev/null | wc -l | tr -d ' ') files"
	@echo "  Instructions: $$(find .aiassisted/instructions -type f 2>/dev/null | wc -l | tr -d ' ') files"
	@echo "  Prompts:      $$(find .aiassisted/prompts -type f 2>/dev/null | wc -l | tr -d ' ') files"
	@echo "  Total:        $$(find .aiassisted -type f ! -name '.version' ! -name 'FILES.txt' 2>/dev/null | wc -l | tr -d ' ') files"
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
	@echo "Cleaning temporary files and build artifacts..."
	@find . -name "*.tmp" -delete
	@find . -name ".DS_Store" -delete
	@rm -rf src/python/.venv
	@rm -rf src/python/__pycache__
	@rm -rf src/python/**/__pycache__
	@rm -rf src/bun/node_modules
	@echo "✓ Cleaned"

# Install Python dependencies
deps-python:
	@echo "Installing Python dependencies..."
	@if command -v uv >/dev/null 2>&1; then \
		cd src/python && uv sync && echo "✓ Python dependencies installed"; \
	else \
		echo "✗ Error: UV not installed"; \
		echo "  Install: https://docs.astral.sh/uv/getting-started/installation/"; \
		exit 1; \
	fi

# Install Bun dependencies
deps-bun:
	@echo "Installing Bun dependencies..."
	@if command -v bun >/dev/null 2>&1; then \
		cd src/bun && bun install && echo "✓ Bun dependencies installed"; \
	else \
		echo "✗ Error: Bun not installed"; \
		echo "  Install: https://bun.sh/docs/installation"; \
		exit 1; \
	fi

# Install all dependencies
deps: deps-python deps-bun
	@echo ""
	@echo "✓ All dependencies installed"

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
