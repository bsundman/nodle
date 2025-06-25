//! GPU-accelerated node rendering using wgpu callbacks

use egui::{Color32, Vec2};
use nodle_core::{Node, NodeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wgpu::util::DeviceExt;
use once_cell::sync::Lazy;

/// Instance data for a single node in GPU memory
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NodeInstanceData {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub bevel_color_top: [f32; 4],      // Bevel layer top color (0.65 grey)
    pub bevel_color_bottom: [f32; 4],   // Bevel layer bottom color (0.15 grey)
    pub background_color_top: [f32; 4], // Background layer top color (0.5 grey)
    pub background_color_bottom: [f32; 4], // Background layer bottom color (0.25 grey)
    pub border_color: [f32; 4],         // Border color (blue if selected, grey if not)
    pub corner_radius: f32,
    pub selected: f32, // 1.0 if selected, 0.0 otherwise
    pub _padding: [f32; 3], // Adjusted padding for alignment
}

/// Instance data for a single port in GPU memory
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PortInstanceData {
    pub position: [f32; 2],
    pub radius: f32,
    pub border_color: [f32; 4],         // Border color
    pub bevel_color: [f32; 4],          // Bevel color (dark grey)
    pub background_color: [f32; 4],     // Background color (port type color)
    pub is_input: f32,                  // 1.0 for input, 0.0 for output
    pub _padding: [f32; 2],
}

impl NodeInstanceData {
    pub fn from_node(node: &Node, selected: bool, zoom: f32) -> Self {
        let rect = node.get_rect();
        
        // CPU-style colors to match exact visual appearance
        // BEVEL layer: 0.65 grey to 0.15 grey
        let bevel_top = Color32::from_rgb(166, 166, 166);    // 0.65 grey
        let bevel_bottom = Color32::from_rgb(38, 38, 38);    // 0.15 grey
        
        // BACKGROUND layer: 0.5 grey to 0.25 grey
        let background_top = Color32::from_rgb(127, 127, 127); // 0.5 grey
        let background_bottom = Color32::from_rgb(64, 64, 64); // 0.25 grey
        
        // BORDER color - blue if selected, dark gray otherwise
        let border_color = if selected {
            Color32::from_rgb(100, 150, 255) // Blue selection
        } else {
            Color32::from_rgb(64, 64, 64)    // 0.25 grey unselected
        };
        
        
        
        // Use the original node size - bevel layer matches node size exactly
        Self {
            position: [rect.min.x, rect.min.y],
            size: [rect.width(), rect.height()],
            bevel_color_top: Self::color_to_array(bevel_top),
            bevel_color_bottom: Self::color_to_array(bevel_bottom),
            background_color_top: Self::color_to_array(background_top),
            background_color_bottom: Self::color_to_array(background_bottom),
            border_color: Self::color_to_array(border_color),
            corner_radius: 5.0,  // Use fixed radius, let shader handle zoom
            selected: if selected { 1.0 } else { 0.0 },
            _padding: [0.0, 0.0, 0.0],
        }
    }
    
    fn color_to_array(color: Color32) -> [f32; 4] {
        [
            color.r() as f32 / 255.0,
            color.g() as f32 / 255.0,
            color.b() as f32 / 255.0,
            color.a() as f32 / 255.0,
        ]
    }
    
    fn lighten_color(color: Color32, factor: f32) -> Color32 {
        Color32::from_rgb(
            ((color.r() as f32 * (1.0 + factor)).min(255.0)) as u8,
            ((color.g() as f32 * (1.0 + factor)).min(255.0)) as u8,
            ((color.b() as f32 * (1.0 + factor)).min(255.0)) as u8,
        )
    }
    
    fn darken_color(color: Color32, factor: f32) -> Color32 {
        Color32::from_rgb(
            ((color.r() as f32 * (1.0 - factor)).max(0.0)) as u8,
            ((color.g() as f32 * (1.0 - factor)).max(0.0)) as u8,
            ((color.b() as f32 * (1.0 - factor)).max(0.0)) as u8,
        )
    }
}

impl PortInstanceData {
    pub fn from_port(position: egui::Pos2, radius: f32, is_connecting: bool, is_input: bool) -> Self {
        let border_color = if is_connecting {
            Color32::from_rgb(100, 150, 255) // Blue when connecting
        } else {
            Color32::from_rgb(64, 64, 64)    // Dark grey normally
        };
        
        let bevel_color = Color32::from_rgb(38, 38, 38); // Dark grey bevel
        
        let background_color = if is_input {
            Color32::from_rgb(70, 120, 90)  // Green for input ports
        } else {
            Color32::from_rgb(120, 70, 70)  // Red for output ports
        };
        
        
        Self {
            position: [position.x, position.y],
            radius,
            border_color: Self::color_to_array(border_color),
            bevel_color: Self::color_to_array(bevel_color),
            background_color: Self::color_to_array(background_color),
            is_input: if is_input { 1.0 } else { 0.0 },
            _padding: [0.0, 0.0],
        }
    }
    
    fn color_to_array(color: Color32) -> [f32; 4] {
        [
            color.r() as f32 / 255.0,
            color.g() as f32 / 255.0,
            color.b() as f32 / 255.0,
            color.a() as f32 / 255.0,
        ]
    }
}

/// Uniform data for the GPU renderer
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    pub view_matrix: [[f32; 4]; 4],
    pub pan_offset: [f32; 2],
    pub zoom: f32,
    pub time: f32,
    pub screen_size: [f32; 2],
    pub _padding: [f32; 2],
}

