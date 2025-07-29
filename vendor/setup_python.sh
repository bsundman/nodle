#!/bin/bash

# Nōdle Python Setup Script
# Sets up Python runtime for Nōdle with minimal system requirements

set -e  # Exit on any error

# Configuration
PYTHON_VERSION="3.9.23"  # Use latest available 3.9.x version
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VENDOR_DIR="$SCRIPT_DIR"
RUNTIME_DIR="$VENDOR_DIR/python-runtime"
PYTHON_DIR="$RUNTIME_DIR/python"
CARGO_CONFIG="$SCRIPT_DIR/../.cargo/config.toml"

# Platform detection - use astral-sh standalone Python builds
case "$(uname -s)" in
    Darwin*)
        PLATFORM="macos"
        case "$(uname -m)" in
            arm64)
                PYTHON_URL="https://github.com/astral-sh/python-build-standalone/releases/download/20250708/cpython-${PYTHON_VERSION}+20250708-aarch64-apple-darwin-install_only.tar.gz"
                PYTHON_ARCHIVE="cpython-${PYTHON_VERSION}+20250708-aarch64-apple-darwin-install_only.tar.gz"
                ;;
            x86_64)
                PYTHON_URL="https://github.com/astral-sh/python-build-standalone/releases/download/20250708/cpython-${PYTHON_VERSION}+20250708-x86_64-apple-darwin-install_only.tar.gz"
                PYTHON_ARCHIVE="cpython-${PYTHON_VERSION}+20250708-x86_64-apple-darwin-install_only.tar.gz"
                ;;
            *)
                echo "❌ Unsupported macOS architecture: $(uname -m)"
                exit 1
                ;;
        esac
        ;;
    Linux*)
        PLATFORM="linux"
        PYTHON_URL="https://github.com/astral-sh/python-build-standalone/releases/download/20250708/cpython-${PYTHON_VERSION}+20250708-x86_64-unknown-linux-gnu-install_only.tar.gz"
        PYTHON_ARCHIVE="cpython-${PYTHON_VERSION}+20250708-x86_64-unknown-linux-gnu-install_only.tar.gz"
        ;;
    CYGWIN*|MINGW32*|MSYS*|MINGW*)
        PLATFORM="windows"
        PYTHON_URL="https://github.com/astral-sh/python-build-standalone/releases/download/20250708/cpython-${PYTHON_VERSION}+20250708-x86_64-pc-windows-msvc-shared-install_only.tar.gz"
        PYTHON_ARCHIVE="cpython-${PYTHON_VERSION}+20250708-x86_64-pc-windows-msvc-shared-install_only.tar.gz"
        ;;
    *)
        echo "❌ Unsupported platform: $(uname -s)"
        exit 1
        ;;
esac

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if Python runtime exists and is working
check_installation() {
    if [ ! -f "$PYTHON_DIR/bin/python3" ] && [ ! -f "$PYTHON_DIR/python.exe" ]; then
        return 1
    fi
    
    local python_cmd
    if [ -f "$PYTHON_DIR/bin/python3" ]; then
        python_cmd="$PYTHON_DIR/bin/python3"
    else
        python_cmd="$PYTHON_DIR/python.exe"
    fi
    
    # Check if USD works
    if ! "$python_cmd" -c "import pxr.Usd" 2>/dev/null; then
        return 1
    fi
    
    return 0
}

# Download file with progress
download_file() {
    local url="$1"
    local output="$2"
    
    log_info "Downloading: $(basename "$url")"
    
    if command -v curl >/dev/null 2>&1; then
        curl -L --progress-bar "$url" -o "$output"
    elif command -v wget >/dev/null 2>&1; then
        wget --progress=bar:force:noscroll "$url" -O "$output"
    else
        log_error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi
}

# Install standalone Python for all platforms
install_standalone_python() {
    local temp_dir="$1"
    local archive_path="$temp_dir/$PYTHON_ARCHIVE"
    
    log_info "Installing standalone Python..."
    
    # Download Python archive
    download_file "$PYTHON_URL" "$archive_path"
    
    # Extract Python
    log_info "Extracting Python..."
    mkdir -p "$RUNTIME_DIR"
    cd "$RUNTIME_DIR"
    tar -xzf "$archive_path"
    
    # The archive extracts to 'python' directory
    if [ ! -d "python" ]; then
        log_error "Python directory not found after extraction"
        return 1
    fi
    
    # Verify Python executable exists
    local python_exe=""
    if [ -f "python/bin/python3" ]; then
        python_exe="python/bin/python3"
    elif [ -f "python/bin/python3.9" ]; then
        python_exe="python/bin/python3.9"
    elif [ -f "python/python.exe" ]; then
        python_exe="python/python.exe"
    else
        log_error "Python executable not found in expected locations"
        return 1
    fi
    
    # Test Python installation
    if ! "$python_exe" --version >/dev/null 2>&1; then
        log_error "Python installation test failed"
        return 1
    fi
    
    log_success "Standalone Python installed successfully"
    return 0
}

