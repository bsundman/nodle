//! Nodle - A node-based visual programming editor
//! 
//! This application provides a visual node editor with vertical flow,
//! where connections flow from top to bottom (inputs on top, outputs on bottom).

use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use std::collections::{HashMap, HashSet};

/// Calculates a point on a cubic Bézier curve at parameter t (0.0 to 1.0).
/// Used for drawing smooth connection curves between nodes.
fn cubic_bezier_point(t: f32, p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2) -> Pos2 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;

    Pos2::new(
        mt3 * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t3 * p3.x,
        mt3 * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t3 * p3.y,
    )
}

/// Calculates the minimum distance from a point to a line segment.
/// Used for detecting clicks on connection curves.
fn distance_to_line_segment(point: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let ap = point - a;
    let ab_len_sq = ab.x * ab.x + ab.y * ab.y;

    if ab_len_sq == 0.0 {
        return (point - a).length();
    }

    let t = ((ap.x * ab.x + ap.y * ab.y) / ab_len_sq).clamp(0.0, 1.0);
    let projection = a + ab * t;
    (point - projection).length()
}

/// Represents a connection point on a node (input or output).
#[derive(Debug, Clone)]
struct Port {
    #[allow(dead_code)]
    id: usize,
    name: String,
    #[allow(dead_code)]
    is_input: bool,
    position: Pos2,
}

/// Represents a visual node in the editor with inputs, outputs, and visual properties.
#[derive(Debug, Clone)]
struct Node {
    #[allow(dead_code)]
    id: usize,
    title: String,
    position: Pos2,
    size: Vec2,
    inputs: Vec<Port>,
    outputs: Vec<Port>,
    color: Color32,
}

impl Node {
    fn new(id: usize, title: &str, position: Pos2) -> Self {
        Self {
            id,
            title: title.to_string(),
            position,
            size: Vec2::new(150.0, 30.0),
            inputs: vec![],
            outputs: vec![],
            color: Color32::from_rgb(60, 60, 60),
        }
    }

    fn add_input(&mut self, name: &str) -> &mut Self {
        let port_id = self.inputs.len();
        self.inputs.push(Port {
            id: port_id,
            name: name.to_string(),
            is_input: true,
            position: Pos2::ZERO,
        });
        self
    }

    fn add_output(&mut self, name: &str) -> &mut Self {
        let port_id = self.outputs.len();
        self.outputs.push(Port {
            id: port_id,
            name: name.to_string(),
            is_input: false,
            position: Pos2::ZERO,
        });
        self
    }

    fn update_port_positions(&mut self) {
        let _port_size = 10.0;
        let port_spacing = 30.0;

        // Inputs on TOP of node
        let input_start_x = if self.inputs.len() > 1 {
            (self.size.x - (self.inputs.len() - 1) as f32 * port_spacing) / 2.0
        } else {
            self.size.x / 2.0
        };

        for (i, input) in self.inputs.iter_mut().enumerate() {
            input.position =
                self.position + Vec2::new(input_start_x + i as f32 * port_spacing, 0.0);
        }

        // Outputs on BOTTOM of node
        let output_start_x = if self.outputs.len() > 1 {
            (self.size.x - (self.outputs.len() - 1) as f32 * port_spacing) / 2.0
        } else {
            self.size.x / 2.0
        };

        for (i, output) in self.outputs.iter_mut().enumerate() {
            output.position =
                self.position + Vec2::new(output_start_x + i as f32 * port_spacing, self.size.y);
        }
    }

    fn get_rect(&self) -> Rect {
        Rect::from_min_size(self.position, self.size)
    }
}

/// Represents a connection between two ports on different nodes.
#[derive(Debug, Clone)]
struct Connection {
    from_node: usize,
    from_port: usize,
    to_node: usize,
    to_port: usize,
}

/// Main application state for the node editor.
struct NodeEditor {
    nodes: HashMap<usize, Node>,
    connections: Vec<Connection>,
    next_node_id: usize,
    #[allow(dead_code)]
    dragging_node: Option<usize>,     // Currently unused, kept for future use
    #[allow(dead_code)]
    drag_offset: Vec2,                // Currently unused, kept for future use
    connecting_from: Option<(usize, usize, bool)>, // (node_id, port_id, is_input)
    selected_nodes: HashSet<usize>,
    selected_connection: Option<usize>,
    context_menu_pos: Option<Pos2>,
    pan_offset: Vec2,
    zoom: f32,
    box_selection_start: Option<Pos2>,
    box_selection_end: Option<Pos2>,
    drag_offsets: HashMap<usize, Vec2>,
}

