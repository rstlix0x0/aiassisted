#!/bin/sh
#
# aiassisted - AI-Assisted Engineering Guidelines Installer
# 
# A simple CLI tool to install and manage .aiassisted directory
# in your projects for consistent AI-assisted development practices.
#
# Usage:
#   aiassisted install [--path=DIR] [--verbose] [--quiet]
#   aiassisted update [--force] [--path=DIR] [--verbose] [--quiet]
#   aiassisted check [--path=DIR]
#   aiassisted version
#   aiassisted self-update
#   aiassisted help
#

set -e

# Version
VERSION="1.0.0"

# GitHub repository
GITHUB_REPO="rstlix0x0/aiassisted"
GITHUB_RAW_URL="https://raw.githubusercontent.com/${GITHUB_REPO}/main"

# Verbosity levels: 0=quiet, 1=normal, 2=verbose
VERBOSITY=1

# Color support detection
if [ -t 1 ] && command -v tput >/dev/null 2>&1; then
    COLORS=$(tput colors 2>/dev/null || echo 0)
    if [ "$COLORS" -ge 8 ]; then
        COLOR_RESET="$(tput sgr0)"
        COLOR_RED="$(tput setaf 1)"
        COLOR_GREEN="$(tput setaf 2)"
        COLOR_YELLOW="$(tput setaf 3)"
        COLOR_BLUE="$(tput setaf 4)"
        COLOR_BOLD="$(tput bold)"
    fi
fi

# Fallback to no colors if not set
COLOR_RESET="${COLOR_RESET:-}"
COLOR_RED="${COLOR_RED:-}"
COLOR_GREEN="${COLOR_GREEN:-}"
COLOR_YELLOW="${COLOR_YELLOW:-}"
COLOR_BLUE="${COLOR_BLUE:-}"
COLOR_BOLD="${COLOR_BOLD:-}"

###########################################
# Logging Functions
###########################################

log_error() {
    printf "%s[ERROR]%s %s\n" "$COLOR_RED" "$COLOR_RESET" "$1" >&2
}

log_success() {
    if [ "$VERBOSITY" -ge 1 ]; then
        printf "%s[SUCCESS]%s %s\n" "$COLOR_GREEN" "$COLOR_RESET" "$1" >&2
    fi
}

log_info() {
    if [ "$VERBOSITY" -ge 1 ]; then
        printf "%s[INFO]%s %s\n" "$COLOR_BLUE" "$COLOR_RESET" "$1" >&2
    fi
}

log_warn() {
    if [ "$VERBOSITY" -ge 1 ]; then
        printf "%s[WARN]%s %s\n" "$COLOR_YELLOW" "$COLOR_RESET" "$1" >&2
    fi
}

log_debug() {
    if [ "$VERBOSITY" -ge 2 ]; then
        printf "%s[DEBUG]%s %s\n" "$COLOR_BLUE" "$COLOR_RESET" "$1" >&2
    fi
}

###########################################
# Utility Functions
###########################################

# Detect download tool (curl or wget)
detect_download_tool() {
    if command -v curl >/dev/null 2>&1; then
        echo "curl"
    elif command -v wget >/dev/null 2>&1; then
        echo "wget"
    else
        log_error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi
}

# Download file
# Usage: download_file <url> <output_file>
download_file() {
    _url="$1"
    _output="$2"
    _tool=$(detect_download_tool)
    
    log_debug "Downloading $_url to $_output using $_tool"
    
    if [ "$_tool" = "curl" ]; then
        if ! curl -fsSL "$_url" -o "$_output"; then
            log_error "Failed to download $_url"
            return 1
        fi
    else
        if ! wget -q "$_url" -O "$_output"; then
            log_error "Failed to download $_url"
            return 1
        fi
    fi
    
    return 0
}

# Check if directory is writable
is_writable() {
    [ -w "$1" ] || [ ! -e "$1" ]
}

# Get value from TOML configuration file
# Usage: toml_get <file> <key> [default]
# Example: toml_get ~/.aiassisted/config.toml "verbosity" "1"
toml_get() {
    _file="$1"
    _key="$2"
    _default="${3:-}"
    
    if [ ! -f "$_file" ]; then
        echo "$_default"
        return 1
    fi
    
    # Extract value for key (handles quotes, comments)
    # Matches lines like: key = "value" or key = 'value' or key = value
    # shellcheck disable=SC1087  # $_key is intentionally expanded in grep pattern
    _value=$(grep -E "^[[:space:]]*$_key[[:space:]]*=" "$_file" 2>/dev/null | \
             sed 's/^[^=]*=[[:space:]]*//; s/"//g; s/'"'"'//g; s/#.*//' | \
             sed 's/^[[:space:]]*//; s/[[:space:]]*$//' | \
             head -n1)
    
    if [ -z "$_value" ]; then
        echo "$_default"
        return 1
    else
        echo "$_value"
        return 0
    fi
}

# Parse version file
# Usage: parse_version_file <file> <key>
parse_version_file() {
    _file="$1"
    _key="$2"
    
    if [ ! -f "$_file" ]; then
        return 1
    fi
    
    # Extract value from KEY=VALUE format
    grep "^${_key}=" "$_file" | cut -d'=' -f2
}

# Detect SHA256 tool
detect_sha256_tool() {
    if command -v sha256sum >/dev/null 2>&1; then
        echo "sha256sum"
    elif command -v shasum >/dev/null 2>&1; then
        echo "shasum"
    elif command -v openssl >/dev/null 2>&1; then
        echo "openssl"
    else
        log_error "No SHA256 tool found (sha256sum, shasum, or openssl required)"
        exit 1
    fi
}

# Calculate SHA256 checksum of a file
# Usage: calculate_sha256 <file>
# Returns: SHA256 hash string
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

# Verify file checksum
# Usage: verify_checksum <file> <expected_hash>
# Returns: 0 if match, 1 if mismatch
verify_checksum() {
    _file="$1"
    _expected="$2"
    
    if [ ! -f "$_file" ]; then
        return 1
    fi
    
    _actual=$(calculate_sha256 "$_file")
    
    if [ "$_actual" = "$_expected" ]; then
        return 0
    else
        log_debug "Checksum mismatch for $_file"
        log_debug "  Expected: $_expected"
        log_debug "  Actual:   $_actual"
        return 1
    fi
}

###########################################
# Core Functions
###########################################

# Fetch remote version info
fetch_remote_version() {
    _temp_version=$(mktemp)
    
    if ! download_file "${GITHUB_RAW_URL}/.aiassisted/.version" "$_temp_version"; then
        rm -f "$_temp_version"
        return 1
    fi
    
    echo "$_temp_version"
}

# Compare versions
# Returns: 0 if same, 1 if different
compare_versions() {
    _local_hash="$1"
    _remote_hash="$2"
    
    if [ "$_local_hash" = "$_remote_hash" ]; then
        return 0
    else
        return 1
    fi
}

