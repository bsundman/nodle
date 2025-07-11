// 3D Wireframe Shader for Nodle 3D Viewport

struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _padding: f32,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Transform position to world space then to clip space
    let world_position = uniforms.model * vec4<f32>(model.position, 1.0);
    out.clip_position = uniforms.view_proj * world_position;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    
    // Simple white wireframe
    out.color = vec4<f32>(0.8, 0.8, 0.8, 1.0);
    
    return out;
}