impl NodeEditor {
    fn new() -> Self {
        let mut editor = Self {
            nodes: HashMap::new(),
            connections: vec![],
            next_node_id: 0,
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
        let mut node = Node::new(self.next_node_id, node_type, position);

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

        self.nodes.insert(self.next_node_id, node);
        self.next_node_id += 1;
    }

    fn add_test_nodes(&mut self) {
        // Math nodes - green colors
        let mut add_node = Node::new(self.next_node_id, "Add", Pos2::new(100.0, 100.0));
        add_node.add_input("A").add_input("B").add_output("Result");
        add_node.color = Color32::from_rgb(80, 120, 80);
        self.nodes.insert(self.next_node_id, add_node);
        self.next_node_id += 1;

        let mut sub_node = Node::new(self.next_node_id, "Subtract", Pos2::new(100.0, 200.0));
        sub_node.add_input("A").add_input("B").add_output("Result");
        sub_node.color = Color32::from_rgb(80, 120, 80);
        self.nodes.insert(self.next_node_id, sub_node);
        self.next_node_id += 1;

        let mut mul_node = Node::new(self.next_node_id, "Multiply", Pos2::new(300.0, 100.0));
        mul_node.add_input("A").add_input("B").add_output("Result");
        mul_node.color = Color32::from_rgb(80, 120, 80);
        self.nodes.insert(self.next_node_id, mul_node);
        self.next_node_id += 1;

        let mut div_node = Node::new(self.next_node_id, "Divide", Pos2::new(300.0, 200.0));
        div_node.add_input("A").add_input("B").add_output("Result");
        div_node.color = Color32::from_rgb(80, 120, 80);
        self.nodes.insert(self.next_node_id, div_node);
        self.next_node_id += 1;

        // Logic nodes - blue colors
        let mut and_node = Node::new(self.next_node_id, "AND", Pos2::new(500.0, 100.0));
        and_node.add_input("A").add_input("B").add_output("Result");
        and_node.color = Color32::from_rgb(80, 80, 120);
        self.nodes.insert(self.next_node_id, and_node);
        self.next_node_id += 1;

        let mut or_node = Node::new(self.next_node_id, "OR", Pos2::new(500.0, 200.0));
        or_node.add_input("A").add_input("B").add_output("Result");
        or_node.color = Color32::from_rgb(80, 80, 120);
        self.nodes.insert(self.next_node_id, or_node);
        self.next_node_id += 1;

        let mut not_node = Node::new(self.next_node_id, "NOT", Pos2::new(700.0, 150.0));
        not_node.add_input("Input").add_output("Result");
        not_node.color = Color32::from_rgb(80, 80, 120);
        self.nodes.insert(self.next_node_id, not_node);
        self.next_node_id += 1;

        // Data nodes - purple colors
        let mut const_node = Node::new(self.next_node_id, "Constant", Pos2::new(100.0, 350.0));
        const_node.add_output("Value");
        const_node.color = Color32::from_rgb(120, 80, 120);
        self.nodes.insert(self.next_node_id, const_node);
        self.next_node_id += 1;

        let mut var_node = Node::new(self.next_node_id, "Variable", Pos2::new(300.0, 350.0));
        var_node.add_input("Set").add_output("Get");
        var_node.color = Color32::from_rgb(120, 80, 120);
        self.nodes.insert(self.next_node_id, var_node);
        self.next_node_id += 1;

        // Output nodes - red colors
        let mut print_node = Node::new(self.next_node_id, "Print", Pos2::new(500.0, 350.0));
        print_node.add_input("Value");
        print_node.color = Color32::from_rgb(120, 80, 80);
        self.nodes.insert(self.next_node_id, print_node);
        self.next_node_id += 1;

        let mut debug_node = Node::new(self.next_node_id, "Debug", Pos2::new(700.0, 350.0));
        debug_node.add_input("Value").add_output("Pass");
        debug_node.color = Color32::from_rgb(120, 80, 80);
        self.nodes.insert(self.next_node_id, debug_node);
        self.next_node_id += 1;
    }
}

impl eframe::App for NodeEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request repaint for smooth updates
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Nodle - Node Editor");
                ui.separator();
                ui.label(format!("Zoom: {:.1}x", self.zoom));
                ui.label(format!(
                    "Pan: ({:.0}, {:.0})",
                    self.pan_offset.x, self.pan_offset.y
                ));
            });

            let response = ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());
            let painter = ui.painter();

            // Apply zoom and pan transforms
            let zoom = self.zoom;
            let pan_offset = self.pan_offset;

            let transform_pos = |pos: Pos2| -> Pos2 {
                Pos2::new(pos.x * zoom + pan_offset.x, pos.y * zoom + pan_offset.y)
            };

            let inverse_transform_pos = |pos: Pos2| -> Pos2 {
                Pos2::new((pos.x - pan_offset.x) / zoom, (pos.y - pan_offset.y) / zoom)
            };

            // Handle pan and zoom
            if ui.input(|i| i.pointer.middle_down()) && response.dragged() {
                self.pan_offset += response.drag_delta();
            }

            // Handle zoom with mouse wheel
            let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
            if scroll_delta != 0.0 {
                if let Some(mouse_pos) = response.hover_pos() {
                    self.zoom_at_point(mouse_pos, scroll_delta * 0.001);
                }
            }

            // Handle right-click for context menu first (before other input handling)
            if response.secondary_clicked() {
                if let Some(screen_pos) = response.interact_pointer_pos() {
                    let pos = inverse_transform_pos(screen_pos);
                    let mut clicked_on_node = false;
                    for (id, node) in &self.nodes {
                        if node.get_rect().contains(pos) {
                            self.selected_nodes.clear();
                            self.selected_nodes.insert(*id);
                            clicked_on_node = true;
                            break;
                        }
                    }

                    if !clicked_on_node {
                        // Right-clicked on empty space, show context menu
                        self.context_menu_pos = Some(screen_pos);
                    }
                }
            }

            // Handle input
            if let Some(screen_pos) = response.interact_pointer_pos() {
                let pos = inverse_transform_pos(screen_pos);
                // Skip node interaction if we're panning
                let is_panning = ui.input(|i| i.pointer.middle_down());

                if !is_panning {
                    // Handle clicks (not just drags)
                    if response.clicked() {
                        let mut clicked_node = None;
                        let mut clicked_port = false;
                        let mut clicked_connection = None;

                        // First check if we clicked on a port
                        for (node_id, node) in &self.nodes {
                            // Check output ports
                            for (port_idx, port) in node.outputs.iter().enumerate() {
                                if (port.position - pos).length() < 10.0 {
                                    clicked_port = true;
                                    // If we already have a connection in progress, try to complete it
                                    if let Some((from_node, from_port, from_is_input)) =
                                        self.connecting_from
                                    {
                                        if from_is_input && *node_id != from_node {
                                            // Connecting from input to output
                                            self.connections.push(Connection {
                                                from_node: *node_id,
                                                from_port: port_idx,
                                                to_node: from_node,
                                                to_port: from_port,
                                            });
                                            self.connecting_from = None;
                                        } else {
                                            // Start new connection from this output
                                            self.connecting_from =
                                                Some((*node_id, port_idx, false));
                                        }
                                    } else {
                                        // Start new connection from this output
                                        self.connecting_from = Some((*node_id, port_idx, false));
                                    }
                                    break;
                                }
                            }
                            if clicked_port {
                                break;
                            }

                            // Check input ports
                            for (port_idx, port) in node.inputs.iter().enumerate() {
                                if (port.position - pos).length() < 10.0 {
                                    clicked_port = true;
                                    // If we already have a connection in progress, try to complete it
                                    if let Some((from_node, from_port, from_is_input)) =
                                        self.connecting_from
                                    {
                                        if !from_is_input && *node_id != from_node {
                                            // Connecting from output to input
                                            self.connections.push(Connection {
                                                from_node,
                                                from_port,
                                                to_node: *node_id,
                                                to_port: port_idx,
                                            });
                                            self.connecting_from = None;
                                        } else {
                                            // Start new connection from this input
                                            self.connecting_from = Some((*node_id, port_idx, true));
                                        }
                                    } else {
                                        // Start new connection from this input
                                        self.connecting_from = Some((*node_id, port_idx, true));
                                    }
                                    break;
                                }
                            }
                            if clicked_port {
                                break;
                            }
                        }

                        // If not clicking on a port, check for node
                        if !clicked_port {
                            for (id, node) in &self.nodes {
                                if node.get_rect().contains(pos) {
                                    clicked_node = Some(*id);
                                    break;
                                }
                            }

                            if let Some(node_id) = clicked_node {
                                // Handle multi-selection with Ctrl/Cmd
                                if ui.input(|i| i.modifiers.ctrl || i.modifiers.command) {
                                    if self.selected_nodes.contains(&node_id) {
                                        self.selected_nodes.remove(&node_id);
                                    } else {
                                        self.selected_nodes.insert(node_id);
                                    }
                                } else {
                                    // Single selection - clear others and select this one
                                    self.selected_nodes.clear();
                                    self.selected_nodes.insert(node_id);
                                }
                                self.selected_connection = None;
                            } else {
                                // Check if clicked on a connection
                                for (idx, connection) in self.connections.iter().enumerate() {
                                    if let (Some(from_node), Some(to_node)) = (
                                        self.nodes.get(&connection.from_node),
                                        self.nodes.get(&connection.to_node),
                                    ) {
                                        if let (Some(from_port), Some(to_port)) = (
                                            from_node.outputs.get(connection.from_port),
                                            to_node.inputs.get(connection.to_port),
                                        ) {
                                            let from_pos = from_port.position;
                                            let to_pos = to_port.position;

                                            // Check if click is near the bezier curve (vertical flow)
                                            let vertical_distance = (to_pos.y - from_pos.y).abs();
                                            let control_offset = if vertical_distance > 10.0 {
                                                vertical_distance * 0.4
                                            } else {
                                                60.0 // Minimum offset for short connections
                                            };
                                            let num_segments = 20;
                                            for i in 0..num_segments {
                                                let t = i as f32 / num_segments as f32;
                                                let t2 = (i + 1) as f32 / num_segments as f32;

                                                let p1 = cubic_bezier_point(
                                                    t,
                                                    from_pos,
                                                    from_pos + Vec2::new(0.0, control_offset),
                                                    to_pos - Vec2::new(0.0, control_offset),
                                                    to_pos,
                                                );
                                                let p2 = cubic_bezier_point(
                                                    t2,
                                                    from_pos,
                                                    from_pos + Vec2::new(0.0, control_offset),
                                                    to_pos - Vec2::new(0.0, control_offset),
                                                    to_pos,
                                                );

                                                // Check distance to line segment
                                                let dist = distance_to_line_segment(pos, p1, p2);
                                                if dist < 5.0 {
                                                    clicked_connection = Some(idx);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    if clicked_connection.is_some() {
                                        break;
                                    }
                                }

                                if let Some(conn_idx) = clicked_connection {
                                    self.selected_connection = Some(conn_idx);
                                    self.selected_nodes.clear();
                                } else {
                                    // Clicked on empty space, deselect all
                                    self.selected_nodes.clear();
                                    self.selected_connection = None;
                                }
                            }
                        }
                    }

                    // Handle drag start for connections, node movement and box selection
                    if response.drag_started() {
                        let mut dragging_port = false;
                        let mut dragging_selected = false;
                        let mut clicked_node_id = None;

                        // First check if we're starting to drag from a port
                        for (node_id, node) in &self.nodes {
                            // Check output ports
                            for (port_idx, port) in node.outputs.iter().enumerate() {
                                if (port.position - pos).length() < 10.0 {
                                    dragging_port = true;
                                    self.connecting_from = Some((*node_id, port_idx, false));
                                    break;
                                }
                            }
                            if dragging_port {
                                break;
                            }

                            // Check input ports
                            for (port_idx, port) in node.inputs.iter().enumerate() {
                                if (port.position - pos).length() < 10.0 {
                                    dragging_port = true;
                                    self.connecting_from = Some((*node_id, port_idx, true));
                                    break;
                                }
                            }
                            if dragging_port {
                                break;
                            }
                        }

                        // If not dragging from a port, handle node dragging
                        if !dragging_port {
                            // Check if we're starting to drag a currently selected node
                            for &node_id in &self.selected_nodes {
                                if let Some(node) = self.nodes.get(&node_id) {
                                    if node.get_rect().contains(pos) {
                                        // Calculate drag offsets for all selected nodes
                                        self.drag_offsets.clear();
                                        for &selected_id in &self.selected_nodes {
                                            if let Some(selected_node) =
                                                self.nodes.get(&selected_id)
                                            {
                                                self.drag_offsets.insert(
                                                    selected_id,
                                                    selected_node.position - pos,
                                                );
                                            }
                                        }
                                        dragging_selected = true;
                                        break;
                                    }
                                }
                            }

                            // If not dragging a selected node, check if we clicked on any node
                            if !dragging_selected {
                                for (node_id, node) in &self.nodes {
                                    if node.get_rect().contains(pos) {
                                        clicked_node_id = Some(*node_id);
                                        break;
                                    }
                                }

                                if let Some(node_id) = clicked_node_id {
                                    // Select the node and start dragging it
                                    self.selected_nodes.clear();
                                    self.selected_nodes.insert(node_id);

                                    // Set up drag offset for this node
                                    self.drag_offsets.clear();
                                    if let Some(node) = self.nodes.get(&node_id) {
                                        self.drag_offsets.insert(node_id, node.position - pos);
                                    }
                                } else {
                                    // Start box selection if not on any node and using left mouse button
                                    if ui.input(|i| i.pointer.primary_down()) {
                                        self.box_selection_start = Some(pos);
                                        self.box_selection_end = Some(pos);
                                    }
                                }
                            }
                        }
                    }

                    // Handle dragging
                    if response.dragged() {
                        if !self.drag_offsets.is_empty() {
                            // Drag all selected nodes
                            for (&node_id, &offset) in &self.drag_offsets {
                                if let Some(node) = self.nodes.get_mut(&node_id) {
                                    node.position = pos + offset;
                                }
                            }
                        } else if self.box_selection_start.is_some() {
                            // Update box selection
                            self.box_selection_end = Some(pos);
                        }
                    }

                    // Handle connection completion
                    if response.drag_stopped() {
                        if let Some((from_node, from_port, from_is_input)) = self.connecting_from {
                            // Check if we released on a port
                            for (node_id, node) in &self.nodes {
                                if from_is_input {
                                    // Connecting from input, look for output
                                    for (port_idx, port) in node.outputs.iter().enumerate() {
                                        if (port.position - pos).length() < 10.0
                                            && *node_id != from_node
                                        {
                                            self.connections.push(Connection {
                                                from_node: *node_id,
                                                from_port: port_idx,
                                                to_node: from_node,
                                                to_port: from_port,
                                            });
                                            break;
                                        }
                                    }
                                } else {
                                    // Connecting from output, look for input
                                    for (port_idx, port) in node.inputs.iter().enumerate() {
                                        if (port.position - pos).length() < 10.0
                                            && *node_id != from_node
                                        {
                                            self.connections.push(Connection {
                                                from_node,
                                                from_port,
                                                to_node: *node_id,
                                                to_port: port_idx,
                                            });
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        self.connecting_from = None;
                    }
                }

                if response.drag_stopped() {
                    self.drag_offsets.clear();

                    // Complete box selection
                    if let (Some(start), Some(end)) =
                        (self.box_selection_start, self.box_selection_end)
                    {
                        let selection_rect = Rect::from_two_pos(start, end);

                        // Handle multi-selection with Ctrl/Cmd
                        if !ui.input(|i| i.modifiers.ctrl || i.modifiers.command) {
                            self.selected_nodes.clear();
                        }

                        // Find all nodes in selection box
                        for (node_id, node) in &self.nodes {
                            if selection_rect.intersects(node.get_rect()) {
                                self.selected_nodes.insert(*node_id);
                            }
                        }

                        self.box_selection_start = None;
                        self.box_selection_end = None;
                    }
                }
            }

            // Handle keyboard input
            if ui.input(|i| i.key_pressed(egui::Key::Delete)) {
                if !self.selected_nodes.is_empty() {
                    // Delete all selected nodes
                    for &node_id in &self.selected_nodes {
                        self.nodes.remove(&node_id);
                        // Remove connections to/from this node
                        self.connections
                            .retain(|conn| conn.from_node != node_id && conn.to_node != node_id);
                    }
                    self.selected_nodes.clear();
                } else if let Some(conn_idx) = self.selected_connection {
                    self.connections.remove(conn_idx);
                    self.selected_connection = None;
                }
            }

            // Handle ESC key to cancel connections
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.connecting_from = None;
            }

            // Update port positions
            for node in self.nodes.values_mut() {
                node.update_port_positions();
            }

            // Draw connections
            for (idx, connection) in self.connections.iter().enumerate() {
                if let (Some(from_node), Some(to_node)) = (
                    self.nodes.get(&connection.from_node),
                    self.nodes.get(&connection.to_node),
                ) {
                    if let (Some(from_port), Some(to_port)) = (
                        from_node.outputs.get(connection.from_port),
                        to_node.inputs.get(connection.to_port),
                    ) {
                        let from_pos = from_port.position;
                        let to_pos = to_port.position;

                        // Transform connection positions
                        let transformed_from = transform_pos(from_pos);
                        let transformed_to = transform_pos(to_pos);

                        // Draw bezier curve (vertical flow: top to bottom)
                        let vertical_distance = (transformed_to.y - transformed_from.y).abs();
                        let control_offset = if vertical_distance > 10.0 {
                            vertical_distance * 0.4
                        } else {
                            60.0 * zoom // Minimum offset for short connections
                        };

                        let points = [
                            transformed_from,
                            transformed_from + Vec2::new(0.0, control_offset),
                            transformed_to - Vec2::new(0.0, control_offset),
                            transformed_to,
                        ];

                        // Highlight selected connection
                        let (stroke_width, stroke_color) = if Some(idx) == self.selected_connection
                        {
                            (4.0 * zoom, Color32::from_rgb(255, 200, 100)) // Orange for selected
                        } else {
                            (2.0 * zoom, Color32::from_rgb(150, 150, 150)) // Gray for normal
                        };

                        painter.add(egui::Shape::CubicBezier(egui::epaint::CubicBezierShape {
                            points,
                            closed: false,
                            fill: Color32::TRANSPARENT,
                            stroke: Stroke::new(stroke_width, stroke_color).into(),
                        }));
                    }
                }
            }

            // Draw nodes
            for (node_id, node) in &self.nodes {
                // Transform node rectangle
                let node_rect = node.get_rect();
                let transformed_rect =
                    Rect::from_two_pos(transform_pos(node_rect.min), transform_pos(node_rect.max));

                // Node background
                painter.rect_filled(transformed_rect, 5.0 * zoom, node.color);

                // Node border - highlight if selected
                let border_color = if self.selected_nodes.contains(node_id) {
                    Color32::from_rgb(255, 200, 100) // Orange for selected
                } else {
                    Color32::from_rgb(100, 100, 100) // Gray for normal
                };
                let border_width = if self.selected_nodes.contains(node_id) {
                    3.0
                } else {
                    2.0
                };

                painter.rect_stroke(
                    transformed_rect,
                    5.0 * zoom,
                    Stroke::new(border_width * zoom, border_color),
                );

                // Title
                painter.text(
                    transform_pos(node.position + Vec2::new(node.size.x / 2.0, 15.0)),
                    egui::Align2::CENTER_CENTER,
                    &node.title,
                    egui::FontId::proportional(12.0 * zoom),
                    Color32::WHITE,
                );

                // Draw ports
                let port_radius = 5.0 * zoom;

                // Input ports (on top)
                for input in &node.inputs {
                    painter.circle_filled(
                        transform_pos(input.position),
                        port_radius,
                        Color32::from_rgb(100, 150, 100),
                    );
                    painter.text(
                        transform_pos(input.position - Vec2::new(0.0, 15.0)),
                        egui::Align2::CENTER_BOTTOM,
                        &input.name,
                        egui::FontId::proportional(10.0 * zoom),
                        Color32::from_gray(200),
                    );
                }

                // Output ports (on bottom)
                for output in &node.outputs {
                    painter.circle_filled(
                        transform_pos(output.position),
                        port_radius,
                        Color32::from_rgb(150, 100, 100),
                    );
                    painter.text(
                        transform_pos(output.position + Vec2::new(0.0, 15.0)),
                        egui::Align2::CENTER_TOP,
                        &output.name,
                        egui::FontId::proportional(10.0 * zoom),
                        Color32::from_gray(200),
                    );
                }
            }

            // Draw current connection being made
            if let Some((from_node, from_port, from_is_input)) = self.connecting_from {
                if let Some(mouse_pos) = response.hover_pos() {
                    if let Some(node) = self.nodes.get(&from_node) {
                        let from_pos = if from_is_input {
                            node.inputs[from_port].position
                        } else {
                            node.outputs[from_port].position
                        };

                        let transformed_from = transform_pos(from_pos);
                        let _to_pos = inverse_transform_pos(mouse_pos);
                        let transformed_to = mouse_pos;

                        // Draw bezier curve for connection preview (vertical flow)
                        let vertical_distance = (transformed_to.y - transformed_from.y).abs();
                        let control_offset = if vertical_distance > 10.0 {
                            vertical_distance * 0.4
                        } else {
                            60.0 * zoom
                        };

                        let points = [
                            transformed_from,
                            transformed_from + Vec2::new(0.0, control_offset),
                            transformed_to - Vec2::new(0.0, control_offset),
                            transformed_to,
                        ];

                        painter.add(egui::Shape::CubicBezier(egui::epaint::CubicBezierShape {
                            points,
                            closed: false,
                            fill: Color32::TRANSPARENT,
                            stroke: Stroke::new(2.0 * zoom, Color32::from_rgb(255, 255, 100))
                                .into(),
                        }));
                    }
                }
            }

            // Draw box selection
            if let (Some(start), Some(end)) = (self.box_selection_start, self.box_selection_end) {
                let selection_rect = Rect::from_two_pos(transform_pos(start), transform_pos(end));

                // Draw selection box background
                painter.rect_filled(
                    selection_rect,
                    0.0,
                    Color32::from_rgba_unmultiplied(100, 150, 255, 30),
                );

                // Draw selection box border
                painter.rect_stroke(
                    selection_rect,
                    0.0,
                    Stroke::new(1.0 * zoom, Color32::from_rgb(100, 150, 255)),
                );
            }

            // Show context menu
            if let Some(menu_screen_pos) = self.context_menu_pos {
                let menu_world_pos = inverse_transform_pos(menu_screen_pos);
                let popup_id = egui::Id::new("context_menu");

                let menu_response =
                    egui::Area::new(popup_id)
                        .fixed_pos(menu_screen_pos)
                        .show(ui.ctx(), |ui| {
                            egui::Frame::popup(ui.style())
                                .show(ui, |ui| {
                                    ui.set_min_width(120.0);

                                    ui.label("Create Node:");
                                    ui.separator();

                                    // Math nodes
                                    if ui.button("Add").clicked() {
                                        self.create_node("Add", menu_world_pos);
                                        self.context_menu_pos = None;
                                    }
                                    if ui.button("Subtract").clicked() {
                                        self.create_node("Subtract", menu_world_pos);
                                        self.context_menu_pos = None;
                                    }
                                    if ui.button("Multiply").clicked() {
                                        self.create_node("Multiply", menu_world_pos);
                                        self.context_menu_pos = None;
                                    }
                                    if ui.button("Divide").clicked() {
                                        self.create_node("Divide", menu_world_pos);
                                        self.context_menu_pos = None;
                                    }

                                    ui.separator();

                                    // Logic nodes
                                    if ui.button("AND").clicked() {
                                        self.create_node("AND", menu_world_pos);
                                        self.context_menu_pos = None;
                                    }
                                    if ui.button("OR").clicked() {
                                        self.create_node("OR", menu_world_pos);
                                        self.context_menu_pos = None;
                                    }
                                    if ui.button("NOT").clicked() {
                                        self.create_node("NOT", menu_world_pos);
                                        self.context_menu_pos = None;
                                    }

                                    ui.separator();

                                    // Data nodes
                                    if ui.button("Constant").clicked() {
                                        self.create_node("Constant", menu_world_pos);
                                        self.context_menu_pos = None;
                                    }
                                    if ui.button("Variable").clicked() {
                                        self.create_node("Variable", menu_world_pos);
                                        self.context_menu_pos = None;
                                    }

                                    ui.separator();

                                    // Output nodes
                                    if ui.button("Print").clicked() {
                                        self.create_node("Print", menu_world_pos);
                                        self.context_menu_pos = None;
                                    }
                                    if ui.button("Debug").clicked() {
                                        self.create_node("Debug", menu_world_pos);
                                        self.context_menu_pos = None;
                                    }
                                })
                                .inner
                        });

                // Close menu if clicked outside the menu area
                if ui.input(|i| i.pointer.primary_clicked()) {
                    if let Some(click_pos) = ui.input(|i| i.pointer.interact_pos()) {
                        if !menu_response.response.rect.contains(click_pos) {
                            self.context_menu_pos = None;
                        }
                    }
                }

                // Also close on Escape key
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.context_menu_pos = None;
                }
            }
        });
    }
}

/// Application entry point.
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Nodle",
        options,
        Box::new(|_cc| Ok(Box::new(NodeEditor::new()))),
    )
}
