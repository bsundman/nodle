# USD Viewport Performance Optimization Strategy

## Current Performance Issues

### 1. **USD File Reloading Every Frame** 🚨 CRITICAL
The viewport is loading the entire USD file **every frame**:
```
✅ USDEngine: Extracted 1788 meshes from USD stage in 1.660933292s
```
This happens because `ViewportNode::get_viewport_data` creates a new `USDRenderer` and loads the stage every time it's called.

### 2. **GPU Buffer Re-upload Every Frame** 🚨 CRITICAL
In `render_scene()`, the code uploads meshes to GPU every frame:
```rust
// Upload mesh to GPU if not already uploaded
if let Err(e) = self.upload_mesh_to_gpu(mesh.id.clone(), mesh) {
```
Even though there's a check, it still iterates through all 1788 meshes every frame.

### 3. **Inefficient Data Flow**
- ViewportNode creates mesh data
- Converts to SDK format
- Converts to GPU format
- Multiple unnecessary copies and allocations

## Optimization Options

### Option 1: **Cache USD Data in ViewportNode** (Quick Fix) ✅ IMPLEMENTING FIRST
Store the loaded USD data in the ViewportNode instance:
```rust
pub struct ViewportNode {
    pub current_stage: String,
    pub viewport_data: ViewportData,
    pub camera_settings: CameraSettings,
    pub viewport_logic: USDViewportLogic,
    pub cached_usd_data: Option<USDSceneData>, // Add this
    pub last_loaded_stage: String,             // Add this
}
```

### Option 2: **Implement Proper Viewport State Management** (Best Solution)
Create a persistent viewport state that:
- Loads USD data once when stage changes
- Keeps GPU buffers allocated
- Only updates when scene actually changes
- Separates camera updates from scene updates

### Option 3: **Implement Level-of-Detail (LOD) System**
- Render distant objects with fewer polygons
- Use bounding box culling
- Implement frustum culling
- Only render visible meshes

### Option 4: **Batch Rendering**
Instead of 1788 individual draw calls:
- Merge meshes with same materials
- Use instanced rendering for repeated objects
- Create mesh atlases

### Option 5: **Async USD Loading**
- Load USD in background thread
- Show progress indicator
- Don't block frame rendering during load

## Immediate Fixes Needed

### 1. **Stop Reloading USD Every Frame**
```rust
// In ViewportNode
pub fn get_viewport_data(node: &Node) -> Option<ViewportData> {
    // Check if stage already loaded
    if let Some(cached_data) = node.cached_viewport_data {
        return Some(cached_data);
    }
    // Only load if not cached
}
```

### 2. **Move GPU Upload to One-Time Operation**
```rust
// In renderer
pub fn prepare_scene(&mut self, viewport_data: &ViewportData) {
    // Only upload meshes that aren't already on GPU
    for mesh in &viewport_data.scene.meshes {
        if !self.gpu_meshes.contains_key(&mesh.id) {
            self.upload_mesh_to_gpu(mesh.id.clone(), mesh);
        }
    }
}
```

### 3. **Separate Camera Updates from Scene Rendering**
```rust
// Only update camera uniforms, not entire scene
pub fn update_camera_only(&mut self, camera_data: &CameraData) {
    // Just update camera uniforms
    self.update_camera_uniforms();
}
```

## Implementation Order

1. **First**: Fix USD reloading (Option 1) - This alone should fix most lag
2. **Second**: Implement proper state management (Option 2)
3. **Third**: Add frustum culling (Option 3)
4. **Later**: Batch rendering and LOD for massive scenes

## Expected Performance Improvements

- **Option 1**: Should reduce frame time from ~1.6s to ~16ms (60 FPS)
- **Option 2**: Better memory usage and smoother interactions
- **Option 3**: Scale to even larger scenes (10k+ meshes)
- **Option 4**: Reduce draw calls by 90%+
- **Option 5**: Non-blocking UI during scene loads