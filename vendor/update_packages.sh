#!/bin/bash

# Nōdle Python Package Update Script
# This script updates all Python packages to their latest versions

set -e  # Exit on any error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PYTHON_BIN="$SCRIPT_DIR/python-runtime/python/bin/python3"

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

# Check if Python exists
if [ ! -f "$PYTHON_BIN" ]; then
    log_error "Python not found at $PYTHON_BIN"
    log_info "Please run ./setup_python.sh first"
    exit 1
fi

echo "🔄 Nōdle Python Package Updater"
echo "================================"
echo "Python: $PYTHON_BIN"
echo ""

# Show current package versions
log_info "Current package versions:"
"$PYTHON_BIN" -m pip list

echo ""
log_info "Updating packages to latest versions..."

# Update pip first
log_info "Updating pip..."
"$PYTHON_BIN" -m pip install --upgrade pip

# Update all packages
log_info "Updating setuptools..."
"$PYTHON_BIN" -m pip install --upgrade setuptools

log_info "Updating wheel..."
"$PYTHON_BIN" -m pip install --upgrade wheel

log_info "Updating numpy..."
"$PYTHON_BIN" -m pip install --upgrade numpy

log_info "Updating usd-core..."
"$PYTHON_BIN" -m pip install --upgrade usd-core

echo ""
log_info "Final package versions:"
"$PYTHON_BIN" -m pip list

echo ""
log_info "Testing installations..."

# Test numpy
"$PYTHON_BIN" -c "import numpy; print(f'✅ NumPy {numpy.__version__} working')"

# Test USD
"$PYTHON_BIN" -c "import pxr.Usd; print(f'✅ USD {pxr.Usd.GetVersion()} working')"

echo ""
log_success "🎉 All packages updated successfully!"
echo ""
echo "💡 Next steps:"
echo "1. Update version numbers in DEPENDENCIES.md"
echo "2. Test the application with: cargo build --features usd"
echo "3. Commit the updated DEPENDENCIES.md file"