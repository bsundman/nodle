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
}

impl ViewportPanel {
    pub fn new() -> Self {
        Self {
            default_size: [900.0, 700.0],
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
            self.render_individual_viewport(ctx, node_id, node, panel_manager, menu_bar_height)
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
            .min_size([600.0, 400.0])
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
                let (control_action, close_requested) = self.render_panel_controls(ui, node_id, panel_manager);
                if control_action != PanelAction::None {
                    panel_action = control_action;
                }
                if close_requested {
                    panel_action = PanelAction::Close;
                }
                
                ui.separator();
                
                // Viewport-specific content
                egui::Frame::none()
                    .inner_margin(egui::Margin::same(8.0))
                    .fill(Color32::from_gray(30))
                    .rounding(egui::Rounding::same(4.0))
                    .show(ui, |ui| {
                        self.render_viewport_content(ui, node_id, node);
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
        static mut SELECTED_VIEWPORT_TAB: usize = 0;
        let selected_tab = unsafe { &mut SELECTED_VIEWPORT_TAB };
        
        // Ensure selected tab is valid
        if *selected_tab >= stacked_node_ids.len() {
            *selected_tab = 0;
        }
        
        // Viewport tabbed stacking positioning - same as floating viewport (top left corner)
        let screen_rect = ctx.screen_rect();
        let position = Pos2::new(screen_rect.min.x, screen_rect.min.y + menu_bar_height);
        
        // Create tabbed viewport window (movable, not fixed)
        let window_title = format!("Viewport Tabs ({})", stacked_node_ids.len());
        egui::Window::new(window_title)
            .id(egui::Id::new("tabbed_viewport_panels"))
            .default_pos(position)
            .default_size(self.default_size)
            .min_size([600.0, 400.0])
            .max_size([1600.0, 1200.0])
            .resizable(true)
            .collapsible(true)
            .open(&mut window_open)
            .constrain_to(egui::Rect::from_min_size(
                egui::Pos2::new(0.0, menu_bar_height), 
                egui::Vec2::new(screen_rect.width(), screen_rect.height() - menu_bar_height)
            ))
            .show(ctx, |ui| {
                // Create tab bar for multiple viewport panels
                ui.horizontal(|ui| {
                    for (i, &node_id) in stacked_node_ids.iter().enumerate() {
                        if let Some(node) = viewed_nodes.get(&node_id) {
                            let tab_text = &node.title;
                            let is_selected = i == *selected_tab;
                            
                            if ui.selectable_label(is_selected, tab_text).clicked() {
                                *selected_tab = i;
                            }
                        }
                    }
                });
                
                ui.separator();
                
                // Render the selected viewport
                if let Some(&selected_node_id) = stacked_node_ids.get(*selected_tab) {
                    if let Some(node) = viewed_nodes.get(&selected_node_id) {
                        // Panel controls for the selected viewport
                        let (control_action, close_requested) = self.render_panel_controls(ui, selected_node_id, panel_manager);
                        if control_action != PanelAction::None {
                            panel_action = control_action;
                        }
                        if close_requested {
                            panel_action = PanelAction::Close;
                        }
                        
                        ui.separator();
                        
                        // Viewport content area
                        egui::Frame::none()
                            .inner_margin(egui::Margin::same(4.0))
                            .fill(Color32::from_gray(25))
                            .rounding(egui::Rounding::same(4.0))
                            .show(ui, |ui| {
                                self.render_viewport_content(ui, selected_node_id, node);
                            });
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
    ) -> (PanelAction, bool) {
        let mut panel_action = PanelAction::None;
        let mut close_requested = false;
        
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

    /// Render viewport-specific content (3D rendering area)
    fn render_viewport_content(
        &mut self,
        ui: &mut egui::Ui,
        _node_id: NodeId,
        node: &Node,
    ) {
        // Check if this is a viewport node via metadata (pure node-centric approach)
        if let NodeType::Regular = node.node_type {
            let registry = crate::nodes::factory::NodeRegistry::default();
            if let Some(metadata) = registry.get_node_metadata(&node.title) {
                if metadata.panel_type == PanelType::Viewport {
                    // Create a viewport node instance for interface rendering
                    let mut viewport_node = crate::nodes::three_d::ViewportNode3D::default();
                    
                    // Render the custom viewport interface
                    use crate::nodes::interface::NodeInterfacePanel;
                    viewport_node.render_custom_ui(ui);
                    return;
                }
            }
        }
        
        // Fallback content if not a proper viewport node
        ui.label("Viewport Display");
        
        // Large area for 3D content
        let available_size = ui.available_size();
        let viewport_size = egui::vec2(
            available_size.x.max(600.0),
            available_size.y.max(400.0)
        );
        
        let (rect, _response) = ui.allocate_exact_size(
            viewport_size,
            egui::Sense::click_and_drag()
        );
        
        // Draw placeholder 3D viewport
        ui.painter().rect_filled(
            rect,
            egui::Rounding::same(4.0),
            Color32::from_gray(20)
        );
        
        // Draw border
        ui.painter().rect_stroke(
            rect,
            egui::Rounding::same(4.0),
            egui::Stroke::new(2.0, Color32::from_gray(100))
        );
        
        // Placeholder text
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "3D Viewport",
            egui::FontId::default(),
            Color32::from_gray(150)
        );
    }
}