# Download .aiassisted directory to temp location
download_aiassisted() {
    _temp_dir=$(mktemp -d)
    _target_dir="$_temp_dir/.aiassisted"
    
    mkdir -p "$_target_dir"
    
    log_info "Downloading .aiassisted directory..."
    
    # Download .version file first
    if ! download_file "${GITHUB_RAW_URL}/.aiassisted/.version" "$_target_dir/.version"; then
        rm -rf "$_temp_dir"
        return 1
    fi
    
    # Download FILES.txt manifest
    _manifest="$_temp_dir/FILES.txt"
    if ! download_file "${GITHUB_RAW_URL}/.aiassisted/FILES.txt" "$_manifest"; then
        log_error "Failed to download FILES.txt manifest"
        rm -rf "$_temp_dir"
        return 1
    fi
    
    # Count total files for progress
    _total_files=$(grep -c -v -E '^(#|$)' "$_manifest" 2>/dev/null || echo "0")
    _current_file=0
    _downloaded_bytes=0
    
    # Read manifest and download each file with checksum verification
    while IFS=: read -r _filepath _expected_hash || [ -n "$_filepath" ]; do
        # Skip empty lines and comments
        case "$_filepath" in
            ''|'#'*) continue ;;
        esac
        
        _current_file=$((_current_file + 1))
        _file_dir="$(dirname "$_filepath")"
        mkdir -p "$_target_dir/$_file_dir"
        
        # Show progress - use simple counter without carriage return
        if [ $((_current_file % 5)) -eq 0 ] || [ $_current_file -eq 1 ] || [ $_current_file -eq $_total_files ]; then
            log_info "Progress: $_current_file/$_total_files files downloaded"
        fi
        
        if ! download_file "${GITHUB_RAW_URL}/.aiassisted/${_filepath}" "$_target_dir/$_filepath"; then
            printf "\n" >&2
            log_error "Failed to download $_filepath"
            rm -rf "$_temp_dir"
            return 1
        fi
        
        # Verify checksum
        if ! verify_checksum "$_target_dir/$_filepath" "$_expected_hash"; then
            printf "\n" >&2
            log_error "Checksum verification failed for $_filepath"
            rm -rf "$_temp_dir"
            return 1
        fi
        
        # Track downloaded size
        if [ -f "$_target_dir/$_filepath" ]; then
            _file_size=$(wc -c < "$_target_dir/$_filepath" 2>/dev/null || echo "0")
            _downloaded_bytes=$((_downloaded_bytes + _file_size))
        fi
    done < "$_manifest"
    
    # Copy FILES.txt to target
    cp "$_manifest" "$_target_dir/FILES.txt"
    
    # Calculate size in human-readable format
    if [ "$_downloaded_bytes" -ge 1048576 ]; then
        _size_mb=$(((_downloaded_bytes + 524288) / 1048576))
        _size_display="${_size_mb}MB"
    elif [ "$_downloaded_bytes" -ge 1024 ]; then
        _size_kb=$(((_downloaded_bytes + 512) / 1024))
        _size_display="${_size_kb}KB"
    else
        _size_display="${_downloaded_bytes}B"
    fi
    
    log_success "Downloaded $_total_files files ($_size_display)"
    echo "$_temp_dir"
}

# Install .aiassisted directory
install_aiassisted() {
    _source="$1"
    _target="$2"
    
    log_debug "Installing from $_source to $_target"
    
    # Create target directory if it doesn't exist
    if [ ! -d "$_target" ]; then
        if ! mkdir -p "$_target"; then
            log_error "Failed to create directory $_target"
            return 1
        fi
    fi
    
    # Copy files
    if ! cp -r "$_source/.aiassisted" "$_target/"; then
        log_error "Failed to copy .aiassisted directory"
        return 1
    fi
    
    log_success "Installed .aiassisted to $_target"
    return 0
}

# Download only changed files based on checksums
# Usage: download_changed_files <target_path> <remote_manifest>
# Returns: temp directory with only changed files
download_changed_files() {
    _target_path="$1"
    _remote_manifest="$2"
    _local_manifest="$_target_path/.aiassisted/FILES.txt"
    
    _temp_dir=$(mktemp -d)
    _target_dir="$_temp_dir/.aiassisted"
    mkdir -p "$_target_dir"
    
    _changed_count=0
    _unchanged_count=0
    
    # Read remote manifest and compare with local
    while IFS=: read -r _filepath _remote_hash || [ -n "$_filepath" ]; do
        case "$_filepath" in
            ''|'#'*) continue ;;
        esac
        
        _needs_download=1
        
        # Check if file exists locally with same hash
        if [ -f "$_local_manifest" ]; then
            _local_hash=$(grep "^${_filepath}:" "$_local_manifest" 2>/dev/null | cut -d: -f2)
            
            if [ -n "$_local_hash" ] && [ "$_local_hash" = "$_remote_hash" ]; then
                _needs_download=0
                _unchanged_count=$((_unchanged_count + 1))
                log_debug "Unchanged: $_filepath"
            fi
        fi
        
        # Download if needed
        if [ $_needs_download -eq 1 ]; then
            _file_dir="$(dirname "$_filepath")"
            mkdir -p "$_target_dir/$_file_dir"
            
            log_debug "Downloading changed file: $_filepath..."
            if ! download_file "${GITHUB_RAW_URL}/.aiassisted/${_filepath}" "$_target_dir/$_filepath"; then
                log_error "Failed to download $_filepath"
                rm -rf "$_temp_dir"
                return 1
            fi
            
            # Verify checksum
            if ! verify_checksum "$_target_dir/$_filepath" "$_remote_hash"; then
                log_error "Checksum verification failed for $_filepath"
                rm -rf "$_temp_dir"
                return 1
            fi
            
            _changed_count=$((_changed_count + 1))
        fi
    done < "$_remote_manifest"
    
    log_info "Changed: $_changed_count file(s), Unchanged: $_unchanged_count file(s)"
    
    echo "$_temp_dir"
}

# Generate diff between two directories
generate_diff() {
    _old="$1"
    _new="$2"
    
    if command -v diff >/dev/null 2>&1; then
        diff -ru "$_old" "$_new" 2>/dev/null || true
    else
        log_warn "diff command not found, cannot show changes"
        return 1
    fi
}

# Apply selective updates (only changed files)
# Usage: apply_selective_update <source_temp_dir> <target_path> <remote_manifest>
apply_selective_update() {
    _source="$1"
    _target="$2"
    _remote_manifest="$3"
    
    log_info "Applying selective updates..."
    
    _updated=0
    _skipped=0
    
    # Download .version file
    if ! cp "$_source/.aiassisted/.version" "$_target/.aiassisted/.version"; then
        log_error "Failed to update .version file"
        return 1
    fi
    
    # Download FILES.txt manifest
    if ! cp "$_remote_manifest" "$_target/.aiassisted/FILES.txt"; then
        log_error "Failed to update FILES.txt manifest"
        return 1
    fi
    
    # Read manifest and copy only changed files from temp
    while IFS=: read -r _filepath _hash || [ -n "$_filepath" ]; do
        case "$_filepath" in
            ''|'#'*) continue ;;
        esac
        
        _source_file="$_source/.aiassisted/$_filepath"
        _target_file="$_target/.aiassisted/$_filepath"
        
        # If file exists in source temp (was downloaded), copy it
        if [ -f "$_source_file" ]; then
            _target_dir="$(dirname "$_target_file")"
            mkdir -p "$_target_dir"
            
            if ! cp "$_source_file" "$_target_file"; then
                log_error "Failed to update $_filepath"
                return 1
            fi
            
            log_debug "Updated: $_filepath"
            _updated=$((_updated + 1))
        else
            _skipped=$((_skipped + 1))
        fi
    done < "$_remote_manifest"
    
    log_success "Updated $_updated file(s), skipped $_skipped unchanged file(s)"
    return 0
}

# Prompt user for confirmation
prompt_confirm() {
    _prompt="$1"
    
    printf "%s%s [y/N]:%s " "$COLOR_YELLOW" "$_prompt" "$COLOR_RESET"
    read -r _response
    
    case "$_response" in
        [yY]|[yY][eE][sS])
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

###########################################
# Command Implementations
###########################################

