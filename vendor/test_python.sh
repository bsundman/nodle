#!/bin/bash

# Nōdle Python Test Script
# Tests if embedded Python is installed and working with all required modules

set -e  # Exit on any error

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VENDOR_DIR="$SCRIPT_DIR"
RUNTIME_DIR="$VENDOR_DIR/python-runtime"
PYTHON_DIR="$RUNTIME_DIR/python"

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

# Test results
TESTS_PASSED=0
TESTS_FAILED=0

# Test function
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -n "Testing $test_name... "
    
    if eval "$test_command" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ PASS${NC}"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}❌ FAIL${NC}"
        ((TESTS_FAILED++))
        return 1
    fi
}

# Test function with output
run_test_with_output() {
    local test_name="$1"
    local test_command="$2"
    
    echo -n "Testing $test_name... "
    
    local output
    if output=$(eval "$test_command" 2>&1); then
        echo -e "${GREEN}✅ PASS${NC}"
        echo "  → $output"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}❌ FAIL${NC}"
        echo "  → $output"
        ((TESTS_FAILED++))
        return 1
    fi
}

# Main test function
main() {
    local verbose=false
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                cat << EOF
Nōdle Python Test Script

USAGE:
    ./test_python.sh [OPTIONS]

OPTIONS:
    -h, --help      Show this help message
    -v, --verbose   Show detailed output for each test

DESCRIPTION:
    This script tests if the embedded Python runtime is properly
    installed and configured with all required modules for Nōdle.

    Tests performed:
    - Python executable exists
    - Python version compatibility
    - Required modules can be imported
    - USD functionality works
    - Cargo configuration is correct

EXAMPLES:
    ./test_python.sh           # Run all tests
    ./test_python.sh -v        # Run tests with verbose output

EOF
                exit 0
                ;;
            -v|--verbose)
                verbose=true
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
    echo "🧪 Nōdle Python Test Suite"
    echo "=========================="
    echo "Target Directory: $RUNTIME_DIR"
    echo ""
    
    # Determine Python executable
    local python_cmd=""
    if [ -f "$PYTHON_DIR/bin/python3" ]; then
        python_cmd="$PYTHON_DIR/bin/python3"
    elif [ -f "$PYTHON_DIR/python.exe" ]; then
        python_cmd="$PYTHON_DIR/python.exe"
    fi
    
    # Test 1: Python executable exists
    log_info "Basic Python Installation Tests"
    run_test "Python executable exists" "[ -n '$python_cmd' ] && [ -f '$python_cmd' ]"
    
    if [ -z "$python_cmd" ] || [ ! -f "$python_cmd" ]; then
        log_error "Python executable not found. Cannot continue with tests."
        echo ""
        log_info "💡 To install Python runtime, run: ./vendor/setup_python.sh"
        exit 1
    fi
    
    # Test 2: Python version
    if [ "$verbose" = true ]; then
        run_test_with_output "Python version" "$python_cmd --version"
    else
        run_test "Python version check" "$python_cmd --version"
    fi
    
    # Test 3: Python can execute basic commands
    run_test "Python basic execution" "$python_cmd -c 'print(\"Hello, Python!\")'"
    
    # Test 4: pip is available
    run_test "pip availability" "$python_cmd -m pip --version"
    
    echo ""
    log_info "Package Import Tests"
    
    # Test 5: Standard library imports
    run_test "Standard library (sys)" "$python_cmd -c 'import sys'"
    run_test "Standard library (os)" "$python_cmd -c 'import os'"
    run_test "Standard library (json)" "$python_cmd -c 'import json'"
    
    # Test 6: Required packages
    run_test "setuptools import" "$python_cmd -c 'import setuptools'"
    run_test "wheel import" "$python_cmd -c 'import wheel'"
    
    # Test 7: NumPy
    if [ "$verbose" = true ]; then
        run_test_with_output "NumPy import and version" "$python_cmd -c 'import numpy; print(f\"NumPy {numpy.__version__}\")'"
    else
        run_test "NumPy import" "$python_cmd -c 'import numpy'"
    fi
    
    # Test 8: NumPy basic functionality
    run_test "NumPy basic operations" "$python_cmd -c 'import numpy as np; arr = np.array([1, 2, 3]); assert arr.sum() == 6'"
    
    # Test 9: USD import
    if [ "$verbose" = true ]; then
        run_test_with_output "USD import and version" "$python_cmd -c 'import pxr.Usd; print(f\"USD {pxr.Usd.GetVersion()}\")'"
    else
        run_test "USD core import" "$python_cmd -c 'import pxr.Usd'"
    fi
    
    # Test 10: USD basic functionality
    run_test "USD basic operations" "$python_cmd -c 'import pxr.Usd; stage = pxr.Usd.Stage.CreateInMemory(); assert stage is not None'"
    
    # Test 11: USD submodules
    run_test "USD Sdf import" "$python_cmd -c 'import pxr.Sdf'"
    run_test "USD Tf import" "$python_cmd -c 'import pxr.Tf'"
    run_test "USD Gf import" "$python_cmd -c 'import pxr.Gf'"
    
    echo ""
    log_info "Configuration Tests"
    
    # Test 12: Cargo configuration exists
    local cargo_config="$SCRIPT_DIR/../.cargo/config.toml"
    run_test "Cargo config exists" "[ -f '$cargo_config' ]"
    
    # Test 13: Cargo configuration has correct Python path
    if [ -f "$cargo_config" ]; then
        run_test "Cargo config Python path" "grep -q 'PYO3_PYTHON.*vendor/python-runtime' '$cargo_config'"
        run_test "Cargo config Python version" "grep -q 'PYO3_PYTHON_VERSION.*3.9' '$cargo_config'"
        run_test "Cargo config PYTHONPATH" "grep -q 'PYTHONPATH.*vendor/python-runtime' '$cargo_config'"
    fi
    
    # Test 14: gitignore configuration
    local gitignore_file="$SCRIPT_DIR/../.gitignore"
    if [ -f "$gitignore_file" ]; then
        run_test "gitignore excludes python-runtime" "grep -q 'vendor/python-runtime' '$gitignore_file'"
    fi
    
    echo ""
    log_info "Package Version Report"
    
    if [ "$verbose" = true ]; then
        echo "📦 All installed packages:"
        "$python_cmd" -m pip list
        echo ""
    fi
    
    # Show specific package versions
    echo "🔍 Key package versions:"
    "$python_cmd" -c "
