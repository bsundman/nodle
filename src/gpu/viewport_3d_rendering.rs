//! 3D rendering system for Nodle viewports
//! 
//! This module implements a 3D rendering pipeline using wgpu for real-time 3D visualization
//! in viewport nodes. It supports mesh rendering, wireframe mode, grid display, and 
//! Maya-style camera navigation.

use wgpu::{
    Buffer, Device, Queue, RenderPipeline, BindGroup, BindGroupLayout, 
    TextureFormat, PresentMode, Surface, SurfaceConfiguration,
    CommandEncoder, RenderPass, BufferUsages, ShaderStages,
    VertexBufferLayout, VertexAttribute, VertexFormat, VertexStepMode,
    PrimitiveTopology, FrontFace, Face, PolygonMode,
    CompareFunction, DepthStencilState, DepthBiasState,
    TextureUsages, TextureDescriptor, TextureDimension, Extent3d,
    TextureView, TextureViewDescriptor,
};
use super::config::GraphicsConfig;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Vec2, Quat};
use std::mem;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex3D {
    const ATTRIBUTES: [VertexAttribute; 3] = [
        VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: VertexFormat::Float32x3,
        },
        VertexAttribute {
            offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
            shader_location: 1,
            format: VertexFormat::Float32x3,
        },
        VertexAttribute {
            offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
            shader_location: 2,
            format: VertexFormat::Float32x2,
        },
    ];

    pub fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: mem::size_of::<Vertex3D>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Uniforms3D {
    pub view_proj: [[f32; 4]; 4],
    pub model: [[f32; 4]; 4],
    pub camera_pos: [f32; 3],
    pub _padding: f32,
}

/// 3D Camera with Maya-style navigation
#[derive(Debug, Clone)]
pub struct Camera3D {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect: f32,
    
    // Maya-style navigation state
    pub orbit_sensitivity: f32,
    pub pan_sensitivity: f32,
    pub zoom_sensitivity: f32,
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            position: Vec3::new(5.0, 5.0, 5.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov: 45.0_f32.to_radians(),
            near: 0.1,
            far: 100.0,
            aspect: 1.0,
            orbit_sensitivity: 0.5,   // Increased from 0.005 to 0.5 for more responsive orbiting
            pan_sensitivity: 1.0,     // Increased from 0.01 to 1.0 for more responsive panning
            zoom_sensitivity: 1.0,    // Increased from 0.1 to 1.0 for more responsive zooming
        }
    }
}