cmd_install() {
    _target_path="."
    
    # Parse arguments
    for _arg in "$@"; do
        case "$_arg" in
            --path=*)
                _target_path="${_arg#*=}"
                ;;
        esac
    done
    
    # Resolve to absolute path
    _target_path=$(cd "$_target_path" 2>/dev/null && pwd || echo "$_target_path")
    
    log_info "Installing .aiassisted to $_target_path"
    
    # Check if .aiassisted already exists
    if [ -d "$_target_path/.aiassisted" ]; then
        if [ -f "$_target_path/.aiassisted/.version" ]; then
            _local_hash=$(parse_version_file "$_target_path/.aiassisted/.version" "COMMIT_HASH")
            
            # Fetch remote version
            if ! _remote_version_file=$(fetch_remote_version); then
                log_error "Failed to fetch remote version information"
                exit 1
            fi
            
            _remote_hash=$(parse_version_file "$_remote_version_file" "COMMIT_HASH")
            rm -f "$_remote_version_file"
            
            if compare_versions "$_local_hash" "$_remote_hash"; then
                log_success ".aiassisted is already up-to-date (version: $_local_hash)"
                exit 0
            else
                log_warn ".aiassisted already exists but is outdated"
                log_info "Current version: $_local_hash"
                log_info "Latest version:  $_remote_hash"
                log_info "Run 'aiassisted update' to update to the latest version"
                exit 0
            fi
        else
            log_warn ".aiassisted exists but no version information found"
            log_info "Run 'aiassisted update --force' to overwrite with the latest version"
            exit 0
        fi
    fi
    
    # Download .aiassisted
    if ! _temp_dir=$(download_aiassisted); then
        log_error "Failed to download .aiassisted"
        exit 1
    fi
    
    # Install
    if ! install_aiassisted "$_temp_dir" "$_target_path"; then
        rm -rf "$_temp_dir"
        exit 1
    fi
    
    rm -rf "$_temp_dir"
    
    # Count installed items
    _guideline_count=$(find "$_target_path/.aiassisted/guidelines" -type f 2>/dev/null | wc -l | tr -d ' ')
    _instruction_count=$(find "$_target_path/.aiassisted/instructions" -type f 2>/dev/null | wc -l | tr -d ' ')
    _template_count=$(find "$_target_path/.aiassisted/templates" -type f 2>/dev/null | wc -l | tr -d ' ')
    
    # Show version info and summary
    _installed_hash=$(parse_version_file "$_target_path/.aiassisted/.version" "COMMIT_HASH")
    _short_hash=$(echo "$_installed_hash" | cut -c1-7)
    
    printf "\n"
    log_success "Installation complete!"
    
    # Show installation summary
    printf "\n%s%sInstallation Summary:%s\n" "$COLOR_BOLD" "$COLOR_GREEN" "$COLOR_RESET"
    printf "  %sLocation:%s     %s\n" "$COLOR_BOLD" "$COLOR_RESET" "$_target_path/.aiassisted"
    printf "  %sVersion:%s      %s\n" "$COLOR_BOLD" "$COLOR_RESET" "$_short_hash"
    printf "  %sGuidelines:%s   %s file(s)\n" "$COLOR_BOLD" "$COLOR_RESET" "$_guideline_count"
    printf "  %sInstructions:%s %s file(s)\n" "$COLOR_BOLD" "$COLOR_RESET" "$_instruction_count"
    printf "  %sTemplates:%s    %s file(s)\n" "$COLOR_BOLD" "$COLOR_RESET" "$_template_count"
    
    # Show next steps
    printf "\n%s%sNext Steps:%s\n" "$COLOR_BOLD" "$COLOR_GREEN" "$COLOR_RESET"
    printf "  1. Setup skills: %saiassisted setup-skills%s (slash commands)\n" "$COLOR_BOLD" "$COLOR_RESET"
    printf "  2. Setup agents: %saiassisted setup-agents%s (custom AI agents)\n" "$COLOR_BOLD" "$COLOR_RESET"
    printf "  3. Update content: %saiassisted update%s\n" "$COLOR_BOLD" "$COLOR_RESET"
    printf "  4. View all commands: %saiassisted help%s\n\n" "$COLOR_BOLD" "$COLOR_RESET"
}

cmd_update() {
    _force=0
    _target_path="."
    
    # Parse arguments
    for _arg in "$@"; do
        case "$_arg" in
            --force)
                _force=1
                ;;
            --path=*)
                _target_path="${_arg#*=}"
                ;;
        esac
    done
    
    # Resolve to absolute path
    _target_path=$(cd "$_target_path" 2>/dev/null && pwd || echo "$_target_path")
    
    log_info "Checking for updates in $_target_path"
    
    # Check if .aiassisted exists
    if [ ! -d "$_target_path/.aiassisted" ]; then
        log_error ".aiassisted not found in $_target_path"
        log_info "Run 'aiassisted install' first"
        exit 1
    fi
    
    # Get local version
    if [ ! -f "$_target_path/.aiassisted/.version" ]; then
        log_warn "No version information found locally"
        _local_hash="unknown"
    else
        _local_hash=$(parse_version_file "$_target_path/.aiassisted/.version" "COMMIT_HASH")
    fi
    
    # Fetch remote version
    if ! _remote_version_file=$(fetch_remote_version); then
        log_error "Failed to fetch remote version information"
        exit 1
    fi
    
    _remote_hash=$(parse_version_file "$_remote_version_file" "COMMIT_HASH")
    rm -f "$_remote_version_file"
    
    log_debug "Local version:  $_local_hash"
    log_debug "Remote version: $_remote_hash"
    
    # Compare versions
    if [ "$_local_hash" != "unknown" ] && compare_versions "$_local_hash" "$_remote_hash"; then
        log_success ".aiassisted is already up-to-date (version: $_local_hash)"
        exit 0
    fi
    
    log_info "Update available!"
    log_info "Current version: $_local_hash"
    log_info "Latest version:  $_remote_hash"
    
    # Download remote manifest first to check what files changed
    _remote_manifest=$(mktemp)
    if ! download_file "${GITHUB_RAW_URL}/.aiassisted/FILES.txt" "$_remote_manifest"; then
        log_error "Failed to download remote manifest"
        rm -f "$_remote_manifest"
        exit 1
    fi
    
    # Download only changed files
    if ! _temp_dir=$(download_changed_files "$_target_path" "$_remote_manifest"); then
        log_error "Failed to download changed files"
        rm -f "$_remote_manifest"
        exit 1
    fi
    
    # Download .version file
    if ! download_file "${GITHUB_RAW_URL}/.aiassisted/.version" "$_temp_dir/.aiassisted/.version"; then
        log_error "Failed to download .version file"
        rm -rf "$_temp_dir"
        rm -f "$_remote_manifest"
        exit 1
    fi
    
    # Show diff unless --force
    if [ $_force -eq 0 ]; then
        # Check if there are any changed files in temp dir
        _has_changes=0
        if [ -d "$_temp_dir/.aiassisted" ] && [ "$(find "$_temp_dir/.aiassisted" -type f | wc -l)" -gt 0 ]; then
            _has_changes=1
        fi
        
        if [ $_has_changes -eq 1 ]; then
            printf "\n%s%sChanges to be applied:%s\n\n" "$COLOR_BOLD" "$COLOR_YELLOW" "$COLOR_RESET"
            
            _diff_output=$(generate_diff "$_target_path/.aiassisted" "$_temp_dir/.aiassisted")
            
            if [ -n "$_diff_output" ]; then
                echo "$_diff_output" | head -n 100
                _diff_lines=$(echo "$_diff_output" | wc -l | tr -d '[:space:]')
                if [ "$_diff_lines" -gt 100 ]; then
                    printf "\n%s... (%d more lines)%s\n\n" "$COLOR_BLUE" "$((_diff_lines - 100))" "$COLOR_RESET"
                else
                    printf "\n"
                fi
            fi
        else
            printf "\n%s%sNo file content changes (version metadata update only)%s\n\n" "$COLOR_BLUE" "$COLOR_BOLD" "$COLOR_RESET"
        fi
        
        if ! prompt_confirm "Apply these changes?"; then
            log_info "Update cancelled"
            rm -rf "$_temp_dir"
            rm -f "$_remote_manifest"
            exit 0
        fi
    fi
    
    # Apply selective updates (only changed files)
    if ! apply_selective_update "$_temp_dir" "$_target_path" "$_remote_manifest"; then
        rm -rf "$_temp_dir"
        rm -f "$_remote_manifest"
        exit 1
    fi
    
    rm -rf "$_temp_dir"
    rm -f "$_remote_manifest"
    
    _short_remote=$(echo "$_remote_hash" | cut -c1-7)
    _short_local=$(echo "$_local_hash" | cut -c1-7)
    
    printf "\n"
    log_success "Update complete!"
    
    # Show update summary
    printf "\n%s%sUpdate Summary:%s\n" "$COLOR_BOLD" "$COLOR_GREEN" "$COLOR_RESET"
    printf "  %sPrevious:%s %s\n" "$COLOR_BOLD" "$COLOR_RESET" "$_short_local"
    printf "  %sCurrent:%s  %s\n" "$COLOR_BOLD" "$COLOR_RESET" "$_short_remote"
    printf "  %sLocation:%s %s\n\n" "$COLOR_BOLD" "$COLOR_RESET" "$_target_path/.aiassisted"
}

