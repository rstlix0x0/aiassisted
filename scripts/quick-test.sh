#!/bin/bash
#
# Quick smoke test - verifies basic functionality
#
# Tests:
# 1. Version command works
# 2. manifest.json can be parsed
# 3. Install command can download from GitHub (requires manifest.json to be pushed)
#

set -e

BINARY="${1:-cargo run -q --}"

echo "Quick Test for aiassisted"
echo "========================="
echo ""

# Test 1: Version
echo "Test 1: Version command"
if $BINARY version | grep -q "aiassisted"; then
    echo "✓ Version command works"
else
    echo "✗ Version command failed"
    exit 1
fi
echo ""

# Test 2: Local manifest.json is valid
echo "Test 2: Validate local manifest.json"
if [ -f .aiassisted/manifest.json ]; then
    if python3 -m json.tool .aiassisted/manifest.json > /dev/null 2>&1; then
        echo "✓ manifest.json is valid JSON"
        echo "  Files: $(jq '.files | length' .aiassisted/manifest.json)"
        echo "  Version: $(jq -r '.version' .aiassisted/manifest.json)"
    else
        echo "✗ manifest.json is invalid JSON"
        exit 1
    fi
else
    echo "✗ manifest.json not found"
    exit 1
fi
echo ""

# Test 3: Check if manifest.json is accessible on GitHub
echo "Test 3: Check GitHub manifest accessibility"
MANIFEST_URL="https://raw.githubusercontent.com/rstlix0x0/aiassisted/main/.aiassisted/manifest.json"

if curl -f -s "$MANIFEST_URL" > /dev/null 2>&1; then
    echo "✓ manifest.json is accessible on GitHub"
    echo "  URL: $MANIFEST_URL"
else
    echo "⚠ manifest.json not yet pushed to GitHub"
    echo "  URL: $MANIFEST_URL"
    echo "  Note: Push changes before testing install"
fi
echo ""

echo "Quick test completed!"