impl Camera3D {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.position, self.target, self.up);
        let proj = Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far);
        proj * view
    }
    
    /// Maya-style orbit around target
    pub fn orbit(&mut self, delta_x: f32, delta_y: f32) {
        let offset = self.position - self.target;
        let radius = offset.length();
        
        // Convert to spherical coordinates
        let mut theta = offset.z.atan2(offset.x); // Azimuth
        let mut phi = (offset.y / radius).acos(); // Elevation
        
        // Apply rotation
        theta += delta_x * self.orbit_sensitivity;
        phi += delta_y * self.orbit_sensitivity;
        
        // Clamp phi to avoid gimbal lock
        phi = phi.clamp(0.01, std::f32::consts::PI - 0.01);
        
        // Convert back to cartesian
        let new_offset = Vec3::new(
            radius * phi.sin() * theta.cos(),
            radius * phi.cos(),
            radius * phi.sin() * theta.sin(),
        );
        
        self.position = self.target + new_offset;
    }
    
    /// Maya-style pan (move target and position together)
    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        let forward = (self.target - self.position).normalize();
        let right = forward.cross(self.up).normalize();
        let up = right.cross(forward).normalize();
        
        let pan_vector = right * delta_x * self.pan_sensitivity 
                        + up * delta_y * self.pan_sensitivity;
        
        self.position += pan_vector;
        self.target += pan_vector;
    }
    
    /// Maya-style zoom (move camera closer/farther from target)
    pub fn zoom(&mut self, delta: f32) {
        let direction = (self.target - self.position).normalize();
        let distance = (self.target - self.position).length();
        let new_distance = (distance + delta * self.zoom_sensitivity).max(0.1);
        
        self.position = self.target - direction * new_distance;
    }
    
    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
    
    /// Convert screen delta to world space movement for 1:1 pan
    pub fn screen_to_world_pan(&self, screen_delta_x: f32, screen_delta_y: f32, viewport_height: f32) -> Vec3 {
        // Calculate the vertical field of view extent at the target distance
        let distance = (self.target - self.position).length();
        let fov_height = 2.0 * distance * (self.fov / 2.0).tan();
        
        // Scale factor to convert screen pixels to world units
        let world_per_pixel = fov_height / viewport_height;
        
        // Calculate camera coordinate system
        let forward = (self.target - self.position).normalize();
        let right = forward.cross(self.up).normalize();
        let up = right.cross(forward).normalize();
        
        // Convert screen deltas to world space movement (fixed Y-axis)
        right * (screen_delta_x * world_per_pixel) + up * (screen_delta_y * world_per_pixel)
    }
    
    /// Get a ray from camera through screen position (normalized 0-1)
    pub fn screen_to_ray(&self, screen_x: f32, screen_y: f32) -> (Vec3, Vec3) {
        // Convert from screen space (0,1) to NDC (-1,1)
        let ndc_x = screen_x * 2.0 - 1.0;
        let ndc_y = 1.0 - screen_y * 2.0; // Flip Y
        
        // Create inverse view-projection matrix
        let view_proj = self.build_view_projection_matrix();
        let inv_view_proj = view_proj.inverse();
        
        // Transform points from NDC to world space
        let near_point = inv_view_proj.project_point3(Vec3::new(ndc_x, ndc_y, -1.0));
        let far_point = inv_view_proj.project_point3(Vec3::new(ndc_x, ndc_y, 1.0));
        
        let ray_origin = near_point;
        let ray_direction = (far_point - near_point).normalize();
        
        (ray_origin, ray_direction)
    }
    
    /// Orbit around a specific point in 3D space
    pub fn orbit_around_point(&mut self, pivot: Vec3, delta_x: f32, delta_y: f32) {
        // Calculate offset from pivot to current camera position
        let position_offset = self.position - pivot;
        let radius = position_offset.length();
        
        if radius < 0.001 {
            return; // Too close to pivot
        }
        
        // Convert to spherical coordinates
        let mut theta = position_offset.z.atan2(position_offset.x);
        let mut phi = (position_offset.y / radius).acos();
        
        // Apply rotation
        theta += delta_x * self.orbit_sensitivity;
        phi += delta_y * self.orbit_sensitivity;
        
        // Clamp phi to avoid gimbal lock
        phi = phi.clamp(0.01, std::f32::consts::PI - 0.01);
        
        // Convert back to cartesian for new position
        let new_position_offset = Vec3::new(
            radius * phi.sin() * theta.cos(),
            radius * phi.cos(),
            radius * phi.sin() * theta.sin(),
        );
        
        // Update camera position and make it look at the pivot point
        self.position = pivot + new_position_offset;
        self.target = pivot;
    }
    
    /// Zoom towards a specific point
    pub fn zoom_to_point(&mut self, target_point: Vec3, delta: f32) {
        let direction = (target_point - self.position).normalize();
        let distance = (target_point - self.position).length();
        
        // Scale zoom amount based on distance and increase sensitivity
        let zoom_amount = delta * self.zoom_sensitivity * distance * 2.0;
        
        // Calculate new position (moving towards target point)
        let new_position = self.position + direction * zoom_amount;
        
        // Ensure we don't zoom past the target point or too close
        let new_distance = (target_point - new_position).length();
        if new_distance > 0.1 {
            self.position = new_position;
            // Update target to maintain the camera's look direction
            let target_direction = (self.target - (self.position - direction * zoom_amount)).normalize();
            self.target = self.position + target_direction * (self.target - self.position).length();
        }
    }
    
    /// Ray-triangle intersection test using Möller-Trumbore algorithm
    pub fn ray_triangle_intersect(&self, ray_origin: Vec3, ray_direction: Vec3, v0: Vec3, v1: Vec3, v2: Vec3) -> Option<f32> {
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let h = ray_direction.cross(edge2);
        let a = edge1.dot(h);
        
        // Ray is parallel to triangle
        if a > -0.00001 && a < 0.00001 {
            return None;
        }
        
        let f = 1.0 / a;
        let s = ray_origin - v0;
        let u = f * s.dot(h);
        
        if u < 0.0 || u > 1.0 {
            return None;
        }
        
        let q = s.cross(edge1);
        let v = f * ray_direction.dot(q);
        
        if v < 0.0 || u + v > 1.0 {
            return None;
        }
        
        let t = f * edge2.dot(q);
        
        if t > 0.00001 {
            Some(t)
        } else {
            None
        }
    }
    
    /// Find the closest intersection point with scene geometry (only in front of camera)
    pub fn find_closest_intersection(&self, ray_origin: Vec3, ray_direction: Vec3) -> Option<Vec3> {
        // USD geometry intersection moved to plugin
        None
    }
    
    /// Find the best orbit pivot point for mouse position using proper ray casting
    pub fn find_orbit_pivot(&self, mouse_x: f32, mouse_y: f32) -> Vec3 {
        let (ray_origin, ray_direction) = self.screen_to_ray(mouse_x, mouse_y);
        
        // First try to find exact intersection with scene geometry
        if let Some(intersection_point) = self.find_closest_intersection(ray_origin, ray_direction) {
            return intersection_point;
        }
        
        // No direct intersection - use a reasonable default distance
        // Use current target distance as a sensible fallback
        let fallback_distance = (self.target - self.position).length();
        let fallback_point = ray_origin + ray_direction * fallback_distance;
        
        fallback_point
    }
}

