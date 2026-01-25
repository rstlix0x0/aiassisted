#!/bin/sh
#
# aiassisted installer
#
# This script installs the aiassisted CLI tool and immediately installs
# the .aiassisted directory to your current directory.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/rstlix0x0/aiassisted/main/install.sh | sh
#

set -e

# GitHub repository
GITHUB_REPO="rstlix0x0/aiassisted"
GITHUB_RAW_URL="https://raw.githubusercontent.com/${GITHUB_REPO}/main"

# Installation directory
INSTALL_DIR="$HOME/.local/bin"
CLI_PATH="$INSTALL_DIR/aiassisted"

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
download_file() {
    _url="$1"
    _output="$2"
    _tool=$(detect_download_tool)
    
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
# Installation Functions
###########################################

install_cli() {
    log_info "Installing aiassisted CLI to $CLI_PATH"
    
    # Create install directory
    if [ ! -d "$INSTALL_DIR" ]; then
        log_info "Creating $INSTALL_DIR"
        if ! mkdir -p "$INSTALL_DIR"; then
            log_error "Failed to create $INSTALL_DIR"
            exit 1
        fi
    fi
    
    # Download CLI script
    _temp_cli=$(mktemp)
    
    log_info "Downloading aiassisted CLI..."
    if ! download_file "${GITHUB_RAW_URL}/bin/aiassisted" "$_temp_cli"; then
        log_error "Failed to download CLI script"
        rm -f "$_temp_cli"
        exit 1
    fi
    
    # Make executable
    chmod +x "$_temp_cli"
    
    # Move to installation directory
    if ! mv "$_temp_cli" "$CLI_PATH"; then
        log_error "Failed to install CLI to $CLI_PATH"
        rm -f "$_temp_cli"
        exit 1
    fi
    
    log_success "Installed aiassisted CLI to $CLI_PATH"
    
    # Check if in PATH
    if ! is_in_path "$INSTALL_DIR"; then
        log_warn "$INSTALL_DIR is not in your PATH"
        add_to_path "$INSTALL_DIR"
    else
        log_success "$INSTALL_DIR is already in your PATH"
    fi
}

install_aiassisted_dir() {
    log_info "Installing .aiassisted directory to current directory"
    
    # Run the CLI to install .aiassisted
    if ! "$CLI_PATH" install; then
        log_error "Failed to install .aiassisted directory"
        exit 1
    fi
}

###########################################
# Main Entry Point
###########################################

main() {
    printf "\n%s%saiassisted Installer%s\n\n" "$COLOR_BOLD" "$COLOR_GREEN" "$COLOR_RESET"
    
    # Install CLI
    install_cli
    
    printf "\n"
    
    # Install .aiassisted directory
    install_aiassisted_dir
    
    printf "\n"
    log_success "Installation complete!"
    
    printf "\n%s%sNext Steps:%s\n" "$COLOR_BOLD" "$COLOR_GREEN" "$COLOR_RESET"
    printf "  1. Restart your terminal or run: %ssource %s%s\n" "$COLOR_BOLD" "$(get_shell_rc_file)" "$COLOR_RESET"
    printf "  2. Use %saiassisted help%s to see all available commands\n" "$COLOR_BOLD" "$COLOR_RESET"
    printf "  3. Run %saiassisted check%s to check for updates\n\n" "$COLOR_BOLD" "$COLOR_RESET"
}

# Run main function
main