cmd_check() {
    _target_path="."
    
    # Parse arguments
    for _arg in "$@"; do
        case "$_arg" in
            --path=*)
                _target_path="${_arg#*=}"
                ;;
        esac
    done
    
    # Resolve to absolute path
    _target_path=$(cd "$_target_path" 2>/dev/null && pwd || echo "$_target_path")
    
    log_info "Checking version in $_target_path"
    
    # Check if .aiassisted exists
    if [ ! -d "$_target_path/.aiassisted" ]; then
        log_error ".aiassisted not found in $_target_path"
        log_info "Run 'aiassisted install' to install"
        exit 1
    fi
    
    # Get local version
    if [ ! -f "$_target_path/.aiassisted/.version" ]; then
        log_warn "No version information found locally"
        _local_hash="unknown"
    else
        _local_hash=$(parse_version_file "$_target_path/.aiassisted/.version" "COMMIT_HASH")
    fi
    
    # Fetch remote version
    if ! _remote_version_file=$(fetch_remote_version); then
        log_error "Failed to fetch remote version information"
        exit 1
    fi
    
    _remote_hash=$(parse_version_file "$_remote_version_file" "COMMIT_HASH")
    rm -f "$_remote_version_file"
    
    printf "\n%sCurrent version:%s %s\n" "$COLOR_BOLD" "$COLOR_RESET" "$_local_hash"
    printf "%sLatest version: %s %s\n\n" "$COLOR_BOLD" "$COLOR_RESET" "$_remote_hash"
    
    if [ "$_local_hash" != "unknown" ] && compare_versions "$_local_hash" "$_remote_hash"; then
        log_success "You are up-to-date!"
    else
        log_warn "An update is available"
        log_info "Run 'aiassisted update' to update to the latest version"
    fi
}

cmd_version() {
    printf "aiassisted version %s\n" "$VERSION"
}

cmd_self_update() {
    log_info "Updating aiassisted..."
    
    # Detect if we're in a git repository
    if [ ! -d "$SCRIPT_DIR/.git" ]; then
        log_error "Git repository not found at $SCRIPT_DIR"
        log_info "It looks like the installation was modified or corrupted"
        log_info "To restore, run:"
        log_info "  rm -rf ~/.aiassisted/source/aiassisted"
        log_info "  curl -fsSL https://raw.githubusercontent.com/rstlix0x0/aiassisted/main/install.sh | sh"
        exit 1
    fi
    
    # Get current commit hash
    _current_hash=$(cd "$SCRIPT_DIR" && git rev-parse --short HEAD)
    log_info "Current version: $_current_hash"
    
    # Pull latest changes
    log_info "Fetching updates from repository..."
    if ! (cd "$SCRIPT_DIR" && git pull); then
        log_error "Failed to update from repository"
        log_info "You may need to resolve conflicts or check your network connection"
        exit 1
    fi
    
    # Get new commit hash
    _new_hash=$(cd "$SCRIPT_DIR" && git rev-parse --short HEAD)
    
    if [ "$_current_hash" = "$_new_hash" ]; then
        log_success "Already up to date ($_current_hash)"
    else
        log_success "Updated: $_current_hash -> $_new_hash"
        log_info "View changes: cd $SCRIPT_DIR && git log --oneline $_current_hash..$_new_hash"
    fi
    
    log_info "Restart your terminal or run 'hash -r' to reload the CLI"
}

###########################################
# Setup Skills Functions
###########################################

# Detect project root (git repository)
detect_project_root() {
    if ! _root=$(git rev-parse --show-toplevel 2>/dev/null); then
        log_error "Not in a git repository"
        printf "\n" >&2
        log_info "This command requires a git repository to determine the project root."
        printf "\n" >&2
        log_info "To fix this:"
        printf "  1. Initialize git: %sgit init%s\n" "$COLOR_BOLD" "$COLOR_RESET" >&2
        printf "  2. Then run this command again\n" >&2
        printf "\n" >&2
        return 1
    fi
    echo "$_root"
}

# Detect if opencode is installed
detect_opencode() {
    command -v opencode >/dev/null 2>&1
}

# Detect if claude is installed
detect_claude() {
    command -v claude >/dev/null 2>&1
}

# Generate Rust guidelines list
generate_rust_guidelines_list() {
    _project_root="$1"
    _guidelines_dir="$_project_root/.aiassisted/guidelines/rust"
    
    if [ ! -d "$_guidelines_dir" ]; then
        echo "No Rust guidelines found"
        return 1
    fi
    
    find "$_guidelines_dir" -maxdepth 1 -name "*.md" -type f | sort | while read -r _file; do
        _basename=$(basename "$_file")
        echo "- $_basename"
    done
}

# Generate architecture guidelines list
generate_arch_guidelines_list() {
    _project_root="$1"
    _guidelines_dir="$_project_root/.aiassisted/guidelines/architecture"
    
    if [ ! -d "$_guidelines_dir" ]; then
        echo "No architecture guidelines found"
        return 1
    fi
    
    find "$_guidelines_dir" -maxdepth 1 -name "*.md" -type f | sort | while read -r _file; do
        _basename=$(basename "$_file")
        echo "- $_basename"
    done
}

# Find template with cascading fallback
# Usage: find_template <relative_path> <project_root>
# Example: find_template "skills/opencode/git-commit.SKILL.md.template" "/path/to/project"
# Returns: Full path to template file
# Exit code: 0 if found, 1 if not found
find_template() {
    _relative_path="$1"
    _project_root="$2"
    _project_template="$_project_root/.aiassisted/templates/$_relative_path"
    _global_template="$HOME/.aiassisted/templates/$_relative_path"
    
    log_debug "Looking for template: $_relative_path"
    
    # Priority 1: Project-specific template
    if [ -f "$_project_template" ]; then
        log_debug "Using project template: $_project_template"
        echo "$_project_template"
        return 0
    fi
    
    # Priority 2: Global template
    if [ -f "$_global_template" ]; then
        log_debug "Using global template: $_global_template"
        echo "$_global_template"
        return 0
    fi
    
    # Not found
    log_error "Template not found: $_relative_path"
    log_error "Searched:"
    log_error "  - $_project_template (project)"
    log_error "  - $_global_template (global)"
    return 1
}

# Substitute template variables
# Usage: substitute_template <template_file> <output_file> <project_root> <rust_list> <arch_list>
substitute_template() {
    _template="$1"
    _output="$2"
    _project_root="$3"
    _rust_list="$4"
    _arch_list="$5"
    
    # Read template and substitute variables
    # This is POSIX-compliant sed - avoiding GNU-specific features
    sed -e "s|{{PROJECT_ROOT}}|$_project_root|g" "$_template" | \
    sed -e "/{{RUST_GUIDELINES_LIST}}/{
r /dev/stdin
d
}" <<EOF | \
sed -e "/{{ARCH_GUIDELINES_LIST}}/{
r /dev/stdin
d
}" > "$_output"
$_rust_list
EOF
}

# Simplified template substitution (working version)
substitute_template_simple() {
    _template="$1"
    _output="$2"
    _project_root="$3"
    _rust_list="$4"
    _arch_list="$5"
    
    # Use a temporary file to build the output
    _temp=$(mktemp)
    
    # Read template line by line and substitute
    while IFS= read -r _line; do
        case "$_line" in
            *"{{PROJECT_ROOT}}"*)
                _line=$(echo "$_line" | sed "s|{{PROJECT_ROOT}}|$_project_root|g")
                ;;
            *"{{RUST_GUIDELINES_LIST}}"*)
                echo "$_rust_list"
                continue
                ;;
            *"{{ARCH_GUIDELINES_LIST}}"*)
                echo "$_arch_list"
                continue
                ;;
        esac
        echo "$_line"
    done < "$_template" > "$_temp"
    
    mv "$_temp" "$_output"
}

