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
    open_submenu: Option<String>, // Track which submenu is open
    submenu_pos: Option<Pos2>,    // Position for the submenu
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
            open_submenu: None,
            submenu_pos: None,
            pan_offset: Vec2::ZERO,
            zoom: 1.0,
            box_selection_start: None,
            box_selection_end: None,
            drag_offsets: HashMap::new(),
        };

        // Start with empty node graph

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
        if let Some(node) = crate::NodeRegistry::create_node(node_type, position) {
            self.graph.add_node(node);
        }
    }

    #[allow(dead_code)]
    fn add_test_nodes(&mut self) {
        crate::create_test_nodes(&mut self.graph);
    }
}

impl eframe::App for NodeEditor {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Request repaint for smooth updates
        ctx.request_repaint();
        
        // Set dark theme for window decorations
        ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(egui::SystemTheme::Dark));

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(40, 40, 40)))
            .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Nōdle - Node Editor");
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
                        let mut new_connection: Option<nodle_core::Connection> = None;
                        let mut new_connecting_from: Option<(nodle_core::NodeId, usize, bool)> = None;

                        // First check if we clicked on a port
                        for (node_id, node) in &self.graph.nodes {
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
                                            new_connection = Some(nodle_core::Connection::new(*node_id, port_idx, from_node, from_port));
                                            new_connecting_from = None;
                                        } else {
                                            // Start new connection from this output
                                            new_connecting_from = Some((*node_id, port_idx, false));
                                        }
                                    } else {
                                        // Start new connection from this output
                                        new_connecting_from = Some((*node_id, port_idx, false));
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
                                            new_connection = Some(nodle_core::Connection::new(from_node, from_port, *node_id, port_idx));
                                            new_connecting_from = None;
                                        } else {
                                            // Start new connection from this input
                                            new_connecting_from = Some((*node_id, port_idx, true));
                                        }
                                    } else {
                                        // Start new connection from this input
                                        new_connecting_from = Some((*node_id, port_idx, true));
                                    }
                                    break;
                                }
                            }
                            if clicked_port {
                                break;
                            }
                        }

                        // Apply any connection changes after the loop
                        let connection_made = new_connection.is_some();
                        if let Some(connection) = new_connection {
                            let _ = self.graph.add_connection(connection);
                        }
                        if let Some(connecting) = new_connecting_from {
                            self.connecting_from = Some(connecting);
                        } else if connection_made {
                            self.connecting_from = None;
                        }

                        // If not clicking on a port, check for node
                        if !clicked_port {
                            for (id, node) in &self.graph.nodes {
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
                                // Clicked on empty space, deselect all
                                self.selected_nodes.clear();
                                self.selected_connection = None;
                            }
                        }
                    }

                    // Handle drag start for connections, node movement and box selection
                    if response.drag_started() {
                        let mut dragging_port = false;
                        let mut dragging_selected = false;
                        let mut clicked_node_id = None;

                        // First check if we're starting to drag from a port
                        for (node_id, node) in &self.graph.nodes {
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
                                if let Some(node) = self.graph.nodes.get(&node_id) {
                                    if node.get_rect().contains(pos) {
                                        // Calculate drag offsets for all selected nodes
                                        self.drag_offsets.clear();
                                        for &selected_id in &self.selected_nodes {
                                            if let Some(selected_node) =
                                                self.graph.nodes.get(&selected_id)
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
                                for (node_id, node) in &self.graph.nodes {
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
                                    if let Some(node) = self.graph.nodes.get(&node_id) {
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
                                if let Some(node) = self.graph.nodes.get_mut(&node_id) {
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
                            let mut drag_connection: Option<nodle_core::Connection> = None;
                            for (node_id, node) in &self.graph.nodes {
                                if from_is_input {
                                    // Connecting from input, look for output
                                    for (port_idx, port) in node.outputs.iter().enumerate() {
                                        if (port.position - pos).length() < 10.0
                                            && *node_id != from_node
                                        {
                                            drag_connection = Some(nodle_core::Connection::new(*node_id, port_idx, from_node, from_port));
                                            break;
                                        }
                                    }
                                } else {
                                    // Connecting from output, look for input
                                    for (port_idx, port) in node.inputs.iter().enumerate() {
                                        if (port.position - pos).length() < 10.0
                                            && *node_id != from_node
                                        {
                                            drag_connection = Some(nodle_core::Connection::new(from_node, from_port, *node_id, port_idx));
                                            break;
                                        }
                                    }
                                }
                                if drag_connection.is_some() {
                                    break;
                                }
                            }
                            
                            // Apply connection after loop
                            if let Some(connection) = drag_connection {
                                let _ = self.graph.add_connection(connection);
                            }
                        }
                        // Cancel connection if we're releasing and not connecting
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
                        for (node_id, node) in &self.graph.nodes {
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
                        self.graph.remove_node(node_id);
                    }
                    self.selected_nodes.clear();
                } else if let Some(conn_idx) = self.selected_connection {
                    self.graph.remove_connection(conn_idx);
                    self.selected_connection = None;
                }
            }

            // Handle ESC key to cancel connections
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.connecting_from = None;
            }

            // Handle right-click for context menu first (before other input handling)
            if response.secondary_clicked() {
                if let Some(screen_pos) = response.interact_pointer_pos() {
                    let pos = inverse_transform_pos(screen_pos);
                    let mut clicked_on_node = false;
                    for (id, node) in &self.graph.nodes {
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

            // Show context menu with submenus
            if let Some(menu_screen_pos) = self.context_menu_pos {
                let menu_world_pos = inverse_transform_pos(menu_screen_pos);
                let popup_id = egui::Id::new("context_menu");

                let menu_response =
                    egui::Area::new(popup_id)
                        .fixed_pos(menu_screen_pos)
                        .show(ui.ctx(), |ui| {
                            egui::Frame::popup(ui.style())
                                .show(ui, |ui| {
                                    // Calculate menu width based on category names
                                    let categories = ["Create Node:", "Math", "Logic", "Data", "Output"];
                                    let text_width = categories.iter()
                                        .map(|text| ui.fonts(|f| f.glyph_width(&egui::FontId::default(), ' ') * text.len() as f32))
                                        .fold(0.0, f32::max);
                                    let menu_width = (text_width + ui.spacing().button_padding.x * 2.0 + 20.0).max(120.0); // +20 for arrow
                                    ui.set_min_width(menu_width);
                                    ui.set_max_width(menu_width);

                                    ui.label("Create Node:");
                                    ui.separator();

                                    // Helper function to create menu items with full-width highlighting and arrow
                                    let submenu_item = |ui: &mut egui::Ui, text: &str, open_submenu: &mut Option<String>, submenu_pos: &mut Option<Pos2>| -> bool {
                                        let desired_size = egui::Vec2::new(menu_width, ui.spacing().button_padding.y * 2.0 + ui.text_style_height(&egui::TextStyle::Body));
                                        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
                                        
                                        if ui.is_rect_visible(rect) {
                                            let visuals = ui.style().interact(&response);
                                            
                                            // Fill background on hover
                                            if response.hovered() {
                                                ui.painter().rect_filled(rect, 0.0, visuals.bg_fill);
                                                *open_submenu = Some(text.to_string());
                                                *submenu_pos = Some(Pos2::new(rect.right(), rect.top()));
                                            }
                                            
                                            // Draw text
                                            ui.painter().text(
                                                rect.left_center() + egui::vec2(ui.spacing().button_padding.x, 0.0),
                                                egui::Align2::LEFT_CENTER,
                                                text,
                                                egui::FontId::default(),
                                                visuals.text_color(),
                                            );
                                            
                                            // Draw arrow
                                            ui.painter().text(
                                                rect.right_center() - egui::vec2(ui.spacing().button_padding.x, 0.0),
                                                egui::Align2::RIGHT_CENTER,
                                                "▶",
                                                egui::FontId::default(),
                                                visuals.text_color(),
                                            );
                                        }
                                        
                                        response.clicked()
                                    };

                                    // Category menu items
                                    submenu_item(ui, "Math", &mut self.open_submenu, &mut self.submenu_pos);
                                    submenu_item(ui, "Logic", &mut self.open_submenu, &mut self.submenu_pos);
                                    submenu_item(ui, "Data", &mut self.open_submenu, &mut self.submenu_pos);
                                    submenu_item(ui, "Output", &mut self.open_submenu, &mut self.submenu_pos);
                                })
                                .inner
                        });

                // Show submenu if one is open
                if let (Some(submenu_name), Some(submenu_screen_pos)) = (self.open_submenu.clone(), self.submenu_pos) {
                    let submenu_id = egui::Id::new(format!("submenu_{}", submenu_name));
                    
                    let submenu_response = egui::Area::new(submenu_id)
                        .fixed_pos(submenu_screen_pos)
                        .show(ui.ctx(), |ui| {
                            egui::Frame::popup(ui.style())
                                .show(ui, |ui| {
                                    // Get node items for this category
                                    let node_items = match submenu_name.as_str() {
                                        "Math" => vec!["Add", "Subtract", "Multiply", "Divide"],
                                        "Logic" => vec!["AND", "OR", "NOT"],
                                        "Data" => vec!["Constant", "Variable"],
                                        "Output" => vec!["Print", "Debug"],
                                        _ => vec![],
                                    };

                                    // Calculate submenu width
                                    let text_width = node_items.iter()
                                        .map(|text| ui.fonts(|f| f.glyph_width(&egui::FontId::default(), ' ') * text.len() as f32))
                                        .fold(0.0, f32::max);
                                    let submenu_width = (text_width + ui.spacing().button_padding.x * 2.0).max(80.0);
                                    ui.set_min_width(submenu_width);
                                    ui.set_max_width(submenu_width);

                                    // Helper for submenu items
                                    let submenu_node_item = |ui: &mut egui::Ui, text: &str| -> bool {
                                        let desired_size = egui::Vec2::new(submenu_width, ui.spacing().button_padding.y * 2.0 + ui.text_style_height(&egui::TextStyle::Body));
                                        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
                                        
                                        if ui.is_rect_visible(rect) {
                                            let visuals = ui.style().interact(&response);
                                            
                                            // Fill background on hover
                                            if response.hovered() {
                                                ui.painter().rect_filled(rect, 0.0, visuals.bg_fill);
                                            }
                                            
                                            // Draw text
                                            ui.painter().text(
                                                rect.left_center() + egui::vec2(ui.spacing().button_padding.x, 0.0),
                                                egui::Align2::LEFT_CENTER,
                                                text,
                                                egui::FontId::default(),
                                                visuals.text_color(),
                                            );
                                        }
                                        
                                        response.clicked()
                                    };

                                    // Draw submenu items
                                    for node_type in node_items {
                                        if submenu_node_item(ui, node_type) {
                                            self.create_node(node_type, menu_world_pos);
                                            self.context_menu_pos = None;
                                            self.open_submenu = None;
                                            self.submenu_pos = None;
                                        }
                                    }
                                })
                                .inner
                        });

                    // Close submenu if mouse moves away from both main menu and submenu
                    if let Some(mouse_pos) = ui.input(|i| i.pointer.interact_pos()) {
                        if !menu_response.response.rect.contains(mouse_pos) && 
                           !submenu_response.response.rect.contains(mouse_pos) {
                            // Add a small delay/buffer area between menu and submenu
                            let buffer_rect = egui::Rect::from_two_pos(
                                menu_response.response.rect.right_top(),
                                submenu_response.response.rect.left_bottom()
                            );
                            if !buffer_rect.contains(mouse_pos) {
                                self.open_submenu = None;
                                self.submenu_pos = None;
                            }
                        }
                    }
                }

                // Close entire menu if clicked outside all menu areas
                if ui.input(|i| i.pointer.primary_clicked()) {
                    if let Some(click_pos) = ui.input(|i| i.pointer.interact_pos()) {
                        let clicked_outside = !menu_response.response.rect.contains(click_pos);
                        
                        // Also check submenu if open
                        if let Some(_) = &self.open_submenu {
                            // We need to get the submenu rect, but it's already been computed above
                            // For now, we'll handle this in the submenu interaction logic
                        }
                        
                        if clicked_outside {
                            self.context_menu_pos = None;
                            self.open_submenu = None;
                            self.submenu_pos = None;
                        }
                    }
                }

                // Close on Escape key
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.context_menu_pos = None;
                    self.open_submenu = None;
                    self.submenu_pos = None;
                }
            }

            // Update port positions
            self.graph.update_all_port_positions();

            // Draw nodes
            for (node_id, node) in &self.graph.nodes {
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

            // Draw connections
            for (idx, connection) in self.graph.connections.iter().enumerate() {
                if let (Some(from_node), Some(to_node)) = (
                    self.graph.nodes.get(&connection.from_node),
                    self.graph.nodes.get(&connection.to_node),
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

            // Draw current connection being made
            if let Some((from_node, from_port, from_is_input)) = self.connecting_from {
                if let Some(mouse_pos) = response.hover_pos() {
                    if let Some(node) = self.graph.nodes.get(&from_node) {
                        let from_pos = if from_is_input {
                            node.inputs[from_port].position
                        } else {
                            node.outputs[from_port].position
                        };

                        let transformed_from = transform_pos(from_pos);
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
        });
    }
}