//! Viewport panel implementation
//! 
//! Handles viewport-type interface panels that are floating windows with 3D content

use egui::{Context, Color32, Pos2};
use crate::nodes::{Node, NodeId, InterfacePanelManager, NodeType};
use crate::nodes::interface::PanelType;
use crate::editor::panels::PanelAction;
use std::collections::HashMap;

/// Viewport panel renderer
pub struct ViewportPanel {
    /// Default viewport size
    default_size: [f32; 2],
    /// Selected tab for each stacked viewport window
    selected_tabs: HashMap<String, usize>,
    /// Viewport node instances (to maintain camera state)
    viewport_instances: HashMap<NodeId, crate::nodes::three_d::output::viewport::ViewportNode>,
}

impl ViewportPanel {
    pub fn new() -> Self {
        Self {
            default_size: [900.0, 700.0],
            selected_tabs: HashMap::new(),
            viewport_instances: HashMap::new(),
        }
    }

    /// Render viewport panels (handles both tabbed stacking and individual floating)
    pub fn render(
        &mut self,
        ctx: &Context,
        node_id: NodeId,
        node: &Node,
        panel_manager: &mut InterfacePanelManager,
        menu_bar_height: f32,
        viewed_nodes: &HashMap<NodeId, Node>,
    ) -> PanelAction {
        // Check if this panel should be stacked
        if panel_manager.is_panel_stacked(node_id) {
            // For stacked panels, only render the shared window from the first stacked node
            // to avoid creating multiple windows
            let stacked_viewport_nodes = panel_manager.get_stacked_panels_by_type(
                PanelType::Viewport, 
                viewed_nodes
            );
            
            if let Some(&first_node_id) = stacked_viewport_nodes.first() {
                if node_id == first_node_id {
                    // This is the first stacked viewport node, render the shared window
                    self.render_tabbed_viewports(ctx, &stacked_viewport_nodes, panel_manager, menu_bar_height, viewed_nodes)
                } else {
                    // This is not the first node, don't render a window (already handled by first node)
                    PanelAction::None
                }
            } else {
                PanelAction::None
            }
        } else {
            // Render as individual floating window (default)
            self.render_individual_viewport(ctx, node_id, node, panel_manager, menu_bar_height, viewed_nodes)
        }
    }

    /// Render an individual viewport panel (floating window)
    fn render_individual_viewport(
        &mut self,
        ctx: &Context,
        node_id: NodeId,
        node: &Node,
        panel_manager: &mut InterfacePanelManager,
        menu_bar_height: f32,
        viewed_nodes: &std::collections::HashMap<NodeId, Node>,
    ) -> PanelAction {
        let panel_id = egui::Id::new(format!("viewport_panel_{}", node_id));
        let mut panel_action = PanelAction::None;
        
        // Get current window open state
        let mut window_open = panel_manager.is_panel_open(node_id);
        
        // Viewport panel specific position - top left corner, touching edges
        let screen_rect = ctx.screen_rect();
        let position = Pos2::new(screen_rect.min.x, screen_rect.min.y + menu_bar_height);
        
        // Create viewport panel window
        let window_title = format!("{} Viewport", node.title);
        egui::Window::new(&window_title)
            .id(panel_id)
            .default_pos(position)
            .default_size(self.default_size)
            .max_size([1600.0, 1200.0])
            .resizable(true)
            .collapsible(true)
            .open(&mut window_open)
            .constrain_to(egui::Rect::from_min_size(
                egui::Pos2::new(0.0, menu_bar_height), 
                egui::Vec2::new(screen_rect.width(), screen_rect.height() - menu_bar_height)
            ))
            .show(ctx, |ui| {
                // Panel controls at the top
                let (control_action, close_requested) = self.render_panel_controls(ui, node_id, panel_manager, viewed_nodes);
                if control_action != PanelAction::None {
                    panel_action = control_action;
                }
                if close_requested {
                    panel_action = PanelAction::Close;
                }
                
                ui.separator();
                
                // Viewport-specific content - get or create viewport interface instance
                if let Some(crate::nodes::interface::PanelType::Viewport) = node.get_panel_type() {
                    let viewport_node = self.viewport_instances.entry(node_id)
                        .or_insert_with(|| crate::nodes::three_d::output::viewport::ViewportNode::default());
                    // Use Pattern A build_interface function
                    let _changes = crate::nodes::three_d::output::viewport::viewport_interface::build_interface(viewport_node, ui);
                } else {
                    ui.label("Error: Node does not have viewport panel type");
                }
            });
        
        // Update the panel manager with the new state
        panel_manager.set_panel_open(node_id, window_open);
        
        // Check if window was closed via X button
        if !window_open {
            panel_action = PanelAction::Close;
        }
        
        panel_action
    }

