#!/bin/sh
#
# Update .aiassisted/.version and FILES.txt manifest with checksums
#
# This script automatically:
# 1. Regenerates the FILES.txt manifest with all files and SHA256 checksums
# 2. Updates the version file with the latest commit hash
#

set -e

###########################################
# Detect SHA256 Tool
###########################################

detect_sha256_tool() {
    if command -v sha256sum >/dev/null 2>&1; then
        echo "sha256sum"
    elif command -v shasum >/dev/null 2>&1; then
        echo "shasum"
    elif command -v openssl >/dev/null 2>&1; then
        echo "openssl"
    else
        echo "ERROR: No SHA256 tool found (sha256sum, shasum, or openssl required)" >&2
        exit 1
    fi
}

###########################################
# Calculate SHA256
###########################################

calculate_sha256() {
    _file="$1"
    _tool=$(detect_sha256_tool)
    
    case "$_tool" in
        sha256sum)
            sha256sum "$_file" | cut -d' ' -f1
            ;;
        shasum)
            shasum -a 256 "$_file" | cut -d' ' -f1
            ;;
        openssl)
            openssl dgst -sha256 "$_file" | cut -d' ' -f2
            ;;
    esac
}

###########################################
# Main
###########################################

echo "Regenerating .aiassisted/FILES.txt manifest with checksums..."

# Change to .aiassisted directory
cd .aiassisted

# Create temp file for manifest
_temp_manifest=$(mktemp)

# Find all files and calculate checksums
_file_count=0
find . -type f ! -name '.version' ! -name 'FILES.txt' | sed 's|^\./||' | sort | while IFS= read -r _file; do
    _hash=$(calculate_sha256 "$_file")
    printf "%s:%s\n" "$_file" "$_hash"
    _file_count=$((_file_count + 1))
done > "$_temp_manifest"

# Move temp file to FILES.txt
mv "$_temp_manifest" FILES.txt

FILES_COUNT=$(wc -l < FILES.txt | tr -d '[:space:]')
echo "  Generated checksums for ${FILES_COUNT} files"

# Return to root
cd ..

# Get the latest commit hash for .aiassisted directory
COMMIT_HASH=$(git log -1 --format="%H" -- .aiassisted/)

# If no commits found for .aiassisted, use latest commit
if [ -z "$COMMIT_HASH" ]; then
    COMMIT_HASH=$(git log -1 --format="%H")
fi

UPDATED_AT=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Write version file
cat > .aiassisted/.version <<EOF
COMMIT_HASH=${COMMIT_HASH}
UPDATED_AT=${UPDATED_AT}
EOF

echo "Updated .aiassisted/.version"
echo "  COMMIT_HASH: ${COMMIT_HASH}"
echo "  UPDATED_AT: ${UPDATED_AT}"
echo ""
echo "Don't forget to:"
echo "  git add .aiassisted/"
echo "  git commit -m 'docs: update .aiassisted content'"
echo "  git push origin main"
