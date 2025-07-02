#!/bin/bash
# Nodle Installation Script
# Sets up all dependencies including local USD

set -e  # Exit on error

echo "╭─────────────────────────────────────────────────────────╮"
echo "│               🎨 Nodle Installation                      │"
echo "│         Node-based 3D Visual Programming                 │"
echo "╰─────────────────────────────────────────────────────────╯"
echo

# Check for Python
echo "🔍 Checking for Python..."
if command -v python3 &> /dev/null; then
    PYTHON_CMD="python3"
elif command -v python &> /dev/null; then
    PYTHON_CMD="python"
else
    echo "❌ Python not found. Please install Python 3.8 or higher."
    echo "   macOS: brew install python3"
    echo "   Ubuntu: sudo apt install python3 python3-pip"
    exit 1
fi

PYTHON_VERSION=$($PYTHON_CMD --version 2>&1 | awk '{print $2}')
echo "✓ Found Python $PYTHON_VERSION"

# Check Python version
PYTHON_MAJOR=$($PYTHON_CMD -c 'import sys; print(sys.version_info.major)')
PYTHON_MINOR=$($PYTHON_CMD -c 'import sys; print(sys.version_info.minor)')

if [ "$PYTHON_MAJOR" -lt 3 ] || ([ "$PYTHON_MAJOR" -eq 3 ] && [ "$PYTHON_MINOR" -lt 8 ]); then
    echo "❌ Python 3.8 or higher is required. Found $PYTHON_VERSION"
    exit 1
fi

# Setup USD
echo
echo "📦 Setting up USD (Universal Scene Description)..."
echo "   This may take 2-5 minutes on first install..."
$PYTHON_CMD scripts/setup_usd.py

# Check if cargo is installed
echo
echo "🔍 Checking for Rust..."
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust not found. Please install from https://rustup.rs/"
    exit 1
fi
echo "✓ Found Rust $(cargo --version)"

# Build Nodle
echo
echo "🔨 Building Nodle..."
cargo build --release --features usd

echo
echo "╭─────────────────────────────────────────────────────────╮"
echo "│               ✅ Installation Complete!                  │"
echo "├─────────────────────────────────────────────────────────┤"
echo "│ Run Nodle with:                                         │"
echo "│   cargo run --release --features usd                    │"
echo "│                                                         │"
echo "│ Or run the built executable directly:                   │"
echo "│   ./target/release/nodle                                │"
echo "╰─────────────────────────────────────────────────────────╯"
echo