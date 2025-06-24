// GPU-accelerated port rendering shader
// This shader efficiently renders thousands of ports with 3-layer styling

struct Uniforms {
    view_matrix: mat4x4<f32>,
    pan_offset: vec2<f32>,
    zoom: f32,
    time: f32,
    screen_size: vec2<f32>,
    _padding: vec2<f32>,
}

struct VertexInput {
    @location(0) position: vec2<f32>,    // Quad vertex position (0-1)
    @location(1) tex_coord: vec2<f32>,   // Texture coordinates (0-1)
}

struct InstanceInput {
    @location(2) port_position: vec2<f32>,        // Port world position
    @location(3) port_radius: f32,                // Port radius
    @location(4) border_color: vec4<f32>,         // Border color
    @location(5) bevel_color: vec4<f32>,          // Bevel color (dark grey)
    @location(6) background_color: vec4<f32>,     // Background color (port type color)
    @location(7) is_input: f32,                   // 1.0 for input, 0.0 for output
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) border_color: vec4<f32>,
    @location(2) bevel_color: vec4<f32>,
    @location(3) background_color: vec4<f32>,
    @location(4) port_radius: f32,
    @location(5) is_input: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    // Create a quad around the port center - match CPU port sizing
    let port_size = instance.port_radius * 2.0 + 2.0; // Smaller border space to match CPU
    let world_pos = instance.port_position + (vertex.position - 0.5) * port_size;
    
    // Apply pan and zoom transforms
    let transformed_pos = world_pos * uniforms.zoom + uniforms.pan_offset;
    
    // Convert to normalized device coordinates (NDC)
    let ndc_x = (transformed_pos.x / uniforms.screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (transformed_pos.y / uniforms.screen_size.y) * 2.0;
    
    var out: VertexOutput;
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.tex_coord = vertex.tex_coord;
    out.border_color = instance.border_color;
    out.bevel_color = instance.bevel_color;
    out.background_color = instance.background_color;
    out.port_radius = instance.port_radius * uniforms.zoom;
    out.is_input = instance.is_input;
    
    return out;
}

// Helper function to calculate distance to circle
fn circle_sdf(pixel_pos: vec2<f32>, center: vec2<f32>, radius: f32) -> f32 {
    return length(pixel_pos - center) - radius;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // SIMPLE TEST: Render bright green circles for ports to verify they're rendering
    let tex_coord = in.tex_coord;
    
    // Simple circle test - distance from center
    let center = vec2<f32>(0.5, 0.5);
    let dist = length(tex_coord - center);
    
    if (dist > 0.5) {
        discard;
    }
    
    // Bright green for input ports, bright red for output ports
    let color = select(
        vec4<f32>(1.0, 0.0, 0.0, 1.0), // Red for output
        vec4<f32>(0.0, 1.0, 0.0, 1.0), // Green for input
        in.is_input > 0.5
    );
    
    return color;
}