    /// Render multiple viewport panels in tabbed stacking mode
    fn render_tabbed_viewports(
        &mut self,
        ctx: &Context,
        stacked_node_ids: &[NodeId],
        panel_manager: &mut InterfacePanelManager,
        menu_bar_height: f32,
        viewed_nodes: &HashMap<NodeId, Node>,
    ) -> PanelAction {
        let mut panel_action = PanelAction::None;
        
        if stacked_node_ids.is_empty() {
            return PanelAction::None;
        }
        
        // Use the first node's open state for the shared window
        let first_node_id = stacked_node_ids[0];
        let mut window_open = panel_manager.is_panel_open(first_node_id);
        
        // Track which viewport tab is selected (default to first)
        let window_id = "tabbed_viewport_panels";
        let current_selected_tab = *self.selected_tabs.entry(window_id.to_string()).or_insert(0);
        
        // Ensure selected tab is valid
        let selected_tab_index = if current_selected_tab >= stacked_node_ids.len() {
            0
        } else {
            current_selected_tab
        };
        
        // Update the stored value if it changed
        if selected_tab_index != current_selected_tab {
            self.selected_tabs.insert(window_id.to_string(), selected_tab_index);
        }
        
        // Viewport tabbed stacking positioning - same as floating viewport (top left corner)
        let screen_rect = ctx.screen_rect();
        let position = Pos2::new(screen_rect.min.x, screen_rect.min.y + menu_bar_height);
        
        // Create tabbed viewport window (movable, not fixed)
        let window_title = format!("🔲 Viewport Tabs ({})", stacked_node_ids.len());
        egui::Window::new(window_title)
            .id(egui::Id::new("tabbed_viewport_panels"))
            .default_pos(position)
            .default_size(self.default_size)
            .min_width(400.0)
            .min_height(300.0)
            .max_size([1600.0, 1200.0])
            .resizable(true)
            .collapsible(true)
            .open(&mut window_open)
            .constrain_to(egui::Rect::from_min_size(
                egui::Pos2::new(0.0, menu_bar_height), 
                egui::Vec2::new(screen_rect.width(), screen_rect.height() - menu_bar_height)
            ))
            .show(ctx, |ui| {
                // Debug: Add a label to see if window content is rendering
                ui.label(format!("Stacked Viewports: {}", stacked_node_ids.len()));
                
                // Create tab bar for multiple viewport panels
                let mut new_selected_tab = selected_tab_index;
                ui.horizontal(|ui| {
                    for (i, &node_id) in stacked_node_ids.iter().enumerate() {
                        if let Some(node) = viewed_nodes.get(&node_id) {
                            // Use custom name if available, otherwise use node title
                            let tab_text = panel_manager.get_node_name(node_id)
                                .cloned()
                                .unwrap_or_else(|| node.title.clone());
                            let is_selected = i == selected_tab_index;
                            
                            if ui.selectable_label(is_selected, tab_text).clicked() {
                                new_selected_tab = i;
                            }
                        }
                    }
                });
                
                // Update selected tab if it changed
                if new_selected_tab != selected_tab_index {
                    self.selected_tabs.insert(window_id.to_string(), new_selected_tab);
                }
                
                ui.separator();
                
                // Render the selected viewport
                if let Some(&selected_node_id) = stacked_node_ids.get(new_selected_tab) {
                    if let Some(node) = viewed_nodes.get(&selected_node_id) {
                        // Panel controls for the selected viewport
                        let (control_action, close_requested) = self.render_panel_controls(ui, selected_node_id, panel_manager, viewed_nodes);
                        if control_action != PanelAction::None {
                            panel_action = control_action;
                        }
                        if close_requested {
                            panel_action = PanelAction::Close;
                        }
                        
                        ui.separator();
                        
                        // Viewport content area - get or create viewport interface instance
                        if let Some(crate::nodes::interface::PanelType::Viewport) = node.get_panel_type() {
                            let viewport_node = self.viewport_instances.entry(selected_node_id)
                                .or_insert_with(|| crate::nodes::three_d::output::viewport::ViewportNode::default());
                            // Use Pattern A build_interface function
                            let _changes = crate::nodes::three_d::output::viewport::viewport_interface::build_interface(viewport_node, ui);
                        } else {
                            ui.label("Error: Node does not have viewport panel type");
                        }
                    }
                }
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

    /// Render panel controls (stack/pin buttons) - viewport specific
    fn render_panel_controls(
        &mut self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        panel_manager: &mut InterfacePanelManager,
        viewed_nodes: &std::collections::HashMap<NodeId, crate::nodes::Node>,
    ) -> (PanelAction, bool) {
        let mut panel_action = PanelAction::None;
        let mut close_requested = false;
        
        // Add name field like parameter panels have
        if let Some(node) = viewed_nodes.get(&node_id) {
            // Get current custom name or use node's default title
            let current_name = panel_manager.get_node_name(node_id)
                .cloned()
                .unwrap_or_else(|| node.title.clone());
            let mut name_buffer = current_name;
            
            // Get current fit name flag
            let mut fit_name = panel_manager.get_fit_name(node_id);
            
            ui.horizontal(|ui| {
                ui.label("Name:");
                
                // Name text field
                let name_response = ui.text_edit_singleline(&mut name_buffer);
                if name_response.changed() {
                    panel_manager.set_node_name(node_id, name_buffer.clone());
                }
                
                // Fit name checkbox
                let fit_response = ui.checkbox(&mut fit_name, "Fit name");
                if fit_response.changed() {
                    panel_manager.set_fit_name(node_id, fit_name);
                }
            });
            
            ui.separator();
        }
        
        ui.horizontal(|ui| {
            ui.label("Viewport controls:");
            
            // Stack button (less prominent for viewport panels)
            let is_stacked = panel_manager.is_panel_stacked(node_id);
            let stack_text = if is_stacked { "📚 Stacked" } else { "📄 Stack" };
            let stack_color = if is_stacked { 
                Color32::from_rgb(100, 150, 255) 
            } else { 
                Color32::from_gray(120) // Dimmer for viewports
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

    /// Auto-load USD stage into a viewport node
    pub fn auto_load_usd_into_viewport(&mut self, viewport_node_id: NodeId, stage_id: &str) {
        // Get the viewport instance and load the USD stage
        if let Some(viewport_instance) = self.viewport_instances.get_mut(&viewport_node_id) {
            println!("Viewport Panel: Auto-loading USD stage {} into viewport {}", stage_id, viewport_node_id);
            viewport_instance.load_usd_scene(stage_id);
        } else {
            // Create a new viewport instance if it doesn't exist
            let mut new_viewport = crate::nodes::three_d::output::viewport::ViewportNode::default();
            new_viewport.load_usd_scene(stage_id);
            self.viewport_instances.insert(viewport_node_id, new_viewport);
            println!("Viewport Panel: Created new viewport instance and loaded USD stage {} into viewport {}", stage_id, viewport_node_id);
        }
    }

}