//! Parameter panel implementation
//! 
//! Handles parameter-type interface panels that are typically stacked on the right side

use egui::{Context, Color32, Pos2};
use crate::nodes::{Node, NodeId, InterfacePanelManager};
use crate::editor::panels::PanelAction;
use std::collections::HashMap;
use log::info;

/// Parameter panel renderer
pub struct ParameterPanel {
    /// Tracks which parameter panels are in stacked mode
    stacked_panels: HashMap<NodeId, bool>,
}

impl ParameterPanel {
    pub fn new() -> Self {
        Self {
            stacked_panels: HashMap::new(),
        }
    }

    /// Render parameter panels (handles both stacked and individual)
    pub fn render(
        &mut self,
        ctx: &Context,
        node_id: NodeId,
        node: &Node,
        panel_manager: &mut InterfacePanelManager,
        menu_bar_height: f32,
        viewed_nodes: &std::collections::HashMap<NodeId, Node>,
        graph: &mut crate::nodes::NodeGraph,
    ) -> PanelAction {
        // Check if this panel should be stacked
        if panel_manager.is_panel_stacked(node_id) {
            // For stacked panels, only render the shared window from the first stacked node
            // to avoid creating multiple windows
            let stacked_parameter_nodes = panel_manager.get_stacked_panels_by_type(
                crate::nodes::interface::PanelType::Parameter, 
                viewed_nodes
            );
            
            if !stacked_parameter_nodes.is_empty() {
                // Find the designated renderer for the stacked window
                let first_node_id = stacked_parameter_nodes[0];
                
                if node_id == first_node_id {
                    // This is the designated renderer, render the shared window
                    self.render_stacked_panels(ctx, &stacked_parameter_nodes, panel_manager, menu_bar_height, viewed_nodes, graph)
                } else {
                    // This is not the designated renderer, but if the designated renderer
                    // is not visible or the first node's panel is closed, allow this node to render
                    let first_node_visible = viewed_nodes.get(&first_node_id)
                        .map(|node| node.visible && panel_manager.is_panel_visible(first_node_id))
                        .unwrap_or(false);
                    
                    if !first_node_visible || !panel_manager.is_panel_open(first_node_id) {
                        // First node can't render the window, so this node should do it
                        self.render_stacked_panels(ctx, &stacked_parameter_nodes, panel_manager, menu_bar_height, viewed_nodes, graph)
                    } else {
                        // First node is handling the window
                        PanelAction::None
                    }
                }
            } else {
                PanelAction::None
            }
        } else {
            self.render_individual_panel(ctx, node_id, node, panel_manager, menu_bar_height, graph)
        }
    }

    /// Render an individual parameter panel
    fn render_individual_panel(
        &mut self,
        ctx: &Context,
        node_id: NodeId,
        node: &Node,
        panel_manager: &mut InterfacePanelManager,
        menu_bar_height: f32,
        graph: &mut crate::nodes::NodeGraph,
    ) -> PanelAction {
        // Check if panel is marked as visible
        if !panel_manager.is_panel_visible(node_id) {
            return PanelAction::None;
        }
        let panel_id = egui::Id::new(format!("parameter_panel_{}", node_id));
        let mut panel_action = PanelAction::None;
        
        // Get current window open state
        let mut window_open = panel_manager.is_panel_open(node_id);
        
        // Parameter panel specific position - top right corner, close to edge
        let screen_rect = ctx.screen_rect();
        let position = Pos2::new(screen_rect.max.x - 10.0, screen_rect.min.y + menu_bar_height + 10.0);
        
        // Create parameter panel window
        let window_title = format!("{} Parameters", node.title);
        egui::Window::new(&window_title)
            .id(panel_id)
            .default_pos(position)
            .default_size(crate::constants::panel::DEFAULT_PARAMETER_SIZE)
            .min_size(crate::constants::panel::MIN_PARAMETER_SIZE)
            .max_size(crate::constants::panel::MAX_PARAMETER_SIZE)
            .resizable(true)
            .collapsible(true)
            .open(&mut window_open)
            .constrain_to(egui::Rect::from_min_size(
                egui::Pos2::new(0.0, menu_bar_height), 
                egui::Vec2::new(screen_rect.width(), screen_rect.height() - menu_bar_height)
            ))
            .show(ctx, |ui| {
                // Panel controls at the top
                let (control_action, close_requested) = self.render_panel_controls(ui, node_id, panel_manager);
                if control_action != PanelAction::None {
                    panel_action = control_action;
                }
                if close_requested {
                    panel_action = PanelAction::Close;
                }
                
                ui.separator();
                
                // Node-specific content
                egui::Frame::none()
                    .inner_margin(egui::Margin::same(8.0))
                    .fill(Color32::from_gray(40))
                    .rounding(egui::Rounding::same(4.0))
                    .show(ui, |ui| {
                        self.render_parameter_content(ui, node_id, node, panel_manager, graph);
                    });
            });
        
        // Update the panel manager with the new state
        panel_manager.set_panel_open(node_id, window_open);
        
        // Check if window was closed via X button
        if !window_open {
            panel_action = PanelAction::Close;
        }
        
        panel_action
    }

