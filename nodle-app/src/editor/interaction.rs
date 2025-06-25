//! Node interaction handling (selection, dragging, connections)

use egui::{Pos2, Vec2};
use std::collections::{HashMap, HashSet};
use crate::nodes::{NodeId, NodeGraph};

/// Manages node interactions and selections
#[derive(Debug, Clone)]
pub struct InteractionManager {
    pub selected_nodes: HashSet<NodeId>,
    pub selected_connection: Option<usize>,
    pub drag_offsets: HashMap<NodeId, Vec2>,
    pub box_selection_start: Option<Pos2>,
    pub box_selection_end: Option<Pos2>,
}

impl InteractionManager {
    /// Creates a new interaction manager
    pub fn new() -> Self {
        Self {
            selected_nodes: HashSet::new(),
            selected_connection: None,
            drag_offsets: HashMap::new(),
            box_selection_start: None,
            box_selection_end: None,
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

    /// Complete box selection and return selected nodes
    pub fn complete_box_selection(&mut self, graph: &NodeGraph, multi_select: bool) -> Vec<NodeId> {
        let mut selected = Vec::new();
        
        if let (Some(start), Some(end)) = (self.box_selection_start, self.box_selection_end) {
            let min_x = start.x.min(end.x);
            let max_x = start.x.max(end.x);
            let min_y = start.y.min(end.y);
            let max_y = start.y.max(end.y);
            
            for (&node_id, node) in &graph.nodes {
                let rect = node.get_rect();
                if rect.left() >= min_x && rect.right() <= max_x &&
                   rect.top() >= min_y && rect.bottom() <= max_y {
                    selected.push(node_id);
                }
            }
            
            if !multi_select {
                self.selected_nodes.clear();
            }
            
            for node_id in &selected {
                self.selected_nodes.insert(*node_id);
            }
        }
        
        self.box_selection_start = None;
        self.box_selection_end = None;
        selected
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