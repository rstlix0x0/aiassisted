#!/bin/sh
#
# aiassisted installer
#
# This script installs the aiassisted CLI tool by cloning the repository
# to ~/.aiassisted/source/aiassisted and creating a symlink to ~/.local/bin.
#
# Requirements:
#   - git
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/rstlix0x0/aiassisted/main/install.sh | sh
#

set -e

# GitHub repository
GITHUB_REPO="rstlix0x0/aiassisted"
GITHUB_URL="https://github.com/${GITHUB_REPO}.git"

# Installation directories
AIASSISTED_HOME="$HOME/.aiassisted"
SOURCE_DIR="$AIASSISTED_HOME/source/aiassisted"
BIN_DIR="$HOME/.local/bin"
CLI_PATH="$BIN_DIR/aiassisted"

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
# Prerequisite Checks
###########################################

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check git
    if ! command -v git >/dev/null 2>&1; then
        log_error "git is required but not found"
        printf "\n%sPlease install git first:%s\n" "$COLOR_BOLD" "$COLOR_RESET"
        printf "  macOS:   %sxcode-select --install%s\n" "$COLOR_BOLD" "$COLOR_RESET"
        printf "  Ubuntu:  %ssudo apt install git%s\n" "$COLOR_BOLD" "$COLOR_RESET"
        printf "  Debian:  %ssudo apt install git%s\n" "$COLOR_BOLD" "$COLOR_RESET"
        printf "  Fedora:  %ssudo dnf install git%s\n" "$COLOR_BOLD" "$COLOR_RESET"
        printf "  Arch:    %ssudo pacman -S git%s\n" "$COLOR_BOLD" "$COLOR_RESET"
        printf "  Alpine:  %ssudo apk add git%s\n\n" "$COLOR_BOLD" "$COLOR_RESET"
        exit 1
    fi
    
    log_success "git found: $(git --version)"
}

###########################################
# Installation Functions
###########################################

install_cli() {
    log_info "Installing aiassisted to $AIASSISTED_HOME"
    
    # Create necessary directories
    if ! mkdir -p "$AIASSISTED_HOME" "$BIN_DIR"; then
        log_error "Failed to create installation directories"
        exit 1
    fi
    
    # Remove existing installation if present
    if [ -d "$SOURCE_DIR" ]; then
        log_warn "Existing installation found at $SOURCE_DIR"
        log_info "Removing old installation..."
        rm -rf "$SOURCE_DIR"
    fi
    
    # Clone repository
    log_info "Cloning repository from $GITHUB_REPO..."
    if ! git clone --depth 1 "$GITHUB_URL" "$SOURCE_DIR"; then
        log_error "Failed to clone repository"
        exit 1
    fi
    
    log_success "Cloned repository to $SOURCE_DIR"
    
    # Create symlink to CLI
    log_info "Creating symlink to $CLI_PATH..."
    if [ -L "$CLI_PATH" ] || [ -f "$CLI_PATH" ]; then
        rm -f "$CLI_PATH"
    fi
    
    if ! ln -sf "$SOURCE_DIR/bin/aiassisted" "$CLI_PATH"; then
        log_error "Failed to create symlink"
        exit 1
    fi
    
    log_success "Created symlink: $CLI_PATH -> $SOURCE_DIR/bin/aiassisted"
    
    # Check if in PATH
    if ! is_in_path "$BIN_DIR"; then
        log_warn "$BIN_DIR is not in your PATH"
        add_to_path "$BIN_DIR"
    else
        log_success "$BIN_DIR is already in your PATH"
    fi
}

setup_global_config() {
    _config_file="$AIASSISTED_HOME/config.toml"
    _templates_dir="$AIASSISTED_HOME/templates"
    _cache_dir="$AIASSISTED_HOME/cache"
    _state_dir="$AIASSISTED_HOME/state"
    
    log_info "Setting up global configuration directory"
    
    # Create directories
    if ! mkdir -p "$_templates_dir" "$_cache_dir" "$_state_dir"; then
        log_error "Failed to create configuration directories"
        exit 1
    fi
    
    # Create default config.toml if not exists
    if [ ! -f "$_config_file" ]; then
        log_info "Creating default configuration file..."
        
        # Check if default config exists in source
        _default_config="$SOURCE_DIR/.aiassisted/config.toml.default"
        if [ -f "$_default_config" ]; then
            cp "$_default_config" "$_config_file"
            log_success "Created configuration file: $_config_file"
        else
            # Create minimal config as fallback
            cat > "$_config_file" << 'EOF'
# aiassisted CLI Configuration
[general]
default_runtime = "auto"
verbosity = 1

[install]
auto_update = true

[templates]
prefer_project = true
EOF
            log_success "Created minimal configuration file: $_config_file"
        fi
    else
        log_info "Configuration file already exists: $_config_file"
    fi
    
    log_success "Global configuration setup complete"
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
    
    # Check prerequisites
    check_prerequisites
    
    printf "\n"
    
    # Install CLI
    install_cli
    
    printf "\n"
    
    # Setup global configuration
    setup_global_config
    
    printf "\n"
    
    # Install .aiassisted directory to current directory
    install_aiassisted_dir
    
    printf "\n"
    log_success "Installation complete!"
    
    printf "\n%s%sNext Steps:%s\n" "$COLOR_BOLD" "$COLOR_GREEN" "$COLOR_RESET"
    printf "  1. Restart your terminal or run: %ssource %s%s\n" "$COLOR_BOLD" "$(get_shell_rc_file)" "$COLOR_RESET"
    printf "  2. Use %saiassisted help%s to see all available commands\n" "$COLOR_BOLD" "$COLOR_RESET"
    printf "  3. Run %saiassisted setup-skills%s to setup AI coding tools\n" "$COLOR_BOLD" "$COLOR_RESET"
    printf "  4. Edit config: %saiassisted config edit%s\n" "$COLOR_BOLD" "$COLOR_RESET"
    printf "\n%s%sInstalled to:%s\n" "$COLOR_BOLD" "$COLOR_BLUE" "$COLOR_RESET"
    printf "  Source:  %s\n" "$SOURCE_DIR"
    printf "  Config:  %s\n" "$AIASSISTED_HOME"
    printf "  CLI:     %s\n\n" "$CLI_PATH"
}

# Run main function
main