# Setup OpenCode skills only (slash commands)
setup_opencode_skills_only() {
    _project_root="$1"
    _dry_run="$2"
    
    log_info "Setting up OpenCode skills..."
    
    # Create .opencode directory if it doesn't exist
    _opencode_dir="$_project_root/.opencode"
    
    if [ "$_dry_run" -eq 0 ]; then
        mkdir -p "$_opencode_dir/skills/git-commit"
        mkdir -p "$_opencode_dir/skills/review-rust"
    fi
    
    # Generate guidelines lists
    _rust_list=$(generate_rust_guidelines_list "$_project_root")
    _arch_list=$(generate_arch_guidelines_list "$_project_root")
    
    # Setup git-commit skill
    log_debug "Creating git-commit skill..."
    if _template_path=$(find_template "skills/opencode/git-commit.SKILL.md.template" "$_project_root"); then
        if [ "$_dry_run" -eq 0 ]; then
            substitute_template_simple \
                "$_template_path" \
                "$_opencode_dir/skills/git-commit/SKILL.md" \
                "$_project_root" \
                "$_rust_list" \
                "$_arch_list"
        else
            echo "  Would create: .opencode/skills/git-commit/SKILL.md"
            echo "    from: $_template_path"
        fi
    else
        log_warn "Skipping git-commit skill (template not found)"
    fi
    
    # Setup review-rust skill
    log_debug "Creating review-rust skill..."
    if _template_path=$(find_template "skills/opencode/review-rust.SKILL.md.template" "$_project_root"); then
        if [ "$_dry_run" -eq 0 ]; then
            substitute_template_simple \
                "$_template_path" \
                "$_opencode_dir/skills/review-rust/SKILL.md" \
                "$_project_root" \
                "$_rust_list" \
                "$_arch_list"
        else
            echo "  Would create: .opencode/skills/review-rust/SKILL.md"
            echo "    from: $_template_path"
        fi
    else
        log_warn "Skipping review-rust skill (template not found)"
    fi
    
    if [ "$_dry_run" -eq 0 ]; then
        log_success "OpenCode skills setup complete!"
        log_info "Created skills: git-commit, review-rust"
    fi
}

# Setup Claude Code skills only (slash commands)
setup_claude_skills_only() {
    _project_root="$1"
    _dry_run="$2"
    
    log_info "Setting up Claude Code skills..."
    
    # Create .claude directory if it doesn't exist
    _claude_dir="$_project_root/.claude"
    
    if [ "$_dry_run" -eq 0 ]; then
        mkdir -p "$_claude_dir/skills/git-commit"
        mkdir -p "$_claude_dir/skills/review-rust"
    fi
    
    # Generate guidelines lists
    _rust_list=$(generate_rust_guidelines_list "$_project_root")
    _arch_list=$(generate_arch_guidelines_list "$_project_root")
    
    # Setup git-commit skill
    log_debug "Creating git-commit skill..."
    if _template_path=$(find_template "skills/claude/git-commit.SKILL.md.template" "$_project_root"); then
        if [ "$_dry_run" -eq 0 ]; then
            substitute_template_simple \
                "$_template_path" \
                "$_claude_dir/skills/git-commit/SKILL.md" \
                "$_project_root" \
                "$_rust_list" \
                "$_arch_list"
        else
            echo "  Would create: .claude/skills/git-commit/SKILL.md"
            echo "    from: $_template_path"
        fi
    else
        log_warn "Skipping git-commit skill (template not found)"
    fi
    
    # Setup review-rust skill
    log_debug "Creating review-rust skill..."
    if _template_path=$(find_template "skills/claude/review-rust.SKILL.md.template" "$_project_root"); then
        if [ "$_dry_run" -eq 0 ]; then
            substitute_template_simple \
                "$_template_path" \
                "$_claude_dir/skills/review-rust/SKILL.md" \
                "$_project_root" \
                "$_rust_list" \
                "$_arch_list"
        else
            echo "  Would create: .claude/skills/review-rust/SKILL.md"
            echo "    from: $_template_path"
        fi
    else
        log_warn "Skipping review-rust skill (template not found)"
    fi
    
    if [ "$_dry_run" -eq 0 ]; then
        log_success "Claude Code skills setup complete!"
        log_info "Created skills: git-commit, review-rust"
    fi
}

# Setup OpenCode agents only (custom subagents)
setup_opencode_agents_only() {
    _project_root="$1"
    _dry_run="$2"
    
    log_info "Setting up OpenCode agents..."
    
    # Create .opencode directory if it doesn't exist
    _opencode_dir="$_project_root/.opencode"
    
    if [ "$_dry_run" -eq 0 ]; then
        mkdir -p "$_opencode_dir/agents"
    fi
    
    # Generate guidelines lists
    _rust_list=$(generate_rust_guidelines_list "$_project_root")
    _arch_list=$(generate_arch_guidelines_list "$_project_root")
    
    # Setup ai-knowledge-rust agent
    log_debug "Creating ai-knowledge-rust agent..."
    if _template_path=$(find_template "agents/opencode/ai-knowledge-rust.md.template" "$_project_root"); then
        if [ "$_dry_run" -eq 0 ]; then
            substitute_template_simple \
                "$_template_path" \
                "$_opencode_dir/agents/ai-knowledge-rust.md" \
                "$_project_root" \
                "$_rust_list" \
                "$_arch_list"
        else
            echo "  Would create: .opencode/agents/ai-knowledge-rust.md"
            echo "    from: $_template_path"
        fi
    else
        log_warn "Skipping ai-knowledge-rust agent (template not found)"
    fi
    
    # Setup ai-knowledge-architecture agent
    log_debug "Creating ai-knowledge-architecture agent..."
    if _template_path=$(find_template "agents/opencode/ai-knowledge-architecture.md.template" "$_project_root"); then
        if [ "$_dry_run" -eq 0 ]; then
            substitute_template_simple \
                "$_template_path" \
                "$_opencode_dir/agents/ai-knowledge-architecture.md" \
                "$_project_root" \
                "$_rust_list" \
                "$_arch_list"
        else
            echo "  Would create: .opencode/agents/ai-knowledge-architecture.md"
            echo "    from: $_template_path"
        fi
    else
        log_warn "Skipping ai-knowledge-architecture agent (template not found)"
    fi
    
    # Setup git-commit agent
    log_debug "Creating git-commit agent..."
    if _template_path=$(find_template "agents/opencode/git-commit.md.template" "$_project_root"); then
        if [ "$_dry_run" -eq 0 ]; then
            substitute_template_simple \
                "$_template_path" \
                "$_opencode_dir/agents/git-commit.md" \
                "$_project_root" \
                "$_rust_list" \
                "$_arch_list"
        else
            echo "  Would create: .opencode/agents/git-commit.md"
            echo "    from: $_template_path"
        fi
    else
        log_warn "Skipping git-commit agent (template not found)"
    fi
    
    if [ "$_dry_run" -eq 0 ]; then
        log_success "OpenCode agents setup complete!"
        log_info "Created agents: ai-knowledge-rust, ai-knowledge-architecture, git-commit"
    fi
}