# Install Python on Windows
install_python_windows() {
    local temp_dir="$1"
    local archive_path="$temp_dir/$PYTHON_ARCHIVE"
    
    log_info "Installing Python on Windows..."
    
    # Download embedded Python
    download_file "$PYTHON_URL" "$archive_path"
    
    # Extract to python directory
    log_info "Extracting Python..."
    mkdir -p "$PYTHON_DIR"
    cd "$PYTHON_DIR"
    
    if command -v unzip >/dev/null 2>&1; then
        unzip -q "$archive_path"
    else
        log_error "unzip not found. Please install unzip or 7zip."
        return 1
    fi
    
    # Create bin directory and symlinks for consistency
    mkdir -p bin
    if [ -f python.exe ]; then
        cp python.exe bin/python3.exe
        # Create a simple python3 script that calls python3.exe
        cat > bin/python3 << 'EOF'
#!/bin/bash
exec "$(dirname "$0")/python3.exe" "$@"
EOF
        chmod +x bin/python3
    fi
    
    # Enable pip for embedded Python
    if [ -f python39._pth ]; then
        echo "import site" >> python39._pth
    fi
    
    # Install pip manually
    log_info "Installing pip..."
    curl -sSL https://bootstrap.pypa.io/get-pip.py | ./python.exe
    
    log_success "Python installed on Windows"
}

# Install Python packages
install_packages() {
    log_info "Installing Python packages..."
    
    # Determine Python executable
    local python_cmd
    if [ -f "$PYTHON_DIR/bin/python3" ]; then
        python_cmd="$PYTHON_DIR/bin/python3"
    elif [ -f "$PYTHON_DIR/python.exe" ]; then
        python_cmd="$PYTHON_DIR/python.exe"
    else
        log_error "Cannot find Python executable"
        return 1
    fi
    
    # Upgrade pip first - force latest version
    log_info "Upgrading pip to latest version..."
    if ! "$python_cmd" -m pip install --upgrade pip --force-reinstall; then
        log_info "Pip upgrade failed, trying to reinstall pip from bootstrap..."
        # Download and install pip from bootstrap as fallback
        curl -sSL https://bootstrap.pypa.io/get-pip.py | "$python_cmd"
    fi
    
    # Install required packages
    log_info "Installing setuptools..."
    "$python_cmd" -m pip install --upgrade setuptools
    
    log_info "Installing wheel..."
    "$python_cmd" -m pip install --upgrade wheel
    
    log_info "Installing numpy..."
    "$python_cmd" -m pip install numpy
    
    log_info "Installing PySide6 for USD UI tools..."
    "$python_cmd" -m pip install PySide6
    
    log_info "Installing PyOpenGL for USD viewer..."
    "$python_cmd" -m pip install PyOpenGL
    
    # Verify installations
    log_info "Verifying package installations..."
    "$python_cmd" -c "import numpy; print(f'✅ NumPy {numpy.__version__} working')"
    "$python_cmd" -c "import PySide6; print(f'✅ PySide6 {PySide6.__version__} working')"
    
    log_success "All packages installed successfully"
}

# Update Cargo configuration
update_cargo_config() {
    log_info "Updating Cargo configuration..."
    
    # Create .cargo directory if it doesn't exist
    mkdir -p "$(dirname "$CARGO_CONFIG")"
    
    # Get absolute paths for portable configuration
    local project_root
    project_root="$(cd "$SCRIPT_DIR/.." && pwd)"
    
    # Determine Python executable path (absolute)
    local python_exec
    if [ "$PLATFORM" = "windows" ]; then
        python_exec="$project_root/vendor/python-runtime/python/python.exe"
    else
        python_exec="$project_root/vendor/python-runtime/python/bin/python3"
    fi
    
    # Create new cargo config with absolute paths
    cat > "$CARGO_CONFIG" << EOF
# Cargo configuration for embedded Python and USD
[env]
# Point PyO3 to our embedded Python (absolute paths for cross-platform compatibility)
PYO3_PYTHON = "${python_exec}"
PYO3_PYTHON_VERSION = "3.9"
PYTHONPATH = "${project_root}/vendor/python-runtime/python/lib/python3.9/site-packages"

# USD paths
NODLE_USD_ROOT = "${project_root}/vendor/python-runtime/python"
USD_INSTALL_ROOT = "${project_root}/vendor/python-runtime/python"

# Library path for embedded Python
DYLD_LIBRARY_PATH = "${project_root}/vendor/python-runtime/python/lib"
LD_LIBRARY_PATH = "${project_root}/vendor/python-runtime/python/lib"

[build]
# Link against our embedded Python library
rustflags = [
    "-L", "${project_root}/vendor/python-runtime/python/lib",
    "-Wl,-rpath,${project_root}/vendor/python-runtime/python/lib"
]
EOF
    
    log_success "Cargo configuration updated with absolute paths"
}

