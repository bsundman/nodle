//! Core GPU renderer for nodes and ports
//! 
//! This module provides the main [`GpuNodeRenderer`] struct which manages wgpu 
//! rendering pipelines, buffers, and draw calls for efficient instanced rendering
//! of nodes and ports.

use super::canvas_instance::{NodeInstanceData, PortInstanceData, ButtonInstanceData, FlagInstanceData, Uniforms};
use super::config::GraphicsConfig;
use std::sync::{Arc, Mutex};
use eframe::wgpu::util::DeviceExt;
use once_cell::sync::Lazy;

/// GPU-accelerated node, port, button, and flag renderer
pub struct GpuNodeRenderer {
    node_render_pipeline: eframe::wgpu::RenderPipeline,
    port_render_pipeline: eframe::wgpu::RenderPipeline,
    button_render_pipeline: eframe::wgpu::RenderPipeline,
    flag_render_pipeline: eframe::wgpu::RenderPipeline,
    vertex_buffer: eframe::wgpu::Buffer,
    index_buffer: eframe::wgpu::Buffer,
    node_instance_buffer: eframe::wgpu::Buffer,
    port_instance_buffer: eframe::wgpu::Buffer,
    button_instance_buffer: eframe::wgpu::Buffer,
    flag_instance_buffer: eframe::wgpu::Buffer,
    uniform_buffer: eframe::wgpu::Buffer,
    uniform_bind_group: eframe::wgpu::BindGroup,
    max_node_instances: usize,
    max_port_instances: usize,
    max_button_instances: usize,
    max_flag_instances: usize,
}