/// Basic mesh data for 3D rendering
pub struct Mesh3D {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
}

impl Mesh3D {
    /// Create a cube mesh
    pub fn cube() -> Self {
        let vertices = vec![
            // Front face
            Vertex3D { position: [-1.0, -1.0,  1.0], normal: [ 0.0,  0.0,  1.0], uv: [0.0, 0.0] },
            Vertex3D { position: [ 1.0, -1.0,  1.0], normal: [ 0.0,  0.0,  1.0], uv: [1.0, 0.0] },
            Vertex3D { position: [ 1.0,  1.0,  1.0], normal: [ 0.0,  0.0,  1.0], uv: [1.0, 1.0] },
            Vertex3D { position: [-1.0,  1.0,  1.0], normal: [ 0.0,  0.0,  1.0], uv: [0.0, 1.0] },
            
            // Back face
            Vertex3D { position: [-1.0, -1.0, -1.0], normal: [ 0.0,  0.0, -1.0], uv: [1.0, 0.0] },
            Vertex3D { position: [-1.0,  1.0, -1.0], normal: [ 0.0,  0.0, -1.0], uv: [1.0, 1.0] },
            Vertex3D { position: [ 1.0,  1.0, -1.0], normal: [ 0.0,  0.0, -1.0], uv: [0.0, 1.0] },
            Vertex3D { position: [ 1.0, -1.0, -1.0], normal: [ 0.0,  0.0, -1.0], uv: [0.0, 0.0] },
            
            // Top face
            Vertex3D { position: [-1.0,  1.0, -1.0], normal: [ 0.0,  1.0,  0.0], uv: [0.0, 1.0] },
            Vertex3D { position: [-1.0,  1.0,  1.0], normal: [ 0.0,  1.0,  0.0], uv: [0.0, 0.0] },
            Vertex3D { position: [ 1.0,  1.0,  1.0], normal: [ 0.0,  1.0,  0.0], uv: [1.0, 0.0] },
            Vertex3D { position: [ 1.0,  1.0, -1.0], normal: [ 0.0,  1.0,  0.0], uv: [1.0, 1.0] },
            
            // Bottom face
            Vertex3D { position: [-1.0, -1.0, -1.0], normal: [ 0.0, -1.0,  0.0], uv: [1.0, 1.0] },
            Vertex3D { position: [ 1.0, -1.0, -1.0], normal: [ 0.0, -1.0,  0.0], uv: [0.0, 1.0] },
            Vertex3D { position: [ 1.0, -1.0,  1.0], normal: [ 0.0, -1.0,  0.0], uv: [0.0, 0.0] },
            Vertex3D { position: [-1.0, -1.0,  1.0], normal: [ 0.0, -1.0,  0.0], uv: [1.0, 0.0] },
            
            // Right face
            Vertex3D { position: [ 1.0, -1.0, -1.0], normal: [ 1.0,  0.0,  0.0], uv: [1.0, 0.0] },
            Vertex3D { position: [ 1.0,  1.0, -1.0], normal: [ 1.0,  0.0,  0.0], uv: [1.0, 1.0] },
            Vertex3D { position: [ 1.0,  1.0,  1.0], normal: [ 1.0,  0.0,  0.0], uv: [0.0, 1.0] },
            Vertex3D { position: [ 1.0, -1.0,  1.0], normal: [ 1.0,  0.0,  0.0], uv: [0.0, 0.0] },
            
            // Left face
            Vertex3D { position: [-1.0, -1.0, -1.0], normal: [-1.0,  0.0,  0.0], uv: [0.0, 0.0] },
            Vertex3D { position: [-1.0, -1.0,  1.0], normal: [-1.0,  0.0,  0.0], uv: [1.0, 0.0] },
            Vertex3D { position: [-1.0,  1.0,  1.0], normal: [-1.0,  0.0,  0.0], uv: [1.0, 1.0] },
            Vertex3D { position: [-1.0,  1.0, -1.0], normal: [-1.0,  0.0,  0.0], uv: [0.0, 1.0] },
        ];
        
        let indices = vec![
            0,  1,  2,   0,  2,  3,    // front
            4,  5,  6,   4,  6,  7,    // back
            8,  9,  10,  8,  10, 11,   // top
            12, 13, 14,  12, 14, 15,   // bottom
            16, 17, 18,  16, 18, 19,   // right
            20, 21, 22,  20, 22, 23,   // left
        ];
        
        Self { vertices, indices }
    }
    