    /// Render multiple parameter panels in stacked mode (all in one window)
    fn render_stacked_panels(
        &mut self,
        ctx: &Context,
        stacked_node_ids: &[NodeId],
        panel_manager: &mut InterfacePanelManager,
        menu_bar_height: f32,
        viewed_nodes: &std::collections::HashMap<NodeId, Node>,
        graph: &mut crate::nodes::NodeGraph,
    ) -> PanelAction {
        let mut panel_action = PanelAction::None;
        
        if stacked_node_ids.is_empty() {
            return PanelAction::None;
        }
        
        // Use the first node's open state for the shared window
        let first_node_id = stacked_node_ids[0];
        let mut window_open = panel_manager.is_panel_open(first_node_id);
        
        // Parameter panel stacked positioning - full height, right edge
        let screen_rect = ctx.screen_rect();
        let panel_width = crate::constants::panel::STACKED_PARAMETER_WIDTH;
        let panel_height = screen_rect.height() - menu_bar_height;
        let window_pos = [screen_rect.max.x - panel_width, menu_bar_height];
        
        // Create stacked parameter panel window (full height, right edge, fixed size)
        let window_title = format!("Parameter Panels ({})", stacked_node_ids.len());
        egui::Window::new(window_title)
            .id(egui::Id::new("stacked_parameter_panels"))
            .fixed_pos(window_pos)
            .fixed_size([panel_width, panel_height])
            .resizable(false) // Fixed size to maintain alignment
            .collapsible(false) // Disable collapse to maintain full height
            .open(&mut window_open)
            .constrain_to(egui::Rect::from_min_size(
                egui::Pos2::new(0.0, menu_bar_height), 
                egui::Vec2::new(screen_rect.width(), screen_rect.height() - menu_bar_height)
            ))
            .show(ctx, |ui| {
                // Scrollable area for all stacked interfaces - use full available height
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // Calculate the consistent content width for all panels
                        // The issue is that egui's ScrollArea reserves different amounts of space
                        // for the first item vs subsequent items. We need to normalize this.
                        let available_width = ui.available_width();
                        // No scrollbar spacing - let egui handle it automatically
                        let scrollbar_width = 0.0;
                        let content_width = (available_width - scrollbar_width).max(100.0);
                        
                        // Render each stacked parameter node
                        for &node_id in stacked_node_ids {
                            if let Some(node) = viewed_nodes.get(&node_id) {
                                
                                // Create a consistent-width container for this panel
                                ui.allocate_ui_with_layout(
                                    egui::vec2(content_width, 0.0),
                                    egui::Layout::top_down(egui::Align::LEFT),
                                    |ui| {
                                        ui.set_width(content_width);
                                        
                                        // Panel controls for this node
                                        let (panel_control_action, close_requested) = self.render_panel_controls(ui, node_id, panel_manager);
                                        if panel_control_action != PanelAction::None {
                                            panel_action = panel_control_action;
                                        }
                                        if close_requested {
                                            panel_action = PanelAction::Close;
                                        }
                                        
                                        // Separator with negative margin to extend to window edge
                                        egui::Frame::none()
                                            .inner_margin(egui::Margin {
                                                left: 0.0,
                                                right: -6.0,  // Negative margin to push closer to edge
                                                top: 0.0,
                                                bottom: 0.0,
                                            })
                                            .show(ui, |ui| {
                                                ui.separator();
                                            });
                                        
                                        // Node content in a contained frame
                                        egui::Frame::none()
                                            .inner_margin(egui::Margin::same(8.0))
                                            .fill(Color32::from_gray(45))
                                            .rounding(egui::Rounding::same(4.0))
                                            .stroke(egui::Stroke::new(1.0, Color32::from_gray(80)))
                                            .show(ui, |ui| {
                                                self.render_parameter_content(ui, node_id, node, panel_manager, graph);
                                            });
                                    });
                                
                                ui.add_space(10.0); // Space between nodes
                            }
                        }
                    });
            });
        
        // Update all stacked nodes' panel open state
        for &node_id in stacked_node_ids {
            panel_manager.set_panel_open(node_id, window_open);
        }
        
        // Check if window was closed via X button
        if !window_open {
            panel_action = PanelAction::Close;
        }
        
        panel_action
    }

    /// Render panel controls (stack/pin buttons)
    fn render_panel_controls(
        &mut self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        panel_manager: &mut InterfacePanelManager,
    ) -> (PanelAction, bool) {
        let mut panel_action = PanelAction::None;
        let mut close_requested = false;
        
        ui.horizontal(|ui| {
            ui.label("Panel controls:");
            
            // Stack button
            let is_stacked = panel_manager.is_panel_stacked(node_id);
            let stack_text = if is_stacked { "📚 Stacked" } else { "📄 Stack" };
            let stack_color = if is_stacked { 
                Color32::from_rgb(100, 150, 255) 
            } else { 
                Color32::from_gray(180) 
            };
            
            if ui.button(egui::RichText::new(stack_text).color(stack_color)).clicked() {
                panel_action = PanelAction::ToggleStack;
            }
            
            // Pin button  
            let is_pinned = panel_manager.is_panel_pinned(node_id);
            let pin_text = if is_pinned { "📌 Pinned" } else { "📍 Pin" };
            let pin_color = if is_pinned { 
                Color32::from_rgb(255, 150, 100) 
            } else { 
                Color32::from_gray(180) 
            };
            
            if ui.button(egui::RichText::new(pin_text).color(pin_color)).clicked() {
                panel_action = PanelAction::TogglePin;
            }
            
            // Close button
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("X").clicked() {
                    close_requested = true;
                }
            });
        });
        
        (panel_action, close_requested)
    }

    /// Render parameter-specific content
    fn render_parameter_content(
        &mut self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        node: &Node,
        panel_manager: &mut InterfacePanelManager,
        graph: &mut crate::nodes::NodeGraph,
    ) {
        println!("📋 PANEL START: Node '{}' (ID: {}) - Title in passed node: '{}'", node.title, node_id, node.title);
        
        // Check what the title is in the graph vs the passed node
        if let Some(graph_node) = graph.nodes.get(&node_id) {
            println!("📋 PANEL START: Title in graph: '{}'", graph_node.title);
            if graph_node.title != node.title {
                println!("⚠️  MISMATCH: Graph title '{}' != passed node title '{}'", graph_node.title, node.title);
            }
        }
        
        // Get the actual node's title for editing
        let mut name_buffer = node.title.clone();
        println!("📋 PANEL START: name_buffer initialized to: '{}'", name_buffer);
        
        // Get current fit name flag
        let mut fit_name = panel_manager.get_fit_name(node_id);
        
        ui.horizontal(|ui| {
            ui.label("Name:");
            
            println!("📝 NAME EDIT: About to render text field with buffer: '{}'", name_buffer);
            
            // Name text field - directly edit the node's title
            let name_response = ui.text_edit_singleline(&mut name_buffer);
            
            println!("📝 NAME EDIT: After text field, buffer is now: '{}'", name_buffer);
            println!("📝 NAME EDIT: Text field changed: {}", name_response.changed());
            
            if name_response.changed() {
                println!("🔄 NAME CHANGE: User changed name from '{}' to '{}'", node.title, name_buffer);
                
                // Update the actual node's title in the graph
                if let Some(node_mut) = graph.nodes.get_mut(&node_id) {
                    println!("🔄 NAME CHANGE: Updating graph node title to '{}'", name_buffer);
                    node_mut.title = name_buffer.clone();
                    
                    // If fit name is enabled, resize the node to fit the new title
                    if fit_name {
                        // Calculate new size based on title length (rough estimate)
                        let char_width = 8.0; // Approximate character width
                        let padding = 40.0; // Extra padding for ports and margins
                        let min_width = 120.0; // Minimum node width
                        let new_width = (name_buffer.len() as f32 * char_width + padding).max(min_width);
                        node_mut.size.x = new_width;
                        node_mut.update_port_positions(); // Update port positions after resize
                        println!("🔄 NAME CHANGE: Resized node to width {}", new_width);
                    }
                    
                    println!("✅ NAME CHANGE: Graph node title updated to '{}'", node_mut.title);
                } else {
                    println!("❌ NAME CHANGE: Could not find node {} in graph!", node_id);
                }
            }
            
            // Fit name checkbox
            let fit_response = ui.checkbox(&mut fit_name, "Fit name");
            if fit_response.changed() {
                panel_manager.set_fit_name(node_id, fit_name);
                
                // If fit name was just enabled, resize the node immediately
                if fit_name {
                    if let Some(node_mut) = graph.nodes.get_mut(&node_id) {
                        let char_width = 8.0;
                        let padding = 40.0;
                        let min_width = 120.0;
                        let new_width = (node_mut.title.len() as f32 * char_width + padding).max(min_width);
                        node_mut.size.x = new_width;
                        node_mut.update_port_positions();
                    }
                }
            }
        });
        
        ui.separator();
        
        // Show node info
        ui.label(format!("Node: {}", node.title));
        ui.label(format!("Type: {:?}", node.node_type));
        ui.label(format!("Position: ({:.1}, {:.1})", node.position.x, node.position.y));
        
        ui.separator();
        
        // Show ports
        ui.label("Input Ports:");
        for (i, input) in node.inputs.iter().enumerate() {
            ui.label(format!("  {}: {}", i, input.name));
        }
        
        ui.label("Output Ports:");
        for (i, output) in node.outputs.iter().enumerate() {
            ui.label(format!("  {}: {}", i, output.name));
        }
        
        ui.separator();
        
        // Use proper parameter interface for all nodes that have build_interface methods
        let handled = if let Some(node_mut) = graph.nodes.get_mut(&node_id) {
            self.render_node_interface(node_mut, ui, node_id)
        } else {
            false
        };
        
        // Fallback: render basic parameter display for nodes without proper interfaces
        if !handled && !node.parameters.is_empty() {
            ui.label("Parameters:");
            
            for (param_name, param_value) in &node.parameters {
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", param_name));
                    match param_value {
                        crate::nodes::interface::NodeData::String(s) => {
                            ui.label(s);
                        }
                        crate::nodes::interface::NodeData::Boolean(b) => {
                            ui.label(format!("{}", b));
                        }
                        crate::nodes::interface::NodeData::Float(f) => {
                            ui.label(format!("{:.2}", f));
                        }
                        crate::nodes::interface::NodeData::Integer(i) => {
                            ui.label(format!("{}", i));
                        }
                        _ => {
                            ui.label("Complex parameter");
                        }
                    }
                });
            }
        } else {
            ui.label("No parameters available");
        }
        
        ui.separator();
        ui.label(format!("Node ID: {}", node_id));
    }
    
    /// Render the proper parameter interface using Pattern A: build_interface method
    fn render_node_interface(&mut self, node: &mut crate::nodes::Node, ui: &mut egui::Ui, node_id: NodeId) -> bool {
        // ONLY Pattern A: build_interface(node, ui) method for ALL nodes
        self.render_build_interface_pattern(node, ui, node_id)
    }
    
    /// Pattern A: build_interface(node, ui) method for ALL nodes
    fn render_build_interface_pattern(&mut self, node: &mut crate::nodes::Node, ui: &mut egui::Ui, node_id: NodeId) -> bool {
        let title = node.title.clone();
        
        // Debug output for every individual node (not workspace)
        if matches!(node.node_type, crate::nodes::NodeType::Regular) {
            println!("🔍 INDIVIDUAL NODE: '{}' (id: {})", title, node_id);
        }
        
        // Handle plugin nodes first
        if let Some(plugin_node) = &mut node.plugin_node {
            println!("🎛️ RENDERING PLUGIN PARAMETERS for: {}", title);
            
            // Render plugin parameters
            let parameter_changes = plugin_node.render_parameters(ui);
            
            // Apply parameter changes to the plugin node
            for change in parameter_changes {
                println!("🔄 Parameter change: {} = {:?}", change.parameter, change.value);
                plugin_node.set_parameter(&change.parameter, change.value);
            }
            
            info!("Rendered parameters for plugin node: {}", title);
            return true;
        }
        
        // USD nodes are now handled by plugins - no core implementation needed
        
        // Math nodes using Pattern A
        if title.contains("Add") || title.contains("Addition") {
            let changes = crate::nodes::math::add::parameters::AddNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id);
            return true;
        }
        
        // Math nodes using Pattern A
        if title.contains("Subtract") || title.contains("Subtraction") {
            let changes = crate::nodes::math::subtract::parameters::SubtractNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id);
            return true;
        }
        
        if title.contains("Multiply") || title.contains("Multiplication") {
            let changes = crate::nodes::math::multiply::parameters::MultiplyNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id);
            return true;
        }
        
        if title.contains("Divide") || title.contains("Division") {
            let changes = crate::nodes::math::divide::parameters::DivideNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id);
            return true;
        }
        
        // Geometry nodes using Pattern A
        if title.contains("Sphere") && !title.contains("USD") {
            let changes = crate::nodes::three_d::geometry::sphere::parameters::SphereNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id);
            return true;
        }
        
        if title.contains("Cube") && !title.contains("USD") {
            let changes = crate::nodes::three_d::geometry::cube::parameters::CubeNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id);
            return true;
        }
        
        // Data nodes using Pattern A
        if title.contains("Constant") {
            let changes = crate::nodes::data::constant::parameters::ConstantNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id);
            return true;
        }
        
        if title.contains("Variable") {
            let changes = crate::nodes::data::variable::parameters::VariableNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id);
            return true;
        }
        
        // Output nodes using Pattern A
        if title.contains("Debug") {
            let changes = crate::nodes::output::debug::parameters::DebugNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id);
            return true;
        }
        
        if title.contains("Print") {
            let changes = crate::nodes::output::print::parameters::PrintNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id);
            return true;
        }
        
        // Logic nodes using Pattern A
        if title.contains("And") && !title.contains("USD") {
            let changes = crate::nodes::logic::and::parameters::AndNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id);
            return true;
        }
        
        if title.contains("Or") && !title.contains("USD") {
            let changes = crate::nodes::logic::or::parameters::OrNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id);
            return true;
        }
        
        if title.contains("Not") && !title.contains("USD") {
            let changes = crate::nodes::logic::not::parameters::NotNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id);
            return true;
        }
        
        // Transform nodes using Pattern A
        if title.contains("Translate") && !title.contains("USD") {
            let changes = crate::nodes::three_d::transform::translate::parameters::TranslateNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id);
            return true;
        }
        
        if title.contains("Scale") && !title.contains("USD") {
            let changes = crate::nodes::three_d::transform::scale::parameters::ScaleNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id);
            return true;
        }
        
        if title.contains("Rotate") && !title.contains("USD") {
            let changes = crate::nodes::three_d::transform::rotate::parameters::RotateNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id);
            return true;
        }
        
        // Lighting nodes using Pattern A
        if title.contains("Spot Light") || (title.contains("Spot") && title.contains("Light")) {
            let changes = crate::nodes::three_d::lighting::spot_light::parameters::SpotLightNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id);
            return true;
        }
        
        if title.contains("Directional Light") || (title.contains("Directional") && title.contains("Light")) {
            let changes = crate::nodes::three_d::lighting::directional_light::parameters::DirectionalLightNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id);
            return true;
        }
        
        if title.contains("Point Light") || (title.contains("Point") && title.contains("Light")) {
            let changes = crate::nodes::three_d::lighting::point_light::parameters::PointLightNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id);
            return true;
        }
        
        // Geometry nodes using Pattern A
        if title.contains("Plane") && !title.contains("USD") {
            let changes = crate::nodes::three_d::geometry::plane::parameters::PlaneNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id);
            return true;
        }
        
        // Utility nodes using Pattern A
        if title.contains("Null") {
            let changes = crate::nodes::utility::null::parameters::NullNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id);
            return true;
        }
        
        if title.contains("Test") {
            println!("🧪 FOUND TEST NODE! Rendering custom interface for: '{}'", title);
            println!("🧪 TEST NODE: Node has {} parameters", node.parameters.len());
            let changes = crate::nodes::utility::test::parameters::TestNode::build_interface(node, ui);
            println!("🧪 TEST NODE: build_interface returned {} parameter changes", changes.len());
            self.apply_parameter_changes(node, changes, &title, node_id);
            println!("🧪 TEST NODE: Applied parameter changes successfully");
            return true;
        }
        
        false
    }
    
    
    
    /// Apply parameter changes from Pattern A build_interface method
    fn apply_parameter_changes(
        &mut self, 
        node: &mut crate::nodes::Node, 
        changes: Vec<crate::nodes::interface::ParameterChange>, 
        title: &str, 
        node_id: NodeId
    ) {
        if !changes.is_empty() {
            info!("Applied {} parameter changes for {} node {}", changes.len(), title, node_id);
            for change in changes {
                node.parameters.insert(change.parameter, change.value);
            }
        }
    }
    
}