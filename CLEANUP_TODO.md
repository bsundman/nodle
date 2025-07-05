# Cleanup TODO

## Files that can be removed

### 1. Old USD stub implementations (replaced by plugin)
- `src/nodes/three_d/usd/usd_material.rs` - Old stub, functionality now in USD plugin
- These contain the basic `USDMaterial`, `USDPreviewSurface`, and `USDTexture` structs that were moved to the plugin

### 2. Test file in plugins
- `plugins/test.txt` - Just a test file to verify plugin scanning

### 3. Potential cleanup candidates (verify they're not in use first)
- `src/context.rs` - If old context system (check if still imported)
- `src/menu_hierarchy.rs` and `src/menu_hierarchy_usd.rs` - If replaced by new menu system

## Files that should NOT be removed

### Test files (legitimate tests)
- `src/nodes/registry_test.rs` - Contains actual unit tests
- `src/contexts/test_phase4.rs` - Contains actual unit tests  
- `src/workspaces/test_phase4.rs` - Contains actual workspace tests
- `src/nodes/three_d/usd/test_usd.rs` - Contains USD tests

### USD shading implementations still in use
- `src/nodes/three_d/usd/shading/material/` - Full implementation still used by other USD nodes
- `src/nodes/three_d/usd/shading/preview_surface.rs` - Full implementation  
- `src/nodes/three_d/usd/shading/texture_reader.rs` - Full implementation

## Notes
- The USD plugin only contains simplified versions of Material, PreviewSurface, and Texture
- The full USD implementation in core is much more comprehensive and is still needed
- Most "test" files are actual test modules, not temporary files