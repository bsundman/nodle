//! Node editor implementation

// Module declarations
pub mod viewport;
pub mod input;
pub mod interaction;
pub mod menus;
pub mod rendering;

// Re-exports
pub use viewport::Viewport;
pub use input::InputState;
pub use interaction::InteractionManager;
pub use menus::MenuManager;
pub use rendering::MeshRenderer;

use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use egui_wgpu;
use crate::nodes::{
    NodeGraph,
};
use crate::context::ContextManager;
use crate::contexts::ContextRegistry;
use crate::gpu::NodeRenderCallback;
use crate::gpu::GpuInstanceManager;

/// Main application state for the node editor
pub struct NodeEditor {
    graph: NodeGraph,
    viewport: Viewport,
    input_state: InputState,      // Centralized input handling
    interaction: InteractionManager, // Node selection and dragging
    menus: MenuManager,           // Context menu management
    context_manager: ContextManager,
    // Performance tracking
    show_performance_info: bool,
    frame_times: Vec<f32>,
    last_frame_time: std::time::Instant,
    // GPU rendering toggle
    use_gpu_rendering: bool,
    // Persistent GPU instance manager
    gpu_instance_manager: GpuInstanceManager,
}

impl NodeEditor {
    pub fn new() -> Self {
        // Use the context registry to create a manager with all available contexts
        let context_manager = ContextRegistry::create_context_manager();
        
        let editor = Self {
            graph: NodeGraph::new(),
            viewport: Viewport::new(),
            input_state: InputState::new(),
            interaction: InteractionManager::new(),
            menus: MenuManager::new(),
            context_manager,
            // Performance tracking
            show_performance_info: false,
            frame_times: Vec::new(),
            last_frame_time: std::time::Instant::now(),
            // GPU rendering
            use_gpu_rendering: true, // Start with GPU rendering enabled
            // Persistent GPU instance manager
            gpu_instance_manager: GpuInstanceManager::new(),
        };

        // Start with empty node graph - use F2/F3/F4 to add test nodes

        editor
    }
    

    fn zoom_at_point(&mut self, screen_point: Pos2, zoom_delta: f32) {
        // Convert zoom delta to multiplication factor for viewport compatibility
        let zoom_factor = 1.0 + zoom_delta;
        self.viewport.zoom_at_point(screen_point, zoom_factor);
    }

    /// Handle context menu rendering and interactions
    fn handle_context_menu(&mut self, ui: &mut egui::Ui, _response: &egui::Response) {
        // Apply transforms for coordinate conversions
        let zoom = self.viewport.zoom;
        let pan_offset = self.viewport.pan_offset;

        let inverse_transform_pos = |pos: Pos2| -> Pos2 {
            Pos2::new(
                (pos.x - pan_offset.x) / zoom,
                (pos.y - pan_offset.y) / zoom,
            )
        };

        // Show context menu using MenuManager
        if let Some(menu_screen_pos) = self.input_state.get_context_menu_pos() {
            let menu_world_pos = self.input_state.get_right_click_world_pos().unwrap_or_else(|| inverse_transform_pos(menu_screen_pos));
            
            // Render the context menu using MenuManager
            let (selected_node_type, menu_response, submenu_response) = 
                self.menus.render_context_menu(ui, menu_screen_pos, &self.context_manager);
            
            // Handle node creation if a node type was selected
            if let Some(node_type) = selected_node_type {
                self.create_node(&node_type, menu_world_pos);
                self.input_state.close_context_menu();
                self.menus.reset();
            }
            
            // Handle mouse movement for submenu management
            if let Some(ref submenu_resp) = submenu_response {
                self.menus.handle_mouse_movement(
                    self.input_state.get_interact_pos(ui),
                    menu_response.rect,
                    Some(submenu_resp.rect)
                );
            }

            // Close entire menu if clicked outside all menu areas
            if self.input_state.primary_clicked(ui) {
                if let Some(click_pos) = self.input_state.get_interact_pos(ui) {
                    let mut clicked_outside = !menu_response.rect.contains(click_pos);
                    
                    // Also check submenu if open
                    if let Some(ref submenu_resp) = submenu_response {
                        clicked_outside = clicked_outside && !submenu_resp.rect.contains(click_pos);
                    }
                    
                    if clicked_outside {
                        self.input_state.close_context_menu();
                        self.menus.reset();
                    }
                }
            }

            // Close on Escape key
            if self.input_state.escape_pressed(ui) {
                self.input_state.close_context_menu();
                self.menus.reset();
            }
        }
    }