# Setup Claude Code agents only (custom subagents)
setup_claude_agents_only() {
    _project_root="$1"
    _dry_run="$2"
    
    log_info "Setting up Claude Code agents..."
    
    # Create .claude directory if it doesn't exist
    _claude_dir="$_project_root/.claude"
    
    if [ "$_dry_run" -eq 0 ]; then
        mkdir -p "$_claude_dir/agents"
    fi
    
    # Generate guidelines lists
    _rust_list=$(generate_rust_guidelines_list "$_project_root")
    _arch_list=$(generate_arch_guidelines_list "$_project_root")
    
    # Setup ai-knowledge-rust agent
    log_debug "Creating ai-knowledge-rust agent..."
    if _template_path=$(find_template "agents/claude/ai-knowledge-rust.md.template" "$_project_root"); then
        if [ "$_dry_run" -eq 0 ]; then
            substitute_template_simple \
                "$_template_path" \
                "$_claude_dir/agents/ai-knowledge-rust.md" \
                "$_project_root" \
                "$_rust_list" \
                "$_arch_list"
        else
            echo "  Would create: .claude/agents/ai-knowledge-rust.md"
            echo "    from: $_template_path"
        fi
    else
        log_warn "Skipping ai-knowledge-rust agent (template not found)"
    fi
    
    # Setup ai-knowledge-architecture agent
    log_debug "Creating ai-knowledge-architecture agent..."
    if _template_path=$(find_template "agents/claude/ai-knowledge-architecture.md.template" "$_project_root"); then
        if [ "$_dry_run" -eq 0 ]; then
            substitute_template_simple \
                "$_template_path" \
                "$_claude_dir/agents/ai-knowledge-architecture.md" \
                "$_project_root" \
                "$_rust_list" \
                "$_arch_list"
        else
            echo "  Would create: .claude/agents/ai-knowledge-architecture.md"
            echo "    from: $_template_path"
        fi
    else
        log_warn "Skipping ai-knowledge-architecture agent (template not found)"
    fi
    
    # Setup git-commit agent
    log_debug "Creating git-commit agent..."
    if _template_path=$(find_template "agents/claude/git-commit.md.template" "$_project_root"); then
        if [ "$_dry_run" -eq 0 ]; then
            substitute_template_simple \
                "$_template_path" \
                "$_claude_dir/agents/git-commit.md" \
                "$_project_root" \
                "$_rust_list" \
                "$_arch_list"
        else
            echo "  Would create: .claude/agents/git-commit.md"
            echo "    from: $_template_path"
        fi
    else
        log_warn "Skipping git-commit agent (template not found)"
    fi
    
    if [ "$_dry_run" -eq 0 ]; then
        log_success "Claude Code agents setup complete!"
        log_info "Created agents: ai-knowledge-rust, ai-knowledge-architecture, git-commit"
    fi
}

# Shared setup preparation logic
prepare_setup() {
    _tool="$1"
    
    # Detect project root
    if ! _project_root=$(detect_project_root); then
        return 1
    fi
    
    log_info "Project root: $_project_root"
    
    # Check .aiassisted directory exists
    if [ ! -d "$_project_root/.aiassisted" ]; then
        log_error ".aiassisted directory not found in project root"
        log_info "Run 'aiassisted install' first to set up the project"
        return 1
    fi
    
    # Verify at least one template source exists (project or global)
    _has_templates=0
    if [ -d "$_project_root/.aiassisted/templates" ] || [ -d "$HOME/.aiassisted/templates" ]; then
        _has_templates=1
    fi
    
    if [ $_has_templates -eq 0 ]; then
        log_error "No templates found"
        log_error "Templates should be in:"
        log_error "  - $_project_root/.aiassisted/templates/ (project)"
        log_error "  - $HOME/.aiassisted/templates/ (global)"
        log_info "Run the installer again to download templates"
        return 1
    fi
    
    # Detect available tools
    _opencode_available=0
    _claude_available=0
    
    if detect_opencode; then
        _opencode_available=1
        log_debug "OpenCode detected"
    fi
    
    if detect_claude; then
        _claude_available=1
        log_debug "Claude Code detected"
    fi
    
    # Check if any tools are available
    if [ $_opencode_available -eq 0 ] && [ $_claude_available -eq 0 ]; then
        log_warn "No AI coding tools detected"
        log_info "Install OpenCode or Claude Code to use this feature"
        log_info "  OpenCode: https://opencode.ai"
        log_info "  Claude Code: https://code.claude.com"
        return 1
    fi
    
    # Determine which tools to setup
    _setup_opencode=0
    _setup_claude=0
    
    case "$_tool" in
        auto)
            _setup_opencode=$_opencode_available
            _setup_claude=$_claude_available
            ;;
        opencode)
            if [ $_opencode_available -eq 0 ]; then
                log_error "OpenCode not found"
                log_info "Install OpenCode: https://opencode.ai"
                return 1
            fi
            _setup_opencode=1
            ;;
        claude)
            if [ $_claude_available -eq 0 ]; then
                log_error "Claude Code not found"
                log_info "Install Claude Code: https://code.claude.com"
                return 1
            fi
            _setup_claude=1
            ;;
        *)
            log_error "Unknown tool: $_tool"
            log_info "Valid options: auto, opencode, claude"
            return 1
            ;;
    esac
    
    # Export variables for caller
    # shellcheck disable=SC2034
    SETUP_PROJECT_ROOT="$_project_root"
    # shellcheck disable=SC2034
    SETUP_OPENCODE=$_setup_opencode
    # shellcheck disable=SC2034
    SETUP_CLAUDE=$_setup_claude
    
    return 0
}

# Command: setup-skills (slash commands only)
cmd_setup_skills() {
    _tool="auto"
    _dry_run=0
    
    # Parse arguments
    for _arg in "$@"; do
        case "$_arg" in
            --tool=*)
                _tool="${_arg#*=}"
                ;;
            --dry-run)
                _dry_run=1
                ;;
        esac
    done
    
    # Run shared preparation
    if ! prepare_setup "$_tool"; then
        exit 1
    fi
    
    # Show what will be done
    if [ $_dry_run -eq 1 ]; then
        printf "\n%s%s[DRY RUN] The following would be created:%s\n\n" "$COLOR_BOLD" "$COLOR_YELLOW" "$COLOR_RESET"
    fi
    
    # Setup OpenCode skills
    if [ "$SETUP_OPENCODE" -eq 1 ]; then
        setup_opencode_skills_only "$SETUP_PROJECT_ROOT" "$_dry_run"
    fi
    
    # Setup Claude Code skills
    if [ "$SETUP_CLAUDE" -eq 1 ]; then
        setup_claude_skills_only "$SETUP_PROJECT_ROOT" "$_dry_run"
    fi
    
    if [ "$_dry_run" -eq 1 ]; then
        printf "\n%sRun without --dry-run to create these files%s\n\n" "$COLOR_YELLOW" "$COLOR_RESET"
    else
        printf "\n%s%sSetup complete!%s\n\n" "$COLOR_BOLD" "$COLOR_GREEN" "$COLOR_RESET"
        
        if [ "$SETUP_OPENCODE" -eq 1 ]; then
            printf "OpenCode skills: %s/git-commit%s, %s/review-rust%s\n" "$COLOR_BOLD" "$COLOR_RESET" "$COLOR_BOLD" "$COLOR_RESET"
        fi
        
        if [ "$SETUP_CLAUDE" -eq 1 ]; then
            printf "Claude Code skills: %s/git-commit%s, %s/review-rust%s\n" "$COLOR_BOLD" "$COLOR_RESET" "$COLOR_BOLD" "$COLOR_RESET"
        fi
        
        printf "\n"
    fi
}

# Command: setup-agents (custom subagents only)
cmd_setup_agents() {
    _tool="auto"
    _dry_run=0
    
    # Parse arguments
    for _arg in "$@"; do
        case "$_arg" in
            --tool=*)
                _tool="${_arg#*=}"
                ;;
            --dry-run)
                _dry_run=1
                ;;
        esac
    done
    
    # Run shared preparation
    if ! prepare_setup "$_tool"; then
        exit 1
    fi
    
    # Show what will be done
    if [ $_dry_run -eq 1 ]; then
        printf "\n%s%s[DRY RUN] The following would be created:%s\n\n" "$COLOR_BOLD" "$COLOR_YELLOW" "$COLOR_RESET"
    fi
    
    # Setup OpenCode agents
    if [ "$SETUP_OPENCODE" -eq 1 ]; then
        setup_opencode_agents_only "$SETUP_PROJECT_ROOT" "$_dry_run"
    fi
    
    # Setup Claude Code agents
    if [ "$SETUP_CLAUDE" -eq 1 ]; then
        setup_claude_agents_only "$SETUP_PROJECT_ROOT" "$_dry_run"
    fi
    
    if [ "$_dry_run" -eq 1 ]; then
        printf "\n%sRun without --dry-run to create these files%s\n\n" "$COLOR_YELLOW" "$COLOR_RESET"
    else
        printf "\n%s%sSetup complete!%s\n\n" "$COLOR_BOLD" "$COLOR_GREEN" "$COLOR_RESET"
        
        if [ "$SETUP_OPENCODE" -eq 1 ]; then
            printf "OpenCode agents: %sai-knowledge-rust%s, %sai-knowledge-architecture%s, %sgit-commit%s\n" "$COLOR_BOLD" "$COLOR_RESET" "$COLOR_BOLD" "$COLOR_RESET" "$COLOR_BOLD" "$COLOR_RESET"
        fi
        
        if [ "$SETUP_CLAUDE" -eq 1 ]; then
            printf "Claude Code agents: %sai-knowledge-rust%s, %sai-knowledge-architecture%s, %sgit-commit%s\n" "$COLOR_BOLD" "$COLOR_RESET" "$COLOR_BOLD" "$COLOR_RESET" "$COLOR_BOLD" "$COLOR_RESET"
        fi
        
        printf "\n"
    fi
}

