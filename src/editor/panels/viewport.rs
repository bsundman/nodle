//! Viewport panel implementation
//! 
//! Handles viewport-type interface panels that are floating windows with 3D content

use egui::{Context, Color32, Pos2};
use crate::nodes::{Node, NodeId, InterfacePanelManager, NodeType};
use crate::nodes::interface::PanelType;
use crate::editor::panels::PanelAction;
use std::collections::HashMap;

// Import viewport data types from SDK
use nodle_plugin_sdk::viewport::{ViewportData, SceneData, CameraManipulation};

/// Viewport panel renderer
pub struct ViewportPanel {
    /// Default viewport size
    default_size: [f32; 2],
    /// Selected tab for each stacked viewport window
    selected_tabs: HashMap<String, usize>,
    /// Viewport node instances (to maintain camera state)
    viewport_instances: HashMap<NodeId, crate::nodes::three_d::output::viewport::ViewportNode>,
    /// 3D rendering callbacks for each viewport (to avoid renderer conflicts)
    viewport_callbacks: HashMap<NodeId, crate::gpu::viewport_3d_callback::ViewportRenderCallback>,
}

impl ViewportPanel {
    pub fn new() -> Self {
        Self {
            default_size: [800.0, 600.0],
            selected_tabs: HashMap::new(),
            viewport_instances: HashMap::new(),
            viewport_callbacks: HashMap::new(),
        }
    }