    /// Create a ground plane for the grid
    pub fn grid_plane(size: f32) -> Self {
        let vertices = vec![
            Vertex3D { position: [-size, 0.0, -size], normal: [0.0, 1.0, 0.0], uv: [0.0, 0.0] },
            Vertex3D { position: [ size, 0.0, -size], normal: [0.0, 1.0, 0.0], uv: [1.0, 0.0] },
            Vertex3D { position: [ size, 0.0,  size], normal: [0.0, 1.0, 0.0], uv: [1.0, 1.0] },
            Vertex3D { position: [-size, 0.0,  size], normal: [0.0, 1.0, 0.0], uv: [0.0, 1.0] },
        ];
        
        let indices = vec![0, 1, 2, 0, 2, 3];
        
        Self { vertices, indices }
    }
}

/// 3D Renderer for viewport nodes
pub struct Renderer3D {
    pub device: Option<Device>,
    pub queue: Option<Queue>,
    pub mesh_pipeline: Option<RenderPipeline>,
    pub wireframe_pipeline: Option<RenderPipeline>,
    pub grid_pipeline: Option<RenderPipeline>,
    pub axis_pipeline: Option<RenderPipeline>,
    pub uniform_buffer: Option<Buffer>,
    pub uniform_bind_group: Option<BindGroup>,
    pub depth_texture: Option<TextureView>,
    pub camera: Camera3D,
    pub cube_mesh: Option<Mesh3D>,
    pub grid_mesh: Option<Mesh3D>,
    pub grid_vertex_buffer: Option<Buffer>,
    pub grid_index_buffer: Option<Buffer>,
    pub grid_index_count: u32,
    pub axis_vertex_buffer: Option<Buffer>,
    pub axis_index_buffer: Option<Buffer>,
    pub axis_index_count: u32,
}

