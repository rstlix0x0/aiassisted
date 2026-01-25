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
    
    # Create install directories for all runtimes
    _data_dir="$HOME/.local/share/aiassisted"
    _shell_dir="$_data_dir/src/shell"
    _python_dir="$_data_dir/src/python"
    _bun_dir="$_data_dir/src/bun"
    
    if ! mkdir -p "$INSTALL_DIR" "$_data_dir" "$_shell_dir" "$_python_dir" "$_bun_dir"; then
        log_error "Failed to create installation directories"
        exit 1
    fi
    
    # Download CLI orchestrator
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
    
    # Download shell runtime
    _temp_runtime=$(mktemp)
    log_info "Downloading shell runtime..."
    if ! download_file "${GITHUB_RAW_URL}/src/shell/core.sh" "$_temp_runtime"; then
        log_error "Failed to download shell runtime"
        rm -f "$_temp_runtime"
        exit 1
    fi
    
    # Make executable and move to runtime directory
    chmod +x "$_temp_runtime"
    if ! mv "$_temp_runtime" "$_shell_dir/core.sh"; then
        log_error "Failed to install shell runtime"
        rm -f "$_temp_runtime"
        exit 1
    fi
    
    # Download README.md for Python runtime (required by pyproject.toml)
    _temp_readme=$(mktemp)
    if download_file "${GITHUB_RAW_URL}/README.md" "$_temp_readme"; then
        mv "$_temp_readme" "$_data_dir/README.md"
    else
        log_warn "Failed to download README.md (needed for Python runtime)"
        rm -f "$_temp_readme"
    fi
    
    # Download Python runtime files
    log_info "Downloading Python runtime backend..."
    _python_files_ok=true
    
    # Python config and source directory
    mkdir -p "$_python_dir/aiassisted"
    
    # Download pyproject.toml
    _temp=$(mktemp)
    if download_file "${GITHUB_RAW_URL}/src/python/pyproject.toml" "$_temp"; then
        mv "$_temp" "$_python_dir/pyproject.toml"
    else
        log_warn "Failed to download pyproject.toml"
        rm -f "$_temp"
        _python_files_ok=false
    fi
    
    # Python source files
    for _file in __init__.py __main__.py cli.py downloader.py installer.py manifest.py; do
        _temp=$(mktemp)
        if download_file "${GITHUB_RAW_URL}/src/python/aiassisted/$_file" "$_temp"; then
            mv "$_temp" "$_python_dir/aiassisted/$_file"
        else
            log_warn "Failed to download aiassisted/$_file"
            rm -f "$_temp"
            _python_files_ok=false
        fi
    done
    
    if [ "$_python_files_ok" = true ]; then
        log_success "Installed Python runtime backend"
    else
        log_warn "Python runtime partially installed (some files missing)"
    fi
    
    # Download Bun runtime files
    log_info "Downloading Bun runtime backend..."
    _bun_files_ok=true
    
    # Bun config and source directory
    mkdir -p "$_bun_dir/src"
    
    # Download config files (tsconfig.json is optional)
    _temp=$(mktemp)
    if download_file "${GITHUB_RAW_URL}/src/bun/package.json" "$_temp"; then
        mv "$_temp" "$_bun_dir/package.json"
    else
        log_warn "Failed to download package.json"
        rm -f "$_temp"
        _bun_files_ok=false
    fi
    
    _temp=$(mktemp)
    if download_file "${GITHUB_RAW_URL}/src/bun/tsconfig.json" "$_temp"; then
        mv "$_temp" "$_bun_dir/tsconfig.json"
    else
        # tsconfig.json is optional, don't mark as failure
        rm -f "$_temp"
    fi
    
    # Bun source files
    for _file in index.ts cli.ts downloader.ts installer.ts manifest.ts; do
        _temp=$(mktemp)
        if download_file "${GITHUB_RAW_URL}/src/bun/src/$_file" "$_temp"; then
            mv "$_temp" "$_bun_dir/src/$_file"
        else
            log_warn "Failed to download src/$_file"
            rm -f "$_temp"
            _bun_files_ok=false
        fi
    done
    
    if [ "$_bun_files_ok" = true ]; then
        log_success "Installed Bun runtime backend"
    else
        log_warn "Bun runtime partially installed (some files missing)"
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

setup_global_config() {
    _config_dir="$HOME/.aiassisted"
    _config_file="$_config_dir/config.toml"
    _templates_dir="$_config_dir/templates"
    _data_dir="$HOME/.local/share/aiassisted/.aiassisted"
    
    log_info "Setting up global configuration directory at $_config_dir"
    
    # Create directories
    if ! mkdir -p "$_config_dir" "$_templates_dir" "$_config_dir/cache" "$_config_dir/state"; then
        log_error "Failed to create global configuration directory"
        exit 1
    fi
    
    # Download default config.toml if not exists
    if [ ! -f "$_config_file" ]; then
        log_info "Creating default configuration file..."
        _temp_config=$(mktemp)
        
        if download_file "${GITHUB_RAW_URL}/.aiassisted/config.toml.default" "$_temp_config"; then
            mv "$_temp_config" "$_config_file"
            log_success "Created configuration file: $_config_file"
        else
            log_warn "Failed to download config template, creating minimal config"
            rm -f "$_temp_config"
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

sync_global_templates() {
    _data_dir="$HOME/.local/share/aiassisted/.aiassisted"
    _templates_src="$_data_dir/templates"
    _templates_dst="$HOME/.aiassisted/templates"
    
    # Check if templates source exists (from aiassisted install)
    if [ ! -d "$_templates_src" ]; then
        log_info "Templates will be available after first 'aiassisted install'"
        return 0
    fi
    
    log_info "Syncing templates to global directory..."
    
    # Copy templates to global directory
    if cp -r "$_templates_src"/* "$_templates_dst/"; then
        log_success "Templates synced to $_templates_dst"
    else
        log_warn "Failed to sync templates (this is OK for first install)"
    fi
}

install_aiassisted_dir() {
    log_info "Installing .aiassisted directory to current directory"
    
    # Run the CLI to install .aiassisted
    if ! "$CLI_PATH" install; then
        log_error "Failed to install .aiassisted directory"
        exit 1
    fi
    
    # After install, sync templates to global directory
    sync_global_templates
}

###########################################
# Main Entry Point
###########################################

main() {
    printf "\n%s%saiassisted Installer%s\n\n" "$COLOR_BOLD" "$COLOR_GREEN" "$COLOR_RESET"
    
    # Install CLI
    install_cli
    
    printf "\n"
    
    # Setup global configuration
    setup_global_config
    
    printf "\n"
    
    # Install .aiassisted directory
    install_aiassisted_dir
    
    printf "\n"
    log_success "Installation complete!"
    
    printf "\n%s%sNext Steps:%s\n" "$COLOR_BOLD" "$COLOR_GREEN" "$COLOR_RESET"
    printf "  1. Restart your terminal or run: %ssource %s%s\n" "$COLOR_BOLD" "$(get_shell_rc_file)" "$COLOR_RESET"
    printf "  2. Use %saiassisted help%s to see all available commands\n" "$COLOR_BOLD" "$COLOR_RESET"
    printf "  3. Run %saiassisted setup-skills%s to setup AI coding tools\n" "$COLOR_BOLD" "$COLOR_RESET"
    printf "  4. Edit config: %saiassisted config edit%s\n\n" "$COLOR_BOLD" "$COLOR_RESET"
}

# Run main function
main
