//! Input handling and event management

use egui::{Pos2, Vec2, Modifiers, Key, PointerButton};
use crate::nodes::{NodeId, PortId, NodeGraph, Connection};

/// Manages input state and event handling for the node editor
#[derive(Debug, Clone)]
pub struct InputState {
    // Mouse state
    pub mouse_pos: Option<Pos2>,
    pub last_mouse_pos: Option<Pos2>,
    pub mouse_world_pos: Option<Pos2>,
    pub click_pos: Option<Pos2>,
    pub drag_start_pos: Option<Pos2>,
    
    // Input states
    pub is_panning: bool,
    pub is_dragging_nodes: bool,
    pub is_box_selecting: bool,
    pub is_connecting: bool,
    
    // Interaction state
    pub modifiers: Modifiers,
    pub clicked_this_frame: bool,
    pub right_clicked_this_frame: bool,
    pub drag_started_this_frame: bool,
    pub drag_stopped_this_frame: bool,
    
    
    // Scroll/zoom
    pub scroll_delta: f32,
    
    // Connection management
    pub connecting_from: Option<(NodeId, PortId, bool)>, // (node_id, port_id, is_input)
    
    // Context menu state
    pub context_menu_pos: Option<Pos2>,
    pub right_click_world_pos: Option<Pos2>,
    
    // Connection cutting state (X key)
    pub is_cutting_mode: bool,
    pub cut_paths: Vec<Vec<Pos2>>, // Multiple cut paths while X is held
    pub current_cut_path: Vec<Pos2>, // Current path being drawn
    
    // Connection drawing state (C key)
    pub is_connecting_mode: bool,
    pub connect_paths: Vec<Vec<Pos2>>, // Multiple connection paths while C is held
    pub current_connect_path: Vec<Pos2>, // Current connection path being drawn
}

impl InputState {
    /// Creates a new input state
    pub fn new() -> Self {
        Self {
            mouse_pos: None,
            last_mouse_pos: None,
            mouse_world_pos: None,
            click_pos: None,
            drag_start_pos: None,
            is_panning: false,
            is_dragging_nodes: false,
            is_box_selecting: false,
            is_connecting: false,
            modifiers: Modifiers::default(),
            clicked_this_frame: false,
            right_clicked_this_frame: false,
            drag_started_this_frame: false,
            drag_stopped_this_frame: false,
            scroll_delta: 0.0,
            connecting_from: None,
            context_menu_pos: None,
            right_click_world_pos: None,
            is_cutting_mode: false,
            cut_paths: Vec::new(),
            current_cut_path: Vec::new(),
            is_connecting_mode: false,
            connect_paths: Vec::new(),
            current_connect_path: Vec::new(),
        }
    }

