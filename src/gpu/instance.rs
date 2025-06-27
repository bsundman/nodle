//! GPU instance data structures and management
//! 
//! This module contains all the GPU instance data structures and the instance manager
//! that efficiently manages node and port instances for GPU rendering.

use egui::{Color32, Pos2, Vec2};
use crate::nodes::{Node, NodeId};
use std::collections::{HashMap, HashSet};

/// Button color variants for gradient colorization
#[derive(Debug, Clone, Copy)]
enum ButtonColor {
    Green,
    Red,
}

/// Button type for positioning and state
#[derive(Debug, Clone, Copy)]
pub enum ButtonType {
    Left,
    Right,
}

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
    pub _padding: [f32; 3], // Padding for alignment
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

/// Instance data for a single visibility flag in GPU memory
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FlagInstanceData {
    pub position: [f32; 2],
    pub radius: f32,
    pub border_color: [f32; 4],         // Border color
    pub bevel_color: [f32; 4],          // Bevel color (dark grey)
    pub background_color: [f32; 4],     // Background color (flag color)
    pub is_visible: f32,                // 1.0 if visible, 0.0 if hidden
    pub _padding: [f32; 2],
}

/// Instance data for a single radial button in GPU memory (for Viewport nodes)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ButtonInstanceData {
    pub position: [f32; 2],             // Button center position
    pub radius: f32,                    // Button radius (10.0 base)
    pub center_color: [f32; 4],         // Center color (bright green)
    pub outer_color: [f32; 4],          // Outer edge color (dark green)
    pub _padding: [f32; 3],
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
        
        // All nodes use same colors - no special cases
        let (bevel_top, bevel_bottom) = (Color32::from_rgb(166, 166, 166), Color32::from_rgb(38, 38, 38));
        let (background_top, background_bottom) = (Color32::from_rgb(127, 127, 127), Color32::from_rgb(64, 64, 64));
        
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
    
    /// Create a visibility toggle port instance with just border and bevel, transparent center
    pub fn from_visibility_toggle(position: Pos2, radius: f32, _is_visible: bool) -> Self {
        let border_color = Color32::from_rgb(64, 64, 64); // Dark grey border
        let bevel_color = Color32::from_rgb(38, 38, 38); // Dark grey bevel
        
        // Transparent background - shows whatever is behind it
        let background_color = Color32::TRANSPARENT;
        
        Self {
            position: [position.x, position.y],
            radius,
            border_color: Self::color_to_array(border_color),
            bevel_color: Self::color_to_array(bevel_color),
            background_color: Self::color_to_array(background_color),
            is_input: 0.0, // Use output port styling
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

impl FlagInstanceData {
    pub fn from_flag(position: Pos2, radius: f32, is_visible: bool) -> Self {
        let border_color = if is_visible {
            Color32::from_rgb(100, 150, 255) // Blue when visible (matches port selected color)
        } else {
            Color32::from_rgb(64, 64, 64)    // Dark grey when hidden
        };
        
        let bevel_color = Color32::from_rgb(38, 38, 38); // Dark grey bevel
        
        let background_color = if is_visible {
            Color32::from_rgb(100, 150, 255) // Blue center when visible (matches border)
        } else {
            Color32::TRANSPARENT             // Transparent when hidden
        };
        
        Self {
            position: [position.x, position.y],
            radius,
            border_color: Self::color_to_array(border_color),
            bevel_color: Self::color_to_array(bevel_color),
            background_color: Self::color_to_array(background_color),
            is_visible: if is_visible { 1.0 } else { 0.0 },
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


impl ButtonInstanceData {
    pub fn from_viewport_node(node: &Node) -> Self {
        // For now, create a left button (green) - later we'll need to create both buttons separately
        Self::create_button(node, ButtonType::Left)
    }
    
    /// Create a button with specific type and colorization
    pub fn create_button(node: &Node, button_type: ButtonType) -> Self {
        // Calculate button position (matches CPU implementation)
        let button_radius = 10.0; // Base radius (zoom applied in shader)
        let margin = button_radius; // Equal distance from edges
        
        // Position depends on button type
        let button_center = match button_type {
            ButtonType::Left => Pos2::new(
                node.position.x + margin + button_radius, // Left edge + margin + radius
                node.position.y + node.size.y / 2.0,     // Centered vertically
            ),
            ButtonType::Right => Pos2::new(
                node.position.x + node.size.x - margin - button_radius, // Right edge - margin - radius
                node.position.y + node.size.y / 2.0,                   // Centered vertically
            ),
        };
        
        // Determine color and active state
        let (color_type, is_active) = match button_type {
            ButtonType::Left => (ButtonColor::Green, node.button_states[0]),
            ButtonType::Right => (ButtonColor::Red, node.button_states[1]),
        };
        
        // Simple colors matching CPU version exactly
        let (center_color, outer_color) = match color_type {
            ButtonColor::Green => {
                if is_active {
                    (Color32::from_rgb(120, 200, 120), Color32::from_rgb(60, 120, 60))
                } else {
                    (Color32::from_rgb(90, 160, 90), Color32::from_rgb(45, 90, 45))
                }
            }
            ButtonColor::Red => {
                if is_active {
                    (Color32::from_rgb(200, 120, 120), Color32::from_rgb(120, 60, 60))
                } else {
                    (Color32::from_rgb(160, 90, 90), Color32::from_rgb(90, 45, 45))
                }
            }
        };
        
        Self {
            position: [button_center.x, button_center.y],
            radius: button_radius,
            center_color: Self::color_to_array(center_color),
            outer_color: Self::color_to_array(outer_color),
            _padding: [0.0, 0.0, 0.0],
        }
    }
    
    /// Colorize the gradient top color while preserving luminance and depth
    fn colorize_gradient_top(base_color: Color32, color: ButtonColor) -> Color32 {
        let luminance = base_color.r() as f32 / 255.0; // Use red channel as luminance since it's grey
        match color {
            ButtonColor::Green => {
                // Green tint with much stronger colorization and brightness boost
                Color32::from_rgb(
                    (luminance * 0.3 * 255.0) as u8,  // Much darker red
                    ((luminance * 1.5 + 0.2) * 255.0).min(255.0) as u8, // Significantly boosted green + brightness
                    (luminance * 0.3 * 255.0) as u8,  // Much darker blue
                )
            }
            ButtonColor::Red => {
                // Red tint with much stronger colorization and brightness boost
                Color32::from_rgb(
                    ((luminance * 1.5 + 0.2) * 255.0).min(255.0) as u8, // Significantly boosted red + brightness
                    (luminance * 0.2 * 255.0) as u8,  // Much darker green
                    (luminance * 0.2 * 255.0) as u8,  // Much darker blue
                )
            }
        }
    }
    
    /// Colorize the gradient bottom color while preserving luminance and depth
    fn colorize_gradient_bottom(base_color: Color32, color: ButtonColor) -> Color32 {
        let luminance = base_color.r() as f32 / 255.0; // Use red channel as luminance since it's grey
        match color {
            ButtonColor::Green => {
                // Green tint with much stronger colorization (much darker for contrast)
                Color32::from_rgb(
                    (luminance * 0.1 * 255.0) as u8,  // Very dark red
                    (luminance * 0.8 * 255.0) as u8,  // Reduced green (darker base)
                    (luminance * 0.1 * 255.0) as u8,  // Very dark blue
                )
            }
            ButtonColor::Red => {
                // Red tint with much stronger colorization (much darker for contrast)
                Color32::from_rgb(
                    (luminance * 0.8 * 255.0) as u8,  // Reduced red (darker base)
                    (luminance * 0.1 * 255.0) as u8,  // Very dark green
                    (luminance * 0.1 * 255.0) as u8,  // Very dark blue
                )
            }
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
    button_instances: Vec<ButtonInstanceData>,
    flag_instances: Vec<FlagInstanceData>,
    node_count: usize,
    port_count: usize,
    button_count: usize,
    flag_count: usize,
    last_frame_node_count: usize,
    // Optimization: only rebuild when needed
    needs_full_rebuild: bool,
}

impl GpuInstanceManager {
    pub fn new() -> Self {
        Self {
            node_instances: Vec::with_capacity(10000),
            port_instances: Vec::with_capacity(50000),
            button_instances: Vec::with_capacity(1000), // Much fewer buttons expected
            flag_instances: Vec::with_capacity(10000), // One flag per node
            node_count: 0,
            port_count: 0,
            button_count: 0,
            flag_count: 0,
            last_frame_node_count: 0,
            needs_full_rebuild: true,
        }
    }
    
    pub fn update_instances(
        &mut self,
        nodes: &HashMap<NodeId, Node>,
        selected_nodes: &HashSet<NodeId>,
        connecting_from: Option<(NodeId, usize, bool)>,
        input_state: &crate::editor::InputState,
        graph: &crate::nodes::NodeGraph,
    ) -> (&[NodeInstanceData], &[PortInstanceData], &[ButtonInstanceData], &[FlagInstanceData]) {
        let current_node_count = nodes.len();
        let _estimated_port_count = current_node_count * 3; // Rough estimate
        
        // INSTANCING OPTIMIZATION DISABLED FOR NOW - rebuild every frame
        // This ensures immediate updates when flag visibility changes
        // TODO: Re-enable optimization logic in the future
        /*
        // Only rebuild if node count changed significantly or forced rebuild
        // Also rebuild during connection drawing mode for real-time port highlighting
        let should_rebuild = self.needs_full_rebuild || 
           current_node_count != self.last_frame_node_count ||
           input_state.is_connecting_mode() || // Force rebuild during connection drawing
           connecting_from.is_some(); // Force rebuild during click-to-connect
        
        if should_rebuild {
        */
            // Rebuild instances every frame for immediate updates
            self.rebuild_all_instances(nodes, selected_nodes, connecting_from, input_state, graph);
            self.last_frame_node_count = current_node_count;
            self.needs_full_rebuild = false;
        /*
        }
        */
        
        (&self.node_instances[..self.node_count], &self.port_instances[..self.port_count], &self.button_instances[..self.button_count], &self.flag_instances[..self.flag_count])
    }
    
    fn rebuild_all_instances(
        &mut self,
        nodes: &HashMap<NodeId, Node>,
        selected_nodes: &HashSet<NodeId>,
        connecting_from: Option<(NodeId, usize, bool)>,
        input_state: &crate::editor::InputState,
        graph: &crate::nodes::NodeGraph,
    ) {
        self.node_instances.clear();
        self.port_instances.clear();
        self.button_instances.clear();
        self.flag_instances.clear();
        
        for (id, node) in nodes {
            let selected = selected_nodes.contains(id);
            let instance = NodeInstanceData::from_node(node, selected, 1.0); // Don't apply zoom here
            self.node_instances.push(instance);
            
            // Add flag instance for this node
            let flag_position = node.get_flag_position();
            let flag_instance = FlagInstanceData::from_flag(flag_position, 5.0, node.visible);
            self.flag_instances.push(flag_instance);
            
            // Add port instances for this node
            for (port_idx, port) in node.inputs.iter().enumerate() {
                // Check if this port is being used for an active connection or connection preview
                let mut is_connecting = if let Some((conn_node, conn_port, is_input)) = connecting_from {
                    conn_node == *id && conn_port == port_idx && is_input
                } else {
                    false
                };
                
                // Also check if this port is in the connection drawing preview
                if !is_connecting && input_state.is_connecting_mode() {
                    // Check for start port preview (before drawing begins)
                    if input_state.get_current_connect_path().is_empty() {
                        if let Some((start_node, start_port, start_is_input)) = input_state.get_connection_start_preview(graph) {
                            if start_node == *id && start_port == port_idx && start_is_input {
                                is_connecting = true;
                            }
                        }
                    } else {
                        // Check for completed connection preview (while drawing)
                        if let Some(((start_node, start_port, start_is_input), (end_node, end_port, end_is_input))) = input_state.get_connection_preview(graph) {
                            if (start_node == *id && start_port == port_idx && start_is_input) ||
                               (end_node == *id && end_port == port_idx && end_is_input) {
                                is_connecting = true;
                            }
                        }
                        // Also check for end port preview (current mouse position)
                        if !is_connecting {
                            if let Some((end_node, end_port, end_is_input)) = input_state.get_connection_end_preview(graph) {
                                if end_node == *id && end_port == port_idx && end_is_input {
                                    is_connecting = true;
                                }
                            }
                        }
                    }
                }
                
                let port_instance = PortInstanceData::from_port(port.position, 5.0, is_connecting, true);
                self.port_instances.push(port_instance);
            }
            
            for (port_idx, port) in node.outputs.iter().enumerate() {
                // Check if this port is being used for an active connection or connection preview
                let mut is_connecting = if let Some((conn_node, conn_port, is_input)) = connecting_from {
                    conn_node == *id && conn_port == port_idx && !is_input
                } else {
                    false
                };
                
                // Also check if this port is in the connection drawing preview
                if !is_connecting && input_state.is_connecting_mode() {
                    // Check for start port preview (before drawing begins)
                    if input_state.get_current_connect_path().is_empty() {
                        if let Some((start_node, start_port, start_is_input)) = input_state.get_connection_start_preview(graph) {
                            if start_node == *id && start_port == port_idx && !start_is_input {
                                is_connecting = true;
                            }
                        }
                    } else {
                        // Check for completed connection preview (while drawing)
                        if let Some(((start_node, start_port, start_is_input), (end_node, end_port, end_is_input))) = input_state.get_connection_preview(graph) {
                            if (start_node == *id && start_port == port_idx && !start_is_input) ||
                               (end_node == *id && end_port == port_idx && !end_is_input) {
                                is_connecting = true;
                            }
                        }
                        // Also check for end port preview (current mouse position)
                        if !is_connecting {
                            if let Some((end_node, end_port, end_is_input)) = input_state.get_connection_end_preview(graph) {
                                if end_node == *id && end_port == port_idx && !end_is_input {
                                    is_connecting = true;
                                }
                            }
                        }
                    }
                }
                
                let port_instance = PortInstanceData::from_port(port.position, 5.0, is_connecting, false);
                self.port_instances.push(port_instance);
            }
            
            // NOTE: Visibility toggle ports are now rendered via CPU overlay in both GPU and CPU modes
            // This ensures they appear as simple outlines rather than filled port structures
        }
        
        self.node_count = self.node_instances.len();
        self.port_count = self.port_instances.len();
        self.button_count = self.button_instances.len();
        self.flag_count = self.flag_instances.len();
    }
    
    // INSTANCING OPTIMIZATION DISABLED - force_rebuild no longer needed
    /*
    pub fn force_rebuild(&mut self) {
        self.needs_full_rebuild = true;
    }
    */
}