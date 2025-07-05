#!/bin/bash
echo "🧪 Testing USD Plugin Separation..."
echo

# Test 1: Check USD plugin exists
PLUGIN_PATH="/Users/brian/nodle-claude/nodle/plugins/libnodle_usd_plugin_comprehensive.dylib"
if [ -f "$PLUGIN_PATH" ]; then
    echo "✅ USD Plugin file found"
    echo "   Size: $(ls -lh "$PLUGIN_PATH" | awk '{print $5}')"
else
    echo "❌ USD Plugin file not found"
fi

# Test 2: Check USD directory removed from core
USD_CORE_PATH="/Users/brian/nodle-claude/nodle/src/nodes/three_d/usd"
if [ ! -d "$USD_CORE_PATH" ]; then
    echo "✅ USD directory removed from core"
else
    echo "❌ USD directory still exists in core"
    echo "   Contents: $(ls -la "$USD_CORE_PATH" 2>/dev/null | wc -l) items"
fi

# Test 3: Check PyO3 dependency removed
CARGO_FILE="/Users/brian/nodle-claude/nodle/Cargo.toml"
if ! grep -q "pyo3" "$CARGO_FILE"; then
    echo "✅ PyO3 dependency removed from core"
else
    echo "❌ PyO3 dependency still in core"
fi

# Test 4: Check USD registrations removed
FACTORY_FILE="/Users/brian/nodle-claude/nodle/src/nodes/factory.rs"
USD_REFS=$(grep -c "USD" "$FACTORY_FILE" 2>/dev/null || echo "0")
if [ "$USD_REFS" -le 2 ]; then  # Allow for comments
    echo "✅ USD registrations removed from factory"
else
    echo "❌ USD registrations still in factory ($USD_REFS references)"
fi

# Test 5: Check workspace USD refs removed  
WORKSPACE_FILE="/Users/brian/nodle-claude/nodle/src/workspaces/workspace_3d.rs"
if ! grep -q "register.*USD" "$WORKSPACE_FILE"; then
    echo "✅ USD registrations removed from 3D workspace"
else
    echo "❌ USD registrations still in 3D workspace"
fi

# Test 6: Check plugin architecture files exist
SDK_PATH="/Users/brian/nodle-claude/nodle-plugin-sdk"
TEMPLATE_PATH="/Users/brian/nodle-claude/nodle-plugin-template"
USD_PLUGIN_PATH="/Users/brian/nodle-claude/nodle-usd-plugin"

if [ -d "$SDK_PATH" ]; then
    echo "✅ Plugin SDK exists"
else
    echo "❌ Plugin SDK missing"
fi

if [ -d "$TEMPLATE_PATH" ]; then
    echo "✅ Plugin template exists"
else
    echo "❌ Plugin template missing"
fi

if [ -d "$USD_PLUGIN_PATH" ]; then
    echo "✅ USD plugin source exists"
    echo "   Files: $(find "$USD_PLUGIN_PATH/src" -name "*.rs" 2>/dev/null | wc -l) Rust files"
else
    echo "❌ USD plugin source missing"
fi

echo
echo "🎉 USD Plugin Separation Verification Complete!"