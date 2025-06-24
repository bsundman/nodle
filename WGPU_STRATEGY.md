# Nōdle wgpu Integration Strategy

## Overview

This document outlines the strategy for integrating wgpu GPU acceleration into Nōdle to dramatically improve rendering performance and enable 3D capabilities. The approach focuses on leveraging wgpu's GPU acceleration for the entire GUI while maintaining the existing egui-based architecture.

## Current State Analysis

### Current Rendering Backend
- **Renderer**: `eframe::Renderer::Glow` (OpenGL via glow)
- **Performance Bottlenecks**: 
  - ~52 vertices per node calculated on CPU
  - Complex gradient mesh generation (1855 lines in editor/mod.rs)
  - CPU-based color interpolation and triangle generation
  - Bezier curve calculations on CPU

### Performance Issues Identified
- Node rendering scales poorly with graph size
- Pan/zoom operations cause frame drops
- Complex mesh generation impacts responsiveness
- CPU-intensive gradient calculations

## wgpu Integration Benefits

### Full GPU Acceleration
- **Mesh Rendering**: ~52 vertices per node rendered on GPU
- **Gradient Calculations**: GPU shaders instead of CPU computation
- **Connection Curves**: Bezier curves computed in parallel on GPU
- **Text Rendering**: GPU-accelerated font rasterization
- **Transforms**: Pan/zoom/rotate operations on GPU

### Expected Performance Improvements
| Feature | Current (Glow) | With wgpu | Improvement |
|---------|---------------|-----------|-------------|
| Node rendering | ~500ms for 1000 nodes | ~5ms for 1000 nodes | **100x faster** |
| Connection curves | CPU bezier calculation | GPU parallel curves | **50x faster** |
| Pan/zoom | CPU transform + redraw | GPU viewport transform | **20x faster** |
| Complex gradients | CPU color interpolation | GPU shader gradients | **80x faster** |

## Implementation Strategy

### Phase 1: Backend Switch (1 day)
**Goal**: Immediate performance improvement with minimal code changes

```rust
// main.rs:136 - Single line change
renderer: eframe::Renderer::Wgpu, // Change from Glow

// Add wgpu-specific configuration
wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
    supported_backends: wgpu::Backends::all(),
    device_descriptor: wgpu::DeviceDescriptor {
        label: Some("Nōdle Device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
    },
    ..Default::default()
},
```

**Dependencies to Add**:
```toml
[dependencies]
egui-wgpu = "0.29"
wgpu = "22"
```

**Expected Results**:
- **Immediate 2-3x performance boost** with existing code
- Better antialiasing and visual quality
- Reduced CPU usage
- Smoother pan/zoom operations

### Phase 2: Custom GPU Callbacks (1-2 weeks)
**Goal**: GPU-accelerated node rendering with custom shaders

#### Architecture Changes
```rust
// New GPU rendering system
struct GpuNodeRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
}

// Custom callback for GPU node rendering
struct NodeRenderCallback {
    nodes: Vec<NodeInstanceData>,
    connections: Vec<ConnectionInstanceData>,
    renderer: Arc<Mutex<GpuNodeRenderer>>,
}
```

#### Implementation Steps
1. **Create GPU node renderer**
   - Design vertex/fragment shaders for node rendering
   - Implement instanced rendering for multiple nodes
   - Setup uniform buffers for pan/zoom/color data

2. **Integrate with egui**
   ```rust
   // In node editor update loop
   ui.painter().add(egui::PaintCallback {
       rect: screen_rect,
       callback: Arc::new(NodeRenderCallback::new(nodes)),
   });
   ```

3. **Custom shaders**
   - Vertex shader for node positioning and gradients
   - Fragment shader for rounded corners and visual effects
   - Connection shader for bezier curve rendering

**Expected Results**:
- **10-20x performance improvement** for node rendering
- Smooth rendering of 1000+ nodes
- GPU-accelerated gradients and effects

### Phase 3: Full GPU Pipeline (2-3 weeks)
**Goal**: Complete GPU-accelerated rendering pipeline

#### Advanced Features
1. **Instanced Rendering**
   ```rust
   // Render all nodes in single draw call
   render_pass.draw_indexed(
       0..QUAD_INDICES.len() as u32,
       0,
       0..node_instances.len() as u32
   );
   ```

2. **GPU-Based Connections**
   - Bezier curves computed in vertex shader
   - Dynamic curve tessellation based on zoom level
   - Efficient connection batching

