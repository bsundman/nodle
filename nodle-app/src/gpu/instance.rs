//! GPU instance data structures and management
//! 
//! This module contains all the GPU instance data structures and the instance manager
//! that efficiently manages node and port instances for GPU rendering.

use egui::{Color32, Pos2, Vec2};
use crate::nodes::{Node, NodeId};
use std::collections::{HashMap, HashSet};

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

impl NodeInstanceData {
    pub fn from_node(node: &Node, selected: bool, _zoom: f32) -> Self {
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
}

impl PortInstanceData {
    pub fn from_port(position: Pos2, radius: f32, is_connecting: bool, is_input: bool) -> Self {
        let border_color = if is_connecting {
            Color32::from_rgb(100, 150, 255) // Blue when connecting
        } else {
            Color32::from_rgb(64, 64, 64)    // Dark grey normally
        };
        
        let bevel_color = Color32::from_rgb(38, 38, 38); // Dark grey bevel
        
        let background_color = if is_input {
            Color32::from_rgb(90, 160, 120)  // Brighter green for input ports
        } else {
            Color32::from_rgb(160, 90, 90)   // Brighter red for output ports
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

/// Persistent GPU instance manager for optimal performance
pub struct GpuInstanceManager {
    node_instances: Vec<NodeInstanceData>,
    port_instances: Vec<PortInstanceData>,
    node_count: usize,
    port_count: usize,
    last_frame_node_count: usize,
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
            needs_full_rebuild: true,
        }
    }
    
    pub fn update_instances(
        &mut self,
        nodes: &HashMap<NodeId, Node>,
        selected_nodes: &HashSet<NodeId>,
        connecting_from: Option<(NodeId, usize, bool)>,
    ) -> (&[NodeInstanceData], &[PortInstanceData]) {
        let current_node_count = nodes.len();
        let _estimated_port_count = current_node_count * 3; // Rough estimate
        
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
        selected_nodes: &HashSet<NodeId>,
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