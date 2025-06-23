//! Node editor implementation

use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use nodle_core::{
    graph::{Connection, NodeGraph},
    math::{cubic_bezier_point, distance_to_line_segment},
    node::{Node, NodeId},
    port::PortId,
};
use std::collections::{HashMap, HashSet};

/// Main application state for the node editor
pub struct NodeEditor {
    graph: NodeGraph,
    #[allow(dead_code)]
    dragging_node: Option<NodeId>,
    #[allow(dead_code)]
    drag_offset: Vec2,
    connecting_from: Option<(NodeId, PortId, bool)>, // (node_id, port_id, is_input)
    selected_nodes: HashSet<NodeId>,
    selected_connection: Option<usize>,
    context_menu_pos: Option<Pos2>,
    pan_offset: Vec2,
    zoom: f32,
    box_selection_start: Option<Pos2>,
    box_selection_end: Option<Pos2>,
    drag_offsets: HashMap<NodeId, Vec2>,
}

impl NodeEditor {
    pub fn new() -> Self {
        let mut editor = Self {
            graph: NodeGraph::new(),
            dragging_node: None,
            drag_offset: Vec2::ZERO,
            connecting_from: None,
            selected_nodes: HashSet::new(),
            selected_connection: None,
            context_menu_pos: None,
            pan_offset: Vec2::ZERO,
            zoom: 1.0,
            box_selection_start: None,
            box_selection_end: None,
            drag_offsets: HashMap::new(),
        };

        // Create test nodes
        editor.add_test_nodes();

        editor
    }

    fn zoom_at_point(&mut self, screen_point: Pos2, zoom_delta: f32) {
        // Convert screen point to world coordinates before zoom
        let world_point = Pos2::new(
            (screen_point.x - self.pan_offset.x) / self.zoom,
            (screen_point.y - self.pan_offset.y) / self.zoom,
        );

        // Apply zoom
        let new_zoom = (self.zoom + zoom_delta).clamp(0.1, 5.0);

        // Calculate new pan offset to keep the world point under the mouse
        let new_pan_offset = Vec2::new(
            screen_point.x - world_point.x * new_zoom,
            screen_point.y - world_point.y * new_zoom,
        );

        self.zoom = new_zoom;
        self.pan_offset = new_pan_offset;
    }

    fn create_node(&mut self, node_type: &str, position: Pos2) {
        let mut node = Node::new(0, node_type, position);

        match node_type {
            // Math nodes - green
            "Add" => {
                node.add_input("A").add_input("B").add_output("Result");
                node.color = Color32::from_rgb(80, 120, 80);
            }
            "Subtract" => {
                node.add_input("A").add_input("B").add_output("Result");
                node.color = Color32::from_rgb(80, 120, 80);
            }
            "Multiply" => {
                node.add_input("A").add_input("B").add_output("Result");
                node.color = Color32::from_rgb(80, 120, 80);
            }
            "Divide" => {
                node.add_input("A").add_input("B").add_output("Result");
                node.color = Color32::from_rgb(80, 120, 80);
            }
            // Logic nodes - blue
            "AND" => {
                node.add_input("A").add_input("B").add_output("Result");
                node.color = Color32::from_rgb(80, 80, 120);
            }
            "OR" => {
                node.add_input("A").add_input("B").add_output("Result");
                node.color = Color32::from_rgb(80, 80, 120);
            }
            "NOT" => {
                node.add_input("Input").add_output("Result");
                node.color = Color32::from_rgb(80, 80, 120);
            }
            // Data nodes - purple
            "Constant" => {
                node.add_output("Value");
                node.color = Color32::from_rgb(120, 80, 120);
            }
            "Variable" => {
                node.add_input("Set").add_output("Get");
                node.color = Color32::from_rgb(120, 80, 120);
            }
            // Output nodes - red
            "Print" => {
                node.add_input("Value");
                node.color = Color32::from_rgb(120, 80, 80);
            }
            "Debug" => {
                node.add_input("Value").add_output("Pass");
                node.color = Color32::from_rgb(120, 80, 80);
            }
            _ => {
                // Default node
                node.add_input("Input").add_output("Output");
                node.color = Color32::from_rgb(100, 100, 100);
            }
        }

        self.graph.add_node(node);
    }

    fn add_test_nodes(&mut self) {
        use crate::nodes::create_test_nodes;
        create_test_nodes(&mut self.graph);
    }
}

impl eframe::App for NodeEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Implementation continues in the original file...
        // This is a placeholder - we'll need to copy the rest of the update method
        ctx.request_repaint();
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Nodle - Node Editor");
            // TODO: Copy the rest of the implementation
        });
    }
}