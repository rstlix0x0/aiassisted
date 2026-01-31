#!/bin/sh
#
# Update .aiassisted/.version and manifest.json with checksums
#
# This script automatically:
# 1. Regenerates the manifest.json with all files and SHA256 checksums
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
# Escape JSON String
###########################################

escape_json_string() {
    # Escape backslashes and quotes for JSON
    printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

###########################################
# Main
###########################################

echo "Regenerating .aiassisted/manifest.json with checksums..."

# Change to .aiassisted directory
cd .aiassisted

# Get version from parent directory commit
cd ..
COMMIT_HASH=$(git log -1 --format="%H" -- .aiassisted/)
if [ -z "$COMMIT_HASH" ]; then
    COMMIT_HASH=$(git log -1 --format="%H")
fi
UPDATED_AT=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
VERSION_STRING="${COMMIT_HASH:0:7}"
cd .aiassisted

# Start JSON file
cat > manifest.json <<EOF
{
  "version": "$VERSION_STRING",
  "files": [
EOF

# Find all files and calculate checksums
_file_count=0
_files=$(find . -type f ! -name '.version' ! -name 'manifest.json' ! -name 'FILES.txt' | sed 's|^\./||' | sort)

for _file in $_files; do
    _hash=$(calculate_sha256 "$_file")
    _file_escaped=$(escape_json_string "$_file")

    # Add comma if not first entry
    if [ $_file_count -gt 0 ]; then
        printf ",\n" >> manifest.json
    fi

    # Add file entry
    printf '    {\n      "path": "%s",\n      "checksum": "%s"\n    }' "$_file_escaped" "$_hash" >> manifest.json

    _file_count=$((_file_count + 1))
done

# Close JSON file
cat >> manifest.json <<EOF

  ]
}
EOF

echo "  Generated checksums for ${_file_count} files"

# Return to root
cd ..

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
echo "  git commit -m 'docs: update .aiassisted manifest'"
echo "  git push origin main"