impl Uniforms {
    pub fn new(pan_offset: Vec2, zoom: f32, screen_size: Vec2) -> Self {
        // Create simple identity matrix for now, let the vertex shader handle transforms
        #[rustfmt::skip]
        let identity_matrix = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        
        Self {
            view_matrix: identity_matrix,
            pan_offset: [pan_offset.x, pan_offset.y],
            zoom,
            time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f32() % 1000.0, // Animation time
            screen_size: [screen_size.x, screen_size.y],
            _padding: [0.0, 0.0],
        }
    }
}

/// GPU-accelerated node and port renderer
pub struct GpuNodeRenderer {
    node_render_pipeline: wgpu::RenderPipeline,
    port_render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    node_instance_buffer: wgpu::Buffer,
    port_instance_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    max_node_instances: usize,
    max_port_instances: usize,
}

impl GpuNodeRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        // Create node shader
        let node_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Node Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/node.wgsl").into()),
        });
        
        // Create port shader
        let port_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Port Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/port.wgsl").into()),
        });
        
        // Create vertex buffer for a quad
        #[rustfmt::skip]
        let vertices: &[f32] = &[
            // Position, TexCoord
            0.0, 0.0,  0.0, 0.0, // Bottom-left
            1.0, 0.0,  1.0, 0.0, // Bottom-right
            1.0, 1.0,  1.0, 1.0, // Top-right
            0.0, 1.0,  0.0, 1.0, // Top-left
        ];
        
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Node Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        // Create index buffer
        let indices: &[u16] = &[0, 1, 2, 2, 3, 0];
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Node Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        // Create node instance buffer
        let max_node_instances = 10000; // Support thousands of nodes
        let node_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Node Instance Buffer"),
            size: (max_node_instances * std::mem::size_of::<NodeInstanceData>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create port instance buffer
        let max_port_instances = 50000; // Support many ports for thousands of nodes
        let port_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Port Instance Buffer"),
            size: (max_port_instances * std::mem::size_of::<PortInstanceData>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create uniform buffer
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Node Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Node Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        
        // Create bind group
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Node Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        
        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Node Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Create node render pipeline
        let node_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            cache: None,
            label: Some("Node Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &node_shader,
                entry_point: "vs_main",
                buffers: &[
                    // Vertex buffer layout (position + texcoord)
                    wgpu::VertexBufferLayout {
                        array_stride: 4 * 4, // 4 floats * 4 bytes
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 8,
                                shader_location: 1,
                            },
                        ],
                    },
                    // Instance buffer layout
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<NodeInstanceData>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            // Position
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 2,
                            },
                            // Size
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 8,
                                shader_location: 3,
                            },
                            // Bevel color top
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 16,
                                shader_location: 4,
                            },
                            // Bevel color bottom
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 32,
                                shader_location: 5,
                            },
                            // Background color top
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 48,
                                shader_location: 6,
                            },
                            // Background color bottom
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 64,
                                shader_location: 7,
                            },
                            // Border color
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 80,
                                shader_location: 8,
                            },
                            // Corner radius
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32,
                                offset: 96,
                                shader_location: 9,
                            },
                            // Selected
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32,
                                offset: 100,
                                shader_location: 10,
                            },
                        ],
                    },
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // Disable culling to see if quads are facing wrong way
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 4, // Match egui's 4x MSAA
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &node_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
        });
        
        // Create port render pipeline with different vertex layout
        let port_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            cache: None,
            label: Some("Port Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &port_shader,
                entry_point: "vs_main",
                buffers: &[
                    // Vertex buffer layout (same as nodes)
                    wgpu::VertexBufferLayout {
                        array_stride: 4 * 4,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 8,
                                shader_location: 1,
                            },
                        ],
                    },
                    // Port instance buffer layout
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<PortInstanceData>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            // Position
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 2,
                            },
                            // Radius
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32,
                                offset: 8,
                                shader_location: 3,
                            },
                            // Border color
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 12,
                                shader_location: 4,
                            },
                            // Bevel color
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 28,
                                shader_location: 5,
                            },
                            // Background color
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 44,
                                shader_location: 6,
                            },
                            // Is input
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32,
                                offset: 60,
                                shader_location: 7,
                            },
                        ],
                    },
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // Disable culling to see if quads are facing wrong way
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &port_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
        });
        
        Self {
            node_render_pipeline,
            port_render_pipeline,
            vertex_buffer,
            index_buffer,
            node_instance_buffer,
            port_instance_buffer,
            uniform_buffer,
            uniform_bind_group,
            max_node_instances,
            max_port_instances,
        }
    }
    
    pub fn update_uniforms(&self, queue: &wgpu::Queue, uniforms: &Uniforms) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));
    }
    
    pub fn update_node_instances(&self, queue: &wgpu::Queue, instances: &[NodeInstanceData]) {
        if instances.len() <= self.max_node_instances {
            queue.write_buffer(
                &self.node_instance_buffer,
                0,
                bytemuck::cast_slice(instances),
            );
        }
    }
    
    pub fn update_port_instances(&self, queue: &wgpu::Queue, instances: &[PortInstanceData]) {
        if instances.len() <= self.max_port_instances {
            queue.write_buffer(
                &self.port_instance_buffer,
                0,
                bytemuck::cast_slice(instances),
            );
        }
    }
    
    pub fn render_nodes(&self, render_pass: &mut wgpu::RenderPass, instance_count: u32) {
        render_pass.set_pipeline(&self.node_render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.node_instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..instance_count);
    }
    
    pub fn render_ports(&self, render_pass: &mut wgpu::RenderPass, instance_count: u32) {
        render_pass.set_pipeline(&self.port_render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.port_instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..instance_count);
    }
}

