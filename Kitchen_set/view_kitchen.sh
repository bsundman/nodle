#!/bin/bash

# Script to launch usdview for Kitchen_set.usd using vendor installations
# This script sets up all necessary environment variables to use the locally built USD

set -e

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"

# Set paths to vendor installations
VENDOR_DIR="${PROJECT_ROOT}/vendor"
PYTHON_BIN="${VENDOR_DIR}/python-runtime/python/bin/python3"
USD_DIR="${VENDOR_DIR}/usd"

# Check if Python exists
if [ ! -f "$PYTHON_BIN" ]; then
    echo "Error: Python not found at $PYTHON_BIN"
    echo "Please run: cd ../vendor && ./setup_python.sh"
    exit 1
fi

# Check if USD is built
if [ ! -d "$USD_DIR" ]; then
    echo "Error: USD not found at $USD_DIR"
    echo "Please run: cd ../vendor && ./build_usd.sh"
    exit 1
fi

# Set environment variables for USD
export PYTHONPATH="${USD_DIR}/lib/python:${VENDOR_DIR}/python-runtime/python/lib/python3.9/site-packages:${PYTHONPATH}"
export PATH="${USD_DIR}/bin:${PATH}"
export USD_INSTALL_ROOT="${USD_DIR}"

# For macOS, set dynamic library paths
export DYLD_LIBRARY_PATH="${USD_DIR}/lib:${VENDOR_DIR}/python-runtime/python/lib:${DYLD_LIBRARY_PATH}"
export DYLD_FALLBACK_LIBRARY_PATH="${USD_DIR}/lib:${VENDOR_DIR}/python-runtime/python/lib:${DYLD_FALLBACK_LIBRARY_PATH}"

# For Linux, set library paths
export LD_LIBRARY_PATH="${USD_DIR}/lib:${VENDOR_DIR}/python-runtime/python/lib:${LD_LIBRARY_PATH}"

# USD file to open
USD_FILE="${SCRIPT_DIR}/Kitchen_set.usd"

# Check if USD file exists
if [ ! -f "$USD_FILE" ]; then
    echo "Error: USD file not found at $USD_FILE"
    exit 1
fi

echo "=========================================="
echo "Launching usdview with:"
echo "  Python: ${PYTHON_BIN}"
echo "  USD: ${USD_DIR}"
echo "  File: ${USD_FILE}"
echo "=========================================="

# Launch usdview
"${USD_DIR}/bin/usdview" "${USD_FILE}"