    /// Update input state from egui response and world position transform
    pub fn update(&mut self, ui: &egui::Ui, response: &egui::Response, inverse_transform: impl Fn(Pos2) -> Pos2) {
        // Store previous mouse position
        self.last_mouse_pos = self.mouse_pos;
        
        // Update current mouse positions
        self.mouse_pos = response.hover_pos();
        self.mouse_world_pos = self.mouse_pos.map(&inverse_transform);
        
        // Update modifiers
        self.modifiers = ui.input(|i| i.modifiers);
        
        // Update click states
        self.clicked_this_frame = response.clicked();
        self.right_clicked_this_frame = response.secondary_clicked();
        self.drag_started_this_frame = response.drag_started();
        self.drag_stopped_this_frame = response.drag_stopped();
        
        // Update click position
        if self.clicked_this_frame || self.right_clicked_this_frame {
            self.click_pos = response.interact_pointer_pos().map(&inverse_transform);
        }
        
        // Update right-click world position for context menu
        if self.right_clicked_this_frame {
            self.right_click_world_pos = self.mouse_world_pos;
            self.context_menu_pos = self.mouse_pos;
        }
        
        // Update drag start position
        if self.drag_started_this_frame {
            self.drag_start_pos = response.interact_pointer_pos().map(&inverse_transform);
        }
        
        // Update panning state
        if response.dragged_by(PointerButton::Middle) {
            self.is_panning = true;
        } else if !ui.input(|i| i.pointer.middle_down()) {
            self.is_panning = false;
        }
        
        // Update scroll delta
        self.scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
        
        // Reset dragging states on drag stop
        if self.drag_stopped_this_frame {
            self.is_dragging_nodes = false;
            self.is_box_selecting = false;
        }
        
        // Close context menu on click (if not right-click)
        if self.clicked_this_frame {
            self.context_menu_pos = None;
        }
        
        // Handle cutting mode (X key)
        let x_key_down = ui.input(|i| i.key_down(egui::Key::X));
        
        if x_key_down && !self.is_cutting_mode {
            // Start cutting mode
            self.is_cutting_mode = true;
            self.cut_paths.clear();
            self.current_cut_path.clear();
        } else if !x_key_down && self.is_cutting_mode {
            // End cutting mode - finalize current path if any
            if !self.current_cut_path.is_empty() {
                self.cut_paths.push(self.current_cut_path.clone());
                self.current_cut_path.clear();
            }
            self.is_cutting_mode = false;
        }
        
        // Update cutting path when in cutting mode
        if self.is_cutting_mode {
            if response.dragged() {
                // Add points to current path while dragging
                if let Some(world_pos) = self.mouse_world_pos {
                    self.current_cut_path.push(world_pos);
                }
            } else if response.drag_stopped() {
                // Finish current path and start a new one
                if !self.current_cut_path.is_empty() {
                    self.cut_paths.push(self.current_cut_path.clone());
                    self.current_cut_path.clear();
                }
            }
        }
        
        // Handle connecting mode (C key)
        let c_key_down = ui.input(|i| i.key_down(egui::Key::C));
        
        if c_key_down && !self.is_connecting_mode {
            // Start connecting mode
            self.is_connecting_mode = true;
            self.connect_paths.clear();
            self.current_connect_path.clear();
        } else if !c_key_down && self.is_connecting_mode {
            // End connecting mode - finalize current path if any
            if !self.current_connect_path.is_empty() {
                self.connect_paths.push(self.current_connect_path.clone());
                self.current_connect_path.clear();
            }
            self.is_connecting_mode = false;
        }
        
        // Update connecting path when in connecting mode
        if self.is_connecting_mode {
            if response.dragged() {
                // Add points to current path while dragging
                if let Some(world_pos) = self.mouse_world_pos {
                    self.current_connect_path.push(world_pos);
                }
            } else if response.drag_stopped() {
                // Finish current path and start a new one
                if !self.current_connect_path.is_empty() {
                    self.connect_paths.push(self.current_connect_path.clone());
                    self.current_connect_path.clear();
                }
            }
        }
    }

    /// Get pan delta for viewport panning
    pub fn get_pan_delta(&self, response: &egui::Response) -> Option<Vec2> {
        if self.is_panning && response.dragged() {
            Some(response.drag_delta())
        } else {
            None
        }
    }

    /// Check if a key is pressed this frame
    pub fn key_pressed(&self, ui: &egui::Ui, key: Key) -> bool {
        ui.input(|i| i.key_pressed(key))
    }
    
    /// Check if multi-select modifier is held (Ctrl/Cmd)
    pub fn is_multi_select(&self) -> bool {
        self.modifiers.ctrl || self.modifiers.command
    }
    
    /// Check if primary mouse button is down
    pub fn is_primary_down(&self, ui: &egui::Ui) -> bool {
        ui.input(|i| i.pointer.primary_down())
    }
    
    /// Check if middle mouse button is down
    pub fn is_middle_down(&self, ui: &egui::Ui) -> bool {
        ui.input(|i| i.pointer.middle_down())
    }
    
    /// Get current mouse interact position
    pub fn get_interact_pos(&self, ui: &egui::Ui) -> Option<Pos2> {
        ui.input(|i| i.pointer.interact_pos())
    }
    
    /// Check if primary mouse was clicked this frame
    pub fn primary_clicked(&self, ui: &egui::Ui) -> bool {
        ui.input(|i| i.pointer.primary_clicked())
    }
    
    
    /// Start connecting state
    pub fn start_connecting(&mut self) {
        self.is_connecting = true;
    }
    
    /// Stop connecting state
    pub fn stop_connecting(&mut self) {
        self.is_connecting = false;
    }
    
    /// Check if mouse is near a point within given radius
    pub fn is_mouse_near(&self, point: Pos2, radius: f32) -> bool {
        if let Some(mouse_pos) = self.mouse_world_pos {
            (mouse_pos - point).length() < radius
        } else {
            false
        }
    }
    
    /// Check if scroll/zoom input occurred
    pub fn has_scroll_input(&self) -> bool {
        self.scroll_delta != 0.0
    }
    
