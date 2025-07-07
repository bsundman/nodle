// USD Mesh Vertex/Fragment Shader

struct USDUniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    camera_pos: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: USDUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Transform vertex position
    let world_position = uniforms.model * vec4<f32>(vertex.position, 1.0);
    out.clip_position = uniforms.view_proj * world_position;
    out.world_position = world_position.xyz;
    
    // Transform normal
    let normal_matrix = uniforms.model; // Assuming uniform scaling
    out.world_normal = normalize((normal_matrix * vec4<f32>(vertex.normal, 0.0)).xyz);
    
    out.uv = vertex.uv;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // USD material colors
    let base_color = vec3<f32>(0.7, 0.7, 0.8); // Light gray
    let ambient = vec3<f32>(0.2, 0.2, 0.2);
    
    // Simple directional lighting
    let light_dir = normalize(vec3<f32>(-0.5, -1.0, -0.5));
    let normal = normalize(in.world_normal);
    let diffuse = max(dot(-light_dir, normal), 0.0);
    
    // Camera-based rim lighting
    let view_dir = normalize(uniforms.camera_pos - in.world_position);
    let rim = 1.0 - max(dot(view_dir, normal), 0.0);
    let rim_factor = pow(rim, 2.0) * 0.3;
    
    let final_color = base_color * (ambient + vec3<f32>(diffuse) + vec3<f32>(rim_factor));
    
    return vec4<f32>(final_color, 1.0);
}