# Config command
# Usage: aiassisted config <subcommand> [args]
cmd_config() {
    _subcommand="${1:-show}"
    _config_file="$HOME/.aiassisted/config.toml"
    
    case "$_subcommand" in
        show)
            if [ -f "$_config_file" ]; then
                cat "$_config_file"
            else
                log_error "Configuration file not found: $_config_file"
                log_info "Run the installer to create default configuration"
                return 1
            fi
            ;;
        get)
            _key="$2"
            if [ -z "$_key" ]; then
                log_error "Usage: aiassisted config get <key>"
                log_info "Example: aiassisted config get verbosity"
                return 1
            fi
            if _value=$(toml_get "$_config_file" "$_key"); then
                echo "$_value"
            else
                log_error "Key not found: $_key"
                return 1
            fi
            ;;
        edit)
            if [ ! -f "$_config_file" ]; then
                log_error "Configuration file not found: $_config_file"
                log_info "Run the installer to create default configuration"
                return 1
            fi
            ${EDITOR:-vi} "$_config_file"
            log_success "Configuration edited"
            ;;
        reset)
            log_warn "This will reset your configuration to defaults."
            printf "Continue? [y/N]: "
            read -r _response
            case "$_response" in
                [yY]|[yY][eE][sS])
                    _config_dir="$HOME/.aiassisted"
                    _temp_config=$(mktemp)
                    
                    # Download default config
                    if download_file "${GITHUB_RAW_URL}/.aiassisted/config/config.toml.default" "$_temp_config"; then
                        mv "$_temp_config" "$_config_file"
                        log_success "Configuration reset to defaults"
                    else
                        log_error "Failed to download default configuration"
                        rm -f "$_temp_config"
                        return 1
                    fi
                    ;;
                *)
                    log_info "Reset cancelled"
                    return 0
                    ;;
            esac
            ;;
        path)
            echo "$_config_file"
            ;;
        *)
            log_error "Unknown config subcommand: $_subcommand"
            log_info "Available subcommands: show, get, edit, reset, path"
            log_info ""
            log_info "Examples:"
            log_info "  aiassisted config show              # View configuration"
            log_info "  aiassisted config get default_runtime"
            log_info "  aiassisted config edit              # Edit in \$EDITOR"
            log_info "  aiassisted config reset             # Reset to defaults"
            log_info "  aiassisted config path              # Show config file path"
            return 1
            ;;
    esac
}

