//! Node interaction handling (selection, dragging, connections)

use egui::{Pos2, Vec2};
use std::collections::{HashMap, HashSet};
use crate::nodes::{NodeId, NodeGraph};

/// Manages node interactions and selections
#[derive(Debug, Clone)]
pub struct InteractionManager {
    pub selected_nodes: HashSet<NodeId>,
    pub selected_connection: Option<usize>, // Keep for backward compatibility
    pub selected_connections: HashSet<usize>, // Support for multiple connections
    pub drag_offsets: HashMap<NodeId, Vec2>,
    pub box_selection_start: Option<Pos2>,
    pub box_selection_end: Option<Pos2>,
    // Double-click tracking
    last_click_time: Option<std::time::Instant>,
    last_clicked_node: Option<NodeId>,
    double_click_threshold: std::time::Duration,
}

impl InteractionManager {
    /// Creates a new interaction manager
    pub fn new() -> Self {
        Self {
            selected_nodes: HashSet::new(),
            selected_connection: None,
            selected_connections: HashSet::new(),
            drag_offsets: HashMap::new(),
            box_selection_start: None,
            box_selection_end: None,
            last_click_time: None,
            last_clicked_node: None,
            double_click_threshold: std::time::Duration::from_millis(500),
        }
    }

    /// Select a single node, optionally keeping existing selection
    pub fn select_node(&mut self, node_id: NodeId, multi_select: bool) {
        if multi_select {
            if self.selected_nodes.contains(&node_id) {
                self.selected_nodes.remove(&node_id);
            } else {
                self.selected_nodes.insert(node_id);
            }
        } else {
            self.selected_nodes.clear();
            self.selected_nodes.insert(node_id);
        }
    }

    /// Clear all selections
    pub fn clear_selection(&mut self) {
        self.selected_nodes.clear();
        self.selected_connection = None;
        self.selected_connections.clear();
    }

    /// Select a connection by index
    pub fn select_connection(&mut self, connection_index: usize) {
        self.selected_nodes.clear(); // Clear node selection when selecting connection
        self.selected_connection = Some(connection_index);
        self.selected_connections.clear();
        self.selected_connections.insert(connection_index);
    }

    /// Select a connection with multi-select support
    pub fn select_connection_multi(&mut self, connection_index: usize, multi_select: bool) {
        if multi_select {
            if self.selected_connections.contains(&connection_index) {
                self.selected_connections.remove(&connection_index);
                // Update single selection for backward compatibility
                self.selected_connection = self.selected_connections.iter().next().copied();
            } else {
                self.selected_connections.insert(connection_index);
                self.selected_connection = Some(connection_index);
            }
        } else {
            self.selected_nodes.clear(); // Clear node selection when selecting connection
            self.selected_connections.clear();
            self.selected_connections.insert(connection_index);
            self.selected_connection = Some(connection_index);
        }
    }

    /// Clear only connection selection
    pub fn clear_connection_selection(&mut self) {
        self.selected_connection = None;
        self.selected_connections.clear();
    }
    
    /// Check if a node was double-clicked and update tracking
    pub fn check_double_click(&mut self, node_id: NodeId) -> bool {
        let now = std::time::Instant::now();
        let is_double_click = if let Some(last_time) = self.last_click_time {
            if let Some(last_node) = self.last_clicked_node {
                last_node == node_id && now.duration_since(last_time) < self.double_click_threshold
            } else {
                false
            }
        } else {
            false
        };
        
        self.last_click_time = Some(now);
        self.last_clicked_node = Some(node_id);
        
        is_double_click
    }


    /// Start dragging selected nodes
    pub fn start_drag(&mut self, drag_start: Pos2, graph: &NodeGraph) {
        self.drag_offsets.clear();
        for &node_id in &self.selected_nodes {
            if let Some(node) = graph.nodes.get(&node_id) {
                self.drag_offsets.insert(node_id, node.position - drag_start);
            }
        }
    }

    /// Update node positions during drag
    pub fn update_drag(&mut self, current_pos: Pos2, graph: &mut NodeGraph) {
        for (&node_id, &offset) in &self.drag_offsets {
            if let Some(node) = graph.nodes.get_mut(&node_id) {
                node.position = current_pos + offset;
                node.update_port_positions();
            }
        }
    }

    /// End dragging
    pub fn end_drag(&mut self) {
        self.drag_offsets.clear();
    }

