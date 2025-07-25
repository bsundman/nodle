// Simple button shader - just solid color circles

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
}

struct InstanceInput {
    @location(2) instance_position: vec2<f32>,
    @location(3) instance_radius: f32,
    @location(4) center_color: vec4<f32>,
    @location(5) outer_color: vec4<f32>,
}

struct Uniforms {
    view_matrix: mat4x4<f32>,
    pan_offset: vec2<f32>,
    zoom: f32,
    time: f32,
    screen_size: vec2<f32>,
    menu_bar_height: f32,
    _padding: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) center_color: vec4<f32>,
    @location(2) world_position: vec2<f32>,
    @location(3) button_center: vec2<f32>,
    @location(4) button_radius: f32,
}

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    let zoomed_radius = instance.instance_radius * uniforms.zoom;
    let world_pos = instance.instance_position + vertex.position * zoomed_radius;
    let panned_pos = world_pos + uniforms.pan_offset;
    
    // Offset Y coordinate to account for menu bar at top
    let screen_pos = vec2<f32>(panned_pos.x, panned_pos.y + uniforms.menu_bar_height);
    
    let ndc_x = (screen_pos.x / uniforms.screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (screen_pos.y / uniforms.screen_size.y) * 2.0;
    
    var out: VertexOutput;
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.tex_coord = vertex.tex_coord;
    out.center_color = instance.center_color;
    out.world_position = world_pos;
    out.button_center = instance.instance_position;
    out.button_radius = zoomed_radius;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let center_offset = in.world_position - in.button_center;
    let distance_from_center = length(center_offset);
    let normalized_distance = distance_from_center / in.button_radius;
    
    if (normalized_distance > 1.0) {
        discard;
    }
    
    // Just use the center color - solid color button
    return in.center_color;
}