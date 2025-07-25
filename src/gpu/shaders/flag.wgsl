// GPU-accelerated flag rendering shader
// This shader efficiently renders visibility flags with 3-layer styling

struct Uniforms {
    view_matrix: mat4x4<f32>,
    pan_offset: vec2<f32>,
    zoom: f32,
    time: f32,
    screen_size: vec2<f32>,
    menu_bar_height: f32,
    _padding: f32,
}

struct VertexInput {
    @location(0) position: vec2<f32>,    // Quad vertex position (0-1)
    @location(1) tex_coord: vec2<f32>,   // Texture coordinates (0-1)
}

struct InstanceInput {
    @location(2) flag_position: vec2<f32>,        // Flag world position
    @location(3) flag_radius: f32,                // Flag radius
    @location(4) border_color: vec4<f32>,         // Border color
    @location(5) bevel_color: vec4<f32>,          // Bevel color (dark grey)
    @location(6) background_color: vec4<f32>,     // Background color (flag color)
    @location(7) is_visible: f32,                 // 1.0 if visible, 0.0 if hidden
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) border_color: vec4<f32>,
    @location(2) bevel_color: vec4<f32>,
    @location(3) background_color: vec4<f32>,
    @location(4) flag_radius: f32,
    @location(5) is_visible: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    // Create a quad around the flag center - accommodate larger border radius
    // Border is now 6px radius, so quad size needs to be 12.0 (6.0 * 2)
    let flag_size = instance.flag_radius * 2.4; // 6.0/5.0 = 1.2, so 2.0 * 1.2 = 2.4
    let world_pos = instance.flag_position + (vertex.position - 0.5) * flag_size;
    
    // Apply pan and zoom transforms
    let transformed_pos = world_pos * uniforms.zoom + uniforms.pan_offset;
    
    // Offset Y coordinate to account for menu bar at top
    let screen_pos = vec2<f32>(transformed_pos.x, transformed_pos.y + uniforms.menu_bar_height);
    
    // Convert to normalized device coordinates (NDC)
    let ndc_x = (screen_pos.x / uniforms.screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (screen_pos.y / uniforms.screen_size.y) * 2.0;
    
    var out: VertexOutput;
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.1, 1.0); // Slightly forward to render on top
    out.tex_coord = vertex.tex_coord;
    out.border_color = instance.border_color;
    out.bevel_color = instance.bevel_color;
    out.background_color = instance.background_color;
    out.flag_radius = instance.flag_radius * uniforms.zoom;
    out.is_visible = instance.is_visible;
    
    return out;
}

// Helper function to calculate distance to circle
fn circle_sdf(pixel_pos: vec2<f32>, center: vec2<f32>, radius: f32) -> f32 {
    return length(pixel_pos - center) - radius;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Match CPU flag rendering with 3-layer approach
    let tex_coord = in.tex_coord;
    let flag_radius = in.flag_radius;
    
    // Calculate pixel position within the flag quad (centered at 0.5, 0.5)
    let center = vec2<f32>(0.5, 0.5);
    // flag_radius already has zoom applied from vertex shader
    let pixel_pos_from_center = (tex_coord - center) * flag_radius * 2.4; // Match vertex shader quad size
    
    // Layer sizing to match CPU exactly (flag_radius already includes zoom):
    // Border: 6px outer, 5px inner = 1px stroke (transparent center)
    // Bevel: 5px outer, 4px inner = 1px stroke (transparent center)
    // Background (dot): 3.5px radius (filled)
    let border_outer_radius = flag_radius * 1.2; // 6.0 * zoom (6/5 = 1.2)
    let border_inner_radius = flag_radius; // 5.0 * zoom (1px stroke width)
    let bevel_outer_radius = flag_radius; // 5.0 * zoom
    let bevel_inner_radius = flag_radius * 0.8; // 4.0 * zoom (1px stroke width)
    let background_radius = flag_radius * 0.7; // 3.5 * zoom (3.5/5 = 0.7)
    
    // Calculate distances using circle SDF (same as port shader)
    let border_outer_dist = circle_sdf(pixel_pos_from_center, vec2<f32>(0.0, 0.0), border_outer_radius);
    let border_inner_dist = circle_sdf(pixel_pos_from_center, vec2<f32>(0.0, 0.0), border_inner_radius);
    let bevel_outer_dist = circle_sdf(pixel_pos_from_center, vec2<f32>(0.0, 0.0), bevel_outer_radius);
    let bevel_inner_dist = circle_sdf(pixel_pos_from_center, vec2<f32>(0.0, 0.0), bevel_inner_radius);
    let background_dist = circle_sdf(pixel_pos_from_center, vec2<f32>(0.0, 0.0), background_radius);
    
    // Layer alphas using smoothstep for anti-aliasing (same as port shader)
    let border_outer_alpha = smoothstep(0.0, 1.0, -border_outer_dist);
    let border_inner_alpha = smoothstep(0.0, 1.0, -border_inner_dist);
    let border_alpha = border_outer_alpha - border_inner_alpha; // Stroke = outer - inner
    
    let bevel_outer_alpha = smoothstep(0.0, 1.0, -bevel_outer_dist);
    let bevel_inner_alpha = smoothstep(0.0, 1.0, -bevel_inner_dist);
    let bevel_alpha = bevel_outer_alpha - bevel_inner_alpha; // Stroke = outer - inner
    
    let background_alpha = smoothstep(0.0, 1.0, -background_dist);
    
    // Early discard for performance - discard if no layer is visible
    if (border_alpha < 0.01 && bevel_alpha < 0.01 && background_alpha < 0.01) {
        discard;
    }
    
    // Layer colors
    let border_color = in.border_color.rgb;
    let bevel_color = in.bevel_color.rgb;
    let background_color = in.background_color.rgb;
    
    // Composite layers exactly like port shader: border -> bevel -> background
    let border_bevel_blend = mix(border_color, bevel_color, bevel_alpha);
    
    // Only show background dot when visible
    var final_color: vec3<f32>;
    var effective_background_alpha: f32;
    if (in.is_visible > 0.5) {
        // Visible: show background dot
        final_color = mix(border_bevel_blend, background_color, background_alpha);
        effective_background_alpha = background_alpha;
    } else {
        // Not visible: hide background dot (make it transparent)
        final_color = border_bevel_blend;
        effective_background_alpha = 0.0; // No background contribution
    }
    
    // Calculate total flag alpha (any visible layer) 
    let total_flag_alpha = max(border_alpha, max(bevel_alpha, effective_background_alpha));
    
    // Return with total alpha for proper anti-aliasing (same as port shader)
    return vec4<f32>(final_color, total_flag_alpha);
}