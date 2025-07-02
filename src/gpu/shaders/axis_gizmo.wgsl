// Axis Gizmo Shader for Nodle 3D Viewport

struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _padding: f32,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Position gizmo in screen space (bottom-left corner)
    let gizmo_size = 0.1;
    let gizmo_offset = vec2<f32>(-0.85, -0.85);
    
    // Create view matrix without translation for gizmo
    let view_rotation = mat3x3<f32>(
        uniforms.view_proj[0].xyz,
        uniforms.view_proj[1].xyz,
        uniforms.view_proj[2].xyz
    );
    
    // Apply only rotation to gizmo
    let rotated_pos = view_rotation * model.position;
    
    // Position in screen space
    out.clip_position = vec4<f32>(
        rotated_pos.x * gizmo_size + gizmo_offset.x,
        rotated_pos.y * gizmo_size + gizmo_offset.y,
        0.0,
        1.0
    );
    
    out.color = model.color;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}