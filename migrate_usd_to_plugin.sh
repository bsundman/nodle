#!/bin/bash
# Script to migrate all USD code to the plugin

# Source and destination directories
SRC_USD="/Users/brian/nodle-claude/nodle/src/nodes/three_d/usd"
DEST_PLUGIN="/Users/brian/nodle-claude/nodle-usd-plugin/src"

echo "Creating directory structure..."
mkdir -p "$DEST_PLUGIN"/{core,stage,geometry,transform,lighting,shading,viewport}

echo "Copying USD core files..."
# Core infrastructure
cp "$SRC_USD"/usd_engine.rs "$DEST_PLUGIN"/core/engine.rs 2>/dev/null || echo "  - usd_engine.rs not found"
cp "$SRC_USD"/usd_engine_extended.rs "$DEST_PLUGIN"/core/engine_extended.rs 2>/dev/null || echo "  - usd_engine_extended.rs not found"
cp "$SRC_USD"/local_usd.rs "$DEST_PLUGIN"/core/local_usd.rs 2>/dev/null || echo "  - local_usd.rs not found"
cp "$SRC_USD"/get_attribute.rs "$DEST_PLUGIN"/core/get_attribute.rs 2>/dev/null || echo "  - get_attribute.rs not found"
cp "$SRC_USD"/set_attribute.rs "$DEST_PLUGIN"/core/set_attribute.rs 2>/dev/null || echo "  - set_attribute.rs not found"

echo "Copying stage management files..."
# Stage management
if [ -d "$SRC_USD/stage" ]; then
    cp -r "$SRC_USD"/stage/* "$DEST_PLUGIN"/stage/ 2>/dev/null
fi
# Also copy individual stage files
cp "$SRC_USD"/create_stage.rs "$DEST_PLUGIN"/stage/ 2>/dev/null || echo "  - create_stage.rs not found"
cp "$SRC_USD"/load_stage.rs "$DEST_PLUGIN"/stage/ 2>/dev/null || echo "  - load_stage.rs not found"
cp "$SRC_USD"/save_stage.rs "$DEST_PLUGIN"/stage/ 2>/dev/null || echo "  - save_stage.rs not found"

echo "Copying geometry files..."
# Geometry
if [ -d "$SRC_USD/geometry" ]; then
    cp -r "$SRC_USD"/geometry/* "$DEST_PLUGIN"/geometry/ 2>/dev/null
fi
# Also copy individual geometry files
cp "$SRC_USD"/usd_cube.rs "$DEST_PLUGIN"/geometry/ 2>/dev/null || echo "  - usd_cube.rs not found"
cp "$SRC_USD"/usd_sphere.rs "$DEST_PLUGIN"/geometry/ 2>/dev/null || echo "  - usd_sphere.rs not found"
cp "$SRC_USD"/usd_mesh.rs "$DEST_PLUGIN"/geometry/ 2>/dev/null || echo "  - usd_mesh.rs not found"

echo "Copying transform files..."
# Transform
if [ -d "$SRC_USD/transform" ]; then
    cp -r "$SRC_USD"/transform/* "$DEST_PLUGIN"/transform/ 2>/dev/null
fi
cp "$SRC_USD"/usd_xform.rs "$DEST_PLUGIN"/transform/ 2>/dev/null || echo "  - usd_xform.rs not found"

echo "Copying lighting files..."
# Lighting
if [ -d "$SRC_USD/lighting" ]; then
    cp -r "$SRC_USD"/lighting/* "$DEST_PLUGIN"/lighting/ 2>/dev/null
fi
cp "$SRC_USD"/usd_light.rs "$DEST_PLUGIN"/lighting/ 2>/dev/null || echo "  - usd_light.rs not found"

echo "Copying shading files..."
# Shading
if [ -d "$SRC_USD/shading" ]; then
    cp -r "$SRC_USD"/shading/* "$DEST_PLUGIN"/shading/ 2>/dev/null
fi
cp "$SRC_USD"/usd_material.rs "$DEST_PLUGIN"/shading/ 2>/dev/null || echo "  - usd_material.rs not found"

echo "Copying viewport files..."
# Viewport
if [ -d "$SRC_USD/usd_viewport" ]; then
    cp -r "$SRC_USD"/usd_viewport/* "$DEST_PLUGIN"/viewport/ 2>/dev/null
fi

echo "Copying other USD files..."
# Other files
cp "$SRC_USD"/usd_camera.rs "$DEST_PLUGIN"/core/ 2>/dev/null || echo "  - usd_camera.rs not found"
cp "$SRC_USD"/usd_composition.rs "$DEST_PLUGIN"/core/ 2>/dev/null || echo "  - usd_composition.rs not found"
cp "$SRC_USD"/test_usd.rs "$DEST_PLUGIN"/core/ 2>/dev/null || echo "  - test_usd.rs not found"
cp "$SRC_USD"/mod.rs "$DEST_PLUGIN"/core/original_mod.rs 2>/dev/null || echo "  - mod.rs not found"

echo ""
echo "Migration complete! Now you need to:"
echo "1. Update imports in all copied files"
echo "2. Create mod.rs files for each module" 
echo "3. Update Cargo.toml with USD dependencies"
echo "4. Remove USD files from core"
echo ""
echo "Files copied to: $DEST_PLUGIN"