/// Global GPU renderer instance shared across all callbacks
static GLOBAL_GPU_RENDERER: Lazy<Arc<Mutex<Option<GpuNodeRenderer>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(None))
});

/// Paint callback for GPU node and port rendering
pub struct NodeRenderCallback {
    pub nodes: Vec<NodeInstanceData>,
    pub ports: Vec<PortInstanceData>,
    pub uniforms: Uniforms,
}

/// Persistent GPU instance manager for optimal performance
pub struct GpuInstanceManager {
    node_instances: Vec<NodeInstanceData>,
    port_instances: Vec<PortInstanceData>,
    node_count: usize,
    port_count: usize,
    last_frame_node_count: usize,
    last_frame_port_count: usize,
    // Optimization: only rebuild when needed
    needs_full_rebuild: bool,
}

impl GpuInstanceManager {
    pub fn new() -> Self {
        Self {
            node_instances: Vec::with_capacity(10000),
            port_instances: Vec::with_capacity(50000),
            node_count: 0,
            port_count: 0,
            last_frame_node_count: 0,
            last_frame_port_count: 0,
            needs_full_rebuild: true,
        }
    }
    
    pub fn update_instances(
        &mut self,
        nodes: &HashMap<NodeId, Node>,
        selected_nodes: &std::collections::HashSet<NodeId>,
        connecting_from: Option<(NodeId, usize, bool)>,
    ) -> (&[NodeInstanceData], &[PortInstanceData]) {
        let current_node_count = nodes.len();
        let estimated_port_count = current_node_count * 3; // Rough estimate
        
        // Only rebuild if node count changed significantly or forced rebuild
        if self.needs_full_rebuild || 
           current_node_count != self.last_frame_node_count ||
           (current_node_count > 0 && (self.last_frame_node_count as f32 / current_node_count as f32 - 1.0).abs() > 0.1) {
            
            // Only log rebuilds for significant changes
            if current_node_count > 1000 || self.last_frame_node_count > 1000 {
                println!("GPU Instance Manager: Rebuilding instances for {} nodes (was {} nodes)", 
                         current_node_count, self.last_frame_node_count);
            }
            self.rebuild_all_instances(nodes, selected_nodes, connecting_from);
            self.last_frame_node_count = current_node_count;
            self.needs_full_rebuild = false;
        }
        
        (&self.node_instances[..self.node_count], &self.port_instances[..self.port_count])
    }
    
