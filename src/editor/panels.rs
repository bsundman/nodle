//! Interface panel management system for the node editor
//!
//! Handles rendering and managing interface panels for nodes, including
//! parameter editing, panel positioning, and state management.

use egui::{Pos2, Ui, Context, Color32};
use crate::nodes::{
    NodeGraph, Node, NodeId, InterfacePanelManager, PanelType,
};
use std::collections::HashMap;

// Import GraphView from the parent module
use super::GraphView;

/// Actions that can be performed on interface panels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelAction {
    None,
    Close,
    CloseAll, // Close all panels in a stacked window
    Minimize,
    Restore,
    ToggleStack,
    TogglePin,
}

/// Manages interface panels for the node editor
pub struct PanelManager {
    /// The core interface panel manager for state tracking
    interface_panel_manager: InterfacePanelManager,
    /// Current menu bar height for window constraints
    current_menu_bar_height: f32,
}

impl PanelManager {
    /// Create a new panel manager
    pub fn new() -> Self {
        Self {
            interface_panel_manager: InterfacePanelManager::new(),
            current_menu_bar_height: 0.0,
        }
    }

    /// Get a reference to the underlying interface panel manager
    pub fn interface_panel_manager(&self) -> &InterfacePanelManager {
        &self.interface_panel_manager
    }

    /// Get a mutable reference to the underlying interface panel manager
    pub fn interface_panel_manager_mut(&mut self) -> &mut InterfacePanelManager {
        &mut self.interface_panel_manager
    }

    /// Set the current menu bar height for window constraints
    pub fn set_menu_bar_height(&mut self, height: f32) {
        self.current_menu_bar_height = height;
    }