# Update gitignore
update_gitignore() {
    local gitignore_file="$SCRIPT_DIR/../.gitignore"
    
    log_info "Updating .gitignore..."
    
    # Check if gitignore exists
    if [ ! -f "$gitignore_file" ]; then
        log_info "Creating .gitignore..."
        touch "$gitignore_file"
    fi
    
    # Add python-runtime to gitignore if not already there
    if ! grep -q "vendor/python-runtime" "$gitignore_file"; then
        echo "" >> "$gitignore_file"
        echo "# Python runtime (regenerated by setup script)" >> "$gitignore_file"
        echo "vendor/python-runtime/" >> "$gitignore_file"
        log_success "Added python-runtime to .gitignore"
    else
        log_info "python-runtime already in .gitignore"
    fi
}

# Clean existing installation
clean_installation() {
    log_info "Cleaning existing Python installation..."
    
    if [ -d "$RUNTIME_DIR" ]; then
        rm -rf "$RUNTIME_DIR"
        log_success "Removed existing Python runtime"
    fi
}

# Main function
main() {
    local clean=false
    local check_only=false
    local force=false
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                cat << EOF
Nōdle Python Setup Script

USAGE:
    ./setup_python.sh [OPTIONS]

OPTIONS:
    -h, --help      Show this help message
    -c, --clean     Clean existing installation first
    -f, --force     Force reinstall even if Python exists
    --check         Check current installation status

DESCRIPTION:
    This script sets up Python ${PYTHON_VERSION} with dependencies
    for the Nōdle project using embedded Python for all platforms.
    
    Platform support:
    - macOS: Downloads standalone Python build (ARM64/x86_64)
    - Linux: Downloads standalone Python build (x86_64)
    - Windows: Downloads standalone Python build (x86_64)

EXAMPLES:
    ./setup_python.sh         # Install Python runtime
    ./setup_python.sh --force # Force reinstall
    ./setup_python.sh --clean # Clean install
    ./setup_python.sh --check # Check installation

EOF
                exit 0
                ;;
            -c|--clean)
                clean=true
                shift
                ;;
            -f|--force)
                force=true
                shift
                ;;
            --check)
                check_only=true
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done
    
    # Show header
    echo "🐍 Nōdle Python Setup"
    echo "======================"
    echo "Python Version: $PYTHON_VERSION"
    echo "Platform: $PLATFORM"
    echo "Target Directory: $RUNTIME_DIR"
    echo ""
    
    # Check only mode
    if [ "$check_only" = true ]; then
        if check_installation; then
            log_success "Installation is valid"
            # Show package versions
            local python_cmd
            if [ -f "$PYTHON_DIR/bin/python3" ]; then
                python_cmd="$PYTHON_DIR/bin/python3"
            else
                python_cmd="$PYTHON_DIR/python.exe"
            fi
            echo ""
            echo "📦 Installed packages:"
            "$python_cmd" -m pip list
            exit 0
        else
            log_error "Installation is invalid or missing"
            exit 1
        fi
    fi
    
    # Clean if requested
    if [ "$clean" = true ]; then
        clean_installation
    fi
    
    # Check if we need to install
    if [ "$force" = false ] && check_installation; then
        log_info "Python runtime already installed and working"
        exit 0
    fi
    
    # Create temporary directory
    local temp_dir
    temp_dir=$(mktemp -d)
    log_info "Using temporary directory: $temp_dir"
    
    # Install standalone Python for all platforms
    install_standalone_python "$temp_dir"
    
    # Cleanup temporary directory
    rm -rf "$temp_dir"
    
    # Install packages
    install_packages
    
    # Update configurations
    update_cargo_config
    update_gitignore
    
    # Final verification
    if check_installation; then
        log_success "🎉 Python runtime setup completed successfully!"
        echo ""
        
        # Show final package list
        local python_cmd
        if [ -f "$PYTHON_DIR/bin/python3" ]; then
            python_cmd="$PYTHON_DIR/bin/python3"
        else
            python_cmd="$PYTHON_DIR/python.exe"
        fi
        
        echo "📦 Installed packages:"
        "$python_cmd" -m pip list
        echo ""
        echo "Next steps:"
        echo "1. Run 'cargo build --features usd' to build with USD support"
        echo "2. Run 'cargo run --features usd' to run Nōdle with USD support"
    else
        log_error "Bootstrap completed but verification failed"
        exit 1
    fi
}

# Run main function
main "$@"