    fn rebuild_all_instances(
        &mut self,
        nodes: &HashMap<NodeId, Node>,
        selected_nodes: &std::collections::HashSet<NodeId>,
        connecting_from: Option<(NodeId, usize, bool)>,
    ) {
        self.node_instances.clear();
        self.port_instances.clear();
        
        for (id, node) in nodes {
            let selected = selected_nodes.contains(id);
            let instance = NodeInstanceData::from_node(node, selected, 1.0); // Don't apply zoom here
            self.node_instances.push(instance);
            
            // Add port instances for this node
            for (port_idx, port) in node.inputs.iter().enumerate() {
                let is_connecting = if let Some((conn_node, conn_port, is_input)) = connecting_from {
                    conn_node == *id && conn_port == port_idx && is_input
                } else {
                    false
                };
                
                let port_instance = PortInstanceData::from_port(port.position, 5.0, is_connecting, true);
                self.port_instances.push(port_instance);
            }
            
            for (port_idx, port) in node.outputs.iter().enumerate() {
                let is_connecting = if let Some((conn_node, conn_port, is_input)) = connecting_from {
                    conn_node == *id && conn_port == port_idx && !is_input
                } else {
                    false
                };
                
                let port_instance = PortInstanceData::from_port(port.position, 5.0, is_connecting, false);
                self.port_instances.push(port_instance);
            }
        }
        
        self.node_count = self.node_instances.len();
        self.port_count = self.port_instances.len();
    }
    
    pub fn force_rebuild(&mut self) {
        self.needs_full_rebuild = true;
    }
}