    /// Get zoom delta based on scroll input
    pub fn get_zoom_delta(&self) -> f32 {
        self.scroll_delta * 0.001
    }
    
    // === CONNECTION MANAGEMENT ===
    
    /// Start a connection from a port
    pub fn start_connection(&mut self, node_id: NodeId, port_id: PortId, is_input: bool) {
        self.connecting_from = Some((node_id, port_id, is_input));
        self.is_connecting = true;
    }
    
    /// Complete a connection by creating the connection object
    pub fn complete_connection(&mut self, to_node: NodeId, to_port: PortId) -> Option<Connection> {
        if let Some((from_node, from_port, from_is_input)) = self.connecting_from {
            let connection = if from_is_input {
                // From input to output (reverse direction)
                Connection::new(to_node, to_port, from_node, from_port)
            } else {
                // From output to input (normal direction)
                Connection::new(from_node, from_port, to_node, to_port)
            };
            self.cancel_connection();
            Some(connection)
        } else {
            None
        }
    }
    
    /// Cancel current connection
    pub fn cancel_connection(&mut self) {
        self.connecting_from = None;
        self.is_connecting = false;
    }
    
    /// Check if we're currently connecting
    pub fn is_connecting_active(&self) -> bool {
        self.connecting_from.is_some()
    }
    
    /// Get the current connection information
    pub fn get_connecting_from(&self) -> Option<(NodeId, PortId, bool)> {
        self.connecting_from
    }
    
    // === PORT INTERACTION ===
    
    /// Check if mouse is near a port (within interaction radius)
    pub fn is_mouse_near_port(&self, port_pos: Pos2, radius: f32) -> bool {
        if let Some(mouse_world_pos) = self.mouse_world_pos {
            (mouse_world_pos - port_pos).length() < radius
        } else {
            false
        }
    }
    
    /// Find which port (if any) is being clicked, returns (node_id, port_idx, is_input)
    pub fn find_clicked_port(&self, graph: &NodeGraph, click_radius: f32) -> Option<(NodeId, usize, bool)> {
        if let Some(pos) = self.mouse_world_pos {
            for (node_id, node) in &graph.nodes {
                // Check output ports
                for (port_idx, port) in node.outputs.iter().enumerate() {
                    if (port.position - pos).length() < click_radius {
                        return Some((*node_id, port_idx, false));
                    }
                }
                // Check input ports
                for (port_idx, port) in node.inputs.iter().enumerate() {
                    if (port.position - pos).length() < click_radius {
                        return Some((*node_id, port_idx, true));
                    }
                }
            }
        }
        None
    }
    
    /// Find existing connection to an input port, returns connection index and source info
    pub fn find_input_connection(&self, graph: &NodeGraph, target_node: NodeId, target_port: PortId) -> Option<(usize, NodeId, PortId)> {
        for (idx, connection) in graph.connections.iter().enumerate() {
            if connection.to_node == target_node && connection.to_port == target_port {
                return Some((idx, connection.from_node, connection.from_port));
            }
        }
        None
    }
    
    /// Check if an input port has an existing connection
    pub fn input_has_connection(&self, graph: &NodeGraph, node_id: NodeId, port_idx: PortId) -> bool {
        self.find_input_connection(graph, node_id, port_idx).is_some()
    }

    /// Find connection curve that was clicked, returns connection index
    pub fn find_clicked_connection(&self, graph: &NodeGraph, click_radius: f32, zoom: f32) -> Option<usize> {
        if let Some(click_pos) = self.mouse_world_pos {
            for (idx, connection) in graph.connections.iter().enumerate() {
                if let (Some(from_node), Some(to_node)) = (
                    graph.nodes.get(&connection.from_node),
                    graph.nodes.get(&connection.to_node),
                ) {
                    if let (Some(from_port), Some(to_port)) = (
                        from_node.outputs.get(connection.from_port),
                        to_node.inputs.get(connection.to_port),
                    ) {
                        let from_pos = from_port.position;
                        let to_pos = to_port.position;

                        // Calculate bezier curve control points (same logic as in rendering)
                        let vertical_distance = (to_pos.y - from_pos.y).abs();
                        let control_offset = if vertical_distance > 10.0 {
                            vertical_distance * 0.4
                        } else {
                            60.0 * zoom
                        };

                        let control_point1 = egui::Pos2::new(from_pos.x, from_pos.y + control_offset);
                        let control_point2 = egui::Pos2::new(to_pos.x, to_pos.y - control_offset);

                        // Check if click is near the bezier curve
                        let distance = crate::nodes::math_utils::distance_to_bezier_curve(
                            click_pos,
                            from_pos,
                            control_point1,
                            control_point2,
                            to_pos,
                        );

                        if distance <= click_radius {
                            return Some(idx);
                        }
                    }
                }
            }
        }
        None
    }
    
