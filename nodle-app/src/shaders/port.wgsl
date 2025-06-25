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
    // Create a quad around the port center - match CPU port sizing exactly
    // CPU uses port_radius + 2.0 for border, so total size is (port_radius + 2) * 2
    let port_size = (instance.port_radius + 2.0) * 2.0;
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
    // Match CPU port rendering with 3-layer approach
    let tex_coord = in.tex_coord;
    let port_radius = in.port_radius;
    
    // Calculate pixel position within the port quad (centered at 0.5, 0.5)
    let center = vec2<f32>(0.5, 0.5);
    let pixel_pos_from_center = (tex_coord - center) * (port_radius + 2.0) * 2.0; // Match quad size from vertex shader
    
    // CPU layer sizing: border = port_radius + 2, bevel = port_radius + 1, background = port_radius
    // Modified for thicker highlight: shrink bevel and background by 1px each
    let border_radius = port_radius + 2.0;
    let bevel_radius = port_radius;         // Shrunk by 1px (was port_radius + 1.0)
    let background_radius = port_radius - 1.0;  // Shrunk by 1px (was port_radius)
    
    // Calculate distances using circle SDF
    let border_dist = circle_sdf(pixel_pos_from_center, vec2<f32>(0.0, 0.0), border_radius);
    let bevel_dist = circle_sdf(pixel_pos_from_center, vec2<f32>(0.0, 0.0), bevel_radius);
    let background_dist = circle_sdf(pixel_pos_from_center, vec2<f32>(0.0, 0.0), background_radius);
    
    // Layer alphas using smoothstep for anti-aliasing
    let border_alpha = smoothstep(0.0, 1.0, -border_dist);
    let bevel_alpha = smoothstep(0.0, 1.0, -bevel_dist);
    let background_alpha = smoothstep(0.0, 1.0, -background_dist);
    
    // Discard pixels outside the border
    if (border_alpha < 0.01) {
        discard;
    }
    
    // Layer colors - match CPU exactly
    let border_color = in.border_color.rgb;
    let bevel_color = in.bevel_color.rgb;
    let background_color = in.background_color.rgb;
    
    // Composite layers: border -> bevel -> background (exactly like CPU)
    let border_bevel_blend = mix(border_color, bevel_color, bevel_alpha);
    let final_color = mix(border_bevel_blend, background_color, background_alpha);
    
    return vec4<f32>(final_color, border_alpha);
}