    /// Start box selection
    pub fn start_box_selection(&mut self, start: Pos2) {
        self.box_selection_start = Some(start);
        self.box_selection_end = Some(start);
    }

    /// Update box selection
    pub fn update_box_selection(&mut self, end: Pos2) {
        self.box_selection_end = Some(end);
    }

    /// Get nodes currently touched by box selection (for highlighting during drag)
    pub fn get_box_selection_preview(&self, graph: &NodeGraph) -> Vec<NodeId> {
        let mut preview_nodes = Vec::new();
        
        if let (Some(start), Some(end)) = (self.box_selection_start, self.box_selection_end) {
            let min_x = start.x.min(end.x);
            let max_x = start.x.max(end.x);
            let min_y = start.y.min(end.y);
            let max_y = start.y.max(end.y);
            
            // Find nodes that intersect with the box
            for (&node_id, node) in &graph.nodes {
                let rect = node.get_rect();
                // Check if rectangles intersect (not just contain)
                if rect.left() <= max_x && rect.right() >= min_x &&
                   rect.top() <= max_y && rect.bottom() >= min_y {
                    preview_nodes.push(node_id);
                }
            }
        }
        
        preview_nodes
    }
    
    /// Complete box selection and return selected nodes
    pub fn complete_box_selection(&mut self, graph: &NodeGraph, multi_select: bool) -> Vec<NodeId> {
        let mut selected_nodes = Vec::new();
        let mut selected_connections = Vec::new();
        
        if let (Some(start), Some(end)) = (self.box_selection_start, self.box_selection_end) {
            let min_x = start.x.min(end.x);
            let max_x = start.x.max(end.x);
            let min_y = start.y.min(end.y);
            let max_y = start.y.max(end.y);
            
            // Select nodes that intersect with the box
            for (&node_id, node) in &graph.nodes {
                let rect = node.get_rect();
                // Check if rectangles intersect (not just contain)
                if rect.left() <= max_x && rect.right() >= min_x &&
                   rect.top() <= max_y && rect.bottom() >= min_y {
                    selected_nodes.push(node_id);
                }
            }
            
            // Select connections that pass through the box
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
                        
                        // Check if connection curve intersects with selection box
                        if self.connection_intersects_box(from_pos, to_pos, min_x, max_x, min_y, max_y) {
                            selected_connections.push(idx);
                        }
                    }
                }
            }
            
            if !multi_select {
                self.selected_nodes.clear();
                self.selected_connections.clear();
                self.selected_connection = None;
            }
            
            // Add selected nodes
            for node_id in &selected_nodes {
                self.selected_nodes.insert(*node_id);
            }
            
            // Add selected connections
            for &connection_idx in &selected_connections {
                self.selected_connections.insert(connection_idx);
            }
            
            // Update single connection for backward compatibility
            if !self.selected_connections.is_empty() {
                self.selected_connection = self.selected_connections.iter().next().copied();
            }
        }
        
        self.box_selection_start = None;
        self.box_selection_end = None;
        selected_nodes
    }
    
    /// Check if a connection curve intersects with a selection box
    fn connection_intersects_box(&self, from_pos: Pos2, to_pos: Pos2, min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> bool {
        // Sample points along the bezier curve to check intersection
        let total_distance = (to_pos - from_pos).length();
        let control_offset = total_distance * 0.3;
        
        let control_point1 = egui::Pos2::new(from_pos.x, from_pos.y + control_offset);
        let control_point2 = egui::Pos2::new(to_pos.x, to_pos.y - control_offset);
        
        // Sample multiple points along the bezier curve
        for i in 0..=20 {
            let t = i as f32 / 20.0;
            let point = crate::nodes::math_utils::cubic_bezier_point(
                t, from_pos, control_point1, control_point2, to_pos
            );
            
            // Check if this point is inside the selection box
            if point.x >= min_x && point.x <= max_x && point.y >= min_y && point.y <= max_y {
                return true;
            }
        }
        
        false
    }

    /// Delete selected nodes
    pub fn delete_selected(&mut self, graph: &mut NodeGraph) {
        for node_id in &self.selected_nodes {
            graph.remove_node(*node_id);
        }
        self.selected_nodes.clear();
    }
}

impl Default for InteractionManager {
    fn default() -> Self {
        Self::new()
    }
}