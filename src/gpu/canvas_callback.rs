//! GPU rendering callback implementation
//!
//! This module contains the NodeRenderCallback struct and its implementation
//! for egui paint callbacks, handling GPU-accelerated node and port rendering.

use egui::Vec2;
use crate::nodes::{Node, NodeId};
use super::{NodeInstanceData, PortInstanceData, ButtonInstanceData, FlagInstanceData, Uniforms, GLOBAL_GPU_RENDERER};
use std::collections::HashMap;

/// Paint callback for GPU node, port, button, and flag rendering
pub struct NodeRenderCallback {
    pub nodes: Vec<NodeInstanceData>,
    pub ports: Vec<PortInstanceData>,
    pub buttons: Vec<ButtonInstanceData>,
    pub flags: Vec<FlagInstanceData>,
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
        let mut button_instances = Vec::new();
        let mut flag_instances = Vec::new();
        
        for (id, node) in &nodes {
            let selected = selected_nodes.contains(&id);
            let instance = NodeInstanceData::from_node(&node, selected, zoom);
            // Note: Debug output removed for cleaner production code
            node_instances.push(instance);
            
            // Add flag instance for this node
            let flag_position = node.get_flag_position();
            let flag_instance = FlagInstanceData::from_flag(flag_position, 5.0, node.visible);
            flag_instances.push(flag_instance);
            
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
            buttons: button_instances,
            flags: flag_instances,
            uniforms,
        }
    }
    
    /// Create from pre-built instances (optimized path)
    pub fn from_instances(
        node_instances: &[NodeInstanceData],
        port_instances: &[PortInstanceData],
        button_instances: &[ButtonInstanceData],
        flag_instances: &[FlagInstanceData],
        pan_offset: Vec2,
        zoom: f32,
        screen_size: Vec2,
    ) -> Self {
        let uniforms = Uniforms::new(pan_offset, zoom, screen_size);
        
        Self {
            nodes: node_instances.to_vec(),
            ports: port_instances.to_vec(),
            buttons: button_instances.to_vec(),
            flags: flag_instances.to_vec(),
            uniforms,
        }
    }
}

impl egui_wgpu::CallbackTrait for NodeRenderCallback {
    fn prepare(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut eframe::wgpu::CommandEncoder,
        _callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<eframe::wgpu::CommandBuffer> {
        // Update GPU resources
        
        // Get or create the global renderer
        let mut renderer_lock = match GLOBAL_GPU_RENDERER.lock() {
            Ok(lock) => lock,
            Err(_) => return Vec::new(), // Skip rendering if mutex is poisoned
        };
        if renderer_lock.is_none() {
            // Use the format that matches egui's surface format
            let format = eframe::wgpu::TextureFormat::Bgra8Unorm; // Match egui's surface format
            // Initialize global renderer
            *renderer_lock = Some(super::GpuNodeRenderer::new(device, format));
        }
        
        if let Some(renderer) = renderer_lock.as_ref() {
            renderer.update_uniforms(queue, &self.uniforms);
            renderer.update_node_instances(queue, &self.nodes);
            renderer.update_port_instances(queue, &self.ports);
            renderer.update_button_instances(queue, &self.buttons);
            renderer.update_flag_instances(queue, &self.flags);
        }
        Vec::new()
    }
    
    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut eframe::wgpu::RenderPass<'static>,
        _callback_resources: &egui_wgpu::CallbackResources,
    ) {
        // Reduce debug output for performance
        
        let renderer_lock = match GLOBAL_GPU_RENDERER.lock() {
            Ok(lock) => lock,
            Err(_) => return, // Skip rendering if mutex is poisoned
        };
        if let Some(renderer) = renderer_lock.as_ref() {
            // Set viewport to match the full screen area scaled by DPI
            // Calculate DPI-aware viewport dimensions
            let pixels_per_point = info.pixels_per_point;
            let viewport_width = (self.uniforms.screen_size[0] * pixels_per_point).max(1.0);
            let viewport_height = (self.uniforms.screen_size[1] * pixels_per_point).max(1.0);
            
            // Validate against actual screen size
            let render_target_width = info.screen_size_px[0] as f32;
            let render_target_height = info.screen_size_px[1] as f32;
            let clamped_width = viewport_width.min(render_target_width);
            let clamped_height = viewport_height.min(render_target_height);
            
            render_pass.set_viewport(
                0.0,
                0.0,
                clamped_width,
                clamped_height,
                0.0,
                1.0,
            );
            
            // Render nodes first (background layer)
            renderer.render_nodes(render_pass, self.nodes.len() as u32);
            
            // Render ports on top
            renderer.render_ports(render_pass, self.ports.len() as u32);
            
            // Render buttons on top of everything
            renderer.render_buttons(render_pass, self.buttons.len() as u32);
            
            // Render flags on top of everything
            renderer.render_flags(render_pass, self.flags.len() as u32);
            
        }
    }
}