    /// Create a constrained window that respects the menu bar
    fn create_window<'a>(title: &'a str, ctx: &Context, menu_bar_height: f32) -> egui::Window<'a> {
        egui::Window::new(title)
            .constrain_to(egui::Rect::from_min_size(
                egui::Pos2::new(0.0, menu_bar_height), 
                egui::Vec2::new(ctx.screen_rect().width(), ctx.screen_rect().height() - menu_bar_height)
            ))
    }

    /// Render all interface panels for the given nodes
    pub fn render_interface_panels(
        &mut self, 
        ui: &mut Ui, 
        viewed_nodes: &HashMap<NodeId, Node>, 
        menu_bar_height: f32,
        current_view: &GraphView,
        graph: &mut NodeGraph,
    ) {
        // Store menu bar height
        self.set_menu_bar_height(menu_bar_height);
        let ctx = ui.ctx();
        
        // Track which nodes to close and which actions to apply (to avoid borrowing issues)
        let mut nodes_to_close: Vec<NodeId> = Vec::new();
        let mut nodes_to_toggle_stack: Vec<NodeId> = Vec::new();
        let mut nodes_to_toggle_pin: Vec<NodeId> = Vec::new();
        
        // Separate nodes into stacked and non-stacked groups
        let mut stacked_nodes: Vec<(NodeId, &Node)> = Vec::new();
        let mut individual_nodes: Vec<(NodeId, &Node)> = Vec::new();
        
        for (&node_id, node) in viewed_nodes {
            if node.visible {
                if self.interface_panel_manager.is_panel_stacked(node_id) {
                    stacked_nodes.push((node_id, node));
                } else {
                    individual_nodes.push((node_id, node));
                }
            }
        }
        
        // Render stacked nodes in a single scrollable window
        if !stacked_nodes.is_empty() {
            let panel_action = self.render_stacked_panels(ctx, &stacked_nodes, menu_bar_height);
            self.handle_stacked_panel_action(panel_action, &stacked_nodes, &mut nodes_to_close, &mut nodes_to_toggle_stack, &mut nodes_to_toggle_pin);
        }
        
        // Render individual nodes in separate windows
        for (node_id, node) in individual_nodes {
            let panel_position = self.get_panel_position(ui, PanelType::Parameter, menu_bar_height);
            let panel_action = self.render_universal_interface_panel(ctx, node_id, node, panel_position);
            
            // Handle panel actions
            match panel_action {
                PanelAction::Close => nodes_to_close.push(node_id),
                PanelAction::CloseAll => nodes_to_close.push(node_id), // Individual panels don't use CloseAll
                PanelAction::ToggleStack => nodes_to_toggle_stack.push(node_id),
                PanelAction::TogglePin => nodes_to_toggle_pin.push(node_id),
                PanelAction::None | PanelAction::Minimize | PanelAction::Restore => {
                    // egui handles minimize/restore automatically with collapsible(true)
                }
            }
        }
        
        // Apply panel actions (after iteration to avoid borrowing conflicts)
        for node_id in nodes_to_close {
            self.close_node_panel(node_id, current_view, graph);
        }
        
        // Apply stack toggle actions
        for node_id in nodes_to_toggle_stack {
            self.interface_panel_manager.toggle_panel_stacked(node_id);
        }
        
        // Apply pin toggle actions  
        for node_id in nodes_to_toggle_pin {
            self.interface_panel_manager.toggle_panel_pinned(node_id);
        }
    }

    /// Get the default position for a panel based on its type
    fn get_panel_position(&self, ui: &Ui, panel_type: PanelType, menu_bar_height: f32) -> Pos2 {
        let screen_rect = ui.ctx().screen_rect();
        
        match panel_type {
            PanelType::Parameter => {
                // Top right corner - close to edge and below menu bar
                Pos2::new(screen_rect.max.x - 10.0, screen_rect.min.y + menu_bar_height + 10.0)
            },
            PanelType::Viewer => {
                // Bottom right corner
                Pos2::new(screen_rect.max.x - 400.0, screen_rect.max.y - 300.0)
            },
            PanelType::Editor => {
                // Center of screen, constrained below menu bar
                Pos2::new(screen_rect.center().x - 200.0, (screen_rect.center().y - 150.0).max(screen_rect.min.y + menu_bar_height + 10.0))
            },
            PanelType::Inspector => {
                // Bottom left corner
                Pos2::new(screen_rect.min.x + 20.0, screen_rect.max.y - 250.0)
            },
        }
    }

    /// Render stacked panels in a single scrollable window
    fn render_stacked_panels(
        &mut self,
        ctx: &Context,
        stacked_nodes: &[(NodeId, &Node)],
        menu_bar_height: f32,
    ) -> (NodeId, PanelAction) {
        let mut result_action = PanelAction::None;
        let mut result_node_id = 0; // Default node ID
        
        // Get window open state for the stack (use first node's ID for tracking)
        if let Some((first_node_id, _)) = stacked_nodes.first() {
            result_node_id = *first_node_id;
            let mut window_open = self.interface_panel_manager.is_panel_open(*first_node_id);
            
            let screen_rect = ctx.screen_rect();
            let panel_width = 400.0;
            let panel_height = screen_rect.height() - menu_bar_height;
            
            Self::create_window("Interface Panel Stack", ctx, menu_bar_height)
                .id(egui::Id::new("stacked_panels"))
                .fixed_pos([screen_rect.max.x - panel_width, menu_bar_height])
                .fixed_size([panel_width, panel_height])
                .resizable(false) // Fixed size to maintain right edge alignment
                .collapsible(false) // Disable collapse to maintain full height
                .open(&mut window_open)
                .show(ctx, |ui| {
                    // Scrollable area for all stacked interfaces - use full available height
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            // Force consistent width for all content
                            let full_width = ui.available_width();
                            
                            for (node_id, node) in stacked_nodes {
                                // Create a full-width container for each panel
                                ui.allocate_ui_with_layout(
                                    egui::vec2(full_width, 0.0),
                                    egui::Layout::top_down(egui::Align::LEFT),
                                    |ui| {
                                        ui.set_width(full_width);
                                    // Panel controls for this node - directly rendered like individual panels
                                    let (panel_control_action, close_requested) = self.render_panel_controls(ui, *node_id);
                                    if panel_control_action != PanelAction::None {
                                        result_action = panel_control_action;
                                        result_node_id = *node_id;
                                    }
                                    if close_requested {
                                        result_action = PanelAction::Close;
                                        result_node_id = *node_id;
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
                                    
                                    // Node content in a contained frame - same as individual panels
                                    egui::Frame::none()
                                        .inner_margin(egui::Margin::same(8.0))
                                        .fill(Color32::from_gray(45))
                                        .rounding(egui::Rounding::same(4.0))
                                        .stroke(egui::Stroke::new(1.0, Color32::from_gray(80)))
                                        .show(ui, |ui| {
                                            let (_, _) = self.render_node_header_and_content(ui, *node_id, node);
                                        });
                                });
                                
                                ui.add_space(10.0); // Space between nodes
                            }
                        });
                });
            
            // Update window open state for the stack
            self.interface_panel_manager.set_panel_open(*first_node_id, window_open);
            
            // If window was closed, close all stacked panels (this is when the window X is clicked)
            if !window_open {
                result_action = PanelAction::CloseAll;
            }
        }
        
        (result_node_id, result_action)
    }
    
    /// Handle panel actions from stacked panels
    fn handle_stacked_panel_action(
        &self,
        (node_id, action): (NodeId, PanelAction),
        stacked_nodes: &[(NodeId, &Node)],
        nodes_to_close: &mut Vec<NodeId>,
        nodes_to_toggle_stack: &mut Vec<NodeId>,
        nodes_to_toggle_pin: &mut Vec<NodeId>,
    ) {
        match action {
            PanelAction::Close => {
                // Only close the specific node that requested closure, not all stacked panels
                nodes_to_close.push(node_id);
            }
            PanelAction::CloseAll => {
                // Close all stacked panels (when the entire stacked window is closed)
                for (stacked_node_id, _) in stacked_nodes {
                    nodes_to_close.push(*stacked_node_id);
                }
            }
            PanelAction::ToggleStack => nodes_to_toggle_stack.push(node_id),
            PanelAction::TogglePin => nodes_to_toggle_pin.push(node_id),
            PanelAction::None | PanelAction::Minimize | PanelAction::Restore => {}
        }
    }

    /// Close a node's interface panel and disable its visibility flag
    fn close_node_panel(
        &mut self, 
        node_id: NodeId, 
        current_view: &GraphView,
        graph: &mut NodeGraph,
    ) {
        // Find the node in the correct graph based on current view and set its visibility to false
        match current_view {
            GraphView::Root => {
                if let Some(node) = graph.nodes.get_mut(&node_id) {
                    node.visible = false;
                }
            }
            GraphView::WorkspaceNode(workspace_node_id) => {
                if let Some(workspace_node) = graph.nodes.get_mut(workspace_node_id) {
                    if let Some(internal_graph) = workspace_node.get_internal_graph_mut() {
                        if let Some(node) = internal_graph.nodes.get_mut(&node_id) {
                            node.visible = false;
                        }
                    }
                }
            }
        }
        
        // Clear all panel state
        self.interface_panel_manager.set_panel_visibility(node_id, false);
        self.interface_panel_manager.set_panel_minimized(node_id, false);
        self.interface_panel_manager.set_panel_open(node_id, true); // Reset for next time
    }

    /// Render universal interface panel for any node type
    fn render_universal_interface_panel(
        &mut self, 
        ctx: &Context, 
        node_id: NodeId, 
        node: &Node, 
        position: Pos2,
    ) -> PanelAction {
        let panel_id = egui::Id::new(format!("interface_panel_{}", node_id));
        let mut panel_action = PanelAction::None;
        
        // Get current window open state (avoiding borrowing conflicts)
        let mut window_open = self.interface_panel_manager.is_panel_open(node_id);
        
        // Use global window creator with automatic menu bar constraint
        let _window_response = Self::create_window(&format!("{} Panel", node.title), ctx, self.current_menu_bar_height)
            .id(panel_id)
            .default_pos(position)
            .resizable(true)
            .collapsible(true) // Enable built-in collapse/minimize button
            .open(&mut window_open) // Track if window is closed via X button
            .constrain(true) // Enable automatic constraint
            .show(ctx, |ui| {
                // Panel controls at the very top
                let (panel_control_action, close_requested) = self.render_panel_controls(ui, node_id);
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
                
                // Node-specific content in a contained section
                egui::Frame::none()
                    .inner_margin(egui::Margin::same(8.0))
                    .fill(Color32::from_gray(40))
                    .rounding(egui::Rounding::same(4.0))
                    .show(ui, |ui| {
                        // Render node header and content
                        let (header_changed, _) = self.render_node_header_and_content(ui, node_id, node);
                        
                        // Apply node name changes if header was modified
                        if header_changed {
                            // Note: This would need access to the graph to apply changes
                            // For now, we'll handle this in the editor
                        }
                    });
            });
        
        // Update the panel manager with the new state
        self.interface_panel_manager.set_panel_open(node_id, window_open);
        
        // Check if window was closed via X button
        if !window_open {
            panel_action = PanelAction::Close;
        }
        
        panel_action
    }

    /// Render panel controls (stack/pin buttons) at the top of the panel
    fn render_panel_controls(&mut self, ui: &mut Ui, node_id: NodeId) -> (PanelAction, bool) {
        let mut panel_action = PanelAction::None;
        let mut close_requested = false;
        
        // Use a frame with negative right margin to push closer to window edge
        egui::Frame::none()
            .inner_margin(egui::Margin {
                left: 0.0,
                right: -6.0,  // Negative margin to push closer to edge
                top: 0.0,
                bottom: 0.0,
            })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Panel controls:");
                    
                    // Stack button
                    let is_stacked = self.interface_panel_manager.is_panel_stacked(node_id);
                    let stack_text = if is_stacked { "📚 Stacked" } else { "📄 Stack" };
                    let stack_color = if is_stacked { 
                        Color32::from_rgb(100, 150, 255) 
                    } else { 
                        Color32::from_gray(180) 
                    };
                    
                    let stack_button = ui.button(egui::RichText::new(stack_text).color(stack_color));
                    if stack_button.clicked() {
                        panel_action = PanelAction::ToggleStack;
                    }
                    
                    // Pin button  
                    let is_pinned = self.interface_panel_manager.is_panel_pinned(node_id);
                    let pin_text = if is_pinned { "📌 Pinned" } else { "📍 Pin" };
                    let pin_color = if is_pinned { 
                        Color32::from_rgb(255, 150, 100) 
                    } else { 
                        Color32::from_gray(180) 
                    };
                    
                    let pin_button = ui.button(egui::RichText::new(pin_text).color(pin_color));
                    if pin_button.clicked() {
                        panel_action = PanelAction::TogglePin;
                    }
                    
                    // Simple approach: use with_layout to create a right-aligned section for the close button
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Close button using standard X character that renders properly
                        let close_button = ui.small_button("X");
                        if close_button.clicked() {
                            close_requested = true;
                        }
                    });
                });
            });
        
        (panel_action, close_requested)
    }

    /// Render node header and content (previously render_standard_panel_header)
    fn render_node_header_and_content(&mut self, ui: &mut Ui, node_id: NodeId, node: &Node) -> (bool, PanelAction) {
        let mut changed = false;
        
        // Get current custom name or use node's default title
        // But strip any existing "..." truncation to show the full name in the editor
        let current_name = self.interface_panel_manager.get_node_name(node_id)
            .cloned()
            .unwrap_or_else(|| {
                // If the node title has "..." truncation, we need to get the original name
                // For now, just use the node's current title
                node.title.clone()
            });
        let mut name_buffer = current_name;
        
        // Get current fit name flag
        let mut fit_name = self.interface_panel_manager.get_fit_name(node_id);
        
        ui.horizontal(|ui| {
            ui.label("Name:");
            
            // Name text field
            let name_response = ui.text_edit_singleline(&mut name_buffer);
            if name_response.changed() {
                self.interface_panel_manager.set_node_name(node_id, name_buffer.clone());
                changed = true;
            }
            
            // Fit name checkbox on the same line
            let fit_response = ui.checkbox(&mut fit_name, "Fit name");
            if fit_response.changed() {
                self.interface_panel_manager.set_fit_name(node_id, fit_name);
                changed = true;
            }
        });
        
        // Also show current effective name and size info for debugging
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Current node title:");
            ui.label(&node.title);
        });
        ui.horizontal(|ui| {
            ui.label("Node width:");
            ui.label(format!("{:.0}px", node.size.x));
        });
        
        ui.separator();
        
        // Render node-specific content
        self.render_node_specific_content(ui, node_id, node);
        
        (changed, PanelAction::None)
    }

    /// Render node-specific content in the interface panel
    fn render_node_specific_content(&mut self, ui: &mut Ui, node_id: NodeId, node: &Node) {
        // Default content for all nodes
        ui.label(format!("Node: {}", node.title));
        ui.label(format!("Type: {:?}", node.node_type));
        ui.label(format!("Position: ({:.1}, {:.1})", node.position.x, node.position.y));
        
        // Additional information
        ui.separator();
        ui.label("Input Ports:");
        for (i, input) in node.inputs.iter().enumerate() {
            ui.label(format!("  {}: {}", i, input.name));
        }
        
        ui.label("Output Ports:");
        for (i, output) in node.outputs.iter().enumerate() {
            ui.label(format!("  {}: {}", i, output.name));
        }
        
        ui.separator();
        ui.label(format!("Node ID: {}", node_id));
    }

    /// Apply node name and sizing changes to the actual node
    pub fn apply_node_name_changes(
        &mut self, 
        node_id: NodeId, 
        current_view: &GraphView,
        graph: &mut NodeGraph,
    ) {
        // Find the node in the appropriate graph
        if let Some(node) = graph.nodes.get_mut(&node_id) {
            // Update node title with custom name if set, handling truncation
            if let Some(custom_name) = self.interface_panel_manager.get_node_name(node_id) {
                let fit_name = self.interface_panel_manager.get_fit_name(node_id);
                
                if fit_name {
                    // Use full name and adjust node size using proper text measurement
                    node.title = custom_name.clone();
                    
                    // Determine minimum width based on node title
                    let min_width: f32 = match node.title.as_str() {
                        "Add" | "Subtract" | "Multiply" | "Divide" => 80.0,
                        "AND" | "OR" | "NOT" => 60.0,
                        "Constant" | "Variable" => 100.0,
                        "Print" | "Debug" => 80.0,
                        _ => 120.0, // Default for complex nodes
                    };
                    
                    // Calculate text width (approximate)
                    let text_width = custom_name.len() as f32 * 8.0; // Rough approximation
                    let padding = 20.0; // Padding on both sides
                    let required_width = text_width + padding;
                    
                    // Use the larger of minimum width or required text width
                    let new_width = min_width.max(required_width);
                    node.size.x = new_width;
                    
                    // Update port positions after size change
                    node.update_port_positions();
                } else {
                    // Use truncated name with original size
                    let max_chars = ((node.size.x - 20.0) / 8.0) as usize; // Rough char calculation
                    if custom_name.len() > max_chars {
                        node.title = format!("{}...", &custom_name[..max_chars.saturating_sub(3)]);
                    } else {
                        node.title = custom_name.clone();
                    }
                }
            }
        }
    }

    /// Render cube interface panel (legacy, specific to certain node types)
    pub fn render_cube_interface_panel(&mut self, ctx: &Context, node_id: NodeId, _node: &Node, position: Pos2) {
        let panel_id = egui::Id::new(format!("cube_interface_{}", node_id));
        
        Self::create_window("Cube Parameters", ctx, self.current_menu_bar_height)
            .id(panel_id)
            .default_pos(position)
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label("Cube Parameters");
                ui.separator();
                
                ui.label("This is a specialized interface for cube nodes.");
                ui.label("Parameters would appear here.");
                
                ui.separator();
                
                if ui.button("Close Panel").clicked() {
                    self.interface_panel_manager.set_panel_visibility(node_id, false);
                }
            });
    }

    /// Render sphere interface panel (legacy, specific to certain node types)
    pub fn render_sphere_interface_panel(&mut self, ctx: &Context, node_id: NodeId, _node: &Node, position: Pos2) {
        let panel_id = egui::Id::new(format!("sphere_interface_{}", node_id));
        
        Self::create_window("Sphere Parameters", ctx, self.current_menu_bar_height)
            .id(panel_id)
            .default_pos(position)
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label("Sphere Parameters");
                ui.separator();
                
                ui.label("This is a specialized interface for sphere nodes.");
                ui.label("Parameters would appear here.");
                
                ui.separator();
                
                if ui.button("Close Panel").clicked() {
                    self.interface_panel_manager.set_panel_visibility(node_id, false);
                }
            });
    }

    /// Render translate interface panel (legacy, specific to certain node types)
    pub fn render_translate_interface_panel(&mut self, ctx: &Context, node_id: NodeId, _node: &Node, position: Pos2) {
        let panel_id = egui::Id::new(format!("translate_interface_{}", node_id));
        
        Self::create_window("Translate Parameters", ctx, self.current_menu_bar_height)
            .id(panel_id)
            .default_pos(position)
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label("Translate Parameters");
                ui.separator();
                
                ui.label("This is a specialized interface for translate nodes.");
                ui.label("Parameters would appear here.");
                
                ui.separator();
                
                if ui.button("Close Panel").clicked() {
                    self.interface_panel_manager.set_panel_visibility(node_id, false);
                }
            });
    }

    /// Render generic interface panel (legacy, fallback for any node type)
    pub fn render_generic_interface_panel(&mut self, ctx: &Context, node_id: NodeId, node: &Node, position: Pos2) {
        let panel_id = egui::Id::new(format!("interface_{}", node_id));
        
        Self::create_window(&format!("{} Interface", node.title), ctx, self.current_menu_bar_height)
            .id(panel_id)
            .default_pos(position)
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label(format!("Interface panel for {}", node.title));
                ui.separator();
                
                ui.label("This node has an interface panel.");
                ui.label("Parameters would appear here.");
                
                ui.separator();
                
                if ui.button("Close Panel").clicked() {
                    self.interface_panel_manager.set_panel_visibility(node_id, false);
                }
            });
    }
}

impl Default for PanelManager {
    fn default() -> Self {
        Self::new()
    }
}