impl std::fmt::Debug for Renderer3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Renderer3D")
            .field("camera", &self.camera)
            .field("has_device", &self.device.is_some())
            .field("has_queue", &self.queue.is_some())
            .field("has_mesh_pipeline", &self.mesh_pipeline.is_some())
            .field("has_wireframe_pipeline", &self.wireframe_pipeline.is_some())
            .field("has_grid_pipeline", &self.grid_pipeline.is_some())
            .finish()
    }
}

impl Default for Renderer3D {
    fn default() -> Self {
        Self {
            device: None,
            queue: None,
            mesh_pipeline: None,
            wireframe_pipeline: None,
            grid_pipeline: None,
            axis_pipeline: None,
            uniform_buffer: None,
            uniform_bind_group: None,
            depth_texture: None,
            camera: Camera3D::default(),
            cube_mesh: Some(Mesh3D::cube()),
            grid_mesh: Some(Mesh3D::grid_plane(10.0)),
            grid_vertex_buffer: None,
            grid_index_buffer: None,
            grid_index_count: 0,
            axis_vertex_buffer: None,
            axis_index_buffer: None,
            axis_index_count: 0,
        }
    }
}

impl Renderer3D {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Initialize the 3D renderer with device and queue
    pub fn initialize(&mut self, device: Device, queue: Queue) {
        // Create uniform buffer
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("3D Uniform Buffer"),
            size: mem::size_of::<Uniforms3D>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("3D Bind Group Layout"),
        });
        
        // Create bind group
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("3D Bind Group"),
        });
        
        // Load shaders and create pipelines
        self.create_pipelines_with_device(&device, &bind_group_layout);
        
        // Store the created resources
        self.uniform_buffer = Some(uniform_buffer);
        self.uniform_bind_group = Some(uniform_bind_group);
        self.device = Some(device);
        self.queue = Some(queue);
        
        // Create grid buffers
        self.create_grid_buffers(20.0, 40); // 20x20 grid with 40 divisions
        
        // Create axis gizmo buffers
        self.create_axis_buffers();
    }
    
    fn create_pipelines_with_device(&mut self, device: &Device, bind_group_layout: &BindGroupLayout) {
        // Load shaders
        let mesh_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("3D Mesh Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/mesh3d.wgsl").into()),
        });
        
        let wireframe_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("3D Wireframe Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/wireframe3d.wgsl").into()),
        });
        
        let grid_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("3D Grid Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/grid3d.wgsl").into()),
        });
        
        let axis_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("3D Axis Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/axis_gizmo.wgsl").into()),
        });
        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("3D Pipeline Layout"),
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Create mesh pipeline
        self.mesh_pipeline = Some(device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("3D Mesh Pipeline"),
            layout: Some(&pipeline_layout),
            cache: None,
            vertex: wgpu::VertexState {
                module: &mesh_shader,
                entry_point: "vs_main",
                buffers: &[Vertex3D::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &mesh_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None, // No depth buffer in egui callback system
            multisample: GraphicsConfig::global().multisample_state(),
            multiview: None,
        }));
        
        // Create wireframe pipeline
        self.wireframe_pipeline = Some(device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("3D Wireframe Pipeline"),
            layout: Some(&pipeline_layout),
            cache: None,
            vertex: wgpu::VertexState {
                module: &wireframe_shader,
                entry_point: "vs_main",
                buffers: &[Vertex3D::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &wireframe_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill, // Use Fill instead of Line to avoid feature requirement
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None, // No depth buffer in egui callback system
            multisample: GraphicsConfig::global().multisample_state(),
            multiview: None,
        }));
        
        // Create grid pipeline
        self.grid_pipeline = Some(device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("3D Grid Pipeline"),
            layout: Some(&pipeline_layout),
            cache: None,
            vertex: wgpu::VertexState {
                module: &grid_shader,
                entry_point: "vs_main",
                buffers: &[
                    VertexBufferLayout {
                        array_stride: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &[
                            VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: VertexFormat::Float32x3,
                            },
                        ],
                    }
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &grid_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None, // No depth buffer in egui callback system
            multisample: GraphicsConfig::global().multisample_state(),
            multiview: None,
        }));
        
        // Create axis gizmo pipeline
        self.axis_pipeline = Some(device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("3D Axis Pipeline"),
            layout: Some(&pipeline_layout),
            cache: None,
            vertex: wgpu::VertexState {
                module: &axis_shader,
                entry_point: "vs_main",
                buffers: &[
                    VertexBufferLayout {
                        array_stride: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress, // position + color
                        step_mode: VertexStepMode::Vertex,
                        attributes: &[
                            VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: VertexFormat::Float32x3,
                            },
                            VertexAttribute {
                                offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                                shader_location: 1,
                                format: VertexFormat::Float32x3,
                            },
                        ],
                    }
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &axis_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None, // No depth testing for gizmo
            multisample: GraphicsConfig::global().multisample_state(),
            multiview: None,
        }));
    }
    
    /// Initialize renderer using references (for callback system)
    /// This creates minimal state needed for rendering but doesn't store Device/Queue
    pub fn initialize_from_refs(&mut self, device: &Device, queue: &Queue) {
        // Create uniform buffer
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("3D Uniform Buffer"),
            size: mem::size_of::<Uniforms3D>() as u64,  
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("3D Bind Group Layout"),
        });
        
        // Create bind group
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("3D Bind Group"),
        });
        
        // Create pipelines
        self.create_pipelines_with_device(device, &bind_group_layout);
        
        // Store created resources (but not device/queue)
        self.uniform_buffer = Some(uniform_buffer);
        self.uniform_bind_group = Some(uniform_bind_group);
        
        // Create grid and axis buffers
        self.create_grid_buffers_from_refs(device, 20.0, 40);
        self.create_axis_buffers_from_refs(device);
        
        // Mark as initialized by setting a flag (we'll use a different approach)
        // Instead of storing device/queue, we'll check for pipeline existence
    }
    
    pub fn resize(&mut self, width: u32, height: u32) {
        self.camera.set_aspect(width as f32 / height as f32);
        
        if let Some(device) = &self.device {
            // Recreate depth texture
            let depth_texture = device.create_texture(&TextureDescriptor {
                label: Some("3D Depth Texture"),
                size: Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            
            self.depth_texture = Some(depth_texture.create_view(&TextureViewDescriptor::default()));
        }
    }
    
    /// Update camera uniforms
    pub fn update_camera_uniforms(&self, queue: &Queue) {
        if let Some(uniform_buffer) = &self.uniform_buffer {
            let view_proj_matrix = self.camera.build_view_projection_matrix();
            let uniforms = Uniforms3D {
                view_proj: view_proj_matrix.to_cols_array_2d(),
                model: Mat4::IDENTITY.to_cols_array_2d(),
                camera_pos: [self.camera.position.x, self.camera.position.y, self.camera.position.z],
                _padding: 0.0,
            };
            
            queue.write_buffer(uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
        }
    }
    
    /// Render mesh geometry
    pub fn render_mesh(&self, render_pass: &mut wgpu::RenderPass, vertex_buffer: &Buffer, index_buffer: &Buffer, index_count: u32) {
        if let (Some(pipeline), Some(bind_group)) = (&self.mesh_pipeline, &self.uniform_bind_group) {
            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..index_count, 0, 0..1);
        }
    }
    
    /// Render wireframe geometry
    pub fn render_wireframe(&self, render_pass: &mut wgpu::RenderPass, vertex_buffer: &Buffer, index_buffer: &Buffer, index_count: u32) {
        if let (Some(pipeline), Some(bind_group)) = (&self.wireframe_pipeline, &self.uniform_bind_group) {
            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..index_count, 0, 0..1);
        }
    }
    
    /// Create grid vertex and index buffers
    pub fn create_grid_buffers(&mut self, size: f32, divisions: u32) {
        if let Some(device) = &self.device {
            let mut vertices = Vec::new();
            let mut indices = Vec::new();
            
            let step = size * 2.0 / divisions as f32;
            let half_size = size;
            
            // Create grid lines
            for i in 0..=divisions {
                let pos = -half_size + i as f32 * step;
                
                // Lines along X
                vertices.push([pos, 0.0, -half_size]);
                vertices.push([pos, 0.0, half_size]);
                
                // Lines along Z
                vertices.push([-half_size, 0.0, pos]);
                vertices.push([half_size, 0.0, pos]);
            }
            
            // Create indices for line list
            for i in 0..vertices.len() {
                indices.push(i as u32);
            }
            
            // Create buffers
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Grid Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: BufferUsages::VERTEX,
            });
            
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Grid Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: BufferUsages::INDEX,
            });
            
            self.grid_vertex_buffer = Some(vertex_buffer);
            self.grid_index_buffer = Some(index_buffer);
            self.grid_index_count = indices.len() as u32;
        }
    }
    
    /// Render grid
    pub fn render_grid(&self, render_pass: &mut wgpu::RenderPass) {
        if let (Some(pipeline), Some(bind_group), Some(vertex_buffer), Some(index_buffer)) = 
            (&self.grid_pipeline, &self.uniform_bind_group, &self.grid_vertex_buffer, &self.grid_index_buffer) {
            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.grid_index_count, 0, 0..1);
        }
    }
    
    /// Create axis gizmo buffers
    pub fn create_axis_buffers(&mut self) {
        if let Some(device) = &self.device {
            #[repr(C)]
            #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
            struct AxisVertex {
                position: [f32; 3],
                color: [f32; 3],
            }
            
            let axis_length = 1.0;
            let vertices = vec![
                // X axis (red)
                AxisVertex { position: [0.0, 0.0, 0.0], color: [1.0, 0.0, 0.0] },
                AxisVertex { position: [axis_length, 0.0, 0.0], color: [1.0, 0.0, 0.0] },
                // Y axis (green)
                AxisVertex { position: [0.0, 0.0, 0.0], color: [0.0, 1.0, 0.0] },
                AxisVertex { position: [0.0, axis_length, 0.0], color: [0.0, 1.0, 0.0] },
                // Z axis (blue)
                AxisVertex { position: [0.0, 0.0, 0.0], color: [0.0, 0.0, 1.0] },
                AxisVertex { position: [0.0, 0.0, axis_length], color: [0.0, 0.0, 1.0] },
            ];
            
            let indices: Vec<u32> = vec![0, 1, 2, 3, 4, 5]; // Line list
            
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Axis Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: BufferUsages::VERTEX,
            });
            
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Axis Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: BufferUsages::INDEX,
            });
            
            self.axis_vertex_buffer = Some(vertex_buffer);
            self.axis_index_buffer = Some(index_buffer);
            self.axis_index_count = indices.len() as u32;
        }
    }
    
    /// Create grid buffers using device reference (for callback system)
    pub fn create_grid_buffers_from_refs(&mut self, device: &Device, size: f32, divisions: u32) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        let step = size * 2.0 / divisions as f32;
        let half_size = size;
        
        // Create grid lines
        for i in 0..=divisions {
            let pos = -half_size + i as f32 * step;
            
            // Lines along X
            vertices.push([pos, 0.0, -half_size]);
            vertices.push([pos, 0.0, half_size]);
            
            // Lines along Z
            vertices.push([-half_size, 0.0, pos]);
            vertices.push([half_size, 0.0, pos]);
        }
        
        // Create indices for line list
        for i in 0..vertices.len() {
            indices.push(i as u32);
        }
        
        // Create buffers
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });
        
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });
        
        self.grid_vertex_buffer = Some(vertex_buffer);
        self.grid_index_buffer = Some(index_buffer);
        self.grid_index_count = indices.len() as u32;
    }
    
    /// Create axis buffers using device reference (for callback system)
    pub fn create_axis_buffers_from_refs(&mut self, device: &Device) {
        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct AxisVertex {
            position: [f32; 3],
            color: [f32; 3],
        }
        
        let axis_length = 1.0;
        let vertices = vec![
            // X axis (red)
            AxisVertex { position: [0.0, 0.0, 0.0], color: [1.0, 0.0, 0.0] },
            AxisVertex { position: [axis_length, 0.0, 0.0], color: [1.0, 0.0, 0.0] },
            // Y axis (green)
            AxisVertex { position: [0.0, 0.0, 0.0], color: [0.0, 1.0, 0.0] },
            AxisVertex { position: [0.0, axis_length, 0.0], color: [0.0, 1.0, 0.0] },
            // Z axis (blue)
            AxisVertex { position: [0.0, 0.0, 0.0], color: [0.0, 0.0, 1.0] },
            AxisVertex { position: [0.0, 0.0, axis_length], color: [0.0, 0.0, 1.0] },
        ];
        
        let indices: Vec<u32> = vec![0, 1, 2, 3, 4, 5]; // Line list
        
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Axis Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });
        
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Axis Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });
        
        self.axis_vertex_buffer = Some(vertex_buffer);
        self.axis_index_buffer = Some(index_buffer);
        self.axis_index_count = indices.len() as u32;
    }
    
    /// Render axis gizmo
    pub fn render_axis_gizmo(&self, render_pass: &mut wgpu::RenderPass) {
        if let (Some(pipeline), Some(bind_group), Some(vertex_buffer), Some(index_buffer)) = 
            (&self.axis_pipeline, &self.uniform_bind_group, &self.axis_vertex_buffer, &self.axis_index_buffer) {
            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.axis_index_count, 0, 0..1);
        }
    }
    
    /// Set camera for rendering
    pub fn set_camera(&mut self, camera: &Camera3D) {
        self.camera = camera.clone();
    }
    
    /// Render a complete scene with plugin viewport data
    pub fn render_scene(&mut self, render_pass: &mut wgpu::RenderPass, viewport_data: &nodle_plugin_sdk::viewport::ViewportData, _viewport_size: (u32, u32)) {
        // Update camera from viewport data
        let plugin_camera = &viewport_data.scene.camera;
        self.camera.position = glam::Vec3::new(plugin_camera.position[0], plugin_camera.position[1], plugin_camera.position[2]);
        self.camera.target = glam::Vec3::new(plugin_camera.target[0], plugin_camera.target[1], plugin_camera.target[2]);
        self.camera.up = glam::Vec3::new(plugin_camera.up[0], plugin_camera.up[1], plugin_camera.up[2]);
        self.camera.fov = plugin_camera.fov;
        self.camera.near = plugin_camera.near;
        self.camera.far = plugin_camera.far;
        self.camera.aspect = plugin_camera.aspect;
        
        // Render basic scene (grid and axis) first
        self.render_basic_scene(render_pass, _viewport_size);
        
        // TODO: Render plugin meshes
        // For now, render basic scene representation
        if !viewport_data.scene.meshes.is_empty() {
            // Render a placeholder cube for each mesh
            for mesh in &viewport_data.scene.meshes {
                println!("🔧 Rendering mesh: {}", mesh.id);
                // TODO: Convert plugin mesh data to wgpu buffers and render
            }
        }
    }
    
    /// Render basic scene (grid, axes) when no plugin data is available
    pub fn render_basic_scene(&self, render_pass: &mut wgpu::RenderPass, _viewport_size: (u32, u32)) {
        // Render grid
        if let (Some(vertex_buffer), Some(index_buffer)) = (&self.grid_vertex_buffer, &self.grid_index_buffer) {
            if let (Some(pipeline), Some(bind_group)) = (&self.grid_pipeline, &self.uniform_bind_group) {
                render_pass.set_pipeline(pipeline);
                render_pass.set_bind_group(0, bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.grid_index_count, 0, 0..1);
            }
        }
        
        // Render axis gizmo
        self.render_axis_gizmo(render_pass);
    }
}