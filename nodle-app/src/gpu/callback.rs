//! GPU rendering callback implementation
//!
//! This module contains the NodeRenderCallback struct and its implementation
//! for egui paint callbacks, handling GPU-accelerated node and port rendering.

use egui::Vec2;
use crate::nodes::{Node, NodeId};
use super::{NodeInstanceData, PortInstanceData, Uniforms, GLOBAL_GPU_RENDERER};
use std::collections::HashMap;

/// Paint callback for GPU node and port rendering
pub struct NodeRenderCallback {
    pub nodes: Vec<NodeInstanceData>,
    pub ports: Vec<PortInstanceData>,
    pub uniforms: Uniforms,
}

impl NodeRenderCallback {
    /// Create callback from node graph data
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
            // Note: Debug output removed for cleaner production code
            node_instances.push(instance);
            
            // Add port instances for this node
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
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
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
            *renderer_lock = Some(super::GpuNodeRenderer::new(device, format));
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