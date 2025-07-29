#!/bin/bash

# Script to debug renderer availability

set -e

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"

# Set paths to vendor installations
VENDOR_DIR="${PROJECT_ROOT}/vendor"
USD_DIR="${VENDOR_DIR}/usd"
CYCLES_DIR="${VENDOR_DIR}/cycles"

# Set environment variables for USD
export PYTHONPATH="${USD_DIR}/lib/python:${VENDOR_DIR}/python-runtime/python/lib/python3.9/site-packages"
export PATH="${USD_DIR}/bin:${PATH}"
export USD_INSTALL_ROOT="${USD_DIR}"

# For macOS, set dynamic library paths
export DYLD_LIBRARY_PATH="${USD_DIR}/lib:${VENDOR_DIR}/python-runtime/python/lib"
export DYLD_FALLBACK_LIBRARY_PATH="${USD_DIR}/lib:${VENDOR_DIR}/python-runtime/python/lib"

echo "Testing USD installation without Cycles plugin:"
echo "=============================================="
echo "Available renderers with base USD:"
"${USD_DIR}/bin/usdrecord" --list-renderers || echo "Command failed"

echo ""
echo "Testing with Cycles plugin path:"
echo "================================"
export PXR_PLUGINPATH="${CYCLES_DIR}/install/hydra:${CYCLES_DIR}/install/usd"
export DYLD_LIBRARY_PATH="${USD_DIR}/lib:${CYCLES_DIR}/install/lib:${VENDOR_DIR}/python-runtime/python/lib"

echo "PXR_PLUGINPATH: $PXR_PLUGINPATH"
echo "Available renderers with Cycles plugin:"
"${USD_DIR}/bin/usdrecord" --list-renderers || echo "Command failed"

echo ""
echo "Plugin debugging:"
echo "================="
echo "Checking if hdCycles.dylib exists:"
ls -la "${CYCLES_DIR}/install/hydra/hdCycles.dylib" || echo "File not found"

echo ""
echo "Checking plugin dependencies:"
otool -L "${CYCLES_DIR}/install/hydra/hdCycles.dylib" | head -10