impl NodeRenderCallback {
    pub fn new(
        nodes: HashMap<NodeId, Node>,
        selected_nodes: &std::collections::HashSet<NodeId>,
        connecting_from: Option<(NodeId, usize, bool)>,
        pan_offset: Vec2,
        zoom: f32,
        screen_size: Vec2,
    ) -> Self {
        let mut node_instances = Vec::new();
        let mut port_instances = Vec::new();
        
        for (id, node) in &nodes {
            let selected = selected_nodes.contains(&id);
            let instance = NodeInstanceData::from_node(&node, selected, zoom);
            // Debug first few nodes
            if node_instances.len() < 3 {
                println!("GPU: Node at position [{}, {}], size [{}, {}]", 
                    instance.position[0], instance.position[1],
                    instance.size[0], instance.size[1]);
            }
            node_instances.push(instance);
            
            // Add port instances for this node
            let node_rect = node.get_rect();
            
            // Input ports on the left
            for (port_idx, port) in node.inputs.iter().enumerate() {
                let port_pos = port.position; // Use the actual port position from the port object
                
                let is_connecting = if let Some((conn_node, conn_port, is_input)) = connecting_from {
                    conn_node == *id && conn_port == port_idx && is_input
                } else {
                    false
                };
                
                let port_instance = PortInstanceData::from_port(port_pos, 5.0, is_connecting, true);
                port_instances.push(port_instance);
            }
            
            // Output ports on the right
            for (port_idx, port) in node.outputs.iter().enumerate() {
                let port_pos = port.position; // Use the actual port position from the port object
                
                let is_connecting = if let Some((conn_node, conn_port, is_input)) = connecting_from {
                    conn_node == *id && conn_port == port_idx && !is_input
                } else {
                    false
                };
                
                let port_instance = PortInstanceData::from_port(port_pos, 5.0, is_connecting, false);
                port_instances.push(port_instance);
            }
        }
        
        let uniforms = Uniforms::new(pan_offset, zoom, screen_size);
        
        Self {
            nodes: node_instances,
            ports: port_instances,
            uniforms,
        }
    }
    
    /// Create from pre-built instances (optimized path)
    pub fn from_instances(
        node_instances: &[NodeInstanceData],
        port_instances: &[PortInstanceData],
        pan_offset: Vec2,
        zoom: f32,
        screen_size: Vec2,
    ) -> Self {
        let uniforms = Uniforms::new(pan_offset, zoom, screen_size);
        
        Self {
            nodes: node_instances.to_vec(),
            ports: port_instances.to_vec(),
            uniforms,
        }
    }
}

impl egui_wgpu::CallbackTrait for NodeRenderCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        _callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        // Reduce debug output for performance
        if self.nodes.len() >= 1000 {
            println!("🟢 GPU: prepare() CALLED with {} nodes!", self.nodes.len());
        }
        
        // Get or create the global renderer
        let mut renderer_lock = GLOBAL_GPU_RENDERER.lock().unwrap();
        if renderer_lock.is_none() {
            // Use the format that matches egui's surface format
            let format = wgpu::TextureFormat::Bgra8Unorm; // Match egui's surface format
            println!("🟢 GPU: Initializing GLOBAL renderer with format: {:?}", format);
            *renderer_lock = Some(GpuNodeRenderer::new(device, format));
        }
        
        if let Some(renderer) = renderer_lock.as_ref() {
            renderer.update_uniforms(queue, &self.uniforms);
            renderer.update_node_instances(queue, &self.nodes);
            renderer.update_port_instances(queue, &self.ports);
        }
        Vec::new()
    }
    
    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        _callback_resources: &egui_wgpu::CallbackResources,
    ) {
        // Reduce debug output for performance
        
        let renderer_lock = GLOBAL_GPU_RENDERER.lock().unwrap();
        if let Some(renderer) = renderer_lock.as_ref() {
            // Set viewport to match the clip rect
            render_pass.set_viewport(
                info.clip_rect.min.x,
                info.clip_rect.min.y,
                info.clip_rect.width(),
                info.clip_rect.height(),
                0.0,
                1.0,
            );
            
            // Render nodes first (background layer)
            renderer.render_nodes(render_pass, self.nodes.len() as u32);
            
            // Render ports on top
            renderer.render_ports(render_pass, self.ports.len() as u32);
        }
    }
}