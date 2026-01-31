#!/bin/sh
#
# aiassisted installer
#
# This script installs the aiassisted CLI tool by downloading a pre-built
# binary from GitHub releases and installing it to ~/.local/bin.
#
# Requirements:
#   - curl or wget
#   - tar
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/rstlix0x0/aiassisted/main/install.sh | sh
#
#   Or with custom version:
#   VERSION=v0.1.0 curl -fsSL https://raw.githubusercontent.com/rstlix0x0/aiassisted/main/install.sh | sh
#

set -e

# GitHub repository
GITHUB_REPO="rstlix0x0/aiassisted"

# Version to install (default: latest)
VERSION="${VERSION:-latest}"

# Installation directory
BIN_DIR="$HOME/.local/bin"

# Binary name
BINARY_NAME="aiassisted"

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
    printf "%s[SUCCESS]%s %s\n" "$COLOR_GREEN" "$COLOR_RESET" "$1"
}

log_info() {
    printf "%s[INFO]%s %s\n" "$COLOR_BLUE" "$COLOR_RESET" "$1"
}

log_warn() {
    printf "%s[WARN]%s %s\n" "$COLOR_YELLOW" "$COLOR_RESET" "$1"
}

###########################################
# Utility Functions
###########################################

# Detect user's shell
detect_shell() {
    _shell_name=$(basename "$SHELL" 2>/dev/null || echo "sh")
    echo "$_shell_name"
}

# Get shell RC file path
get_shell_rc_file() {
    _shell=$(detect_shell)

    case "$_shell" in
        bash)
            if [ -f "$HOME/.bashrc" ]; then
                echo "$HOME/.bashrc"
            elif [ -f "$HOME/.bash_profile" ]; then
                echo "$HOME/.bash_profile"
            else
                echo "$HOME/.bashrc"
            fi
            ;;
        zsh)
            if [ -f "$HOME/.zshrc" ]; then
                echo "$HOME/.zshrc"
            else
                echo "$HOME/.zshrc"
            fi
            ;;
        fish)
            echo "$HOME/.config/fish/config.fish"
            ;;
        *)
            if [ -f "$HOME/.profile" ]; then
                echo "$HOME/.profile"
            else
                echo "$HOME/.profile"
            fi
            ;;
    esac
}

# Check if directory is in PATH
is_in_path() {
    _dir="$1"

    # Normalize path
    _dir=$(cd "$_dir" 2>/dev/null && pwd || echo "$_dir")

    # Check each PATH entry
    _old_ifs="$IFS"
    IFS=":"
    for _path_entry in $PATH; do
        _path_entry=$(cd "$_path_entry" 2>/dev/null && pwd || echo "$_path_entry")
        if [ "$_path_entry" = "$_dir" ]; then
            IFS="$_old_ifs"
            return 0
        fi
    done
    IFS="$_old_ifs"

    return 1
}

# Add directory to PATH in shell RC file
add_to_path() {
    _dir="$1"
    _rc_file=$(get_shell_rc_file)

    log_info "Adding $_dir to PATH in $_rc_file"

    # Create RC file if it doesn't exist
    if [ ! -f "$_rc_file" ]; then
        touch "$_rc_file"
    fi

    # Check if already added
    if grep -q "export PATH=\"\$PATH:$_dir\"" "$_rc_file" 2>/dev/null; then
        log_info "PATH already configured in $_rc_file"
        return 0
    fi

    if grep -q "export PATH=\"$_dir:\$PATH\"" "$_rc_file" 2>/dev/null; then
        log_info "PATH already configured in $_rc_file"
        return 0
    fi

    # Add to PATH
    printf "\n# Added by aiassisted installer\nexport PATH=\"\$PATH:%s\"\n" "$_dir" >> "$_rc_file"

    log_success "Added to PATH in $_rc_file"
    log_warn "Please restart your terminal or run: source $_rc_file"
}

###########################################
# Platform Detection
###########################################