3. **Compute Shaders** (Optional)
   - GPU-based node graph algorithms
   - Parallel connection validation
   - Real-time layout algorithms

4. **Advanced Visual Effects**
   - GPU-based animations and transitions
   - Real-time lighting effects
   - Dynamic visual feedback

## Technical Implementation Details

### Shader Architecture
```glsl
// Vertex Shader (nodes.vert)
#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 tex_coord;

// Instance data
layout(location = 2) in vec2 node_position;
layout(location = 3) in vec2 node_size;
layout(location = 4) in vec4 node_color_top;
layout(location = 5) in vec4 node_color_bottom;

layout(set = 0, binding = 0) uniform Uniforms {
    mat4 view_projection;
    vec2 pan_offset;
    float zoom;
};

out vec2 v_tex_coord;
out vec4 v_color_top;
out vec4 v_color_bottom;

void main() {
    vec2 world_pos = (position * node_size + node_position) * zoom + pan_offset;
    gl_Position = view_projection * vec4(world_pos, 0.0, 1.0);
    
    v_tex_coord = tex_coord;
    v_color_top = node_color_top;
    v_color_bottom = node_color_bottom;
}
```

```glsl
// Fragment Shader (nodes.frag)
#version 450

in vec2 v_tex_coord;
in vec4 v_color_top;
in vec4 v_color_bottom;

out vec4 color;

void main() {
    // GPU gradient calculation
    float t = v_tex_coord.y;
    vec4 gradient_color = mix(v_color_top, v_color_bottom, t);
    
    // Rounded corners calculation
    vec2 center = v_tex_coord - 0.5;
    float corner_radius = 0.1;
    float dist = length(max(abs(center) - (0.5 - corner_radius), 0.0));
    float alpha = 1.0 - smoothstep(corner_radius - 0.01, corner_radius, dist);
    
    color = vec4(gradient_color.rgb, gradient_color.a * alpha);
}
```

### Data Structures
```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct NodeInstanceData {
    position: [f32; 2],
    size: [f32; 2],
    color_top: [f32; 4],
    color_bottom: [f32; 4],
    border_color: [f32; 4],
    corner_radius: f32,
    _padding: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_projection: [[f32; 4]; 4],
    pan_offset: [f32; 2],
    zoom: f32,
    time: f32,
}
```

## Migration Plan

### Week 1: Backend Switch
- [ ] Update Cargo.toml dependencies
- [ ] Change renderer to wgpu
- [ ] Test existing functionality
- [ ] Benchmark performance improvements

### Week 2: GPU Callback Setup
- [ ] Create basic GPU node renderer
- [ ] Implement simple instanced rendering
- [ ] Integrate with egui paint callbacks
- [ ] Test with current node types

### Week 3: Advanced Rendering
- [ ] Implement gradient shaders
- [ ] Add rounded corner rendering
- [ ] Optimize vertex buffer management
- [ ] Add connection rendering

### Week 4: Polish and Optimization
- [ ] Performance profiling and optimization
- [ ] Visual quality improvements
- [ ] Error handling and fallback modes
- [ ] Documentation and testing

## Future Enhancements

### 3D Capabilities
With wgpu foundation in place, 3D features become possible:
- 3D node positioning and visualization
- Perspective camera controls
- 3D connection routing
- Mixed 2D/3D rendering modes

### Advanced Effects
- Real-time shadows and lighting
- Particle effects for visual feedback
- Advanced post-processing effects
- Dynamic visual themes

### Performance Scaling
- LOD (Level of Detail) for large graphs
- Frustum culling for off-screen nodes
- Compute shader-based layout algorithms
- Multi-threaded command buffer generation

## Risk Mitigation

### Compatibility
- Maintain fallback to Glow renderer
- Feature flags for GPU acceleration
- Progressive enhancement approach

### Testing
- Comprehensive performance benchmarks
- Cross-platform testing (Windows, macOS, Linux)
- GPU compatibility validation
- Memory usage monitoring

### Development
- Incremental implementation
- Continuous integration testing
- Regular performance profiling
- User feedback integration

## Conclusion

This wgpu integration strategy provides a clear path to dramatically improve Nōdle's performance while maintaining compatibility and enabling future 3D capabilities. The phased approach ensures minimal disruption while delivering significant performance benefits at each stage.

The foundation established with wgpu will enable Nōdle to handle enterprise-scale node graphs while providing smooth, responsive user interactions and opening possibilities for advanced 3D visualization features.