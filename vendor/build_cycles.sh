#!/bin/bash

# Build script for Cycles using embedded Python runtime and USD
# This script builds Cycles with hdCycles plugin compatible with our vendor USD

set -e  # Exit on error

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Set paths
CYCLES_DIR="${SCRIPT_DIR}/cycles"
BUILD_DIR="${CYCLES_DIR}/build"
INSTALL_DIR="${CYCLES_DIR}/install"
PYTHON_BIN="${SCRIPT_DIR}/python-runtime/python/bin/python3"
USD_DIR="${SCRIPT_DIR}/usd"

# Check if Python exists
if [ ! -f "${PYTHON_BIN}" ]; then
    echo "Error: Python not found at ${PYTHON_BIN}"
    echo "Please ensure the python-runtime is properly installed."
    exit 1
fi

# Check if USD exists
if [ ! -d "${USD_DIR}" ]; then
    echo "Error: USD not found at ${USD_DIR}"
    echo "Please ensure USD is built in the vendor directory."
    exit 1
fi

# Check if Cycles source exists
if [ ! -d "${CYCLES_DIR}" ]; then
    echo "Error: Cycles source not found at ${CYCLES_DIR}"
    echo "Please ensure Cycles is copied to the vendor directory."
    exit 1
fi

echo "Using Python: ${PYTHON_BIN}"
echo "Python version:"
"${PYTHON_BIN}" --version

echo "Using USD: ${USD_DIR}"
echo "Building Cycles with hdCycles plugin..."
echo "Build output will be in: ${INSTALL_DIR}"

# Set environment variables for the build
export PATH="${USD_DIR}/bin:${PATH}"
export PYTHONPATH="${USD_DIR}/lib/python:${SCRIPT_DIR}/python-runtime/python/lib/python3.9/site-packages:${PYTHONPATH}"
export USD_INSTALL_ROOT="${USD_DIR}"

# Set custom TBB paths
export TBB_ROOT="${SCRIPT_DIR}/tbb/tbb_install"
export CMAKE_PREFIX_PATH="${TBB_ROOT}:${USD_DIR}:${CMAKE_PREFIX_PATH}"

# For macOS, set dynamic library paths for build process (include custom TBB)
export DYLD_LIBRARY_PATH="${TBB_ROOT}/lib:${USD_DIR}/lib:${SCRIPT_DIR}/python-runtime/python/lib:${DYLD_LIBRARY_PATH}"
export DYLD_FALLBACK_LIBRARY_PATH="${TBB_ROOT}/lib:${USD_DIR}/lib:${SCRIPT_DIR}/python-runtime/python/lib:${DYLD_FALLBACK_LIBRARY_PATH}"

# For Linux, set library paths (include custom TBB)
export LD_LIBRARY_PATH="${TBB_ROOT}/lib:${USD_DIR}/lib:${SCRIPT_DIR}/python-runtime/python/lib:${LD_LIBRARY_PATH}"

# Navigate to Cycles directory
cd "${CYCLES_DIR}"

# Clean previous build if it exists
if [ -d "${BUILD_DIR}" ]; then
    echo "Cleaning previous build..."
    rm -rf "${BUILD_DIR}"
fi

if [ -d "${INSTALL_DIR}" ]; then
    echo "Cleaning previous install..."
    rm -rf "${INSTALL_DIR}"
fi

# Download precompiled libraries (legacy version for USD compatibility)
echo "Downloading precompiled libraries..."
if [ ! -f "GNUmakefile" ]; then
    echo "Error: GNUmakefile not found. Are you in the Cycles source directory?"
    exit 1
fi

# Build all libraries from source instead of using precompiled libraries
echo "Building all dependencies from source with our TBB 2020.3..."

# Configure build with CMake
echo "Configuring build with CMake..."
mkdir -p "${BUILD_DIR}"

# Platform-specific configuration
case "$(uname -s)" in
    Darwin*)
        PLATFORM_FLAGS=""
        if [ "$(uname -m)" = "arm64" ]; then
            PLATFORM_FLAGS="-DCMAKE_OSX_ARCHITECTURES=arm64"
        fi
        ;;
    Linux*)
        PLATFORM_FLAGS=""
        ;;
    *)
        echo "Warning: Unsupported platform $(uname -s)"
        PLATFORM_FLAGS=""
        ;;