    /// Render viewport panels (handles both tabbed stacking and individual floating in same window)
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
                    // This is the first stacked viewport node, render the window with tabbed content
                    self.render_viewport_window(ctx, node_id, node, panel_manager, menu_bar_height, viewed_nodes, true, &stacked_viewport_nodes)
                } else {
                    // This is not the first node, don't render a window (already handled by first node)
                    PanelAction::None
                }
            } else {
                PanelAction::None
            }
        } else {
            // Render as individual floating window
            self.render_viewport_window(ctx, node_id, node, panel_manager, menu_bar_height, viewed_nodes, false, &[node_id])
        }
    }

    /// Unified viewport window renderer (handles both individual and stacked modes)
    fn render_viewport_window(
        &mut self,
        ctx: &Context,
        primary_node_id: NodeId,
        primary_node: &Node,
        panel_manager: &mut InterfacePanelManager,
        menu_bar_height: f32,
        viewed_nodes: &HashMap<NodeId, Node>,
        is_stacked: bool,
        node_ids: &[NodeId],
    ) -> PanelAction {
        // Check if panel is marked as visible
        if !panel_manager.is_panel_visible(primary_node_id) {
            return PanelAction::None;
        }
        
        // Always use the primary node ID for the window to preserve position when stacking/unstacking
        let panel_id = egui::Id::new(format!("viewport_panel_{}", primary_node_id));
        let mut panel_action = PanelAction::None;
        
        // Get current window open state
        let mut window_open = panel_manager.is_panel_open(primary_node_id);
        
        // Only set default position for new nodes - preserve existing position otherwise
        let screen_rect = ctx.screen_rect();
        let default_position = Pos2::new(screen_rect.min.x, screen_rect.min.y + menu_bar_height);
        
        // Create window title based on mode
        let window_title = if is_stacked {
            format!("🔲 Viewport Tabs ({})", node_ids.len())
        } else {
            format!("{} Viewport", primary_node.title)
        };
        
        let window_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            egui::Window::new(&window_title)
                .id(panel_id)
                .default_pos(default_position)
                .default_size(self.default_size)
                .min_size([200.0, 200.0])
                .max_size([1600.0, 1200.0])
        }));
        
        let window = match window_result {
            Ok(w) => w,
            Err(_) => return PanelAction::None,
        }
            .resizable(true)
            .collapsible(true)
            .open(&mut window_open)
            .constrain_to(egui::Rect::from_min_size(
                egui::Pos2::new(0.0, menu_bar_height), 
                egui::Vec2::new(screen_rect.width(), screen_rect.height() - menu_bar_height)
            ));
        
        let show_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            window.show(ctx, |ui| {
                if is_stacked {
                    // Render stacked content with tabs
                    panel_action = self.render_stacked_content(ui, node_ids, panel_manager, viewed_nodes);
                } else {
                    // Render individual content
                    panel_action = self.render_individual_content(ui, primary_node_id, primary_node, panel_manager, viewed_nodes);
                }
            })
        }));
        
        match show_result {
            Ok(_) => {},
            Err(_) => return PanelAction::None,
        }
        
        // Update the panel manager with the new state
        if is_stacked {
            // Update all stacked nodes' panel open state
            for &node_id in node_ids {
                panel_manager.set_panel_open(node_id, window_open);
            }
        } else {
            panel_manager.set_panel_open(primary_node_id, window_open);
        }
        
        // Check if window was closed via X button
        if !window_open {
            panel_action = PanelAction::Close;
        }
        
        panel_action
    }

    /// Render content for individual (non-stacked) viewport
    fn render_individual_content(
        &mut self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        node: &Node,
        panel_manager: &mut InterfacePanelManager,
        viewed_nodes: &HashMap<NodeId, Node>,
    ) -> PanelAction {
        let mut panel_action = PanelAction::None;
        
        // Panel controls at the top
        let (control_action, close_requested) = self.render_panel_controls(ui, node_id, panel_manager, viewed_nodes);
        if control_action != PanelAction::None {
            panel_action = control_action;
        }
        if close_requested {
            panel_action = PanelAction::Close;
        }
        
        ui.separator();
        
        // Viewport-specific content - check if this is a plugin viewport node
        if let Some(crate::nodes::interface::PanelType::Viewport) = node.get_panel_type() {
            // Check if this might be a plugin node by looking for plugin-specific indicators
            if let Some(plugin_manager) = crate::workspace::get_global_plugin_manager() {
                if let Ok(mut manager) = plugin_manager.lock() {
                    // Try to get plugin node instance for viewport data
                    if let Some(plugin_node) = manager.get_plugin_node_for_rendering(node_id, &node.title) {
                        // This is a plugin viewport node - use data-driven rendering
                        // Get viewport data from plugin (safe, no egui rendering)
                        if let Some(viewport_data) = plugin_node.get_viewport_data() {
                            // Render viewport using core's 3D rendering system
                            self.render_plugin_viewport_data(ui, viewport_data, plugin_node.as_mut(), node_id);
                        } else {
                            ui.label("🎬 Plugin Viewport");
                            ui.label("No viewport data available from plugin");
                        }
                    } else {
                        // This is a core viewport node - use the standard viewport interface
                        let viewport_node = self.viewport_instances.entry(node_id)
                            .or_insert_with(|| crate::nodes::three_d::output::viewport::ViewportNode::default());
                        let _changes = crate::nodes::three_d::output::viewport::viewport_interface::build_interface(viewport_node, ui);
                    }
                } else {
                    // Plugin manager lock failed - fall back to core viewport
                    let viewport_node = self.viewport_instances.entry(node_id)
                        .or_insert_with(|| crate::nodes::three_d::output::viewport::ViewportNode::default());
                    let _changes = crate::nodes::three_d::output::viewport::viewport_interface::build_interface(viewport_node, ui);
                }
            } else {
                // No plugin manager - fall back to core viewport
                let viewport_node = self.viewport_instances.entry(node_id)
                    .or_insert_with(|| crate::nodes::three_d::output::viewport::ViewportNode::default());
                let _changes = crate::nodes::three_d::output::viewport::viewport_interface::build_interface(viewport_node, ui);
            }
        } else {
            ui.label("Error: Node does not have viewport panel type");
        }
        
        panel_action
    }

    /// Render content for stacked viewport with tabs
    fn render_stacked_content(
        &mut self,
        ui: &mut egui::Ui,
        node_ids: &[NodeId],
        panel_manager: &mut InterfacePanelManager,
        viewed_nodes: &HashMap<NodeId, Node>,
    ) -> PanelAction {
        let mut panel_action = PanelAction::None;
        
        if node_ids.is_empty() {
            return PanelAction::None;
        }
        
        // Track which viewport tab is selected (default to first)
        let window_id = format!("viewport_panel_{}", node_ids[0]); // Use first node's ID for consistency
        let current_selected_tab = *self.selected_tabs.entry(window_id.clone()).or_insert(0);
        
        // Ensure selected tab is valid
        let selected_tab_index = if current_selected_tab >= node_ids.len() {
            0
        } else {
            current_selected_tab
        };
        
        // Update the stored value if it changed
        if selected_tab_index != current_selected_tab {
            self.selected_tabs.insert(window_id.clone(), selected_tab_index);
        }
        
        // Debug: Add a label to see if window content is rendering
        ui.label(format!("Stacked Viewports: {}", node_ids.len()));
        
        // Create tab bar for multiple viewport panels
        let mut new_selected_tab = selected_tab_index;
        ui.horizontal(|ui| {
            for (i, &node_id) in node_ids.iter().enumerate() {
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
            self.selected_tabs.insert(window_id, new_selected_tab);
        }
        
        ui.separator();
        
        // Render the selected viewport
        if let Some(&selected_node_id) = node_ids.get(new_selected_tab) {
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
                
                // Viewport content area - check if this is a plugin viewport node
                if let Some(crate::nodes::interface::PanelType::Viewport) = node.get_panel_type() {
                    // Check if this might be a plugin node by looking for plugin-specific indicators
                    if let Some(plugin_manager) = crate::workspace::get_global_plugin_manager() {
                        if let Ok(mut manager) = plugin_manager.lock() {
                            // Try to get plugin node instance for viewport data
                            if let Some(plugin_node) = manager.get_plugin_node_for_rendering(selected_node_id, &node.title) {
                                // This is a plugin viewport node - use data-driven rendering
                                // Get viewport data from plugin (safe, no egui rendering)
                                if let Some(viewport_data) = plugin_node.get_viewport_data() {
                                    // Render viewport using core's 3D rendering system
                                    self.render_plugin_viewport_data(ui, viewport_data, plugin_node.as_mut(), selected_node_id);
                                } else {
                                    ui.label("🎬 Plugin Viewport");
                                    ui.label("No viewport data available from plugin");
                                }
                            } else {
                                // This is a core viewport node - use the standard viewport interface
                                let viewport_node = self.viewport_instances.entry(selected_node_id)
                                    .or_insert_with(|| crate::nodes::three_d::output::viewport::ViewportNode::default());
                                let _changes = crate::nodes::three_d::output::viewport::viewport_interface::build_interface(viewport_node, ui);
                            }
                        } else {
                            // Plugin manager lock failed - fall back to core viewport
                            let viewport_node = self.viewport_instances.entry(selected_node_id)
                                .or_insert_with(|| crate::nodes::three_d::output::viewport::ViewportNode::default());
                            let _changes = crate::nodes::three_d::output::viewport::viewport_interface::build_interface(viewport_node, ui);
                        }
                    } else {
                        // No plugin manager - fall back to core viewport
                        let viewport_node = self.viewport_instances.entry(selected_node_id)
                            .or_insert_with(|| crate::nodes::three_d::output::viewport::ViewportNode::default());
                        let _changes = crate::nodes::three_d::output::viewport::viewport_interface::build_interface(viewport_node, ui);
                    }
                } else {
                    ui.label("Error: Node does not have viewport panel type");
                }
            }
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
    
    /// Render plugin viewport data using the core's 3D rendering system
    fn render_plugin_viewport_data(&mut self, ui: &mut egui::Ui, viewport_data: ViewportData, plugin_node: &mut dyn nodle_plugin_sdk::PluginNode, node_id: NodeId) {
        // 3D Viewport area with actual wgpu rendering - no extra UI elements
        // Create viewport area - use all available space
        let available_size = ui.available_size();
        let viewport_size = egui::vec2(
            available_size.x.max(100.0),
            available_size.y.max(100.0)
        );
        let (rect, response) = ui.allocate_exact_size(viewport_size, egui::Sense::drag());
            
        // Get or create 3D rendering callback for this specific viewport node
        let callback = self.viewport_callbacks.entry(node_id)
            .or_insert_with(|| crate::gpu::viewport_3d_callback::ViewportRenderCallback::new());
        callback.update_viewport_data(viewport_data.clone());
        callback.update_viewport_size(viewport_size.x as u32, viewport_size.y as u32);
        
        // Handle mouse interactions for camera control
        if response.dragged() {
            let delta = response.drag_delta();
            let ctx = ui.ctx();
            let modifiers = ctx.input(|i| i.modifiers);
            
            // Create camera manipulation based on modifiers
            let manipulation = if modifiers.alt {
                // Alt + drag = orbit
                nodle_plugin_sdk::viewport::CameraManipulation::Orbit {
                    delta_x: delta.x * 0.01,
                    delta_y: delta.y * 0.01,
                }
            } else if modifiers.shift {
                // Shift + drag = pan
                nodle_plugin_sdk::viewport::CameraManipulation::Pan {
                    delta_x: delta.x * 0.01,
                    delta_y: delta.y * 0.01,
                }
            } else {
                // Default drag = orbit
                nodle_plugin_sdk::viewport::CameraManipulation::Orbit {
                    delta_x: delta.x * 0.01,
                    delta_y: delta.y * 0.01,
                }
            };
            
            // Send manipulation to plugin node to update its camera state
            plugin_node.handle_viewport_camera(manipulation.clone());
            
            // Also update the callback for immediate rendering
            match manipulation {
                nodle_plugin_sdk::viewport::CameraManipulation::Orbit { delta_x, delta_y } => {
                    callback.handle_camera_manipulation(
                        delta_x, 
                        delta_y, 
                        crate::gpu::viewport_3d_callback::CameraManipulationType::Orbit
                    );
                }
                nodle_plugin_sdk::viewport::CameraManipulation::Pan { delta_x, delta_y } => {
                    callback.handle_camera_manipulation(
                        delta_x, 
                        delta_y, 
                        crate::gpu::viewport_3d_callback::CameraManipulationType::Pan
                    );
                }
                _ => {}
            }
        }
        
        // Handle scroll for zoom
        if response.hovered() {
            let ctx = ui.ctx();
            ctx.input(|i| {
                if i.raw_scroll_delta.y != 0.0 {
                    // Create zoom manipulation for plugin
                    let zoom_manipulation = nodle_plugin_sdk::viewport::CameraManipulation::Zoom {
                        delta: i.raw_scroll_delta.y * 0.01,
                    };
                    
                    // Send to plugin node
                    plugin_node.handle_viewport_camera(zoom_manipulation);
                    
                    // Also update callback for immediate rendering
                    callback.handle_camera_manipulation(
                        i.raw_scroll_delta.y * 0.01, 
                        0.0, 
                        crate::gpu::viewport_3d_callback::CameraManipulationType::Zoom
                    );
                }
            });
        }
        
        // Add the 3D rendering callback to egui (clone it since egui takes ownership)
        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            callback.clone(),
        ));
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