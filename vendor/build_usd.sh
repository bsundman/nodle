#!/bin/bash

# Build script for Pixar's USD using embedded Python runtime
# This script clones USD and builds it using the Python from vendor/python-runtime

set -e  # Exit on error

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Set paths
USD_DIR="${SCRIPT_DIR}/USD"
BUILD_DIR="${SCRIPT_DIR}/usd"
PYTHON_BIN="${SCRIPT_DIR}/python-runtime/python/bin/python3"

# Check if Python exists
if [ ! -f "${PYTHON_BIN}" ]; then
    echo "Error: Python not found at ${PYTHON_BIN}"
    echo "Please ensure the python-runtime is properly installed."
    exit 1
fi

echo "Using Python: ${PYTHON_BIN}"
echo "Python version:"
"${PYTHON_BIN}" --version

# Clone USD if it doesn't exist
if [ ! -d "${USD_DIR}" ]; then
    echo "Cloning USD from GitHub..."
    git clone https://github.com/PixarAnimationStudios/USD.git "${USD_DIR}"
else
    echo "USD repository already exists at ${USD_DIR}"
    echo "Using existing checkout"
fi

# Create build directory if it doesn't exist
if [ ! -d "${BUILD_DIR}" ]; then
    mkdir -p "${BUILD_DIR}"
fi

# Add Python bin directory to PATH for PySide6 tools
export PATH="${SCRIPT_DIR}/python-runtime/python/bin:${PATH}"

# Navigate to USD directory
cd "${USD_DIR}"

# Build USD
echo "Building USD with monolithic libraries..."
echo "Build output will be in: ${BUILD_DIR}"

# Check for build options
BUILD_ARGS="--build-monolithic"

# Check if PySide is available
if ! "${PYTHON_BIN}" -c "import PySide2" 2>/dev/null && ! "${PYTHON_BIN}" -c "import PySide6" 2>/dev/null; then
    echo "Warning: PySide not found. Building without UI tools..."
    echo "To build with UI tools, run: ./update_packages.sh"
    BUILD_ARGS="${BUILD_ARGS} --no-usdview"
else
    echo "PySide found - building with usdview support"
fi

# Run the build script
"${PYTHON_BIN}" build_scripts/build_usd.py ${BUILD_ARGS} "${BUILD_DIR}"

echo "USD build complete!"
echo "USD installation path: ${BUILD_DIR}"

# Set up environment variables hint
echo ""
echo "To use this USD installation, you may need to set:"
echo "export PYTHONPATH=${BUILD_DIR}/lib/python:\$PYTHONPATH"
echo "export PATH=${BUILD_DIR}/bin:\$PATH"