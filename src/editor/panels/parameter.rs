//! Parameter panel implementation
//! 
//! Handles parameter-type interface panels that are typically stacked on the right side

use egui::{Context, Color32, Pos2};
use crate::nodes::{Node, NodeId, InterfacePanelManager};
use crate::nodes::interface::NodeData;
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
        execution_engine: &mut crate::nodes::NodeGraphEngine,
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
                    self.render_stacked_panels(ctx, &stacked_parameter_nodes, panel_manager, menu_bar_height, viewed_nodes, graph, execution_engine)
                } else {
                    // This is not the designated renderer, but if the designated renderer
                    // is not visible or the first node's panel is closed, allow this node to render
                    let first_node_visible = viewed_nodes.get(&first_node_id)
                        .map(|node| node.visible && panel_manager.is_panel_visible(first_node_id))
                        .unwrap_or(false);
                    
                    if !first_node_visible || !panel_manager.is_panel_open(first_node_id) {
                        // First node can't render the window, so this node should do it
                        self.render_stacked_panels(ctx, &stacked_parameter_nodes, panel_manager, menu_bar_height, viewed_nodes, graph, execution_engine)
                    } else {
                        // First node is handling the window
                        PanelAction::None
                    }
                }
            } else {
                PanelAction::None
            }
        } else {
            self.render_individual_panel(ctx, node_id, node, panel_manager, menu_bar_height, graph, execution_engine)
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
        execution_engine: &mut crate::nodes::NodeGraphEngine,
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
                egui::Frame::default()
                    .inner_margin(egui::Margin::same(8))
                    .fill(Color32::from_gray(40))
                    .corner_radius(4.0)
                    .show(ui, |ui| {
                        self.render_parameter_content(ui, node_id, panel_manager, graph, execution_engine);
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
        execution_engine: &mut crate::nodes::NodeGraphEngine,
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
                                        egui::Frame::default()
                                            .inner_margin(egui::Margin {
                                                left: 0,
                                                right: -6,  // Negative margin to push closer to edge
                                                top: 0,
                                                bottom: 0,
                                            })
                                            .show(ui, |ui| {
                                                ui.separator();
                                            });
                                        
                                        // Node content in a contained frame
                                        egui::Frame::default()
                                            .inner_margin(egui::Margin::same(8))
                                            .fill(Color32::from_gray(45))
                                            .corner_radius(4.0)
                                            .stroke(egui::Stroke::new(1.0, Color32::from_gray(80)))
                                            .show(ui, |ui| {
                                                self.render_parameter_content(ui, node_id, panel_manager, graph, execution_engine);
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
        panel_manager: &mut InterfacePanelManager,
        graph: &mut crate::nodes::NodeGraph,
        execution_engine: &mut crate::nodes::NodeGraphEngine,
    ) {
        // ALWAYS get fresh node data from graph - never use stale viewed_nodes
        let (mut name_buffer, node_type, node_position, node_inputs, node_outputs, node_parameters) = {
            let Some(fresh_node) = graph.nodes.get(&node_id) else {
                ui.label("Node not found in graph");
                return;
            };
            
            // Clone the data we need to avoid borrow checker issues
            (
                fresh_node.title.clone(),
                fresh_node.node_type.clone(),
                fresh_node.position,
                fresh_node.inputs.clone(),
                fresh_node.outputs.clone(),
                fresh_node.parameters.clone()
            )
        };
        // Name buffer initialized
        
        // Get current fit name flag
        let mut fit_name = panel_manager.get_fit_name(node_id);
        
        ui.horizontal(|ui| {
            ui.label("Name:");
            
            // Rendering name text field
            
            // Name text field - directly edit the node's title
            let name_response = ui.text_edit_singleline(&mut name_buffer);
            
            // Name field rendered
            
            if name_response.changed() {
                // Name changed by user
                
                // Update the actual node's title in the graph
                if let Some(node_mut) = graph.nodes.get_mut(&node_id) {
                    // Updating graph node title
                    node_mut.title = name_buffer.clone();
                    
                    // If fit name is enabled, resize the node to fit the new title
                    if fit_name {
                        // Calculate new size based on actual text width with 15px padding on each side
                        let font_id = egui::FontId::proportional(12.0);
                        let text_width = ui.fonts(|fonts| {
                            fonts.layout_no_wrap(name_buffer.clone(), font_id, egui::Color32::WHITE).size().x
                        });
                        let padding = 60.0; // 30px padding on each side (extra 30px to avoid visibility flag)
                        let min_width = 120.0; // Minimum node width
                        let new_width = (text_width + padding).max(min_width);
                        node_mut.size.x = new_width;
                        node_mut.update_port_positions(); // Update port positions after resize
                    }
                    
                } else {
                }
            }
            
            // Fit name checkbox
            let fit_response = ui.checkbox(&mut fit_name, "Fit name");
            if fit_response.changed() {
                panel_manager.set_fit_name(node_id, fit_name);
                
                // Handle fit name toggle - resize immediately or restore default
                if fit_name {
                    // Fit name was just enabled - resize to fit text
                    if let Some(node_mut) = graph.nodes.get_mut(&node_id) {
                        // Calculate new size based on actual text width with 15px padding on each side
                        let font_id = egui::FontId::proportional(12.0);
                        let text_width = ui.fonts(|fonts| {
                            fonts.layout_no_wrap(node_mut.title.clone(), font_id, egui::Color32::WHITE).size().x
                        });
                        let padding = 60.0; // 30px padding on each side (extra 30px to avoid visibility flag)
                        let min_width = 120.0; // Minimum node width
                        let new_width = (text_width + padding).max(min_width);
                        node_mut.size.x = new_width;
                        node_mut.update_port_positions();
                    }
                } else {
                    // Fit name was just disabled - restore default width
                    if let Some(node_mut) = graph.nodes.get_mut(&node_id) {
                        let default_width = 150.0; // Standard default node width
                        node_mut.size.x = default_width;
                        node_mut.update_port_positions();
                    }
                }
            }
        });
        
        ui.separator();
        
        // Show node info - using fresh graph data
        ui.label(format!("Node: {}", name_buffer));
        ui.label(format!("Type: {:?}", node_type));
        ui.label(format!("Position: ({:.1}, {:.1})", node_position.x, node_position.y));
        
        ui.separator();
        
        // Show connection debug info
        ui.label(format!("Graph Connections: {}", graph.connections.len()));
        if graph.connections.len() > 0 {
            ui.collapsing("Debug: All Connections", |ui| {
                for (i, conn) in graph.connections.iter().enumerate() {
                    let from_name = graph.nodes.get(&conn.from_node)
                        .map(|n| n.title.as_str()).unwrap_or("Unknown");
                    let to_name = graph.nodes.get(&conn.to_node)
                        .map(|n| n.title.as_str()).unwrap_or("Unknown");
                    ui.label(format!("  {}: {} port {} → {} port {}", 
                        i, from_name, conn.from_port, to_name, conn.to_port));
                }
            });
        }
        
        ui.separator();
        
        // Show ports with connection information - using fresh graph data
        ui.label("Input Ports:");
        for (i, input) in node_inputs.iter().enumerate() {
            // Find connections to this input port
            let connected_from = graph.connections.iter()
                .find(|conn| conn.to_node == node_id && conn.to_port == i)
                .map(|conn| {
                    let source_node = graph.nodes.get(&conn.from_node)
                        .map(|n| n.title.as_str())
                        .unwrap_or("Unknown");
                    format!("← {} port {}", source_node, conn.from_port)
                });
            
            if let Some(connection_info) = connected_from {
                ui.colored_label(egui::Color32::from_rgb(100, 255, 100), 
                    format!("  🔗 {}: {} {}", i, input.name, connection_info));
            } else {
                ui.colored_label(egui::Color32::from_rgb(150, 150, 150), 
                    format!("  ○ {}: {} (not connected)", i, input.name));
            }
        }
        
        ui.label("Output Ports:");
        for (i, output) in node_outputs.iter().enumerate() {
            // Find connections from this output port
            let connected_to: Vec<String> = graph.connections.iter()
                .filter(|conn| conn.from_node == node_id && conn.from_port == i)
                .map(|conn| {
                    let target_node = graph.nodes.get(&conn.to_node)
                        .map(|n| n.title.as_str())
                        .unwrap_or("Unknown");
                    format!("{} port {}", target_node, conn.to_port)
                })
                .collect();
            
            if !connected_to.is_empty() {
                ui.colored_label(egui::Color32::from_rgb(100, 255, 100), 
                    format!("  🔗 {}: {} → {}", i, output.name, connected_to.join(", ")));
            } else {
                ui.colored_label(egui::Color32::from_rgb(150, 150, 150), 
                    format!("  ○ {}: {} (not connected)", i, output.name));
            }
        }
        
        ui.separator();
        
        // Use proper parameter interface for all nodes that have build_interface methods
        let handled = if graph.nodes.contains_key(&node_id) {
            self.render_node_interface_safe(ui, node_id, execution_engine, graph)
        } else {
            false
        };
        
        // Fallback: render basic parameter display for nodes without proper interfaces
        if !handled && !node_parameters.is_empty() {
            ui.label("Parameters:");
            
            for (param_name, param_value) in &node_parameters {
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
    fn render_node_interface(
        &mut self, 
        node: &mut crate::nodes::Node, 
        ui: &mut egui::Ui, 
        node_id: NodeId,
        execution_engine: &mut crate::nodes::NodeGraphEngine,
        graph: &crate::nodes::NodeGraph,
    ) -> bool {
        // ONLY Pattern A: build_interface(node, ui) method for ALL nodes
        self.render_build_interface_pattern(node, ui, node_id, execution_engine, graph)
    }
    
    /// Safe version that handles borrowing conflicts
    fn render_node_interface_safe(
        &mut self,
        ui: &mut egui::Ui, 
        node_id: NodeId,
        execution_engine: &mut crate::nodes::NodeGraphEngine,
        graph: &mut crate::nodes::NodeGraph,
    ) -> bool {
        // Extended approach: handle all node types with parameter change detection
        // This avoids the borrowing conflict while still enabling parameter change notifications
        let mut changes_applied = false;
        let mut handled = false;
        
        if let Some(node) = graph.nodes.get_mut(&node_id) {
            let title = node.title.clone();
            // Rendering node interface
            
            // Try to handle all known node types with build_interface methods
            let changes = match node.type_id.as_str() {
                // Data nodes
                "Data_UsdFileReader" => {
                    // Using USD File Reader interface
                    crate::nodes::data::usd_file_reader::parameters::UsdFileReaderParameters::build_interface(node, ui)
                },
                
                // Test nodes
                "Test" => {
                    // Using Test node interface
                    crate::nodes::utility::test::parameters::TestNode::build_interface(node, ui)
                },
                
                // 3D Transform nodes
                "Translate" | "3D_Translate" => {
                    // Using Translate interface
                    crate::nodes::three_d::transform::translate::parameters::TranslateNode::build_interface(node, ui)
                },
                "Rotate" | "3D_Rotate" => {
                    // Using Rotate interface
                    crate::nodes::three_d::transform::rotate::parameters::RotateNode::build_interface(node, ui)
                },
                "Scale" | "3D_Scale" => {
                    // Using Scale interface
                    crate::nodes::three_d::transform::scale::parameters::ScaleNode::build_interface(node, ui)
                },
                
                // 3D Geometry nodes
                "Cube" | "3D_Cube" => {
                    // Using Cube interface
                    crate::nodes::three_d::geometry::cube::parameters::CubeParameters::build_interface(node, ui)
                },
                "Sphere" | "3D_Sphere" => {
                    // Using Sphere interface
                    crate::nodes::three_d::geometry::sphere::parameters::SphereParameters::build_interface(node, ui)
                },
                "Cylinder" | "3D_Cylinder" => {
                    // Using Cylinder interface
                    crate::nodes::three_d::geometry::cylinder::parameters::CylinderParameters::build_interface(node, ui)
                },
                "Cone" | "3D_Cone" => {
                    // Using Cone interface
                    crate::nodes::three_d::geometry::cone::parameters::ConeParameters::build_interface(node, ui)
                },
                "Plane" | "3D_Plane" => {
                    // Using Plane interface
                    crate::nodes::three_d::geometry::plane::parameters::PlaneParameters::build_interface(node, ui)
                },
                "Capsule" | "3D_Capsule" => {
                    // Using Capsule interface
                    crate::nodes::three_d::geometry::capsule::parameters::CapsuleParameters::build_interface(node, ui)
                },
                
                // 3D Lighting nodes
                "Point Light" => {
                    // Using Point Light interface
                    crate::nodes::three_d::lighting::point_light::parameters::PointLightNode::build_interface(node, ui)
                },
                "Directional Light" => {
                    // Using Directional Light interface
                    crate::nodes::three_d::lighting::directional_light::parameters::DirectionalLightNode::build_interface(node, ui)
                },
                "Spot Light" => {
                    // Using Spot Light interface
                    crate::nodes::three_d::lighting::spot_light::parameters::SpotLightNode::build_interface(node, ui)
                },
                
                // 3D Modify nodes
                "Reverse" | "3D_Reverse" => {
                    // Using Reverse interface
                    crate::nodes::three_d::modify::reverse::parameters::ReverseNode::build_interface(node, ui)
                },
                
                // 3D Output nodes
                "3D_Render" => {
                    // Using Render interface
                    crate::nodes::three_d::output::render::parameters::RenderParameters::build_interface(node, ui)
                },
                
                // Other node types - check if it's a plugin node first, otherwise use generic interface
                _ => {
                    // Check if this is a plugin node that should be handled by plugin system
                    if node.plugin_node.is_some() {
                        // This is a plugin node - don't handle it here, let plugin handling take over
                        Vec::new() // Return empty changes - plugin handler will take over
                    } else {
                        // Using generic parameter interface
                        // Fall back to basic parameter editing for unknown types
                        // This creates a simple interface for any parameters the node has
                        self.build_generic_parameter_interface(node, ui)
                    }
                }
            };
            
            // Apply changes if any were detected
            if !changes.is_empty() {
                info!("Applied {} parameter changes for {} node {}", changes.len(), title, node_id);
                // Applying parameter changes
                for change in changes {
                    node.parameters.insert(change.parameter.clone(), change.value.clone());
                    
                    // Special handling for render node trigger_render parameter
                    if node.type_id == "3D_Render" && change.parameter == "trigger_render" {
                        if let NodeData::Boolean(true) = change.value {
                            // If trigger_render was set to true, we need to schedule it to be reset
                            // after the execution completes. For now, we'll rely on the execution
                            // engine logic to only execute when this flag is true.
                            println!("🎬 Render trigger activated for node {}", node_id);
                        }
                    }
                }
                changes_applied = true;
                
                // Notify execution engine immediately after changes are applied
                // Notifying execution engine
            } else {
                // No parameter changes
            }
            
            // Only mark as handled if this is NOT a plugin node (plugin nodes will be handled later)
            if node.plugin_node.is_none() {
                handled = true;
            }
        }
        
        // Notify execution engine outside the mutable borrow scope if changes were made
        if changes_applied {
            // Notifying execution engine about parameter changes
            execution_engine.on_node_parameter_changed(node_id, graph);
            
            // Special handling for render node: reset trigger_render after execution
            if let Some(node) = graph.nodes.get_mut(&node_id) {
                if node.type_id == "3D_Render" {
                    if let Some(NodeData::Boolean(true)) = node.parameters.get("trigger_render") {
                        // Reset trigger_render to false after the execution engine has processed it
                        // This ensures the render only happens once per button click
                        node.parameters.insert("trigger_render".to_string(), NodeData::Boolean(false));
                        println!("🔧 Parameter Panel: Reset trigger_render to false after execution");
                    }
                }
            }
        }
        
        // If not handled by the main match statement, check for plugin nodes
        if !handled {
            handled = self.handle_plugin_nodes_safe(ui, node_id, execution_engine, graph);
        }
        
        handled
    }
    
    /// Handle plugin nodes separately after the main match statement
    fn handle_plugin_nodes_safe(
        &mut self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        execution_engine: &mut crate::nodes::NodeGraphEngine,
        graph: &mut crate::nodes::NodeGraph,
    ) -> bool {
        if let Some(node) = graph.nodes.get_mut(&node_id) {
            let title = node.title.clone();
            
            // Check for plugin nodes that weren't handled by the main match statement
            
            if let Some(plugin_node) = &mut node.plugin_node {
                
                // Get UI description from plugin using normal Rust types
                let ui_description = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let result = plugin_node.get_parameter_ui();
                        result
                })) {
                    Ok(ui_desc) => ui_desc,
                    Err(e) => {
                        ui.colored_label(egui::Color32::RED, format!("Plugin '{}' parameter UI crashed", title));
                        return true;
                    }
                };
                
                
                // Render UI elements 
                let ui_actions = self.render_ui_elements(ui, &ui_description.elements);
                
                // Handle UI actions and convert to parameter changes
                if !ui_actions.is_empty() {
                    for action in ui_actions {
                        match action {
                            crate::plugins::UIAction::ParameterChanged { parameter, value } => {
                                
                                // Convert plugin NodeData to core NodeData
                                let core_value = match value {
                                    crate::plugins::NodeData::Float(f) => crate::nodes::interface::NodeData::Float(f),
                                    crate::plugins::NodeData::Vector3(v) => crate::nodes::interface::NodeData::Vector3(v),
                                    crate::plugins::NodeData::Color(c) => crate::nodes::interface::NodeData::Color([c[0], c[1], c[2], 1.0]),
                                    crate::plugins::NodeData::String(s) => crate::nodes::interface::NodeData::String(s),
                                    crate::plugins::NodeData::Boolean(b) => crate::nodes::interface::NodeData::Boolean(b),
                                    crate::plugins::NodeData::USDScene(s) => crate::nodes::interface::NodeData::String(s), // Convert USD to string
                                    // Rich data types - convert to JSON strings for now
                                    crate::plugins::NodeData::Scene(_) => crate::nodes::interface::NodeData::String("[Scene Data]".to_string()),
                                    crate::plugins::NodeData::Geometry(_) => crate::nodes::interface::NodeData::String("[Geometry Data]".to_string()),
                                    crate::plugins::NodeData::Material(_) => crate::nodes::interface::NodeData::String("[Material Data]".to_string()),
                                    crate::plugins::NodeData::Stage(_) => crate::nodes::interface::NodeData::String("[Stage Data]".to_string()),
                                    crate::plugins::NodeData::USDSceneData(_) => crate::nodes::interface::NodeData::String("[USD Scene Data]".to_string()),
                                    crate::plugins::NodeData::USDScenegraphMetadata(_) => crate::nodes::interface::NodeData::String("[USD Metadata]".to_string()),
                                    crate::plugins::NodeData::Light(_) => crate::nodes::interface::NodeData::String("[Light Data]".to_string()),
                                    crate::plugins::NodeData::Image(_) => crate::nodes::interface::NodeData::String("[Image Data]".to_string()),
                                    crate::plugins::NodeData::Integer(i) => crate::nodes::interface::NodeData::Integer(i),
                                    crate::plugins::NodeData::Any(s) => crate::nodes::interface::NodeData::String(s),
                                    crate::plugins::NodeData::None => crate::nodes::interface::NodeData::String("None".to_string()),
                                };
                                node.parameters.insert(parameter, core_value);
                            }
                            crate::plugins::UIAction::ButtonClicked { action } => {
                                // Handle button clicks if needed - for now just log
                            }
                            crate::plugins::UIAction::FileSelected { parameter, path } => {
                                // Handle file selections - convert to string parameter
                                let core_value = crate::nodes::interface::NodeData::String(path);
                                node.parameters.insert(parameter, core_value);
                            }
                        }
                    }
                    
                    // Notify execution engine
                    execution_engine.on_node_parameter_changed(node_id, graph);
                }
                
                return true;
            }
        }
        
        false
    }
    
    /// Build a generic parameter interface for nodes without specialized interfaces
    fn build_generic_parameter_interface(
        &mut self, 
        node: &mut crate::nodes::Node, 
        ui: &mut egui::Ui
    ) -> Vec<crate::nodes::interface::ParameterChange> {
        let mut changes = Vec::new();
        
        if node.parameters.is_empty() {
            ui.label("No parameters available");
            return changes;
        }
        
        ui.label("Parameters:");
        ui.separator();
        
        // Clone the parameters to avoid borrowing issues during iteration
        let parameters: Vec<(String, crate::nodes::interface::NodeData)> = node.parameters.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        
        for (param_name, param_value) in parameters {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", param_name));
                
                match param_value {
                    crate::nodes::interface::NodeData::String(mut s) => {
                        if ui.text_edit_singleline(&mut s).changed() {
                            changes.push(crate::nodes::interface::ParameterChange {
                                parameter: param_name,
                                value: crate::nodes::interface::NodeData::String(s),
                            });
                        }
                    }
                    crate::nodes::interface::NodeData::Boolean(mut b) => {
                        if ui.checkbox(&mut b, "").changed() {
                            changes.push(crate::nodes::interface::ParameterChange {
                                parameter: param_name,
                                value: crate::nodes::interface::NodeData::Boolean(b),
                            });
                        }
                    }
                    crate::nodes::interface::NodeData::Float(mut f) => {
                        if ui.add(egui::DragValue::new(&mut f).speed(0.1)).changed() {
                            changes.push(crate::nodes::interface::ParameterChange {
                                parameter: param_name,
                                value: crate::nodes::interface::NodeData::Float(f),
                            });
                        }
                    }
                    crate::nodes::interface::NodeData::Integer(mut i) => {
                        if ui.add(egui::DragValue::new(&mut i)).changed() {
                            changes.push(crate::nodes::interface::ParameterChange {
                                parameter: param_name,
                                value: crate::nodes::interface::NodeData::Integer(i),
                            });
                        }
                    }
                    _ => {
                        ui.label(format!("{:?}", param_value));
                    }
                }
            });
        }
        
        changes
    }
    
    /// Pattern A: build_interface(node, ui) method for ALL nodes
    fn render_build_interface_pattern(
        &mut self, 
        node: &mut crate::nodes::Node, 
        ui: &mut egui::Ui, 
        node_id: NodeId,
        execution_engine: &mut crate::nodes::NodeGraphEngine,
        graph: &crate::nodes::NodeGraph,
    ) -> bool {
        let title = node.title.clone();
        
        // Debug output for every individual node (not workspace)
        
        // Handle plugin nodes using FFI-SAFE methods
        // Check core node storage first (non-viewport nodes)
        if let Some(plugin_node) = &mut node.plugin_node {
            
            // Try calling a simpler method first
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let id = plugin_node.id();
            })) {
                Ok(_) => {},
                Err(e) => {
                    ui.colored_label(egui::Color32::RED, format!("Plugin '{}' vtable corrupted", title));
                    return true;
                }
            }
            
            // Get UI description from plugin using normal Rust types
            let ui_description = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let result = plugin_node.get_parameter_ui();
                result
            })) {
                Ok(ui_desc) => {
                    ui_desc
                },
                Err(e) => {
                    ui.colored_label(egui::Color32::RED, format!("Plugin '{}' crashed getting UI description", title));
                    return true;
                }
            };
            
            
            // CORE renders the UI based on normal Rust description
            let ui_actions = self.render_ui_elements(ui, &ui_description.elements);
            
            // Send actions back to plugin using normal Rust types and get parameter changes
            for action in ui_actions {
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    plugin_node.handle_ui_action(action)
                })) {
                    Ok(changes) => {
                        for change in changes {
                            // Apply parameter changes
                            plugin_node.set_parameter(&change.parameter, change.value);
                        }
                    }
                    Err(e) => {
                    }
                }
            }
            
            return true;
        }
        
        // Check global plugin manager for viewport nodes (stored separately)
        if let Some(plugin_manager) = crate::workspace::get_global_plugin_manager() {
            if let Ok(mut manager) = plugin_manager.lock() {
                if let Some(plugin_node) = manager.get_plugin_node_for_rendering(node_id, &title) {
                    
                    // Get UI description from plugin using normal Rust types
                    let ui_description = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        plugin_node.get_parameter_ui()
                    })) {
                        Ok(ui_desc) => ui_desc,
                        Err(e) => {
                                    ui.colored_label(egui::Color32::RED, format!("Plugin '{}' crashed getting UI description", title));
                            return true;
                        }
                    };
                    
                    // CORE renders the UI based on normal Rust description
                    let ui_actions = self.render_ui_elements(ui, &ui_description.elements);
                    
                    // Send actions back to plugin using normal Rust types and get parameter changes
                    for action in ui_actions {
                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            plugin_node.handle_ui_action(action)
                        })) {
                            Ok(changes) => {
                                for change in changes {
                                    // Apply parameter changes
                                    plugin_node.set_parameter(&change.parameter, change.value);
                                }
                            }
                            Err(e) => {
                                    }
                        }
                    }
                    
                    return true;
                }
            }
        }
        
        // USD nodes are now handled by plugins - no core implementation needed
        
        // Math nodes using Pattern A
        if node.type_id.contains("Add") || node.type_id.contains("Addition") {
            let changes = crate::nodes::math::add::parameters::AddNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id, execution_engine, graph);
            return true;
        }
        
        // Math nodes using Pattern A
        if node.type_id.contains("Subtract") || node.type_id.contains("Subtraction") {
            let changes = crate::nodes::math::subtract::parameters::SubtractNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id, execution_engine, graph);
            return true;
        }
        
        if node.type_id.contains("Multiply") || node.type_id.contains("Multiplication") {
            let changes = crate::nodes::math::multiply::parameters::MultiplyNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id, execution_engine, graph);
            return true;
        }
        
        if node.type_id.contains("Divide") || node.type_id.contains("Division") {
            let changes = crate::nodes::math::divide::parameters::DivideNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id, execution_engine, graph);
            return true;
        }
        
        // Geometry nodes using Pattern A
        if node.type_id.contains("Sphere") && !node.type_id.contains("USD") {
            let changes = crate::nodes::three_d::geometry::sphere::parameters::SphereParameters::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id, execution_engine, graph);
            return true;
        }
        
        if node.type_id.contains("Cube") && !node.type_id.contains("USD") {
            let changes = crate::nodes::three_d::geometry::cube::parameters::CubeParameters::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id, execution_engine, graph);
            return true;
        }
        
        // Data nodes using Pattern A
        if title.contains("Constant") {
            let changes = crate::nodes::data::constant::parameters::ConstantNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id, execution_engine, graph);
            return true;
        }
        
        if title.contains("Variable") {
            let changes = crate::nodes::data::variable::parameters::VariableNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id, execution_engine, graph);
            return true;
        }
        
        // Output nodes using Pattern A
        if title.contains("Debug") {
            let changes = crate::nodes::output::debug::parameters::DebugNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id, execution_engine, graph);
            return true;
        }
        
        if title.contains("Print") {
            let changes = crate::nodes::output::print::parameters::PrintNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id, execution_engine, graph);
            return true;
        }
        
        // Logic nodes using Pattern A
        if title.contains("And") && !title.contains("USD") {
            let changes = crate::nodes::logic::and::parameters::AndNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id, execution_engine, graph);
            return true;
        }
        
        if title.contains("Or") && !title.contains("USD") {
            let changes = crate::nodes::logic::or::parameters::OrNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id, execution_engine, graph);
            return true;
        }
        
        if title.contains("Not") && !title.contains("USD") {
            let changes = crate::nodes::logic::not::parameters::NotNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id, execution_engine, graph);
            return true;
        }
        
        // Transform nodes using Pattern A
        if title.contains("Translate") && !title.contains("USD") {
            let changes = crate::nodes::three_d::transform::translate::parameters::TranslateNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id, execution_engine, graph);
            return true;
        }
        
        if title.contains("Scale") && !title.contains("USD") {
            let changes = crate::nodes::three_d::transform::scale::parameters::ScaleNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id, execution_engine, graph);
            return true;
        }
        
        if title.contains("Rotate") && !title.contains("USD") {
            let changes = crate::nodes::three_d::transform::rotate::parameters::RotateNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id, execution_engine, graph);
            return true;
        }
        
        // Lighting nodes using Pattern A
        if title.contains("Spot Light") || (title.contains("Spot") && title.contains("Light")) {
            let changes = crate::nodes::three_d::lighting::spot_light::parameters::SpotLightNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id, execution_engine, graph);
            return true;
        }
        
        if title.contains("Directional Light") || (title.contains("Directional") && title.contains("Light")) {
            let changes = crate::nodes::three_d::lighting::directional_light::parameters::DirectionalLightNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id, execution_engine, graph);
            return true;
        }
        
        if title.contains("Point Light") || (title.contains("Point") && title.contains("Light")) {
            let changes = crate::nodes::three_d::lighting::point_light::parameters::PointLightNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id, execution_engine, graph);
            return true;
        }
        
        // Geometry nodes using Pattern A
        if title.contains("Plane") && !title.contains("USD") {
            let changes = crate::nodes::three_d::geometry::plane::parameters::PlaneParameters::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id, execution_engine, graph);
            return true;
        }
        
        // Utility nodes using Pattern A
        if title.contains("Null") {
            let changes = crate::nodes::utility::null::parameters::NullNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id, execution_engine, graph);
            return true;
        }
        
        if node.type_id.contains("Test") {
            let changes = crate::nodes::utility::test::parameters::TestNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id, execution_engine, graph);
            return true;
        }
        
        // Data nodes using Pattern A
        if node.type_id.contains("Data_UsdFileReader") {
            let changes = crate::nodes::data::usd_file_reader::UsdFileReaderNode::build_interface(node, ui);
            self.apply_parameter_changes(node, changes, &title, node_id, execution_engine, graph);
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
        node_id: NodeId,
        execution_engine: &mut crate::nodes::NodeGraphEngine,
        graph: &crate::nodes::NodeGraph,
    ) {
        if !changes.is_empty() {
            let changes_count = changes.len();
            info!("Applied {} parameter changes for {} node {}", changes_count, title, node_id);
            for change in changes {
                node.parameters.insert(change.parameter, change.value);
            }
            
            // Notify execution engine that parameters changed
            println!("🔧 PARAMETER PANEL: Notifying execution engine of {} parameter changes for node {}", changes_count, node_id);
            execution_engine.on_node_parameter_changed(node_id, graph);
            
            // CRITICAL: For nodes that output USD scene data, force viewport refresh immediately
            // This ensures viewport gets fresh data when parameters change
            if let Some(current_node) = graph.nodes.get(&node_id) {
                let is_usd_source_node = match current_node.type_id.as_str() {
                    // USD File Reader
                    "Data_UsdFileReader" => true,
                    // All 3D geometry nodes that output USD scene data
                    "3D_Cube" | "3D_Sphere" | "3D_Cylinder" | "3D_Cone" | "3D_Plane" | "3D_Capsule" => true,
                    // Any other node types that output USD data
                    _ => current_node.type_id.contains("USD") || current_node.type_id.contains("3D_"),
                };
                
                if is_usd_source_node {
                    
                    // Find all downstream viewport nodes and force them to refresh immediately
                    let downstream_nodes = self.find_downstream_nodes(node_id, graph);
                    let mut connected_viewport_nodes = Vec::new();
                    
                    for downstream_id in &downstream_nodes {
                        if let Some(node) = graph.nodes.get(downstream_id) {
                            if node.type_id == "Viewport" || node.type_id == "3D_Viewport" {
                                connected_viewport_nodes.push(*downstream_id);
                            }
                        }
                    }
                    
                    // SIMPLIFIED: Clear only GPU caches when USD source parameters change
                    // Unified cache system handles data cache invalidation automatically
                    if !connected_viewport_nodes.is_empty() {
                        
                        // Clear GPU mesh caches (wgpu buffers)
                        crate::gpu::viewport_3d_callback::clear_all_gpu_mesh_caches();
                        
                        // Clear USD renderer cache (GPU resources)
                        use crate::nodes::three_d::ui::viewport::USD_RENDERER_CACHE;
                        if let Ok(mut cache) = USD_RENDERER_CACHE.lock() {
                            cache.renderers.clear();
                            cache.scene_bounds.clear();
                        }
                        
                        // Clear GPU viewport cache for connected viewports
                        use crate::nodes::three_d::ui::viewport::GPU_VIEWPORT_CACHE;
                        if let Ok(mut gpu_cache) = GPU_VIEWPORT_CACHE.lock() {
                            for viewport_id in &connected_viewport_nodes {
                                gpu_cache.remove(viewport_id);
                            }
                        }
                        
                        // CRITICAL: Mark the USD source node as dirty - unified cache handles the rest
                        execution_engine.mark_dirty(node_id, graph);
                        
                    }
                }
            }
            
            // NOTE: No need to call execute_dirty_nodes() here because
            // on_node_parameter_changed() already handles execution in Auto mode
            // This prevents double execution of the same nodes
            
        }
    }
    
    /// Find all nodes downstream from the given node
    fn find_downstream_nodes(&self, node_id: NodeId, graph: &crate::nodes::NodeGraph) -> Vec<NodeId> {
        let mut downstream = Vec::new();
        
        for connection in &graph.connections {
            if connection.from_node == node_id {
                downstream.push(connection.to_node);
            }
        }
        
        downstream
    }
    

    /// Render UI elements based on plugin description and return actions (legacy method)
    fn render_ui_elements(&self, ui: &mut egui::Ui, elements: &[crate::plugins::UIElement]) -> Vec<crate::plugins::UIAction> {
        let mut actions = Vec::new();
        
        for element in elements {
            match element {
                crate::plugins::UIElement::Heading(text) => {
                    ui.heading(text);
                }
                crate::plugins::UIElement::Label(text) => {
                    ui.label(text);
                }
                crate::plugins::UIElement::Separator => {
                    ui.separator();
                }
                crate::plugins::UIElement::TextEdit { label, value, parameter_name } => {
                    ui.horizontal(|ui| {
                        if !label.is_empty() {
                            ui.label(label);
                        }
                        let mut text_value = value.clone();
                        if ui.text_edit_singleline(&mut text_value).changed() {
                            actions.push(crate::plugins::UIAction::ParameterChanged {
                                parameter: parameter_name.clone(),
                                value: crate::plugins::NodeData::String(text_value),
                            });
                        }
                    });
                }
                crate::plugins::UIElement::Checkbox { label, value, parameter_name } => {
                    let mut checkbox_value = *value;
                    if ui.checkbox(&mut checkbox_value, label).changed() {
                        actions.push(crate::plugins::UIAction::ParameterChanged {
                            parameter: parameter_name.clone(),
                            value: crate::plugins::NodeData::Boolean(checkbox_value),
                        });
                    }
                }
                crate::plugins::UIElement::Button { label, action } => {
                    if ui.button(label).clicked() {
                        actions.push(crate::plugins::UIAction::ButtonClicked {
                            action: action.clone(),
                        });
                    }
                }
                crate::plugins::UIElement::Slider { label, value, min, max, parameter_name } => {
                    ui.horizontal(|ui| {
                        ui.label(label);
                        let mut slider_value = *value;
                        if ui.add(egui::Slider::new(&mut slider_value, *min..=*max)).changed() {
                            actions.push(crate::plugins::UIAction::ParameterChanged {
                                parameter: parameter_name.clone(),
                                value: crate::plugins::NodeData::Float(slider_value),
                            });
                        }
                    });
                }
                crate::plugins::UIElement::Vec3Edit { label, value, parameter_name } => {
                    ui.horizontal(|ui| {
                        ui.label(label);
                        let mut vec_value = *value;
                        let mut changed = false;
                        changed |= ui.add(egui::DragValue::new(&mut vec_value[0]).prefix("X:")).changed();
                        changed |= ui.add(egui::DragValue::new(&mut vec_value[1]).prefix("Y:")).changed();
                        changed |= ui.add(egui::DragValue::new(&mut vec_value[2]).prefix("Z:")).changed();
                        if changed {
                            actions.push(crate::plugins::UIAction::ParameterChanged {
                                parameter: parameter_name.clone(),
                                value: crate::plugins::NodeData::Vector3(vec_value),
                            });
                        }
                    });
                }
                crate::plugins::UIElement::ColorEdit { label, value, parameter_name } => {
                    ui.horizontal(|ui| {
                        ui.label(label);
                        let mut color_value = *value;
                        if ui.color_edit_button_rgb(&mut color_value).changed() {
                            actions.push(crate::plugins::UIAction::ParameterChanged {
                                parameter: parameter_name.clone(),
                                value: crate::plugins::NodeData::Color([color_value[0], color_value[1], color_value[2], 1.0]),
                            });
                        }
                    });
                }
                crate::plugins::UIElement::Horizontal(sub_elements) => {
                    ui.horizontal(|ui| {
                        let sub_actions = self.render_ui_elements(ui, sub_elements);
                        actions.extend(sub_actions);
                    });
                }
                crate::plugins::UIElement::Vertical(sub_elements) => {
                    ui.vertical(|ui| {
                        let sub_actions = self.render_ui_elements(ui, sub_elements);
                        actions.extend(sub_actions);
                    });
                }
                crate::plugins::UIElement::ColorPicker { label, value, parameter_name } => {
                    ui.horizontal(|ui| {
                        ui.label(label);
                        let mut color_value = *value;
                        if ui.color_edit_button_srgba(&mut egui::Color32::from_rgba_premultiplied(
                            (color_value[0] * 255.0) as u8,
                            (color_value[1] * 255.0) as u8,
                            (color_value[2] * 255.0) as u8,
                            (color_value[3] * 255.0) as u8,
                        )).changed() {
                            actions.push(crate::plugins::UIAction::ParameterChanged {
                                parameter: parameter_name.clone(),
                                value: crate::plugins::NodeData::Color(color_value),
                            });
                        }
                    });
                }
                crate::plugins::UIElement::ComboBox { label, selected, options, parameter_name } => {
                    ui.horizontal(|ui| {
                        ui.label(label);
                        let current_text = &options[*selected];
                        egui::ComboBox::from_id_salt(label)
                            .selected_text(current_text)
                            .show_ui(ui, |ui| {
                                for (i, option) in options.iter().enumerate() {
                                    let mut temp_selected = *selected;
                                    if ui.selectable_value(&mut temp_selected, i, option).changed() {
                                        actions.push(crate::plugins::UIAction::ParameterChanged {
                                            parameter: parameter_name.clone(),
                                            value: crate::plugins::NodeData::String(option.clone()),
                                        });
                                    }
                                }
                            });
                    });
                }
                crate::plugins::UIElement::Vector3Input { label, value, parameter_name } => {
                    ui.horizontal(|ui| {
                        ui.label(label);
                        let mut local_value = *value;
                        let mut changed = false;
                        changed |= ui.add(egui::DragValue::new(&mut local_value[0]).prefix("X:")).changed();
                        changed |= ui.add(egui::DragValue::new(&mut local_value[1]).prefix("Y:")).changed();
                        changed |= ui.add(egui::DragValue::new(&mut local_value[2]).prefix("Z:")).changed();
                        if changed {
                            actions.push(crate::plugins::UIAction::ParameterChanged {
                                parameter: parameter_name.clone(),
                                value: crate::plugins::NodeData::Vector3(local_value),
                            });
                        }
                    });
                }
                crate::plugins::UIElement::FilePicker { label, value, parameter_name, .. } => {
                    ui.horizontal(|ui| {
                        ui.label(label);
                        let mut local_value = value.clone();
                        let mut changed = ui.text_edit_singleline(&mut local_value).changed();
                        if ui.button("Browse").clicked() {
                            // File dialog would be handled by the main application
                            // For now, just mark as changed
                            changed = true;
                        }
                        if changed {
                            actions.push(crate::plugins::UIAction::ParameterChanged {
                                parameter: parameter_name.clone(),
                                value: crate::plugins::NodeData::String(local_value),
                            });
                        }
                    });
                }
                crate::plugins::UIElement::Group { label, children } => {
                    ui.group(|ui| {
                        ui.label(label);
                        let sub_actions = self.render_ui_elements(ui, children);
                        actions.extend(sub_actions);
                    });
                }
                crate::plugins::UIElement::Collapsible { label, children, .. } => {
                    ui.collapsing(label, |ui| {
                        let sub_actions = self.render_ui_elements(ui, children);
                        actions.extend(sub_actions);
                    });
                }
            }
        }
        
        actions
    }
    
}