    // === NODE SELECTION ===
    
    /// Find which node (if any) contains the current mouse position
    pub fn find_node_under_mouse(&self, graph: &NodeGraph) -> Option<NodeId> {
        if let Some(pos) = self.mouse_world_pos {
            for (node_id, node) in &graph.nodes {
                if node.get_rect().contains(pos) {
                    return Some(*node_id);
                }
            }
        }
        None
    }
    
    
    // === KEYBOARD SHORTCUTS ===
    
    /// Check for delete key press
    pub fn delete_pressed(&self, ui: &egui::Ui) -> bool {
        self.key_pressed(ui, Key::Delete)
    }
    
    /// Check for escape key press
    pub fn escape_pressed(&self, ui: &egui::Ui) -> bool {
        self.key_pressed(ui, Key::Escape)
    }
    
    /// Check for F1 key press (performance info toggle)
    pub fn f1_pressed(&self, ui: &egui::Ui) -> bool {
        self.key_pressed(ui, Key::F1)
    }
    
    /// Check for F2 key press (add 10 nodes)
    pub fn f2_pressed(&self, ui: &egui::Ui) -> bool {
        self.key_pressed(ui, Key::F2)
    }
    
    /// Check for F3 key press (add 25 nodes)
    pub fn f3_pressed(&self, ui: &egui::Ui) -> bool {
        self.key_pressed(ui, Key::F3)
    }
    
    /// Check for F4 key press (stress test)
    pub fn f4_pressed(&self, ui: &egui::Ui) -> bool {
        self.key_pressed(ui, Key::F4)
    }
    
    /// Check for F5 key press (clear all)
    pub fn f5_pressed(&self, ui: &egui::Ui) -> bool {
        self.key_pressed(ui, Key::F5)
    }
    
    /// Check for F6 key press (toggle GPU/CPU rendering)
    pub fn f6_pressed(&self, ui: &egui::Ui) -> bool {
        self.key_pressed(ui, Key::F6)
    }
    
    // === CONTEXT MENU ===
    
    /// Check if context menu should be shown
    pub fn should_show_context_menu(&self) -> bool {
        self.context_menu_pos.is_some()
    }
    
    /// Get context menu position in screen coordinates
    pub fn get_context_menu_pos(&self) -> Option<Pos2> {
        self.context_menu_pos
    }
    
    /// Get the world position where right-click occurred
    pub fn get_right_click_world_pos(&self) -> Option<Pos2> {
        self.right_click_world_pos
    }
    
    /// Close context menu
    pub fn close_context_menu(&mut self) {
        self.context_menu_pos = None;
        self.right_click_world_pos = None;
    }
    
    /// Check if we clicked outside of a given rectangle (for closing menus)
    pub fn clicked_outside_rect(&self, rect: egui::Rect) -> bool {
        if self.clicked_this_frame {
            if let Some(mouse_pos) = self.mouse_pos {
                return !rect.contains(mouse_pos);
            }
        }
        false
    }
    
    // === CONNECTION CUTTING ===
    
    /// Check if we're in cutting mode
    pub fn is_cutting_mode(&self) -> bool {
        self.is_cutting_mode
    }
    
    /// Get all cut paths for rendering
    pub fn get_cut_paths(&self) -> &Vec<Vec<Pos2>> {
        &self.cut_paths
    }
    
    /// Get current cut path being drawn
    pub fn get_current_cut_path(&self) -> &Vec<Pos2> {
        &self.current_cut_path
    }
    
    /// Find connections that intersect with any cut path and return their indices
    pub fn find_cut_connections(&self, graph: &NodeGraph, zoom: f32) -> Vec<usize> {
        let mut cut_connections = Vec::new();
        
        // Check all completed cut paths plus current path
        let mut all_paths = self.cut_paths.clone();
        if !self.current_cut_path.is_empty() {
            all_paths.push(self.current_cut_path.clone());
        }
        
        for (idx, connection) in graph.connections.iter().enumerate() {
            if let (Some(from_node), Some(to_node)) = (
                graph.nodes.get(&connection.from_node),
                graph.nodes.get(&connection.to_node),
            ) {
                if let (Some(from_port), Some(to_port)) = (
                    from_node.outputs.get(connection.from_port),
                    to_node.inputs.get(connection.to_port),
                ) {
                    let from_pos = from_port.position;
                    let to_pos = to_port.position;
                    
                    // Check if any cut path intersects this connection
                    for cut_path in &all_paths {
                        if self.path_intersects_connection(cut_path, from_pos, to_pos, zoom) {
                            cut_connections.push(idx);
                            break; // Only add once per connection
                        }
                    }
                }
            }
        }
        
        cut_connections
    }
    
