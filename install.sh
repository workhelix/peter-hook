#!/bin/bash

# Git Hook Manager Installation Script
# Usage: curl -fsSL https://raw.githubusercontent.com/example/git-hook-manager/main/install.sh | bash

set -e

# Configuration
VERSION="${VERSION:-latest}"
REPO="example/git-hook-manager"
BINARY_NAME="git-hook-manager"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Utility functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect OS and architecture
detect_platform() {
    local os arch
    
    case "$(uname -s)" in
        Linux*)
            os="linux"
            ;;
        Darwin*)
            os="darwin"
            ;;
        CYGWIN*|MINGW*|MSYS*)
            os="windows"
            ;;
        *)
            log_error "Unsupported operating system: $(uname -s)"
            exit 1
            ;;
    esac
    
    case "$(uname -m)" in
        x86_64|amd64)
            arch="amd64"
            ;;
        aarch64|arm64)
            arch="arm64"
            ;;
        *)
            log_error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac
    
    echo "${os}-${arch}"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Download and verify binary
download_binary() {
    local platform="$1"
    local version="$2"
    local download_url base_url archive_name
    
    if [ "$version" = "latest" ]; then
        base_url="https://github.com/${REPO}/releases/latest/download"
    else
        base_url="https://github.com/${REPO}/releases/download/${version}"
    fi
    
    if [ "$(echo "$platform" | grep -c windows)" -eq 1 ]; then
        archive_name="${BINARY_NAME}-${version}-${platform}.zip"
    else
        archive_name="${BINARY_NAME}-${version}-${platform}.tar.gz"
    fi
    
    download_url="${base_url}/${archive_name}"
    local checksum_url="${download_url}.sha256"
    
    log_info "Downloading ${BINARY_NAME} ${version} for ${platform}..."
    log_info "URL: ${download_url}"
    
    # Create temporary directory
    local temp_dir
    temp_dir=$(mktemp -d)
    trap "rm -rf $temp_dir" EXIT
    
    # Download archive and checksum
    if command_exists curl; then
        curl -fsSL "$download_url" -o "$temp_dir/$archive_name"
        curl -fsSL "$checksum_url" -o "$temp_dir/$archive_name.sha256"
    elif command_exists wget; then
        wget -q "$download_url" -O "$temp_dir/$archive_name"
        wget -q "$checksum_url" -O "$temp_dir/$archive_name.sha256"
    else
        log_error "Neither curl nor wget is available. Please install one of them."
        exit 1
    fi
    
    # Verify checksum
    log_info "Verifying checksum..."
    if command_exists sha256sum; then
        (cd "$temp_dir" && sha256sum -c "$archive_name.sha256")
    elif command_exists shasum; then
        (cd "$temp_dir" && shasum -a 256 -c "$archive_name.sha256")
    else
        log_warning "No checksum utility found. Skipping verification."
    fi
    
    # Extract archive
    log_info "Extracting archive..."
    if [ "$(echo "$platform" | grep -c windows)" -eq 1 ]; then
        if command_exists unzip; then
            unzip -q "$temp_dir/$archive_name" -d "$temp_dir"
            binary_path="$temp_dir/${BINARY_NAME}.exe"
        else
            log_error "unzip is required to extract Windows archives"
            exit 1
        fi
    else
        tar -xzf "$temp_dir/$archive_name" -C "$temp_dir"
        binary_path="$temp_dir/$BINARY_NAME"
    fi
    
    echo "$binary_path"
}

# Install binary
install_binary() {
    local binary_path="$1"
    
    # Create install directory if it doesn't exist
    mkdir -p "$INSTALL_DIR"
    
    # Copy binary to install directory
    cp "$binary_path" "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    
    log_success "Installed $BINARY_NAME to $INSTALL_DIR/$BINARY_NAME"
}

