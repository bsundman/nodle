// GPU-accelerated node rendering shader
// This shader efficiently renders thousands of nodes with gradients and rounded corners

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
    @location(2) node_position: vec2<f32>,        // Node world position
    @location(3) node_size: vec2<f32>,            // Node size
    @location(4) bevel_color_top: vec4<f32>,      // Bevel layer top color (0.65 grey)
    @location(5) bevel_color_bottom: vec4<f32>,   // Bevel layer bottom color (0.15 grey)
    @location(6) background_color_top: vec4<f32>, // Background layer top color (0.5 grey)
    @location(7) background_color_bottom: vec4<f32>, // Background layer bottom color (0.25 grey)
    @location(8) border_color: vec4<f32>,         // Border color
    @location(9) corner_radius: f32,              // Corner radius
    @location(10) selected: f32,                  // 1.0 if selected, 0.0 otherwise
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) bevel_color_top: vec4<f32>,
    @location(2) bevel_color_bottom: vec4<f32>,
    @location(3) background_color_top: vec4<f32>,
    @location(4) background_color_bottom: vec4<f32>,
    @location(5) border_color: vec4<f32>,
    @location(6) corner_radius: f32,
    @location(7) selected: f32,
    @location(8) node_size: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    // Expand render quad to accommodate border layer (1px beyond node size on all sides)
    let border_expand = 1.0;
    let expanded_size = instance.node_size + vec2<f32>(border_expand * 2.0, border_expand * 2.0);
    let expanded_position = instance.node_position - vec2<f32>(border_expand, border_expand);
    
    // Calculate the world position of this vertex
    let world_pos = expanded_position + vertex.position * expanded_size;
    
    // Apply pan and zoom transforms
    let transformed_pos = world_pos * uniforms.zoom + uniforms.pan_offset;
    
    // Convert to normalized device coordinates (NDC)
    let ndc_x = (transformed_pos.x / uniforms.screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (transformed_pos.y / uniforms.screen_size.y) * 2.0;
    
    var out: VertexOutput;
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.tex_coord = vertex.tex_coord;
    out.bevel_color_top = instance.bevel_color_top;
    out.bevel_color_bottom = instance.bevel_color_bottom;
    out.background_color_top = instance.background_color_top;
    out.background_color_bottom = instance.background_color_bottom;
    out.border_color = instance.border_color;
    out.corner_radius = instance.corner_radius * uniforms.zoom;
    out.selected = instance.selected;
    out.node_size = instance.node_size * uniforms.zoom;
    
    return out;
}

// Helper function to calculate distance to rounded rectangle
fn rounded_rect_sdf(pixel_pos: vec2<f32>, size: vec2<f32>, radius: f32) -> f32 {
    let half_size = size * 0.5;
    let center = half_size;
    let corner_box = half_size - vec2<f32>(radius);
    let dist_to_box = length(max(abs(pixel_pos - center) - corner_box, vec2<f32>(0.0)));
    return dist_to_box - radius;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_coord = in.tex_coord;
    let node_size = in.node_size;
    let corner_radius = in.corner_radius;
    
    // CPU layer sizing: border = full node size + 1px, bevel = shrunk by 1px, background = shrunk by 3px total
    let bevel_shrink = 1.0;      // Bevel shrunk by 1px from border
    let background_shrink = 2.0; // Background shrunk by 2px from bevel (3px total from border)
    let border_expand = 1.0;     // Border extends 1px beyond node size
    
    // Since we expanded the quad, tex_coord covers the expanded area
    let expanded_size = node_size + vec2<f32>(border_expand * 2.0, border_expand * 2.0);
    let expanded_pixel_pos = tex_coord * expanded_size;
    
    // Border layer uses the full expanded area
    let border_sdf_dist = rounded_rect_sdf(expanded_pixel_pos, expanded_size, corner_radius);
    
    // Bevel layer: shrunk by 1px from border
    let bevel_size = node_size - vec2<f32>(bevel_shrink * 2.0, bevel_shrink * 2.0);
    let bevel_pixel_pos = expanded_pixel_pos - vec2<f32>(border_expand + bevel_shrink, border_expand + bevel_shrink);
    let bevel_sdf_dist = rounded_rect_sdf(bevel_pixel_pos, bevel_size, corner_radius);
    
    // Background layer: shrunk by additional 2px from bevel (3px total from border)
    let background_size = bevel_size - vec2<f32>(background_shrink * 2.0, background_shrink * 2.0);
    let background_pixel_pos = bevel_pixel_pos - vec2<f32>(background_shrink, background_shrink);
    let background_sdf_dist = rounded_rect_sdf(background_pixel_pos, background_size, corner_radius);
    
    // Layer shape alphas
    let border_alpha = smoothstep(0.0, 1.0, -border_sdf_dist);
    let background_alpha = smoothstep(0.0, 1.0, -background_sdf_dist);
    let bevel_alpha = smoothstep(0.0, 1.0, -bevel_sdf_dist);
    
    // Discard pixels outside the border
    if (border_alpha < 0.01) {
        discard;
    }
    
    // Create vertical gradient parameter
    let gradient_t = tex_coord.y;
    
    // Start with border layer (solid color - no gradient for border)
    let border_color = in.border_color.rgb;
    
    // Add bevel layer on top of border
    let bevel_color = mix(in.bevel_color_top, in.bevel_color_bottom, gradient_t);
    let border_bevel_blend = mix(border_color, bevel_color.rgb, bevel_alpha);
    
    // Add background layer on top of bevel+border
    let background_color = mix(in.background_color_top, in.background_color_bottom, gradient_t);
    let final_color = mix(border_bevel_blend, background_color.rgb, background_alpha);
    
    return vec4<f32>(final_color, border_alpha);
}