detect_platform() {
    _os=$(uname -s | tr '[:upper:]' '[:lower:]')
    _arch=$(uname -m)

    # Map architecture names
    case "$_arch" in
        x86_64)
            _arch="x86_64"
            ;;
        arm64|aarch64)
            _arch="aarch64"
            ;;
        *)
            log_error "Unsupported architecture: $_arch"
            printf "\nSupported architectures: x86_64, aarch64 (arm64)\n" >&2
            exit 1
            ;;
    esac

    # Map OS names to target triples
    case "$_os" in
        linux)
            _target="${_arch}-unknown-linux-gnu"
            _ext=""
            ;;
        darwin)
            _target="${_arch}-apple-darwin"
            _ext=""
            ;;
        msys*|mingw*|cygwin*|windows*)
            _target="${_arch}-pc-windows-msvc"
            _ext=".exe"
            ;;
        *)
            log_error "Unsupported operating system: $_os"
            printf "\nSupported OS: Linux, macOS, Windows\n" >&2
            exit 1
            ;;
    esac

    PLATFORM="$_target"
    BINARY_EXT="$_ext"
}

###########################################
# Prerequisite Checks
###########################################

check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check for curl or wget
    if command -v curl >/dev/null 2>&1; then
        DOWNLOADER="curl"
        log_success "curl found: $(curl --version | head -1)"
    elif command -v wget >/dev/null 2>&1; then
        DOWNLOADER="wget"
        log_success "wget found: $(wget --version | head -1)"
    else
        log_error "curl or wget is required but neither was found"
        printf "\n%sPlease install curl or wget first:%s\n" "$COLOR_BOLD" "$COLOR_RESET"
        printf "  macOS:   %sbrew install curl%s\n" "$COLOR_BOLD" "$COLOR_RESET"
        printf "  Ubuntu:  %ssudo apt install curl%s\n" "$COLOR_BOLD" "$COLOR_RESET"
        printf "  Fedora:  %ssudo dnf install curl%s\n" "$COLOR_BOLD" "$COLOR_RESET"
        printf "  Arch:    %ssudo pacman -S curl%s\n" "$COLOR_BOLD" "$COLOR_RESET"
        exit 1
    fi

    # Check for tar
    if ! command -v tar >/dev/null 2>&1; then
        log_error "tar is required but not found"
        printf "\n%sPlease install tar first:%s\n" "$COLOR_BOLD" "$COLOR_RESET"
        exit 1
    fi

    log_success "tar found: $(tar --version 2>&1 | head -1 || echo 'tar')"
}

###########################################
# Download Functions
###########################################

download_file() {
    _url="$1"
    _output="$2"

    if [ "$DOWNLOADER" = "curl" ]; then
        curl -fsSL "$_url" -o "$_output"
    else
        wget -q "$_url" -O "$_output"
    fi
}

###########################################
# Installation Functions
###########################################