    /// Check if a cut path intersects with a connection bezier curve
    fn path_intersects_connection(&self, cut_path: &[Pos2], from_pos: Pos2, to_pos: Pos2, zoom: f32) -> bool {
        if cut_path.len() < 2 {
            return false;
        }
        
        // Calculate bezier curve control points (same logic as rendering)
        let vertical_distance = (to_pos.y - from_pos.y).abs();
        let control_offset = if vertical_distance > 10.0 {
            vertical_distance * 0.4
        } else {
            60.0 * zoom
        };
        
        let control_point1 = egui::Pos2::new(from_pos.x, from_pos.y + control_offset);
        let control_point2 = egui::Pos2::new(to_pos.x, to_pos.y - control_offset);
        
        // Sample points along the bezier curve
        for i in 0..=20 {
            let t = i as f32 / 20.0;
            let curve_point = crate::nodes::math_utils::cubic_bezier_point(
                t, from_pos, control_point1, control_point2, to_pos
            );
            
            // Check if this curve point is close to any segment of the cut path
            for window in cut_path.windows(2) {
                let seg_start = window[0];
                let seg_end = window[1];
                
                let distance = crate::nodes::math_utils::distance_to_line_segment(
                    curve_point, seg_start, seg_end
                );
                
                if distance < 10.0 { // Intersection threshold
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Clear all cut paths (called when cutting mode ends and cuts are applied)
    pub fn clear_cut_paths(&mut self) {
        self.cut_paths.clear();
        self.current_cut_path.clear();
    }
    
    // === CONNECTION DRAWING ===
    
    /// Check if we're in connecting mode
    pub fn is_connecting_mode(&self) -> bool {
        self.is_connecting_mode
    }
    
    /// Get all connect paths for rendering
    pub fn get_connect_paths(&self) -> &Vec<Vec<Pos2>> {
        &self.connect_paths
    }
    
    /// Get current connect path being drawn
    pub fn get_current_connect_path(&self) -> &Vec<Pos2> {
        &self.current_connect_path
    }
    
    /// Find the nearest port to a given position within a search radius
    pub fn find_nearest_port(&self, graph: &NodeGraph, position: Pos2, search_radius: f32) -> Option<(NodeId, usize, bool)> {
        let mut nearest_port = None;
        let mut min_distance = search_radius;
        
        for (node_id, node) in &graph.nodes {
            // Check output ports
            for (port_idx, port) in node.outputs.iter().enumerate() {
                let distance = (port.position - position).length();
                if distance < min_distance {
                    min_distance = distance;
                    nearest_port = Some((*node_id, port_idx, false)); // false = output
                }
            }
            
            // Check input ports
            for (port_idx, port) in node.inputs.iter().enumerate() {
                let distance = (port.position - position).length();
                if distance < min_distance {
                    min_distance = distance;
                    nearest_port = Some((*node_id, port_idx, true)); // true = input
                }
            }
        }
        
        nearest_port
    }
    
    /// Create connections from drawn paths to nearest ports
    pub fn create_connections_from_paths(&self, graph: &NodeGraph) -> Vec<Connection> {
        let mut connections = Vec::new();
        let search_radius = 80.0; // Increased search radius for easier targeting
        
        // Process all completed paths plus current path
        let mut all_paths = self.connect_paths.clone();
        if !self.current_connect_path.is_empty() {
            all_paths.push(self.current_connect_path.clone());
        }
        
        for path in &all_paths {
            if path.len() < 2 {
                continue; // Need at least start and end points
            }
            
            // Instead of using exact start/end points, use the general area
            // Take the first few and last few points to find the best port match
            let start_area_points = if path.len() >= 4 {
                &path[0..3] // First 3 points
            } else {
                &path[0..1] // Just first point if path is short
            };
            
            let end_area_points = if path.len() >= 4 {
                &path[path.len()-3..] // Last 3 points
            } else {
                &path[path.len()-1..] // Just last point if path is short
            };
            
            // Find best port near the start area
            let start_port = self.find_best_port_near_area(graph, start_area_points, search_radius);
            
            // Find best port near the end area
            let end_port = self.find_best_port_near_area(graph, end_area_points, search_radius);
            
            if let (Some((start_node, start_port_idx, start_is_input)), Some((end_node, end_port_idx, end_is_input))) = (start_port, end_port) {
                // Only create connection if we have different nodes and valid port types
                if start_node != end_node && start_is_input != end_is_input {
                    let connection = if start_is_input {
                        // Start from input, end at output: reverse the connection
                        Connection::new(end_node, end_port_idx, start_node, start_port_idx)
                    } else {
                        // Start from output, end at input: normal connection
                        Connection::new(start_node, start_port_idx, end_node, end_port_idx)
                    };
                    connections.push(connection);
                }
            }
        }
        
        connections
    }
    
    /// Find the best port near an area defined by multiple points
    fn find_best_port_near_area(&self, graph: &NodeGraph, area_points: &[Pos2], search_radius: f32) -> Option<(NodeId, usize, bool)> {
        let mut best_port = None;
        let mut best_score = f32::MAX;
        
        // Calculate the center of the area for reference
        let area_center = if area_points.is_empty() {
            return None;
        } else if area_points.len() == 1 {
            area_points[0]
        } else {
            let sum = area_points.iter().fold(Pos2::ZERO, |acc, &p| Pos2::new(acc.x + p.x, acc.y + p.y));
            Pos2::new(sum.x / area_points.len() as f32, sum.y / area_points.len() as f32)
        };
        
        for (node_id, node) in &graph.nodes {
            // Check output ports
            for (port_idx, port) in node.outputs.iter().enumerate() {
                // Find minimum distance from port to any point in the area
                let min_distance_to_area = area_points.iter()
                    .map(|&point| (port.position - point).length())
                    .fold(f32::MAX, f32::min);
                
                if min_distance_to_area < search_radius {
                    // Score combines distance to area and distance to area center
                    // This helps pick ports that are both close to the path and in the right general area
                    let distance_to_center = (port.position - area_center).length();
                    let score = min_distance_to_area * 0.7 + distance_to_center * 0.3;
                    
                    if score < best_score {
                        best_score = score;
                        best_port = Some((*node_id, port_idx, false)); // false = output
                    }
                }
            }
            
            // Check input ports
            for (port_idx, port) in node.inputs.iter().enumerate() {
                // Find minimum distance from port to any point in the area
                let min_distance_to_area = area_points.iter()
                    .map(|&point| (port.position - point).length())
                    .fold(f32::MAX, f32::min);
                
                if min_distance_to_area < search_radius {
                    // Score combines distance to area and distance to area center
                    let distance_to_center = (port.position - area_center).length();
                    let score = min_distance_to_area * 0.7 + distance_to_center * 0.3;
                    
                    if score < best_score {
                        best_score = score;
                        best_port = Some((*node_id, port_idx, true)); // true = input
                    }
                }
            }
        }
        
        best_port
    }
    
    /// Get preview of ports that would be connected by current connection path
    pub fn get_connection_preview(&self, graph: &NodeGraph) -> Option<((NodeId, usize, bool), (NodeId, usize, bool))> {
        if self.current_connect_path.len() < 2 {
            return None;
        }
        
        let search_radius = 80.0;
        
        // Get start and end areas
        let start_area_points = if self.current_connect_path.len() >= 4 {
            &self.current_connect_path[0..3]
        } else {
            &self.current_connect_path[0..1]
        };
        
        let end_area_points = if self.current_connect_path.len() >= 4 {
            &self.current_connect_path[self.current_connect_path.len()-3..]
        } else {
            &self.current_connect_path[self.current_connect_path.len()-1..]
        };
        
        // Find ports that would be connected
        let start_port = self.find_best_port_near_area(graph, start_area_points, search_radius);
        let end_port = self.find_best_port_near_area(graph, end_area_points, search_radius);
        
        if let (Some(start), Some(end)) = (start_port, end_port) {
            // Only return valid connections (different nodes, compatible port types)
            if start.0 != end.0 && start.2 != end.2 {
                return Some((start, end));
            }
        }
        
        None
    }
    
    /// Clear all connect paths (called when connecting mode ends and connections are applied)
    pub fn clear_connect_paths(&mut self) {
        self.connect_paths.clear();
        self.current_connect_path.clear();
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}