impl GpuNodeRenderer {
    pub fn new(device: &eframe::wgpu::Device, format: eframe::wgpu::TextureFormat) -> Self {
        // Create node shader
        let node_shader = device.create_shader_module(eframe::wgpu::ShaderModuleDescriptor {
            label: Some("Node Shader"),
            source: eframe::wgpu::ShaderSource::Wgsl(include_str!("./shaders/node.wgsl").into()),
        });
        
        // Create port shader
        let port_shader = device.create_shader_module(eframe::wgpu::ShaderModuleDescriptor {
            label: Some("Port Shader"),
            source: eframe::wgpu::ShaderSource::Wgsl(include_str!("./shaders/port.wgsl").into()),
        });
        
        // Create button shader
        let button_shader = device.create_shader_module(eframe::wgpu::ShaderModuleDescriptor {
            label: Some("Button Shader"),
            source: eframe::wgpu::ShaderSource::Wgsl(include_str!("./shaders/button.wgsl").into()),
        });
        
        // Create flag shader
        let flag_shader = device.create_shader_module(eframe::wgpu::ShaderModuleDescriptor {
            label: Some("Flag Shader"),
            source: eframe::wgpu::ShaderSource::Wgsl(include_str!("./shaders/flag.wgsl").into()),
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
        
        let vertex_buffer = device.create_buffer_init(&eframe::wgpu::util::BufferInitDescriptor {
            label: Some("Node Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: eframe::wgpu::BufferUsages::VERTEX,
        });
        
        // Create index buffer
        let indices: &[u16] = &[0, 1, 2, 2, 3, 0];
        let index_buffer = device.create_buffer_init(&eframe::wgpu::util::BufferInitDescriptor {
            label: Some("Node Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: eframe::wgpu::BufferUsages::INDEX,
        });
        
        // Create node instance buffer
        let max_node_instances = 10000; // Support thousands of nodes
        let node_instance_buffer = device.create_buffer(&eframe::wgpu::BufferDescriptor {
            label: Some("Node Instance Buffer"),
            size: (max_node_instances * std::mem::size_of::<NodeInstanceData>()) as u64,
            usage: eframe::wgpu::BufferUsages::VERTEX | eframe::wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create port instance buffer
        let max_port_instances = 50000; // Support many ports for thousands of nodes
        let port_instance_buffer = device.create_buffer(&eframe::wgpu::BufferDescriptor {
            label: Some("Port Instance Buffer"),
            size: (max_port_instances * std::mem::size_of::<PortInstanceData>()) as u64,
            usage: eframe::wgpu::BufferUsages::VERTEX | eframe::wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create button instance buffer
        let max_button_instances = 1000; // Much fewer buttons expected
        let button_instance_buffer = device.create_buffer(&eframe::wgpu::BufferDescriptor {
            label: Some("Button Instance Buffer"),
            size: (max_button_instances * std::mem::size_of::<ButtonInstanceData>()) as u64,
            usage: eframe::wgpu::BufferUsages::VERTEX | eframe::wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create flag instance buffer
        let max_flag_instances = 10000; // One flag per node
        let flag_instance_buffer = device.create_buffer(&eframe::wgpu::BufferDescriptor {
            label: Some("Flag Instance Buffer"),
            size: (max_flag_instances * std::mem::size_of::<FlagInstanceData>()) as u64,
            usage: eframe::wgpu::BufferUsages::VERTEX | eframe::wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create uniform buffer
        let uniform_buffer = device.create_buffer(&eframe::wgpu::BufferDescriptor {
            label: Some("Node Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: eframe::wgpu::BufferUsages::UNIFORM | eframe::wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&eframe::wgpu::BindGroupLayoutDescriptor {
            label: Some("Node Bind Group Layout"),
            entries: &[eframe::wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: eframe::wgpu::ShaderStages::VERTEX | eframe::wgpu::ShaderStages::FRAGMENT,
                ty: eframe::wgpu::BindingType::Buffer {
                    ty: eframe::wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        
        // Create bind group
        let uniform_bind_group = device.create_bind_group(&eframe::wgpu::BindGroupDescriptor {
            label: Some("Node Bind Group"),
            layout: &bind_group_layout,
            entries: &[eframe::wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        
        
        
        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&eframe::wgpu::PipelineLayoutDescriptor {
            label: Some("Node Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Create node render pipeline
        let node_render_pipeline = device.create_render_pipeline(&eframe::wgpu::RenderPipelineDescriptor {
            cache: None,
            label: Some("Node Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: eframe::wgpu::VertexState {
                module: &node_shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    // Vertex buffer layout (position + texcoord)
                    eframe::wgpu::VertexBufferLayout {
                        array_stride: 4 * 4, // 4 floats * 4 bytes
                        step_mode: eframe::wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            },
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x2,
                                offset: 8,
                                shader_location: 1,
                            },
                        ],
                    },
                    // Instance buffer layout
                    eframe::wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<NodeInstanceData>() as u64,
                        step_mode: eframe::wgpu::VertexStepMode::Instance,
                        attributes: &[
                            // Position
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 2,
                            },
                            // Size
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x2,
                                offset: 8,
                                shader_location: 3,
                            },
                            // Bevel color top
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x4,
                                offset: 16,
                                shader_location: 4,
                            },
                            // Bevel color bottom
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x4,
                                offset: 32,
                                shader_location: 5,
                            },
                            // Background color top
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x4,
                                offset: 48,
                                shader_location: 6,
                            },
                            // Background color bottom
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x4,
                                offset: 64,
                                shader_location: 7,
                            },
                            // Border color
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x4,
                                offset: 80,
                                shader_location: 8,
                            },
                            // Corner radius
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32,
                                offset: 96,
                                shader_location: 9,
                            },
                            // Selected
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32,
                                offset: 100,
                                shader_location: 10,
                            },
                            // Left button active
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32,
                                offset: 104,
                                shader_location: 11,
                            },
                            // Right button active
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32,
                                offset: 108,
                                shader_location: 12,
                            },
                        ],
                    },
                ],
                compilation_options: eframe::wgpu::PipelineCompilationOptions::default(),
            },
            primitive: eframe::wgpu::PrimitiveState {
                topology: eframe::wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: eframe::wgpu::FrontFace::Ccw,
                cull_mode: None, // Disable culling to see if quads are facing wrong way
                unclipped_depth: false,
                polygon_mode: eframe::wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: GraphicsConfig::global().multisample_state(),
            fragment: Some(eframe::wgpu::FragmentState {
                module: &node_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(eframe::wgpu::ColorTargetState {
                    format,
                    blend: Some(eframe::wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: eframe::wgpu::ColorWrites::ALL,
                })],
                compilation_options: eframe::wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
        });
        
        // Create port render pipeline with different vertex layout
        let port_render_pipeline = device.create_render_pipeline(&eframe::wgpu::RenderPipelineDescriptor {
            cache: None,
            label: Some("Port Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: eframe::wgpu::VertexState {
                module: &port_shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    // Vertex buffer layout (same as nodes)
                    eframe::wgpu::VertexBufferLayout {
                        array_stride: 4 * 4,
                        step_mode: eframe::wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            },
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x2,
                                offset: 8,
                                shader_location: 1,
                            },
                        ],
                    },
                    // Port instance buffer layout
                    eframe::wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<PortInstanceData>() as u64,
                        step_mode: eframe::wgpu::VertexStepMode::Instance,
                        attributes: &[
                            // Position
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 2,
                            },
                            // Radius
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32,
                                offset: 8,
                                shader_location: 3,
                            },
                            // Border color
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x4,
                                offset: 12,
                                shader_location: 4,
                            },
                            // Bevel color
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x4,
                                offset: 28,
                                shader_location: 5,
                            },
                            // Background color
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x4,
                                offset: 44,
                                shader_location: 6,
                            },
                            // Is input
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32,
                                offset: 60,
                                shader_location: 7,
                            },
                        ],
                    },
                ],
                compilation_options: eframe::wgpu::PipelineCompilationOptions::default(),
            },
            primitive: eframe::wgpu::PrimitiveState {
                topology: eframe::wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: eframe::wgpu::FrontFace::Ccw,
                cull_mode: None, // Disable culling to see if quads are facing wrong way
                unclipped_depth: false,
                polygon_mode: eframe::wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: GraphicsConfig::global().multisample_state(),
            fragment: Some(eframe::wgpu::FragmentState {
                module: &port_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(eframe::wgpu::ColorTargetState {
                    format,
                    blend: Some(eframe::wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: eframe::wgpu::ColorWrites::ALL,
                })],
                compilation_options: eframe::wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
        });
        
        // Create button render pipeline with button-specific vertex layout
        let button_render_pipeline = device.create_render_pipeline(&eframe::wgpu::RenderPipelineDescriptor {
            cache: None,
            label: Some("Button Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: eframe::wgpu::VertexState {
                module: &button_shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    // Vertex buffer layout (same as nodes and ports)
                    eframe::wgpu::VertexBufferLayout {
                        array_stride: 4 * 4, // 4 floats * 4 bytes
                        step_mode: eframe::wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            },
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x2,
                                offset: 8,
                                shader_location: 1,
                            },
                        ],
                    },
                    // Button instance buffer layout
                    eframe::wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<ButtonInstanceData>() as u64,
                        step_mode: eframe::wgpu::VertexStepMode::Instance,
                        attributes: &[
                            // Position
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 2,
                            },
                            // Radius
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32,
                                offset: 8,
                                shader_location: 3,
                            },
                            // Center color
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x4,
                                offset: 12,
                                shader_location: 4,
                            },
                            // Outer color
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x4,
                                offset: 28,
                                shader_location: 5,
                            },
                        ],
                    },
                ],
                compilation_options: eframe::wgpu::PipelineCompilationOptions::default(),
            },
            primitive: eframe::wgpu::PrimitiveState {
                topology: eframe::wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: eframe::wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: eframe::wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: GraphicsConfig::global().multisample_state(),
            fragment: Some(eframe::wgpu::FragmentState {
                module: &button_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(eframe::wgpu::ColorTargetState {
                    format,
                    blend: Some(eframe::wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: eframe::wgpu::ColorWrites::ALL,
                })],
                compilation_options: eframe::wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
        });
        
        // Create flag render pipeline with flag-specific vertex layout
        let flag_render_pipeline = device.create_render_pipeline(&eframe::wgpu::RenderPipelineDescriptor {
            cache: None,
            label: Some("Flag Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: eframe::wgpu::VertexState {
                module: &flag_shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    // Vertex buffer layout (same as nodes and ports)
                    eframe::wgpu::VertexBufferLayout {
                        array_stride: 4 * 4, // 4 floats * 4 bytes
                        step_mode: eframe::wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            },
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x2,
                                offset: 8,
                                shader_location: 1,
                            },
                        ],
                    },
                    // Flag instance buffer layout
                    eframe::wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<FlagInstanceData>() as u64,
                        step_mode: eframe::wgpu::VertexStepMode::Instance,
                        attributes: &[
                            // Position
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 2,
                            },
                            // Radius
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32,
                                offset: 8,
                                shader_location: 3,
                            },
                            // Border color
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x4,
                                offset: 12,
                                shader_location: 4,
                            },
                            // Bevel color
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x4,
                                offset: 28,
                                shader_location: 5,
                            },
                            // Background color
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32x4,
                                offset: 44,
                                shader_location: 6,
                            },
                            // Is visible
                            eframe::wgpu::VertexAttribute {
                                format: eframe::wgpu::VertexFormat::Float32,
                                offset: 60,
                                shader_location: 7,
                            },
                        ],
                    },
                ],
                compilation_options: eframe::wgpu::PipelineCompilationOptions::default(),
            },
            primitive: eframe::wgpu::PrimitiveState {
                topology: eframe::wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: eframe::wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: eframe::wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: GraphicsConfig::global().multisample_state(),
            fragment: Some(eframe::wgpu::FragmentState {
                module: &flag_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(eframe::wgpu::ColorTargetState {
                    format,
                    blend: Some(eframe::wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: eframe::wgpu::ColorWrites::ALL,
                })],
                compilation_options: eframe::wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
        });
        
        Self {
            node_render_pipeline,
            port_render_pipeline,
            button_render_pipeline,
            flag_render_pipeline,
            vertex_buffer,
            index_buffer,
            node_instance_buffer,
            port_instance_buffer,
            button_instance_buffer,
            flag_instance_buffer,
            uniform_buffer,
            uniform_bind_group,
            max_node_instances,
            max_port_instances,
            max_button_instances,
            max_flag_instances,
        }
    }
    
    pub fn update_uniforms(&self, queue: &eframe::wgpu::Queue, uniforms: &Uniforms) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));
    }
    
    pub fn update_node_instances(&self, queue: &eframe::wgpu::Queue, instances: &[NodeInstanceData]) {
        if instances.len() <= self.max_node_instances {
            queue.write_buffer(
                &self.node_instance_buffer,
                0,
                bytemuck::cast_slice(instances),
            );
        }
    }
    
    pub fn update_port_instances(&self, queue: &eframe::wgpu::Queue, instances: &[PortInstanceData]) {
        if instances.len() <= self.max_port_instances {
            queue.write_buffer(
                &self.port_instance_buffer,
                0,
                bytemuck::cast_slice(instances),
            );
        }
    }
    
    pub fn update_button_instances(&self, queue: &eframe::wgpu::Queue, instances: &[ButtonInstanceData]) {
        if instances.len() <= self.max_button_instances {
            queue.write_buffer(
                &self.button_instance_buffer,
                0,
                bytemuck::cast_slice(instances),
            );
        }
    }
    
    pub fn update_flag_instances(&self, queue: &eframe::wgpu::Queue, instances: &[FlagInstanceData]) {
        if instances.len() <= self.max_flag_instances {
            queue.write_buffer(
                &self.flag_instance_buffer,
                0,
                bytemuck::cast_slice(instances),
            );
        }
    }
    
    
    pub fn render_nodes(&self, render_pass: &mut eframe::wgpu::RenderPass, instance_count: u32) {
        render_pass.set_pipeline(&self.node_render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.node_instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), eframe::wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..instance_count);
    }
    
    pub fn render_ports(&self, render_pass: &mut eframe::wgpu::RenderPass, instance_count: u32) {
        render_pass.set_pipeline(&self.port_render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.port_instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), eframe::wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..instance_count);
    }
    
    pub fn render_buttons(&self, render_pass: &mut eframe::wgpu::RenderPass, instance_count: u32) {
        render_pass.set_pipeline(&self.button_render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.button_instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), eframe::wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..instance_count);
    }
    
    pub fn render_flags(&self, render_pass: &mut eframe::wgpu::RenderPass, instance_count: u32) {
        render_pass.set_pipeline(&self.flag_render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.flag_instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), eframe::wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..instance_count);
    }
    
}

/// Global GPU renderer instance shared across all callbacks
pub static GLOBAL_GPU_RENDERER: Lazy<Arc<Mutex<Option<GpuNodeRenderer>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(None))
});