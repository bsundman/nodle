#!/bin/bash

# Build script for Intel TBB (oneTBB) v2021.12 from source
# This creates a modern TBB build compatible with Cycles precompiled libraries

set -e  # Exit on error

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Set paths
TBB_SOURCE_DIR="${SCRIPT_DIR}/tbb"
TBB_BUILD_DIR="${TBB_SOURCE_DIR}/build"
TBB_INSTALL_DIR="${TBB_SOURCE_DIR}/tbb_install"

# Check if TBB source exists
if [ ! -d "${TBB_SOURCE_DIR}" ]; then
    echo "Error: TBB source not found at ${TBB_SOURCE_DIR}"
    echo "Please ensure TBB has been cloned to the vendor directory."
    exit 1
fi

echo "Building Intel TBB from source..."
echo "Source: ${TBB_SOURCE_DIR}"
echo "Install: ${TBB_INSTALL_DIR}"

# Navigate to TBB source directory
cd "${TBB_SOURCE_DIR}"

# Check if this is the newer CMake-based build or older make-based build first
if [ -f "CMakeLists.txt" ]; then
    # CMake build - safe to clean build directory
    if [ -d "${TBB_BUILD_DIR}" ]; then
        echo "Cleaning previous CMake build..."
        rm -rf "${TBB_BUILD_DIR}"
    fi
    # Create build directory
    mkdir -p "${TBB_BUILD_DIR}"
else
    # Legacy make build - don't clean build directory as it contains source files
    echo "Legacy build detected - preserving build directory..."
fi

# Always clean install directory
if [ -d "${TBB_INSTALL_DIR}" ]; then
    echo "Cleaning previous install..."
    rm -rf "${TBB_INSTALL_DIR}"
fi

# Check if this is the newer CMake-based build or older make-based build
if [ -f "CMakeLists.txt" ]; then
    echo "Using CMake build system..."
    
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

    # Configure with CMake
    echo "Configuring TBB with CMake..."
    cmake -B "${TBB_BUILD_DIR}" \
        -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_INSTALL_PREFIX="${TBB_INSTALL_DIR}" \
        -DTBB_TEST=OFF \
        -DTBB_EXAMPLES=OFF \
        -DTBB_STRICT=OFF \
        ${PLATFORM_FLAGS} \
        .

    # Build TBB
    echo "Building TBB (this should be quick)..."
    make -C "${TBB_BUILD_DIR}" -j$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)

    # Install TBB
    echo "Installing TBB..."
    make -C "${TBB_BUILD_DIR}" install
else
    echo "Using legacy make build system..."
    
    # Platform-specific configuration for legacy build
    case "$(uname -s)" in
        Darwin*)
            if [ "$(uname -m)" = "arm64" ]; then
                ARCH_FLAG="arch=arm64"
            else
                ARCH_FLAG="arch=intel64"
            fi
            ;;
        Linux*)
            ARCH_FLAG="arch=intel64"
            ;;
        *)
            echo "Warning: Unsupported platform $(uname -s)"
            ARCH_FLAG=""
            ;;
    esac

    # Build TBB using legacy Makefile
    echo "Building TBB with legacy make system..."
    make ${ARCH_FLAG} -j$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
    
    # Install manually for legacy build
    echo "Installing TBB (manual installation)..."
    mkdir -p "${TBB_INSTALL_DIR}/lib"
    mkdir -p "${TBB_INSTALL_DIR}/include"
    
    # Copy libraries
    if [ -d "${TBB_BUILD_DIR}" ]; then
        find "${TBB_BUILD_DIR}" -name "*.dylib" -o -name "*.so" | while read lib; do
            cp "$lib" "${TBB_INSTALL_DIR}/lib/"
        done
    fi
    
    # Copy headers
    cp -r include/tbb "${TBB_INSTALL_DIR}/include/"
    
    # Create cmake config files
    mkdir -p "${TBB_INSTALL_DIR}/lib/cmake/TBB"
    cat > "${TBB_INSTALL_DIR}/lib/cmake/TBB/TBBConfig.cmake" << EOF
# TBB Config file
set(TBB_FOUND TRUE)
set(TBB_INCLUDE_DIRS "${TBB_INSTALL_DIR}/include")
set(TBB_LIBRARIES "${TBB_INSTALL_DIR}/lib/libtbb.dylib;${TBB_INSTALL_DIR}/lib/libtbbmalloc.dylib")
set(TBB_VERSION "2020.3")

# Create imported targets that USD expects
if(NOT TARGET TBB::tbb)
    add_library(TBB::tbb SHARED IMPORTED)
    set_target_properties(TBB::tbb PROPERTIES
        IMPORTED_LOCATION "${TBB_INSTALL_DIR}/lib/libtbb.dylib"
        INTERFACE_INCLUDE_DIRECTORIES "${TBB_INSTALL_DIR}/include"
    )
endif()

if(NOT TARGET TBB::tbbmalloc)
    add_library(TBB::tbbmalloc SHARED IMPORTED)
    set_target_properties(TBB::tbbmalloc PROPERTIES
        IMPORTED_LOCATION "${TBB_INSTALL_DIR}/lib/libtbbmalloc.dylib"
        INTERFACE_INCLUDE_DIRECTORIES "${TBB_INSTALL_DIR}/include"
    )
endif()

if(NOT TARGET TBB::tbbmalloc_proxy)
    add_library(TBB::tbbmalloc_proxy SHARED IMPORTED)
    set_target_properties(TBB::tbbmalloc_proxy PROPERTIES
        IMPORTED_LOCATION "${TBB_INSTALL_DIR}/lib/libtbbmalloc_proxy.dylib"
        INTERFACE_INCLUDE_DIRECTORIES "${TBB_INSTALL_DIR}/include"
    )
endif()
EOF
fi

echo "TBB build complete!"
echo "Installation path: ${TBB_INSTALL_DIR}"

# Verify installation
TBB_LIB_PATH="${TBB_INSTALL_DIR}/lib"
if [ -d "${TBB_LIB_PATH}" ]; then
    echo "✅ TBB libraries installed:"
    ls -la "${TBB_LIB_PATH}"/libtbb*
    
    # Show version info if available
    if [ -f "${TBB_LIB_PATH}/libtbb.dylib" ]; then
        echo "TBB library info:"
        otool -L "${TBB_LIB_PATH}/libtbb.dylib" | head -3
    elif [ -f "${TBB_LIB_PATH}/libtbb.so" ]; then
        echo "TBB library info:"
        ldd "${TBB_LIB_PATH}/libtbb.so" | head -3
    fi
else
    echo "⚠️  TBB libraries not found in expected location"
fi

echo ""
echo "To use this TBB build:"
echo "export TBB_ROOT=${TBB_INSTALL_DIR}"
echo "export CMAKE_PREFIX_PATH=${TBB_INSTALL_DIR}:\$CMAKE_PREFIX_PATH"