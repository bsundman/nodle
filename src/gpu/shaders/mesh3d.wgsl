// 3D Mesh Vertex and Fragment Shader for Nodle 3D Viewport

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
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) vertex_color: vec3<f32>,
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
    
    // Transform normal to world space
    let normal_matrix = mat3x3<f32>(
        uniforms.model[0].xyz,
        uniforms.model[1].xyz,
        uniforms.model[2].xyz
    );
    out.world_normal = normalize(normal_matrix * model.normal);
    
    out.uv = model.uv;
    out.vertex_color = model.color;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    
    // Simple PBR-like lighting
    // Use vertex color if available, otherwise use default color
    let base_color = in.vertex_color;
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let view_dir = normalize(uniforms.camera_pos - in.world_position);
    
    // Lambertian diffuse
    let n_dot_l = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse = base_color * n_dot_l;
    
    // Simple specular
    let half_vec = normalize(light_dir + view_dir);
    let n_dot_h = max(dot(in.world_normal, half_vec), 0.0);
    let specular = vec3<f32>(0.3) * pow(n_dot_h, 32.0);
    
    // Ambient
    let ambient = base_color * 0.1;
    
    let final_color = ambient + diffuse + specular;
    out.color = vec4<f32>(final_color, 1.0);
    
    return out;
}