# Templates command
# Usage: aiassisted templates <subcommand> [args]
cmd_templates() {
    _subcommand="${1:-list}"
    _global_templates="$HOME/.aiassisted/templates"
    
    # Detect project root
    if ! _project_root=$(detect_project_root 2>/dev/null); then
        _project_root="$(pwd)"
        _in_git_repo=0
    else
        _in_git_repo=1
    fi
    
    _project_templates="$_project_root/.aiassisted/templates"
    
    case "$_subcommand" in
        list)
            log_info "Template locations:"
            printf "\n%s%sGlobal templates%s (%s):\n" "$COLOR_BOLD" "$COLOR_BLUE" "$COLOR_RESET" "$_global_templates"
            
            if [ -d "$_global_templates" ]; then
                find "$_global_templates" -name "*.template" -type f | while read -r _template; do
                    # shellcheck disable=SC2295  # Intentional pattern substitution
                    _rel_path="${_template#$_global_templates/}"
                    printf "  %s%s%s\n" "$COLOR_GREEN" "$_rel_path" "$COLOR_RESET"
                done
            else
                printf "  %s(none - run installer to create)%s\n" "$COLOR_YELLOW" "$COLOR_RESET"
            fi
            
            printf "\n%s%sProject templates%s (%s):\n" "$COLOR_BOLD" "$COLOR_BLUE" "$COLOR_RESET" "$_project_templates"
            
            if [ -d "$_project_templates" ]; then
                _template_files=$(find "$_project_templates" -name "*.template" -type f 2>/dev/null)
                if [ -n "$_template_files" ]; then
                    echo "$_template_files" | while read -r _template; do
                        # shellcheck disable=SC2295  # Intentional pattern substitution
                        _rel_path="${_template#$_project_templates/}"
                        printf "  %s%s%s (custom)\n" "$COLOR_CYAN" "$_rel_path" "$COLOR_RESET"
                    done
                else
                    printf "  %s(none - run 'aiassisted templates init' to customize)%s\n" "$COLOR_YELLOW" "$COLOR_RESET"
                fi
            else
                printf "  %s(none - run 'aiassisted templates init' to customize)%s\n" "$COLOR_YELLOW" "$COLOR_RESET"
            fi
            printf "\n"
            ;;
        
        show)
            _template_path="$2"
            if [ -z "$_template_path" ]; then
                log_error "Usage: aiassisted templates show <path>"
                log_info "Example: aiassisted templates show skills/opencode/git-commit.SKILL.md.template"
                return 1
            fi
            
            # Try to find the template
            if _full_path=$(find_template "$_template_path" "$_project_root" 2>/dev/null); then
                log_info "Showing: $_full_path"
                printf "\n"
                cat "$_full_path"
            else
                log_error "Template not found: $_template_path"
                log_info "Use 'aiassisted templates list' to see available templates"
                return 1
            fi
            ;;
        
        init)
            _force=0
            if [ "$2" = "--force" ]; then
                _force=1
            fi
            
            # Check if .aiassisted directory exists
            if [ ! -d "$_project_root/.aiassisted" ]; then
                log_error ".aiassisted directory not found in project root"
                log_info "Run 'aiassisted install' first to set up the project"
                return 1
            fi
            
            # Check if templates already exist
            if [ -d "$_project_templates" ] && [ $_force -eq 0 ]; then
                log_warn "Project templates directory already exists: $_project_templates"
                printf "Overwrite existing templates? [y/N]: "
                read -r _response
                case "$_response" in
                    [yY]|[yY][eE][sS])
                        log_info "Overwriting existing templates..."
                        ;;
                    *)
                        log_info "Cancelled"
                        return 0
                        ;;
                esac
            fi
            
            # Check if global templates exist
            if [ ! -d "$_global_templates" ]; then
                log_error "Global templates not found: $_global_templates"
                log_info "Run the installer again to download templates"
                return 1
            fi
            
            # Copy templates
            log_info "Copying templates from global to project..."
            
            if ! mkdir -p "$_project_templates"; then
                log_error "Failed to create project templates directory"
                return 1
            fi
            
            if ! cp -r "$_global_templates"/* "$_project_templates"/; then
                log_error "Failed to copy templates"
                return 1
            fi
            
            _count=$(find "$_project_templates" -name "*.template" -type f | wc -l | tr -d ' ')
            
            log_success "Copied $_count templates to: $_project_templates"
            log_info ""
            log_info "Next steps:"
            log_info "  1. Customize templates: vim $_project_templates/skills/opencode/..."
            log_info "  2. Generate skills: aiassisted setup-skills"
            log_info "  3. Generate agents: aiassisted setup-agents"
            log_info "  4. Commit to git: git add .aiassisted/templates/"
            ;;
        
        sync)
            _force=0
            if [ "$2" = "--force" ]; then
                _force=1
            fi
            
            # Check if project templates exist
            if [ ! -d "$_project_templates" ]; then
                log_error "Project templates directory not found: $_project_templates"
                log_info "Run 'aiassisted templates init' first to initialize templates"
                return 1
            fi
            
            # Check if global templates exist
            if [ ! -d "$_global_templates" ]; then
                log_error "Global templates not found: $_global_templates"
                log_info "Run the installer again to download templates"
                return 1
            fi
            
            # Show what would be updated
            log_info "Comparing project templates with global templates..."
            printf "\n"
            
            # Check for changes (avoid subshell issue)
            _changes_output=$(find "$_global_templates" -name "*.template" -type f 2>/dev/null | while read -r _global_file; do
                # shellcheck disable=SC2295  # Intentional pattern substitution
                _rel_path="${_global_file#$_global_templates/}"
                _project_file="$_project_templates/$_rel_path"
                
                if [ ! -f "$_project_file" ]; then
                    printf "  + %s (new)\n" "$_rel_path"
                elif ! cmp -s "$_global_file" "$_project_file"; then
                    printf "  ~ %s (modified)\n" "$_rel_path"
                fi
            done)
            
            if [ -z "$_changes_output" ]; then
                log_success "Project templates are up to date"
                return 0
            fi
            
            # Display changes with colors
            echo "$_changes_output" | while read -r _line; do
                if echo "$_line" | grep -q "^  + "; then
                    _rel=$(echo "$_line" | sed 's/^  + //' | sed 's/ (new)$//')
                    printf "  %s+ %s%s (new)\n" "$COLOR_GREEN" "$_rel" "$COLOR_RESET"
                elif echo "$_line" | grep -q "^  ~ "; then
                    _rel=$(echo "$_line" | sed 's/^  ~ //' | sed 's/ (modified)$//')
                    printf "  %s~ %s%s (modified)\n" "$COLOR_YELLOW" "$_rel" "$COLOR_RESET"
                fi
            done
            
            printf "\n"
            
            if [ $_force -eq 0 ]; then
                log_warn "This will overwrite your custom templates"
                printf "Continue? [y/N]: "
                read -r _response
                case "$_response" in
                    [yY]|[yY][eE][sS])
                        log_info "Syncing templates..."
                        ;;
                    *)
                        log_info "Cancelled"
                        return 0
                        ;;
                esac
            fi
            
            # Sync templates
            if ! cp -r "$_global_templates"/* "$_project_templates"/; then
                log_error "Failed to sync templates"
                return 1
            fi
            
            log_success "Templates synced from global"
            ;;
        
        path)
            printf "Global:  %s\n" "$_global_templates"
            printf "Project: %s\n" "$_project_templates"
            ;;
        
        diff)
            _template_path="$2"
            
            if [ -z "$_template_path" ]; then
                # Show all diffs
                log_info "Comparing all templates..."
                find "$_global_templates" -name "*.template" -type f 2>/dev/null | while read -r _global_file; do
                    # shellcheck disable=SC2295  # Intentional pattern substitution
                    _rel_path="${_global_file#$_global_templates/}"
                    _project_file="$_project_templates/$_rel_path"
                    
                    if [ -f "$_project_file" ]; then
                        if ! cmp -s "$_global_file" "$_project_file"; then
                            printf "\n%s%s=== %s ===%s\n" "$COLOR_BOLD" "$COLOR_BLUE" "$_rel_path" "$COLOR_RESET"
                            diff -u "$_global_file" "$_project_file" || true
                        fi
                    fi
                done
            else
                # Show specific template diff
                _global_file="$_global_templates/$_template_path"
                _project_file="$_project_templates/$_template_path"
                
                if [ ! -f "$_global_file" ]; then
                    log_error "Global template not found: $_template_path"
                    return 1
                fi
                
                if [ ! -f "$_project_file" ]; then
                    log_error "Project template not found: $_template_path"
                    log_info "Run 'aiassisted templates init' first"
                    return 1
                fi
                
                diff -u "$_global_file" "$_project_file" || true
            fi
            ;;
        
        *)
            log_error "Unknown templates subcommand: $_subcommand"
            log_info "Available subcommands: list, show, init, sync, path, diff"
            log_info ""
            log_info "Examples:"
            log_info "  aiassisted templates list                                      # List all templates"
            log_info "  aiassisted templates show skills/opencode/git-commit.SKILL.md.template"
            log_info "  aiassisted templates init                                      # Copy to project"
            log_info "  aiassisted templates sync                                      # Update from global"
            log_info "  aiassisted templates diff                                      # Show differences"
            return 1
            ;;
    esac
}

cmd_help() {
    cat <<'EOF'
aiassisted - AI-Assisted Engineering Guidelines Installer

Usage:
  aiassisted <command> [options]

Commands:
  install [--path=DIR]              Install .aiassisted to directory (default: current)
  update [--force] [--path=DIR]     Update existing .aiassisted installation
  check [--path=DIR]                Check if updates are available
  setup-skills [--tool=TOOL]        Setup AI skills/slash commands (opencode, claude, or auto)
  setup-agents [--tool=TOOL]        Setup AI custom agents (opencode, claude, or auto)
  templates <subcommand>            Manage templates (list, show, init, sync, path, diff)
  config <subcommand>               Manage configuration (show, get, edit, reset, path)
  version                           Show CLI version
  self-update                       Update the aiassisted CLI itself
  help                              Show this help message

Options:
  --path=DIR                        Target directory (default: current directory)
  --force                           Skip confirmation prompts during update
  --tool=TOOL                       AI tool to setup (opencode, claude, auto)
  --dry-run                         Show what would be created without creating
  --verbose                         Show detailed output
  --quiet                           Show only errors

Examples:
  # Install to current directory
  aiassisted install

  # Install to specific project
  aiassisted install --path=/path/to/project

  # Check for updates
  aiassisted check

  # Update with confirmation
  aiassisted update

  # Force update without confirmation
  aiassisted update --force

  # Setup AI skills/slash commands (auto-detect tools)
  aiassisted setup-skills

  # Setup AI custom agents (auto-detect tools)
  aiassisted setup-agents

  # Setup for specific tool
  aiassisted setup-skills --tool=opencode
  aiassisted setup-agents --tool=claude

  # Preview what would be created
  aiassisted setup-skills --dry-run
  aiassisted setup-agents --dry-run

  # Manage templates
  aiassisted templates list          # List all templates
  aiassisted templates init          # Copy to project for customization
  aiassisted templates sync          # Update from global

  # Manage configuration
  aiassisted config show                 # View current config
  aiassisted config get verbosity        # Get specific value
  aiassisted config edit                 # Edit in $EDITOR

  # Update CLI tool itself
  aiassisted self-update

For more information, visit:
  https://github.com/rstlix0x0/aiassisted
EOF
}

###########################################
# Main Entry Point
###########################################

main() {
    # Parse global flags first
    for _arg in "$@"; do
        case "$_arg" in
            --verbose)
                VERBOSITY=2
                ;;
            --quiet)
                VERBOSITY=0
                ;;
        esac
    done
    
    # Get command
    _command="${1:-help}"
    shift || true
    
    # Route to command
    case "$_command" in
        install)
            cmd_install "$@"
            ;;
        update)
            cmd_update "$@"
            ;;
        check)
            cmd_check "$@"
            ;;
        setup-skills)
            cmd_setup_skills "$@"
            ;;
        setup-agents)
            cmd_setup_agents "$@"
            ;;
        templates)
            cmd_templates "$@"
            ;;
        config)
            cmd_config "$@"
            ;;
        version)
            cmd_version
            ;;
        self-update)
            cmd_self_update
            ;;
        help|--help|-h)
            cmd_help
            ;;
        *)
            log_error "Unknown command: $_command"
            printf "Run 'aiassisted help' for usage information\n"
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
