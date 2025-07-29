#!/bin/bash

# Script to render Kitchen_set.usd using usdrecord with Storm renderer
# Storm is USD's high-quality GPU renderer that comes built-in with USD

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

# USD file to render
USD_FILE="${SCRIPT_DIR}/Kitchen_set.usd"

# Output image file
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
OUTPUT_IMAGE="${SCRIPT_DIR}/render_kitchen_storm_${TIMESTAMP}.png"

# Check if USD file exists
if [ ! -f "$USD_FILE" ]; then
    echo "Error: USD file not found at $USD_FILE"
    exit 1
fi

# Default render settings
WIDTH=${WIDTH:-1920}
COMPLEXITY=${COMPLEXITY:-high}

echo "=========================================="
echo "Rendering with Storm renderer:"
echo "  Python: ${PYTHON_BIN}"
echo "  USD: ${USD_DIR}"
echo "  Input: ${USD_FILE}"
echo "  Output: ${OUTPUT_IMAGE}"
echo "  Image Width: ${WIDTH} (height auto-calculated from camera aspect ratio)"
echo "  Complexity: ${COMPLEXITY}"
echo "=========================================="

# Check available renderers
echo "Available renderers:"
"${USD_DIR}/bin/usdrecord" --renderer Storm --help 2>/dev/null | grep -A5 "renderer" || echo "Storm renderer available"

echo ""
echo "Starting render..."

# Run usdrecord with Storm renderer
"${USD_DIR}/bin/usdrecord" \
    --renderer Storm \
    --imageWidth ${WIDTH} \
    --complexity ${COMPLEXITY} \
    --colorCorrectionMode sRGB \
    "${USD_FILE}" \
    "${OUTPUT_IMAGE}"

# Check if render was successful
if [ -f "${OUTPUT_IMAGE}" ]; then
    echo "=========================================="
    echo "Render complete!"
    echo "Output saved to: ${OUTPUT_IMAGE}"
    echo "=========================================="
    
    # Try to open the image (macOS)
    if command -v open >/dev/null 2>&1; then
        open "${OUTPUT_IMAGE}"
    fi
else
    echo "Error: Render failed - no output image generated"
    exit 1
fi