    fn create_node(&mut self, node_type: &str, position: Pos2) {
        // Map MaterialX display names to internal node types
        let internal_node_type = match node_type {
            "Noise" => "MaterialX_Noise",
            "Texture" => "MaterialX_Texture", 
            "Mix" => "MaterialX_Mix",
            "Standard Surface" => "MaterialX_StandardSurface",
            "3D View" => "MaterialX_3DView",
            "2D View" => "MaterialX_2DView",
            _ => node_type, // Use original name for generic nodes
        };
        
        // Try context-specific nodes first
        if let Some(context) = self.context_manager.get_active_context() {
            if let Some(node) = crate::NodeRegistry::create_context_node(context, internal_node_type, position) {
                self.graph.add_node(node);
                self.gpu_instance_manager.force_rebuild();
                return;
            }
        }
        
        // Fall back to generic nodes
        if let Some(node) = crate::NodeRegistry::create_node(internal_node_type, position) {
            self.graph.add_node(node);
            self.gpu_instance_manager.force_rebuild();
        }
    }

    /// Add benchmark nodes in a grid pattern for performance testing
    fn add_benchmark_nodes(&mut self, count: usize) {
        
        let node_types = ["Add", "Subtract", "Multiply", "Divide", "AND", "OR", "NOT", "Constant", "Variable", "Print", "Debug"];
        let spacing = 120.0;
        let grid_cols = (count as f32).sqrt().ceil() as usize;
        
        for i in 0..count {
            let col = i % grid_cols;
            let row = i / grid_cols;
            let x = 50.0 + col as f32 * spacing;
            let y = 100.0 + row as f32 * spacing;
            let node_type = node_types[i % node_types.len()];
            
            if let Some(node) = crate::NodeRegistry::create_node(node_type, Pos2::new(x, y)) {
                self.graph.add_node(node);
                self.gpu_instance_manager.force_rebuild();
            }
        }
    }

    /// Add a large number of nodes with many connections for serious performance stress testing
    fn add_performance_stress_test(&mut self, count: usize) {
        let start_time = std::time::Instant::now();
        let node_types = ["Add", "Subtract", "Multiply", "Divide", "AND", "OR", "NOT", "Constant", "Variable", "Print", "Debug"];
        
        // Calculate grid that fits in reasonable space with compact spacing
        let spacing = 80.0; // Tighter spacing for 1000 nodes
        let grid_cols = (count as f32).sqrt().ceil() as usize;
        
        // Create all nodes first
        let mut node_ids = Vec::new();
        for i in 0..count {
            let col = i % grid_cols;
            let row = i / grid_cols;
            let x = 50.0 + col as f32 * spacing;
            let y = 100.0 + row as f32 * spacing;
            let node_type = node_types[i % node_types.len()];
            
            if let Some(node) = crate::NodeRegistry::create_node(node_type, Pos2::new(x, y)) {
                let node_id = self.graph.add_node(node);
                node_ids.push(node_id);
            }
        }
        
        let _node_creation_time = start_time.elapsed();
        
        // Create many connections for performance testing
        let connection_start = std::time::Instant::now();
        let connection_count = (count / 2).min(500); // Create up to 500 connections
        
        for i in 0..connection_count {
            if i + 1 < node_ids.len() {
                let from_id = node_ids[i];
                let to_id = node_ids[i + 1];
                
                // Try to create a connection (may fail if ports don't match)
                let connection = crate::nodes::Connection::new(from_id, 0, to_id, 0);
                let _ = self.graph.add_connection(connection); // Ignore errors for stress test
            }
            
            // Also create some random long-distance connections
            if i % 10 == 0 && i + 20 < node_ids.len() {
                let from_id = node_ids[i];
                let to_id = node_ids[i + 20];
                let connection = crate::nodes::Connection::new(from_id, 0, to_id, 0);
                let _ = self.graph.add_connection(connection);
            }
        }
        
        let _connection_time = connection_start.elapsed();
        let _total_time = start_time.elapsed();
        
        // Force GPU instance rebuild after adding many nodes
        self.gpu_instance_manager.force_rebuild();
    }

