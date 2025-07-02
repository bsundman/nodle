# USD Integration Strategy

## Current Implementation: PyO3 + Python USD API

We use PyO3 (Rust-Python bindings) to call Pixar's official USD Python API from Rust.

### Architecture
```
Rust Code → PyO3 → Python Interpreter → USD Python API → USD C++ Core
```

## Performance Analysis

### PyO3 Overhead Sources
1. **Function call overhead:** ~24-44ns per call
2. **Type conversion:** Python objects ↔ Rust types  
3. **GIL management:** Global Interpreter Lock handling
4. **Python interpreter:** Additional abstraction layer

### When Overhead Matters vs. Doesn't Matter

#### ❌ High Impact (Avoid)
- Frequent small operations (creating many primitives in tight loops)
- Real-time rendering loops (60 FPS viewport updates)
- Large scene traversals every frame
- Fine-grained attribute access in render loops

#### ✅ Low Impact (Acceptable)
- Batch operations (loading/saving stages)
- One-time scene setup
- User-triggered node modifications
- Configuration/metadata operations

## Viewport Rendering Strategy

### The Key Insight: Viewport ≠ USD Performance

**Viewport rendering (60 FPS):**
```rust
// This runs every frame - needs to be fast
fn render_viewport() {
    // Use cached geometry data (already in GPU buffers)
    wgpu_renderer.draw_meshes(&cached_mesh_data);
    wgpu_renderer.draw_lights(&cached_light_data);
}
```

**USD operations (user-triggered):**
```rust
// This runs when user modifies nodes - can be slower
fn on_parameter_change() {
    usd_engine.set_attribute(prim, "radius", new_value);  // PyO3 call
    let updated_geometry = usd_engine.get_mesh_data(prim); // PyO3 call
    gpu_cache.update_mesh(updated_geometry);              // Update GPU cache
}
```

### Performance Pattern
- **USD updates:** 1-10 times per second (when user edits nodes)
- **Viewport rendering:** 60 FPS (using cached GPU data)

### Recommended Architecture
```
USD Stage (Source of Truth)
    ↓ PyO3 calls when nodes change
Cached GPU Data
    ↓ Used for real-time rendering
60 FPS Viewport
```

## Alternative Approaches Evaluated

### Rust FFI Bindings to USD C++

**Existing Projects:**
- **vfx-rs/usd-bind:** Not production ready, USD 21.11 only, incomplete API
- **AndrejOrsula/pxr_rs:** Early development, 50-minute build times, lacking docs
- **luke-titley/usd-rs:** Partial implementation, missing key features

**Status:** All experimental/incomplete (as of 2025)

**Implementation Difficulty:** Very difficult from scratch due to:
- USD's complex C++ features (templates, exceptions, RTTI)
- Massive API surface (~1000+ classes)
- Complex memory management across language boundaries

### Performance Comparison

| Aspect | PyO3 (Current) | FFI Bindings |
|--------|----------------|--------------|
| **Maturity** | ✅ Stable, production-ready | ❌ All experimental |
| **API Coverage** | ✅ Complete USD Python API | ❌ Partial/incomplete |
| **Maintenance** | ✅ Maintained by PyO3 team | ❌ Individual projects |
| **Documentation** | ✅ Full USD docs apply | ❌ Lacking documentation |
| **Setup** | ✅ Simple `pip install usd-core` | ❌ Complex build processes |

## Optimization Strategies for PyO3

### 1. Batch Operations
```rust
// ❌ Bad: Many small calls
for prim in prims {
    create_sphere(prim.path, prim.radius)  // PyO3 call each iteration
}

// ✅ Good: Single batch call
create_spheres_batch(sphere_data_vec)     // One PyO3 call
```

### 2. Minimize Boundary Crossings
```rust
// ❌ Bad: Frequent USD queries
fn render_frame() {
    for prim in usd_stage.traverse() {    // PyO3 call every frame
        let mesh = prim.get_mesh();       // PyO3 call every frame
        render(mesh);
    }
}

// ✅ Good: Cache and update pattern
fn on_node_change() {
    usd_stage.set_attribute(prim, value);     // PyO3 call when needed
    gpu_cache.update_from_usd(usd_stage);     // Batch PyO3 calls
}

fn render_frame() {
    gpu_renderer.draw(&gpu_cache);            // Pure GPU, no USD calls
}
```

### 3. Async USD Updates
- Perform USD operations on background threads
- Update GPU cache when complete
- Keep viewport responsive during complex USD operations

## Decision: Stick with PyO3

**Rationale:**
1. **Production ready** vs experimental alternatives
2. **Complete API coverage** vs partial implementations  
3. **Performance acceptable** for node editor use case
4. **Simple maintenance** vs complex FFI management

**When to reconsider:**
1. Measured performance bottlenecks in actual usage
2. USD Rust bindings reach production maturity (1-2 years)
3. Need for features not available in Python API

## Node Editor Specific Considerations

### USD Call Patterns in Node Editors

**High-Frequency Operations (need optimization):**
- Interactive parameter changes during modeling
- Node connections/disconnections
- Real-time material preview updates

**Moderate-Frequency Operations:**
- File import/export
- Complex node graph execution
- Scene hierarchy changes

**Low-Frequency Operations (PyO3 overhead negligible):**
- Initial scene setup
- Saving projects
- Menu operations

### Implementation Guidelines

1. **Cache USD data in Rust** for frequently accessed information
2. **Batch USD operations** when possible
3. **Use async updates** for non-blocking USD operations
4. **Profile before optimizing** - measure actual bottlenecks

## Future Migration Path

If performance becomes critical:

1. **Hybrid approach:** PyO3 for most operations, FFI for hot paths
2. **Selective FFI:** Implement FFI only for performance-critical USD operations
3. **Wait for maturity:** Monitor USD Rust binding development

The current PyO3 approach provides the best balance of functionality, reliability, and performance for a node-based 3D editor.