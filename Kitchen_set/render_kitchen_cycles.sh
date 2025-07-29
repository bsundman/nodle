#!/bin/bash

# Script to render Kitchen_set.usd using usdrecord with hdCycles
# This script sets up all necessary environment variables to use the locally built USD and Cycles

set -e

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"

# Set paths to vendor installations
VENDOR_DIR="${PROJECT_ROOT}/vendor"
PYTHON_BIN="${VENDOR_DIR}/python-runtime/python/bin/python3"
USD_DIR="${VENDOR_DIR}/usd"
CYCLES_DIR="${VENDOR_DIR}/cycles"

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

# Check if Cycles exists
if [ ! -d "$CYCLES_DIR" ]; then
    echo "Error: Cycles not found at $CYCLES_DIR"
    echo "Please ensure Cycles is copied to the vendor directory"
    exit 1
fi

# Set environment variables for USD
export PYTHONPATH="${USD_DIR}/lib/python:${VENDOR_DIR}/python-runtime/python/lib/python3.9/site-packages:${PYTHONPATH}"
export PATH="${USD_DIR}/bin:${PATH}"
export USD_INSTALL_ROOT="${USD_DIR}"

# hdCycles plugin is now installed in USD plugin directory
# No additional plugin path needed

# For macOS, set dynamic library paths - use our custom modern TBB for compatibility
CUSTOM_TBB_LIB="${VENDOR_DIR}/tbb/tbb_install/lib"
ESSENTIAL_CYCLES_LIBS="${CYCLES_DIR}/lib/macos_arm64/embree/lib:${CYCLES_DIR}/lib/macos_arm64/osl/lib:${CYCLES_DIR}/lib/macos_arm64/opensubdiv/lib:${CYCLES_DIR}/lib/macos_arm64/opencolorio/lib:${CYCLES_DIR}/lib/macos_arm64/openimageio/lib:${CYCLES_DIR}/lib/macos_arm64/openexr/lib:${CYCLES_DIR}/lib/macos_arm64/imath/lib:${CYCLES_DIR}/lib/macos_arm64/openimagedenoise/lib"
export DYLD_LIBRARY_PATH="${CUSTOM_TBB_LIB}:${ESSENTIAL_CYCLES_LIBS}:${USD_DIR}/lib:${CYCLES_DIR}/install/lib:${VENDOR_DIR}/python-runtime/python/lib:${DYLD_LIBRARY_PATH}"
export DYLD_FALLBACK_LIBRARY_PATH="${CUSTOM_TBB_LIB}:${ESSENTIAL_CYCLES_LIBS}:${USD_DIR}/lib:${CYCLES_DIR}/install/lib:${VENDOR_DIR}/python-runtime/python/lib:${DYLD_FALLBACK_LIBRARY_PATH}"

# For Linux, set library paths including all Cycles dependencies  
CYCLES_LIB_DIRS_LINUX="${CYCLES_DIR}/lib/linux_x86_64/osl/lib:${CYCLES_DIR}/lib/linux_x86_64/embree/lib:${CYCLES_DIR}/lib/linux_x86_64/openimagedenoise/lib:${CYCLES_DIR}/lib/linux_x86_64/openimageio/lib:${CYCLES_DIR}/lib/linux_x86_64/openexr/lib:${CYCLES_DIR}/lib/linux_x86_64/openvdb/lib:${CYCLES_DIR}/lib/linux_x86_64/materialx/lib:${CYCLES_DIR}/lib/linux_x86_64/opencolorio/lib:${CYCLES_DIR}/lib/linux_x86_64/tbb/lib:${CYCLES_DIR}/lib/linux_x86_64/openmp/lib:${CYCLES_DIR}/lib/linux_x86_64/imath/lib:${CYCLES_DIR}/lib/linux_x86_64/opensubdiv/lib"
export LD_LIBRARY_PATH="${USD_DIR}/lib:${CYCLES_LIB_DIRS_LINUX}:${VENDOR_DIR}/python-runtime/python/lib:${LD_LIBRARY_PATH}"