esac

# Check if custom TBB exists
if [ ! -d "${TBB_ROOT}" ]; then
    echo "Error: Custom TBB not found at ${TBB_ROOT}"
    echo "Please run ./build_tbb.sh first to build TBB from source"
    exit 1
fi

echo "Using custom TBB from: ${TBB_ROOT}"

# Configure with CMake - build all dependencies from source
cmake -B "${BUILD_DIR}" \
    -DPXR_ROOT="${USD_DIR}" \
    -DWITH_CYCLES_HYDRA_RENDER_DELEGATE=ON \
    -DWITH_PYTHON_INSTALL=OFF \
    -DPYTHON_EXECUTABLE="${PYTHON_BIN}" \
    -DCMAKE_INSTALL_PREFIX="${INSTALL_DIR}" \
    -DCMAKE_BUILD_TYPE=Release \
    -DTBB_ROOT="${TBB_ROOT}" \
    -DTBB_INCLUDE_DIR="${TBB_ROOT}/include" \
    -DTBB_LIBRARY="${TBB_ROOT}/lib/libtbb.dylib" \
    -DTBB_LIBRARY_DEBUG="${TBB_ROOT}/lib/libtbb.dylib" \
    -DTBB_MALLOC_LIBRARY="${TBB_ROOT}/lib/libtbbmalloc.dylib" \
    -DTBB_MALLOC_PROXY_LIBRARY="${TBB_ROOT}/lib/libtbbmalloc_proxy.dylib" \
    -DMATERIALX_ROOT="${USD_DIR}" \
    -DMATERIALX_INCLUDE_DIR="${USD_DIR}/include" \
    -DMATERIALX_LIBRARY="${USD_DIR}/lib/libMaterialXCore.dylib" \
    -DWITH_CYCLES_EMBREE=OFF \
    -DWITH_CYCLES_OPENVDB=ON \
    -DWITH_CYCLES_OSL=ON \
    -DWITH_CYCLES_OPENIMAGEIO=ON \
    -DWITH_CYCLES_OPENEXR=ON \
    -DWITH_CYCLES_OPENCOLORIO=ON \
    -DWITH_OPENIMAGEDENOISE=ON \
    -DWITH_PRECOMPILED_LIBRARIES=OFF \
    -DCMAKE_PREFIX_PATH="${TBB_ROOT};${USD_DIR}" \
    ${PLATFORM_FLAGS}

# Build Cycles
echo "Building Cycles (this may take 20-40 minutes)..."
make -C "${BUILD_DIR}" -j$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)

# Install Cycles
echo "Installing Cycles..."
make -C "${BUILD_DIR}" install

echo "Cycles build complete!"
echo "Installation path: ${INSTALL_DIR}"

# Verify hdCycles plugin was built
HDCYCLES_PLUGIN="${INSTALL_DIR}/hydra/hdCycles.dylib"
if [ -f "${HDCYCLES_PLUGIN}" ] || [ -f "${INSTALL_DIR}/hydra/hdCycles.so" ]; then
    echo "✅ hdCycles plugin built successfully"
    
    # Show plugin info
    if [ -f "${HDCYCLES_PLUGIN}" ]; then
        echo "Plugin location: ${HDCYCLES_PLUGIN}"
        echo "Plugin size: $(ls -lh "${HDCYCLES_PLUGIN}" | awk '{print $5}')"
    else
        HDCYCLES_PLUGIN="${INSTALL_DIR}/hydra/hdCycles.so"
        echo "Plugin location: ${HDCYCLES_PLUGIN}"
        echo "Plugin size: $(ls -lh "${HDCYCLES_PLUGIN}" | awk '{print $5}')"
    fi
else
    echo "⚠️  hdCycles plugin not found - check build logs"
fi

echo ""
echo "To use hdCycles with USD:"
echo "export PXR_PLUGINPATH=${INSTALL_DIR}/hydra:\$PXR_PLUGINPATH"
echo "export DYLD_LIBRARY_PATH=${INSTALL_DIR}/lib:\$DYLD_LIBRARY_PATH"