import sys
print(f'  Python: {sys.version.split()[0]}')

try:
    import pip
    print(f'  pip: {pip.__version__}')
except: pass

try:
    import setuptools
    print(f'  setuptools: {setuptools.__version__}')
except: pass

try:
    import wheel
    print(f'  wheel: {wheel.__version__}')
except: pass

try:
    import numpy
    print(f'  numpy: {numpy.__version__}')
except: pass

try:
    import pxr.Usd
    version = pxr.Usd.GetVersion()
    print(f'  usd-core: {version[0]}.{version[1]}.{version[2]}')
except: pass
"
    
    echo ""
    log_info "Test Results Summary"
    
    local total_tests=$((TESTS_PASSED + TESTS_FAILED))
    echo "📊 Tests run: $total_tests"
    echo "✅ Passed: $TESTS_PASSED"
    echo "❌ Failed: $TESTS_FAILED"
    
    if [ $TESTS_FAILED -eq 0 ]; then
        echo ""
        log_success "🎉 All tests passed! Python runtime is fully functional."
        echo ""
        echo "💡 Ready to build Nōdle:"
        echo "   cargo build --features usd"
        echo "   cargo run --features usd"
        exit 0
    else
        echo ""
        log_error "Some tests failed. Python runtime may not be fully functional."
        echo ""
        echo "💡 To fix issues, try:"
        echo "   ./vendor/setup_python.sh --clean  # Clean reinstall"
        echo "   ./vendor/setup_python.sh --check  # Check installation"
        exit 1
    fi
}

# Run main function
main "$@"