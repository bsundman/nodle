// 3D Grid Shader for Nodle 3D Viewport

struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _padding: f32,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Transform position to world space
    let world_position = uniforms.model * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;
    
    // Transform to clip space
    out.clip_position = uniforms.view_proj * world_position;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    
    // Grid pattern
    let grid_size = 1.0;
    let line_width = 0.05;
    
    let grid_coord = in.world_position.xz / grid_size;
    let grid_lines = abs(fract(grid_coord - 0.5) - 0.5) / fwidth(grid_coord);
    let line = min(grid_lines.x, grid_lines.y);
    
    let grid_alpha = 1.0 - min(line, 1.0);
    
    // Fade grid with distance
    let distance_to_camera = length(uniforms.camera_pos - in.world_position);
    let fade_factor = 1.0 - smoothstep(10.0, 50.0, distance_to_camera);
    
    out.color = vec4<f32>(0.5, 0.5, 0.5, grid_alpha * fade_factor * 0.3);
    
    return out;
}