install_binary() {
    log_info "Installing aiassisted binary for $PLATFORM"

    # Create binary directory
    if ! mkdir -p "$BIN_DIR"; then
        log_error "Failed to create directory: $BIN_DIR"
        exit 1
    fi

    # Construct download URL
    if [ "$VERSION" = "latest" ]; then
        DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/latest/download"
    else
        DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/download/${VERSION}"
    fi

    ARCHIVE_NAME="${BINARY_NAME}-${PLATFORM}.tar.gz"
    DOWNLOAD_URL="${DOWNLOAD_URL}/${ARCHIVE_NAME}"

    log_info "Downloading from: $DOWNLOAD_URL"

    # Create temporary directory
    TMP_DIR=$(mktemp -d 2>/dev/null || mktemp -d -t 'aiassisted')
    TMP_ARCHIVE="$TMP_DIR/$ARCHIVE_NAME"

    # Download archive
    if ! download_file "$DOWNLOAD_URL" "$TMP_ARCHIVE"; then
        log_error "Failed to download binary from: $DOWNLOAD_URL"
        rm -rf "$TMP_DIR"
        printf "\nThis could mean:\n" >&2
        printf "  - The version %s doesn't exist\n" "$VERSION" >&2
        printf "  - No binary available for platform: %s\n" "$PLATFORM" >&2
        printf "  - Network connection issue\n\n" >&2
        printf "Try visiting: https://github.com/%s/releases\n" "$GITHUB_REPO" >&2
        exit 1
    fi

    log_success "Downloaded binary archive"

    # Extract archive
    log_info "Extracting binary..."
    if ! tar -xzf "$TMP_ARCHIVE" -C "$TMP_DIR"; then
        log_error "Failed to extract archive"
        rm -rf "$TMP_DIR"
        exit 1
    fi

    # Find and move binary
    BINARY_PATH="$TMP_DIR/${BINARY_NAME}${BINARY_EXT}"
    if [ ! -f "$BINARY_PATH" ]; then
        log_error "Binary not found in archive: ${BINARY_NAME}${BINARY_EXT}"
        rm -rf "$TMP_DIR"
        exit 1
    fi

    # Install binary
    INSTALL_PATH="$BIN_DIR/${BINARY_NAME}${BINARY_EXT}"
    if ! mv "$BINARY_PATH" "$INSTALL_PATH"; then
        log_error "Failed to install binary to: $INSTALL_PATH"
        rm -rf "$TMP_DIR"
        exit 1
    fi

    # Make executable
    if ! chmod +x "$INSTALL_PATH"; then
        log_error "Failed to make binary executable"
        rm -rf "$TMP_DIR"
        exit 1
    fi

    # Clean up
    rm -rf "$TMP_DIR"

    log_success "Installed binary to: $INSTALL_PATH"

    # Check if in PATH
    if ! is_in_path "$BIN_DIR"; then
        log_warn "$BIN_DIR is not in your PATH"
        add_to_path "$BIN_DIR"
    else
        log_success "$BIN_DIR is already in your PATH"
    fi
}

verify_installation() {
    INSTALL_PATH="$BIN_DIR/${BINARY_NAME}${BINARY_EXT}"

    log_info "Verifying installation..."

    # Test if binary is executable
    if [ -x "$INSTALL_PATH" ]; then
        log_success "Binary is executable"
    else
        log_error "Binary is not executable: $INSTALL_PATH"
        exit 1
    fi

    # Try to get version (if binary supports --version)
    if "$INSTALL_PATH" --version >/dev/null 2>&1; then
        _version=$("$INSTALL_PATH" --version 2>&1 | head -1)
        log_success "Installation verified: $_version"
    else
        log_success "Binary installed successfully"
    fi
}

###########################################
# Main Entry Point
###########################################

main() {
    printf "\n%s%saiassisted Installer%s\n\n" "$COLOR_BOLD" "$COLOR_GREEN" "$COLOR_RESET"

    # Detect platform
    detect_platform
    log_info "Detected platform: $PLATFORM"

    printf "\n"

    # Check prerequisites
    check_prerequisites

    printf "\n"

    # Install binary
    install_binary

    printf "\n"

    # Verify installation
    verify_installation

    printf "\n"
    log_success "Installation complete!"

    printf "\n%s%sNext Steps:%s\n" "$COLOR_BOLD" "$COLOR_GREEN" "$COLOR_RESET"
    printf "  1. Restart your terminal or run: %ssource %s%s\n" "$COLOR_BOLD" "$(get_shell_rc_file)" "$COLOR_RESET"
    printf "  2. Run %saiassisted --help%s to see available commands\n" "$COLOR_BOLD" "$COLOR_RESET"
    printf "  3. Run %saiassisted install%s in your project to set up .aiassisted directory\n" "$COLOR_BOLD" "$COLOR_RESET"
    printf "  4. Configure with: %saiassisted config%s\n" "$COLOR_BOLD" "$COLOR_RESET"
    printf "\n%s%sInstalled to:%s %s\n\n" "$COLOR_BOLD" "$COLOR_BLUE" "$COLOR_RESET" "$BIN_DIR/${BINARY_NAME}${BINARY_EXT}"
}

# Run main function
main
