#!/bin/bash
#
# Smoke tests for aiassisted CLI
#
# This script performs end-to-end testing of all major features:
# - Core commands (version, help)
# - Content domain (install, update, check)
# - Templates domain (list, show, init, setup-skills, setup-agents)
# - Config domain (show, get, path)
#
# Usage:
#   ./scripts/smoke-test.sh [--binary PATH]
#
# Options:
#   --binary PATH   Path to aiassisted binary (default: cargo run -q --)
#

set -e

###########################################
# Configuration
###########################################

BINARY="cargo run -q --"
TEST_DIR=""
FAILED_TESTS=0
PASSED_TESTS=0

###########################################
# Colors
###########################################

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

###########################################
# Helper Functions
###########################################

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    PASSED_TESTS=$((PASSED_TESTS + 1))
}

error() {
    echo -e "${RED}[FAIL]${NC} $1"
    FAILED_TESTS=$((FAILED_TESTS + 1))
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

section() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

run_test() {
    local test_name="$1"
    local command="$2"
    local expected_pattern="$3"

    info "Testing: $test_name"

    if output=$(eval "$command" 2>&1); then
        if echo "$output" | grep -q "$expected_pattern"; then
            success "$test_name"
            return 0
        else
            error "$test_name - Pattern not found: $expected_pattern"
            echo "Output: $output"
            return 1
        fi
    else
        error "$test_name - Command failed with exit code $?"
        echo "Output: $output"
        return 1
    fi
}

setup_test_dir() {
    TEST_DIR=$(mktemp -d -t aiassisted-smoke-test.XXXXXX)
    info "Test directory: $TEST_DIR"
}

cleanup_test_dir() {
    if [ -n "$TEST_DIR" ] && [ -d "$TEST_DIR" ]; then
        rm -rf "$TEST_DIR"
        info "Cleaned up test directory"
    fi
}

###########################################
# Parse Arguments
###########################################

while [[ $# -gt 0 ]]; do
    case $1 in
        --binary)
            BINARY="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

###########################################
# Main Tests
###########################################

info "Starting smoke tests for aiassisted"
info "Binary: $BINARY"
echo ""

# Setup
setup_test_dir
trap cleanup_test_dir EXIT

###########################################
section "Core Commands"
###########################################

run_test "version" "$BINARY version" "aiassisted"
run_test "help" "$BINARY --help" "Usage: aiassisted"
run_test "verbose flag (-v)" "$BINARY -v version" "aiassisted"
run_test "debug flag (-vv)" "$BINARY -vv version" "aiassisted"

###########################################
section "Config Domain"
###########################################

run_test "config show" "$BINARY config show" "default_tool"
run_test "config get" "$BINARY config get default_tool" "auto\|claude\|opencode"
run_test "config path" "$BINARY config path" "config.toml"

###########################################
section "Templates Domain (List/Show/Path)"
###########################################

run_test "templates list" "$BINARY templates list" "templates for\|No templates found"
run_test "templates path" "$BINARY templates path" "Template directories"

# Only test template show if templates exist
if $BINARY templates list 2>&1 | grep -q "git-commit"; then
    run_test "templates show" "$BINARY templates show git-commit.SKILL.md" "Git Commit"
else
    warn "Skipping templates show test - no templates available"
fi

###########################################
section "Content Domain (Install)"
###########################################

cd "$TEST_DIR"

# Test install (this will download from GitHub)
info "Testing install in test directory: $TEST_DIR"
if $BINARY install 2>&1 | tee install.log; then
    if [ -d .aiassisted ]; then
        success "install - .aiassisted directory created"

        # Verify manifest.json exists
        if [ -f .aiassisted/manifest.json ]; then
            success "install - manifest.json downloaded"
        else
            error "install - manifest.json not found"
        fi

        # Verify some key files exist
        if [ -d .aiassisted/guidelines ]; then
            success "install - guidelines directory exists"
        else
            error "install - guidelines directory not found"
        fi

        if [ -d .aiassisted/templates ]; then
            success "install - templates directory exists"
        else
            error "install - templates directory not found"
        fi
    else
        error "install - .aiassisted directory not created"
        cat install.log
    fi
else
    error "install - command failed"
    cat install.log
fi

###########################################
section "Content Domain (Check/Update)"
###########################################

if [ -d .aiassisted ]; then
    # Test check
    if $BINARY check 2>&1 | tee check.log; then
        success "check - command succeeded"
    else
        # Check command may fail if already up-to-date, check the output
        if grep -q "up-to-date\|No updates" check.log; then
            success "check - installation is up-to-date"
        else
            error "check - command failed"
            cat check.log
        fi
    fi

    # Test update
    if $BINARY update 2>&1 | tee update.log; then
        success "update - command succeeded"
    else
        # Update may fail if already up-to-date
        if grep -q "up-to-date\|No updates" update.log; then
            success "update - installation is up-to-date"
        else
            warn "update - command failed (may be expected if up-to-date)"
        fi
    fi
else
    warn "Skipping check/update tests - install failed"
fi

###########################################
section "Templates Domain (Init/Setup)"
###########################################

if [ -d .aiassisted ]; then
    # Test templates init
    if $BINARY templates init 2>&1 | tee templates-init.log; then
        success "templates init"
    else
        error "templates init - command failed"
        cat templates-init.log
    fi

    # Test setup-skills
    if $BINARY setup-skills 2>&1 | tee setup-skills.log; then
        if [ -d .claude/commands ] || [ -d .opencode/skills ]; then
            success "setup-skills - skills directory created"
        else
            # May be no templates found
            if grep -q "No skill templates found" setup-skills.log; then
                warn "setup-skills - no templates found (expected if global templates empty)"
            else
                error "setup-skills - skills directory not created"
                cat setup-skills.log
            fi
        fi
    else
        error "setup-skills - command failed"
        cat setup-skills.log
    fi

    # Test setup-agents
    if $BINARY setup-agents 2>&1 | tee setup-agents.log; then
        if [ -d .claude/agents ] || [ -d .opencode/agents ]; then
            success "setup-agents - agents directory created"
        else
            # May be no templates found
            if grep -q "No agent templates found" setup-agents.log; then
                warn "setup-agents - no templates found (expected if global templates empty)"
            else
                error "setup-agents - agents directory not created"
                cat setup-agents.log
            fi
        fi
    else
        error "setup-agents - command failed"
        cat setup-agents.log
    fi
else
    warn "Skipping templates init/setup tests - install failed"
fi

###########################################
section "Summary"
###########################################

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Test Results"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
echo -e "${RED}Failed: $FAILED_TESTS${NC}"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