# Add to PATH if needed
update_path() {
    local shell_profile
    
    # Detect shell and profile file
    case "$SHELL" in
        */bash)
            shell_profile="$HOME/.bashrc"
            [ ! -f "$shell_profile" ] && shell_profile="$HOME/.bash_profile"
            ;;
        */zsh)
            shell_profile="$HOME/.zshrc"
            ;;
        */fish)
            shell_profile="$HOME/.config/fish/config.fish"
            ;;
        *)
            shell_profile="$HOME/.profile"
            ;;
    esac
    
    # Check if install directory is in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        log_warning "$INSTALL_DIR is not in your PATH"
        
        if [ -f "$shell_profile" ]; then
            log_info "Adding $INSTALL_DIR to PATH in $shell_profile"
            echo "" >> "$shell_profile"
            echo "# Added by git-hook-manager installer" >> "$shell_profile"
            echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$shell_profile"
            log_success "Added $INSTALL_DIR to PATH in $shell_profile"
            log_info "Please restart your terminal or run: source $shell_profile"
        else
            log_warning "Please add $INSTALL_DIR to your PATH manually"
            log_info "Add this line to your shell profile:"
            log_info "export PATH=\"$INSTALL_DIR:\$PATH\""
        fi
    fi
}

# Verify installation
verify_installation() {
    if [ -x "$INSTALL_DIR/$BINARY_NAME" ]; then
        local version_output
        version_output=$("$INSTALL_DIR/$BINARY_NAME" --version 2>/dev/null || echo "unknown")
        log_success "Installation verified: $version_output"
        
        log_info ""
        log_info "Git Hook Manager has been installed successfully!"
        log_info ""
        log_info "Quick start:"
        log_info "  1. Create a hooks.toml file in your project:"
        log_info "     echo '[hooks.test]' > hooks.toml"
        log_info "     echo 'command = \"echo hello world\"' >> hooks.toml"
        log_info ""
        log_info "  2. Validate your configuration:"
        log_info "     $BINARY_NAME validate"
        log_info ""
        log_info "  3. Run your hooks:"
        log_info "     $BINARY_NAME run test"
        log_info ""
        log_info "For more information, run: $BINARY_NAME --help"
        return 0
    else
        log_error "Installation verification failed"
        return 1
    fi
}

# Main installation function
main() {
    log_info "Starting Git Hook Manager installation..."
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --version)
                VERSION="$2"
                shift 2
                ;;
            --install-dir)
                INSTALL_DIR="$2"
                shift 2
                ;;
            --help)
                echo "Git Hook Manager Installer"
                echo ""
                echo "Usage: $0 [options]"
                echo ""
                echo "Options:"
                echo "  --version VERSION    Install specific version (default: latest)"
                echo "  --install-dir DIR    Installation directory (default: \$HOME/.local/bin)"
                echo "  --help              Show this help message"
                echo ""
                echo "Environment variables:"
                echo "  VERSION             Same as --version"
                echo "  INSTALL_DIR         Same as --install-dir"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                log_info "Use --help for usage information"
                exit 1
                ;;
        esac
    done
    
    # Detect platform
    local platform
    platform=$(detect_platform)
    log_info "Detected platform: $platform"
    
    # Download binary
    local binary_path
    binary_path=$(download_binary "$platform" "$VERSION")
    
    # Install binary
    install_binary "$binary_path"
    
    # Update PATH if needed
    update_path
    
    # Verify installation
    if verify_installation; then
        log_success "Git Hook Manager installation completed successfully!"
    else
        log_error "Installation completed but verification failed"
        exit 1
    fi
}

# Check for required tools
check_requirements() {
    local missing_tools=()
    
    if ! command_exists curl && ! command_exists wget; then
        missing_tools+=("curl or wget")
    fi
    
    if ! command_exists tar; then
        missing_tools+=("tar")
    fi
    
    if [ ${#missing_tools[@]} -gt 0 ]; then
        log_error "Missing required tools: ${missing_tools[*]}"
        log_info "Please install the missing tools and try again"
        exit 1
    fi
}

# Run installation
if [ "${BASH_SOURCE[0]}" = "${0}" ]; then
    check_requirements
    main "$@"
fi