    /// Draw a dashed path for connection cutting visualization
    fn draw_dashed_path(&self, painter: &egui::Painter, path: &[Pos2], transform_pos: &impl Fn(Pos2) -> Pos2, zoom: f32, color: Color32) {
        if path.len() < 2 {
            return;
        }
        
        let dash_length = 8.0 * zoom;
        let gap_length = 4.0 * zoom;
        let stroke_width = 2.0 * zoom;
        
        for window in path.windows(2) {
            let start = transform_pos(window[0]);
            let end = transform_pos(window[1]);
            
            let segment_length = (end - start).length();
            let direction = (end - start) / segment_length;
            
            let mut distance = 0.0;
            let mut drawing_dash = true;
            
            while distance < segment_length {
                let next_distance = if drawing_dash {
                    (distance + dash_length).min(segment_length)
                } else {
                    (distance + gap_length).min(segment_length)
                };
                
                if drawing_dash {
                    let dash_start = start + direction * distance;
                    let dash_end = start + direction * next_distance;
                    
                    painter.line_segment(
                        [dash_start, dash_end],
                        Stroke::new(stroke_width, color),
                    );
                }
                
                distance = next_distance;
                drawing_dash = !drawing_dash;
            }
        }
    }


}

impl eframe::App for NodeEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request repaint
        ctx.request_repaint();

        // Track frame time for performance monitoring
        let current_time = std::time::Instant::now();
        let frame_time = current_time.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = current_time;
        
        self.frame_times.push(frame_time);
        if self.frame_times.len() > 60 { // Keep last 60 frames (1 second at 60fps)
            self.frame_times.remove(0);
        }
        
        // Set dark theme for window decorations
        ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(egui::SystemTheme::Dark));

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(22, 27, 34)))
            .show(ctx, |ui| {
            // Add padding around the top menu bar
            ui.add_space(8.0); // Top padding
            
            // Draw menu bar background
            let menu_height = 40.0; // Approximate height for menu bar
            let menu_rect = egui::Rect::from_min_size(
                egui::Pos2::new(0.0, 8.0), 
                egui::Vec2::new(ui.available_width(), menu_height)
            );
            ui.painter().rect_filled(
                menu_rect,
                0.0,
                Color32::from_rgb(28, 28, 28), // Standard background color
            );
            
            ui.horizontal(|ui| {
                ui.add_space(12.0); // Left padding
                
                // Context selection
                ui.label("Context:");
                
                // Collect context information to avoid borrowing issues
                let current_context_name = self.context_manager.get_active_context()
                    .map(|c| c.display_name())
                    .unwrap_or("None");
                let context_names: Vec<String> = self.context_manager.get_contexts()
                    .iter()
                    .map(|c| c.display_name().to_string())
                    .collect();
                
                let mut selected_context = self.context_manager.get_active_context().map(|_| 0);
                egui::ComboBox::from_label("")
                    .selected_text(current_context_name)
                    .show_ui(ui, |ui| {
                        if ui.selectable_value(&mut selected_context, None, "None").clicked() {
                            self.context_manager.set_active_context(None);
                        }
                        
                        for (i, context_name) in context_names.iter().enumerate() {
                            if ui.selectable_value(&mut selected_context, Some(i), context_name).clicked() {
                                self.context_manager.set_active_context(Some(i));
                            }
                        }
                    });
                
                ui.separator();
                ui.label(format!("Zoom: {:.1}x", self.viewport.zoom));
                ui.label(format!(
                    "Pan: ({:.0}, {:.0})",
                    self.viewport.pan_offset.x, self.viewport.pan_offset.y
                ));
                
                ui.add_space(12.0); // Right padding
            });
            ui.add_space(8.0); // Bottom padding

            let response = ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());
            
            // Set cursor based on special modes
            if self.input_state.is_cutting_mode() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair); // Use crosshair for cutting (X key)
            } else if self.input_state.is_connecting_mode() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); // Use pointing hand for connecting (C key)
            }
            
            // Handle context menu before creating the painter (to avoid borrow conflicts)
            self.handle_context_menu(ui, &response);
            
            let painter = ui.painter();

            // Draw node graph background
            painter.rect_filled(
                response.rect,
                0.0,
                Color32::from_rgb(28, 28, 28), // Standard background color
            );

            // Apply zoom and pan transforms using viewport
            let zoom = self.viewport.zoom;
            let pan_offset = self.viewport.pan_offset;

            let transform_pos = |pos: Pos2| -> Pos2 {
                Pos2::new(
                    pos.x * zoom + pan_offset.x,
                    pos.y * zoom + pan_offset.y,
                )
            };

            let inverse_transform_pos = |pos: Pos2| -> Pos2 {
                Pos2::new(
                    (pos.x - pan_offset.x) / zoom,
                    (pos.y - pan_offset.y) / zoom,
                )
            };

            // Update input state
            self.input_state.update(ui, &response, inverse_transform_pos);

            // Handle pan and zoom using input state
            if let Some(pan_delta) = self.input_state.get_pan_delta(&response) {
                self.viewport.pan(pan_delta);
            }

            // Handle zoom with mouse wheel using input state
            if self.input_state.has_scroll_input() {
                if let Some(mouse_pos) = response.hover_pos() {
                    self.zoom_at_point(mouse_pos, self.input_state.get_zoom_delta());
                }
            }

            // Handle special modes (cutting and connecting)
            if self.input_state.is_cutting_mode() {
                // In cutting mode - skip normal interactions
                // Cutting is handled in the input state update
            } else if self.input_state.is_connecting_mode() {
                // In connecting mode - skip normal interactions
                // Connecting is handled in the input state update
            } else if let Some(pos) = self.input_state.mouse_world_pos {
                // Skip node interaction if we're panning
                if !self.input_state.is_panning {
                    // Handle clicks (not just drags)
                    if self.input_state.clicked_this_frame {
                        // Check if we clicked on a port first
                        if let Some((node_id, port_idx, is_input)) = self.input_state.find_clicked_port(&self.graph, 10.0) {
                            // Handle connection logic
                            if self.input_state.is_connecting_active() {
                                // Try to complete connection
                                if let Some(connection) = self.input_state.complete_connection(node_id, port_idx) {
                                    // Check if target is an input port and already has a connection
                                    if is_input {
                                        if let Some((existing_idx, _, _)) = self.input_state.find_input_connection(&self.graph, node_id, port_idx) {
                                            // Remove existing connection to input port
                                            self.graph.remove_connection(existing_idx);
                                        }
                                    }
                                    let _ = self.graph.add_connection(connection);
                                    self.gpu_instance_manager.force_rebuild();
                                } else {
                                    // Start new connection from this port
                                    self.input_state.start_connection(node_id, port_idx, is_input);
                                    self.gpu_instance_manager.force_rebuild();
                                }
                            } else {
                                // Not currently connecting - check if clicking on connected input port
                                if is_input {
                                    if let Some((conn_idx, from_node, from_port)) = self.input_state.find_input_connection(&self.graph, node_id, port_idx) {
                                        // Disconnect and start new connection from original source
                                        self.graph.remove_connection(conn_idx);
                                        self.input_state.start_connection(from_node, from_port, false);
                                        self.gpu_instance_manager.force_rebuild();
                                        return; // Skip starting connection from input port
                                    }
                                }
                                // Start new connection from this port
                                self.input_state.start_connection(node_id, port_idx, is_input);
                                self.gpu_instance_manager.force_rebuild();
                            }
                        } else if let Some(node_id) = self.input_state.find_node_under_mouse(&self.graph) {
                            // Handle node selection
                            self.interaction.select_node(node_id, self.input_state.is_multi_select());
                            self.gpu_instance_manager.force_rebuild();
                        } else if let Some(connection_idx) = self.input_state.find_clicked_connection(&self.graph, 8.0, self.viewport.zoom) {
                            // Handle connection selection with multi-select support
                            self.interaction.select_connection_multi(connection_idx, self.input_state.is_multi_select());
                            self.gpu_instance_manager.force_rebuild();
                        } else {
                            // Clicked on empty space - deselect all and cancel connections
                            self.interaction.clear_selection();
                            self.input_state.cancel_connection();
                            self.gpu_instance_manager.force_rebuild();
                        }
                    }

                    // Handle drag start for connections, node movement and box selection
                    if self.input_state.drag_started_this_frame {
                        // Check if we're starting to drag from a port for connections
                        if let Some((node_id, port_idx, is_input)) = self.input_state.find_clicked_port(&self.graph, 10.0) {
                            // Handle input port disconnection on drag
                            if is_input {
                                if let Some((conn_idx, from_node, from_port)) = self.input_state.find_input_connection(&self.graph, node_id, port_idx) {
                                    // Disconnect and start new connection from original source
                                    self.graph.remove_connection(conn_idx);
                                    self.input_state.start_connection(from_node, from_port, false);
                                    self.gpu_instance_manager.force_rebuild();
                                } else {
                                    // No existing connection, start from input port
                                    self.input_state.start_connection(node_id, port_idx, is_input);
                                    self.gpu_instance_manager.force_rebuild();
                                }
                            } else {
                                // Output port - start connection normally
                                self.input_state.start_connection(node_id, port_idx, is_input);
                                self.gpu_instance_manager.force_rebuild();
                            }
                        } else {
                            // Check if we're starting to drag a selected node
                            let mut dragging_selected = false;
                            for &node_id in &self.interaction.selected_nodes {
                                if let Some(node) = self.graph.nodes.get(&node_id) {
                                    if node.get_rect().contains(pos) {
                                        // Start dragging selected nodes
                                        self.interaction.start_drag(pos, &self.graph);
                                        dragging_selected = true;
                                        break;
                                    }
                                }
                            }
                            
                            // If not dragging selected nodes, check for clicking on any node
                            if !dragging_selected {
                                if let Some(node_id) = self.input_state.find_node_under_mouse(&self.graph) {
                                    // Select the node and start dragging it
                                    self.interaction.select_node(node_id, false);
                                    self.interaction.start_drag(pos, &self.graph);
                                } else {
                                    // Start box selection if not on any node and using left mouse button
                                    if self.input_state.is_primary_down(ui) {
                                        self.interaction.start_box_selection(pos);
                                    }
                                }
                            }
                        }
                    }

                    // Handle dragging
                    if response.dragged() {
                        if !self.interaction.drag_offsets.is_empty() {
                            // Drag all selected nodes
                            self.interaction.update_drag(pos, &mut self.graph);
                            // Force GPU instance manager to rebuild when nodes are moved
                            self.gpu_instance_manager.force_rebuild();
                        } else if self.interaction.box_selection_start.is_some() {
                            // Update box selection
                            self.interaction.update_box_selection(pos);
                        }
                    }

                    // Handle connection completion
                    if self.input_state.drag_stopped_this_frame {
                        if self.input_state.is_connecting_active() {
                            // Check if we released on a port to complete connection
                            if let Some((node_id, port_idx, _)) = self.input_state.find_clicked_port(&self.graph, 10.0) {
                                if let Some(connection) = self.input_state.complete_connection(node_id, port_idx) {
                                    let _ = self.graph.add_connection(connection);
                                }
                            } else {
                                // Cancel connection if we didn't release on a port
                                self.input_state.cancel_connection();
                            }
                            self.gpu_instance_manager.force_rebuild();
                        }
                    }
                }

                if self.input_state.drag_stopped_this_frame {
                    // Ensure final positions are updated in GPU
                    if self.use_gpu_rendering {
                        self.gpu_instance_manager.force_rebuild();
                    }

                    // Complete box selection
                    if self.interaction.box_selection_start.is_some() {
                        self.interaction.complete_box_selection(&self.graph, self.input_state.is_multi_select());
                        self.gpu_instance_manager.force_rebuild();
                    }
                    
                    // End any dragging operations
                    self.interaction.end_drag();
                }
            }

            // Handle keyboard input using input state
            if self.input_state.delete_pressed(ui) {
                if !self.interaction.selected_nodes.is_empty() {
                    // Delete all selected nodes
                    self.interaction.delete_selected(&mut self.graph);
                    self.gpu_instance_manager.force_rebuild();
                } else if !self.interaction.selected_connections.is_empty() {
                    // Delete all selected connections (in reverse order to maintain indices)
                    let mut connection_indices: Vec<usize> = self.interaction.selected_connections.iter().copied().collect();
                    connection_indices.sort_by(|a, b| b.cmp(a)); // Sort in reverse order
                    
                    for conn_idx in connection_indices {
                        self.graph.remove_connection(conn_idx);
                    }
                    
                    self.interaction.clear_connection_selection();
                    self.gpu_instance_manager.force_rebuild();
                }
            }

            // Handle ESC key to cancel connections
            if self.input_state.escape_pressed(ui) {
                self.input_state.cancel_connection();
                self.gpu_instance_manager.force_rebuild();
            }

            // Handle connection cutting when X key is released
            if !self.input_state.is_cutting_mode() && (!self.input_state.get_cut_paths().is_empty() || !self.input_state.get_current_cut_path().is_empty()) {
                // X key was just released - apply cuts
                let cut_connections = self.input_state.find_cut_connections(&self.graph, self.viewport.zoom);
                
                if !cut_connections.is_empty() {
                    // Sort in reverse order to maintain indices during deletion
                    let mut sorted_cuts = cut_connections;
                    sorted_cuts.sort_by(|a, b| b.cmp(a));
                    
                    for conn_idx in sorted_cuts {
                        self.graph.remove_connection(conn_idx);
                    }
                    
                    self.gpu_instance_manager.force_rebuild();
                }
                
                // Clear cut paths after applying
                self.input_state.clear_cut_paths();
            }

            // Handle connection drawing when C key is released
            if !self.input_state.is_connecting_mode() && (!self.input_state.get_connect_paths().is_empty() || !self.input_state.get_current_connect_path().is_empty()) {
                // C key was just released - create connections from drawn paths
                let new_connections = self.input_state.create_connections_from_paths(&self.graph);
                
                if !new_connections.is_empty() {
                    for connection in new_connections {
                        // Check if target is an input port and already has a connection
                        if let Some((existing_idx, _, _)) = self.input_state.find_input_connection(&self.graph, connection.to_node, connection.to_port) {
                            // Remove existing connection to input port
                            self.graph.remove_connection(existing_idx);
                        }
                        
                        let _ = self.graph.add_connection(connection);
                    }
                    
                    self.gpu_instance_manager.force_rebuild();
                }
                
                // Clear connect paths after applying
                self.input_state.clear_connect_paths();
            }

            // Handle F1 to toggle performance info
            if self.input_state.f1_pressed(ui) {
                self.show_performance_info = !self.show_performance_info;
            }

            // Handle F2-F4 to add different numbers of nodes
            if self.input_state.f2_pressed(ui) {
                self.add_benchmark_nodes(10);
            }
            if self.input_state.f3_pressed(ui) {
                self.add_benchmark_nodes(25);
            }
            if self.input_state.f4_pressed(ui) {
                self.add_performance_stress_test(5000);
            }

            // Handle F5 to clear all nodes
            if self.input_state.f5_pressed(ui) {
                self.graph.nodes.clear();
                self.graph.connections.clear();
                self.interaction.clear_selection();
                self.input_state.cancel_connection();
                self.gpu_instance_manager.force_rebuild();
            }

            // Handle F6 to toggle GPU/CPU rendering
            if self.input_state.f6_pressed(ui) {
                self.use_gpu_rendering = !self.use_gpu_rendering;
            }

            // Handle right-click for context menu first (before other input handling)
            if self.input_state.right_clicked_this_frame {
                if let Some(node_id) = self.input_state.find_node_under_mouse(&self.graph) {
                    // Right-clicked on a node - select it
                    self.interaction.select_node(node_id, false);
                } else {
                    // Right-clicked on empty space - context menu is handled in InputState update
                    // (context_menu_pos is automatically set)
                }
            }


            // Update port positions
            self.graph.update_all_port_positions();

            // Draw nodes - GPU vs CPU rendering
            if self.use_gpu_rendering && !self.graph.nodes.is_empty() {
                // Calculate viewport bounds for GPU callback
                let viewport_rect = response.rect;
                
                // Create GPU callback for node body rendering  
                // Use the full screen size, not just the response rect size, to match GPU viewport
                let screen_size = Vec2::new(
                    ui.ctx().screen_rect().width(),
                    ui.ctx().screen_rect().height()
                );
                
                // Use persistent instance manager for optimal performance
                let (node_instances, port_instances) = self.gpu_instance_manager.update_instances(
                    &self.graph.nodes,
                    &self.interaction.selected_nodes,
                    self.input_state.get_connecting_from(),
                );
                
                let gpu_callback = NodeRenderCallback::from_instances(
                    node_instances,
                    port_instances,
                    self.viewport.pan_offset,
                    self.viewport.zoom,
                    screen_size,
                );
                
                // Add the GPU paint callback using egui_wgpu::Callback - this will trigger prepare() and paint() methods
                painter.add(egui_wgpu::Callback::new_paint_callback(
                    viewport_rect,
                    gpu_callback,
                ));
                
                // Render node titles using CPU (GPU handles node bodies and ports)
                for (_node_id, node) in &self.graph.nodes {
                    // Node titles (CPU-rendered text)
                    painter.text(
                        transform_pos(node.position + Vec2::new(node.size.x / 2.0, 15.0)),
                        egui::Align2::CENTER_CENTER,
                        &node.title,
                        egui::FontId::proportional(12.0 * self.viewport.zoom),
                        Color32::WHITE,
                    );
                    
                    // Port names on hover (CPU-rendered text)
                    if let Some(mouse_world_pos) = self.input_state.mouse_world_pos {
                        // Input port names
                        for input in &node.inputs {
                            if (input.position - mouse_world_pos).length() < 10.0 {
                                painter.text(
                                    transform_pos(input.position - Vec2::new(0.0, 15.0)),
                                    egui::Align2::CENTER_BOTTOM,
                                    &input.name,
                                    egui::FontId::proportional(10.0 * self.viewport.zoom),
                                    Color32::WHITE,
                                );
                            }
                        }
                        
                        // Output port names
                        for output in &node.outputs {
                            if (output.position - mouse_world_pos).length() < 10.0 {
                                painter.text(
                                    transform_pos(output.position + Vec2::new(0.0, 15.0)),
                                    egui::Align2::CENTER_TOP,
                                    &output.name,
                                    egui::FontId::proportional(10.0 * self.viewport.zoom),
                                    Color32::WHITE,
                                );
                            }
                        }
                    }
                }
                
            } else if !self.graph.nodes.is_empty() {
                // CPU rendering path - fallback mode using MeshRenderer
                for (node_id, node) in &self.graph.nodes {
                    let is_selected = self.interaction.selected_nodes.contains(&node_id);
                    
                    // Render complete node using MeshRenderer
                    MeshRenderer::render_node_complete_cpu(
                        &painter,
                        node,
                        is_selected,
                        zoom,
                        &transform_pos,
                    );

                    // Draw ports using MeshRenderer
                    // Input ports (on top)
                    for (port_idx, input) in node.inputs.iter().enumerate() {
                        // Check if this port is being used for an active connection or connection preview
                        let mut is_connecting_port = if let Some((from_node, from_port, from_is_input)) = self.input_state.get_connecting_from() {
                            from_node == *node_id && from_port == port_idx && from_is_input
                        } else {
                            false
                        };
                        
                        // Also check if this port is in the connection drawing preview
                        if !is_connecting_port && self.input_state.is_connecting_mode() {
                            if let Some(((start_node, start_port, start_is_input), (end_node, end_port, end_is_input))) = self.input_state.get_connection_preview(&self.graph) {
                                if (start_node == *node_id && start_port == port_idx && start_is_input) ||
                                   (end_node == *node_id && end_port == port_idx && end_is_input) {
                                    is_connecting_port = true;
                                }
                            }
                        }
                        
                        // Render port using MeshRenderer
                        MeshRenderer::render_port_complete_cpu(
                            &painter,
                            input.position,
                            true, // is_input
                            is_connecting_port,
                            zoom,
                            &transform_pos,
                        );
                        
                        // Render port name on hover using MeshRenderer
                        MeshRenderer::render_port_name_on_hover(
                            &painter,
                            input.position,
                            &input.name,
                            true, // is_input
                            self.input_state.mouse_world_pos,
                            zoom,
                            &transform_pos,
                        );
                    }

                    // Output ports (on bottom)
                    for (port_idx, output) in node.outputs.iter().enumerate() {
                        // Check if this port is being used for an active connection or connection preview
                        let mut is_connecting_port = if let Some((from_node, from_port, from_is_input)) = self.input_state.get_connecting_from() {
                            from_node == *node_id && from_port == port_idx && !from_is_input
                        } else {
                            false
                        };
                        
                        // Also check if this port is in the connection drawing preview
                        if !is_connecting_port && self.input_state.is_connecting_mode() {
                            if let Some(((start_node, start_port, start_is_input), (end_node, end_port, end_is_input))) = self.input_state.get_connection_preview(&self.graph) {
                                if (start_node == *node_id && start_port == port_idx && !start_is_input) ||
                                   (end_node == *node_id && end_port == port_idx && !end_is_input) {
                                    is_connecting_port = true;
                                }
                            }
                        }
                        
                        // Render port using MeshRenderer
                        MeshRenderer::render_port_complete_cpu(
                            &painter,
                            output.position,
                            false, // is_input
                            is_connecting_port,
                            zoom,
                            &transform_pos,
                        );
                        
                        // Render port name on hover using MeshRenderer
                        MeshRenderer::render_port_name_on_hover(
                            &painter,
                            output.position,
                            &output.name,
                            false, // is_input
                            self.input_state.mouse_world_pos,
                            zoom,
                            &transform_pos,
                        );
                    }
                }
            } // End of CPU rendering mode
            

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

                        // Highlight selected connections
                        let (stroke_width, stroke_color) = if self.interaction.selected_connections.contains(&idx)
                        {
                            (4.0 * zoom, Color32::from_rgb(88, 166, 255)) // Blue accent for selected
                        } else {
                            (2.0 * zoom, Color32::from_rgb(100, 110, 120)) // Darker gray for normal
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
            if let Some((from_node, from_port, from_is_input)) = self.input_state.get_connecting_from() {
                if let Some(mouse_pos) = self.input_state.mouse_pos {
                    if let Some(node) = self.graph.nodes.get(&from_node) {
                        let from_pos = if from_is_input {
                            node.inputs[from_port].position
                        } else {
                            node.outputs[from_port].position
                        };

                        let transformed_from = transform_pos(from_pos);
                        let transformed_to = mouse_pos;

                        // Draw bezier curve for connection preview (vertical flow)
                        // Use fixed control offset to prevent popping when curve goes horizontal
                        let control_offset = 60.0 * zoom;

                        // Control points should aim in the correct direction based on port type
                        let from_control = if from_is_input {
                            transformed_from - Vec2::new(0.0, control_offset) // Input ports: aim up
                        } else {
                            transformed_from + Vec2::new(0.0, control_offset) // Output ports: aim down
                        };
                        
                        let to_control = if from_is_input {
                            transformed_to + Vec2::new(0.0, control_offset) // When connecting from input: aim up from mouse
                        } else {
                            transformed_to - Vec2::new(0.0, control_offset) // When connecting from output: aim down to mouse
                        };

                        let points = [
                            transformed_from,
                            from_control,
                            to_control,
                            transformed_to,
                        ];

                        painter.add(egui::Shape::CubicBezier(egui::epaint::CubicBezierShape {
                            points,
                            closed: false,
                            fill: Color32::TRANSPARENT,
                            stroke: Stroke::new(2.0 * zoom, Color32::from_rgb(100, 180, 255))
                                .into(),
                        }));
                    }
                }
            }

            // Draw cut paths (dashed lines)
            if self.input_state.is_cutting_mode() {
                // Draw completed cut paths
                for cut_path in self.input_state.get_cut_paths() {
                    self.draw_dashed_path(&painter, cut_path, &transform_pos, zoom, Color32::from_rgb(255, 100, 100));
                }
                
                // Draw current cut path being drawn
                if !self.input_state.get_current_cut_path().is_empty() {
                    self.draw_dashed_path(&painter, self.input_state.get_current_cut_path(), &transform_pos, zoom, Color32::from_rgb(255, 150, 150));
                }
            }

            // Draw connect paths (dashed lines)
            if self.input_state.is_connecting_mode() {
                // Draw completed connect paths
                for connect_path in self.input_state.get_connect_paths() {
                    self.draw_dashed_path(&painter, connect_path, &transform_pos, zoom, Color32::from_rgb(100, 255, 100));
                }
                
                // Draw current connect path being drawn
                if !self.input_state.get_current_connect_path().is_empty() {
                    self.draw_dashed_path(&painter, self.input_state.get_current_connect_path(), &transform_pos, zoom, Color32::from_rgb(150, 255, 150));
                }
            }

            // Draw box selection
            if let (Some(start), Some(end)) = (self.interaction.box_selection_start, self.interaction.box_selection_end) {
                let selection_rect = egui::Rect::from_two_pos(start, end);
                let transformed_rect = Rect::from_two_pos(transform_pos(selection_rect.min), transform_pos(selection_rect.max));

                // Draw selection box background
                painter.rect_filled(
                    transformed_rect,
                    0.0,
                    Color32::from_rgba_unmultiplied(100, 150, 255, 30),
                );

                // Draw selection box border
                painter.rect_stroke(
                    transformed_rect,
                    0.0,
                    Stroke::new(1.0 * zoom, Color32::from_rgb(100, 150, 255)),
                );
            }

            // Performance info overlay
            if self.show_performance_info && !self.frame_times.is_empty() {
                let avg_frame_time = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
                let fps = 1.0 / avg_frame_time;
                let rendering_mode = if self.use_gpu_rendering { "GPU" } else { "CPU" };
                
                egui::Window::new("Performance")
                    .default_pos([10.0, 10.0])
                    .default_size([200.0, 100.0])
                    .resizable(false)
                    .show(ui.ctx(), |ui| {
                        ui.label(format!("FPS: {:.1}", fps));
                        ui.label(format!("Frame time: {:.2}ms", avg_frame_time * 1000.0));
                        ui.label(format!("Rendering: {}", rendering_mode));
                        ui.label(format!("Nodes: {}", self.graph.nodes.len()));
                        ui.separator();
                        ui.label("F1: Toggle performance info");
                        ui.label("F2: Add 10 nodes");
                        ui.label("F3: Add 25 nodes");
                        ui.label("F4: Stress test (5000 nodes + connections)");
                        ui.label("F5: Clear all nodes");
                        ui.label("F6: Toggle GPU/CPU rendering");
                    });
            }
        });
    }
}