# USD file to render - use the Cycles-optimized version
USD_FILE="${SCRIPT_DIR}/Kitchen_set_cycles.usd"

# Output image file
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
OUTPUT_IMAGE="${SCRIPT_DIR}/render_kitchen_cycles_${TIMESTAMP}.png"

# Check if USD file exists
if [ ! -f "$USD_FILE" ]; then
    echo "Error: USD file not found at $USD_FILE"
    exit 1
fi

# Default render settings
WIDTH=${WIDTH:-1920}
HEIGHT=${HEIGHT:-1080}
SAMPLES=${SAMPLES:-64}

echo "=========================================="
echo "Rendering with hdCycles:"
echo "  Python: ${PYTHON_BIN}"
echo "  USD: ${USD_DIR}"
echo "  Cycles: ${CYCLES_DIR}"
echo "  Plugin Path: ${PXR_PLUGINPATH}"
echo "  Input: ${USD_FILE}"
echo "  Output: ${OUTPUT_IMAGE}"
echo "  Resolution: ${WIDTH}x${HEIGHT}"
echo "  Samples: ${SAMPLES}"
echo "=========================================="

# Test if hdCycles plugin is available
echo "Checking available renderers..."
AVAILABLE_RENDERERS=$("${USD_DIR}/bin/usdrecord" --help 2>&1 | grep -A1 "renderer" || echo "")
if [[ "$AVAILABLE_RENDERERS" == *"Cycles"* ]]; then
    echo "✅ Cycles renderer found"
else
    echo "❌ Cycles renderer not found"
    echo "Available renderers:"
    echo "$AVAILABLE_RENDERERS"
    echo ""
    echo "Troubleshooting:"
    echo "1. Verify Cycles installation at: ${CYCLES_DIR}/install/"
    echo "2. Plugin is in USD directory: ${USD_DIR}/plugin/usd/"
    echo "3. Verify hdCycles.dylib exists at: ${USD_DIR}/plugin/usd/hdCycles.dylib"
    exit 1
fi

# Create a temporary USD file with render settings for Cycles
TEMP_USD="${SCRIPT_DIR}/.render_settings_${TIMESTAMP}.usda"
cat > "${TEMP_USD}" << EOF
#usda 1.0
(
    defaultPrim = "Scene"
    upAxis = "Y"
)

def Scope "Scene" (
    append references = @./Kitchen_set_cycles.usd@
)
{
    def RenderSettings "renderSettings"
    {
        uniform token aspectRatioConformPolicy = "expandAperture"
        uniform int2 resolution = (${WIDTH}, ${HEIGHT})
    }
}
EOF

# Run usdrecord with hdCycles
echo "Starting render..."
echo "Command: ${USD_DIR}/bin/usdrecord --renderer Cycles --imageWidth ${WIDTH} ${TEMP_USD} ${OUTPUT_IMAGE}"

# Set all required environment variables for the render command
export PYTHONPATH="${USD_DIR}/lib/python:${VENDOR_DIR}/python-runtime/python/lib/python3.9/site-packages:${PYTHONPATH}"
export PATH="${USD_DIR}/bin:${PATH}"
export USD_INSTALL_ROOT="${USD_DIR}"
export DYLD_LIBRARY_PATH="${CUSTOM_TBB_LIB}:${ESSENTIAL_CYCLES_LIBS}:${USD_DIR}/lib:${CYCLES_DIR}/install/lib:${VENDOR_DIR}/python-runtime/python/lib:${DYLD_LIBRARY_PATH}"
export DYLD_FALLBACK_LIBRARY_PATH="${CUSTOM_TBB_LIB}:${ESSENTIAL_CYCLES_LIBS}:${USD_DIR}/lib:${CYCLES_DIR}/install/lib:${VENDOR_DIR}/python-runtime/python/lib:${DYLD_FALLBACK_LIBRARY_PATH}"

"${USD_DIR}/bin/usdrecord" \
    --renderer Cycles \
    --imageWidth ${WIDTH} \
    --camera /Scene/Camera \
    "${TEMP_USD}" \
    "${OUTPUT_IMAGE}"

# Clean up temporary file
rm -f "${TEMP_USD}"

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