//! Node editor implementation

// Module declarations
pub mod canvas;
pub mod input;
pub mod interaction;
pub mod menus;
pub mod canvas_rendering;
pub mod navigation;
pub mod file_manager;
pub mod panels;
pub mod debug_tools;
pub mod workspace_builder;

// Re-exports
pub use canvas::Canvas;
pub use input::InputState;
pub use interaction::InteractionManager;
pub use menus::MenuManager;
pub use canvas_rendering::MeshRenderer;
pub use navigation::{NavigationManager, NavigationAction, GraphView};
pub use file_manager::FileManager;
pub use panels::{PanelManager, PanelAction};
pub use debug_tools::DebugToolsManager;
pub use workspace_builder::{WorkspaceBuilder, NodeCompatibility};

use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use egui_wgpu;
use crate::nodes::{
    NodeGraph, Node, NodeId, Connection, PanelType,
};
use std::collections::HashMap;
use std::path::Path;
use crate::workspace::WorkspaceManager;
use crate::workspaces::WorkspaceRegistry;
use crate::gpu::NodeRenderCallback;
use crate::gpu::GpuInstanceManager;

/// Main application state for the node editor
pub struct NodeEditor {
    graph: NodeGraph,
    canvas: Canvas,
    input_state: InputState,      // Centralized input handling
    interaction: InteractionManager, // Node selection and dragging
    menus: MenuManager,           // Context menu management
    navigation: NavigationManager, // Workspace navigation and breadcrumbs
    workspace_manager: WorkspaceManager,
    // Interface panel system
    panel_manager: PanelManager,
    // Debug and performance monitoring
    debug_tools: DebugToolsManager,
    // GPU rendering toggle
    use_gpu_rendering: bool,
    // Persistent GPU instance manager
    gpu_instance_manager: GpuInstanceManager,
    // File management
    file_manager: FileManager,
    // Menu state
    show_file_menu: bool,
    // Layout constraints
    current_menu_bar_height: f32,
}


impl NodeEditor {
    /// Creates a window that automatically respects the menu bar constraint
    /// Use this instead of egui::Window::new() for all windows in the app
    fn create_window<'a>(title: &'a str, ctx: &egui::Context, menu_bar_height: f32) -> egui::Window<'a> {
        egui::Window::new(title)
            .constrain_to(egui::Rect::from_min_size(
                egui::Pos2::new(0.0, menu_bar_height), 
                egui::Vec2::new(ctx.screen_rect().width(), ctx.screen_rect().height() - menu_bar_height)
            ))
    }

    pub fn new() -> Self {
        // Use the workspace registry to create a manager with all available workspaces
        let workspace_manager = WorkspaceRegistry::create_workspace_manager();
        
        let mut editor = Self {
            graph: NodeGraph::new(),
            canvas: Canvas::new(),
            input_state: InputState::new(),
            interaction: InteractionManager::new(),
            menus: MenuManager::new(),
            navigation: NavigationManager::new(),
            workspace_manager,
            // Interface panel system
            panel_manager: PanelManager::new(),
            // Debug and performance monitoring
            debug_tools: DebugToolsManager::new(),
            // GPU rendering
            use_gpu_rendering: true, // Start with GPU rendering enabled
            // Persistent GPU instance manager
            gpu_instance_manager: GpuInstanceManager::new(),
            // File management
            file_manager: FileManager::new(),
            // Menu state
            show_file_menu: false,
            // Layout constraints
            current_menu_bar_height: 0.0,
        };

        // Start with empty node graph - nodes created at 150.0px x 30.0px

        editor
    }
    
    /// Store the current menu bar height for window constraints
    fn store_menu_bar_height(&mut self, height: f32) {
        self.current_menu_bar_height = height;
    }
    
    /// Get the nodes to render based on current view
    fn get_viewed_nodes(&self) -> HashMap<NodeId, Node> {
        self.navigation.get_viewed_nodes(&self.graph)
    }
    
    /// Get the connections to render based on current view
    fn get_viewed_connections(&self) -> Vec<Connection> {
        self.navigation.get_viewed_connections(&self.graph)
    }
    
    /// Build a temporary graph for GPU processing
    fn build_temp_graph(&self, nodes: &HashMap<NodeId, Node>) -> NodeGraph {
        self.navigation.build_temp_graph(nodes, &self.graph)
    }
    
    /// Get mutable reference to a workspace node's internal graph
    fn get_workspace_graph_mut(&mut self, node_id: NodeId) -> Option<&mut NodeGraph> {
        if let Some(node) = self.graph.nodes.get_mut(&node_id) {
            node.get_internal_graph_mut()
        } else {
            None
        }
    }
    
    /// Get the currently active graph for reading (root or workspace internal graph)
    fn get_active_graph(&self) -> &NodeGraph {
        self.navigation.get_active_graph(&self.graph)
    }
    
    /// Add a connection to the appropriate graph based on current view
    fn add_connection_to_active_graph(&mut self, connection: Connection) -> Result<(), &'static str> {
        match self.navigation.current_view() {
            GraphView::Root => {
                self.graph.add_connection(connection)
            }
            GraphView::WorkspaceNode(workspace_node_id) => {
                if let Some(workspace_node) = self.graph.nodes.get_mut(workspace_node_id) {
                    if let Some(internal_graph) = workspace_node.get_internal_graph_mut() {
                        internal_graph.add_connection(connection)
                    } else {
                        Err("Workspace node has no internal graph")
                    }
                } else {
                    Err("Workspace node not found")
                }
            }
        }
    }
    
    /// Remove a connection from the appropriate graph based on current view
    fn remove_connection_from_active_graph(&mut self, idx: usize) {
        match self.navigation.current_view() {
            GraphView::Root => {
                self.graph.remove_connection(idx);
            }
            GraphView::WorkspaceNode(workspace_node_id) => {
                if let Some(workspace_node) = self.graph.nodes.get_mut(workspace_node_id) {
                    if let Some(internal_graph) = workspace_node.get_internal_graph_mut() {
                        internal_graph.remove_connection(idx);
                    }
                }
            }
        }
    }
    

    fn zoom_at_point(&mut self, screen_point: Pos2, zoom_delta: f32) {
        // Convert zoom delta to multiplication factor for viewport compatibility
        let zoom_factor = 1.0 + zoom_delta;
        self.canvas.zoom_at_point(screen_point, zoom_factor);
    }

    /// Handle context menu rendering and interactions
    fn handle_context_menu(&mut self, ui: &mut egui::Ui, _response: &egui::Response) {
        // Apply transforms for coordinate conversions
        let zoom = self.canvas.zoom;
        let pan_offset = self.canvas.pan_offset;

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
                self.menus.render_workspace_menu(ui, menu_screen_pos, &self.workspace_manager, &self.navigation);
            
            // Handle node creation or navigation if a node type was selected
            if let Some(node_type) = selected_node_type {
                if node_type.starts_with("SUBWORKSPACE:") {
                    // Handle subworkspace navigation
                    let workspace_name = node_type.strip_prefix("SUBWORKSPACE:").unwrap();
                    self.navigation.enter_workspace(workspace_name);
                    // Synchronize workspace manager with navigation state
                    self.workspace_manager.set_active_workspace_by_id(Some(workspace_name));
                } else {
                    // Handle regular node creation
                    self.create_node(&node_type, menu_world_pos);
                }
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
        // Delegate to WorkspaceBuilder for all node creation logic
        if let Some(node_id) = WorkspaceBuilder::create_node(
            node_type,
            position,
            &self.navigation,
            &self.workspace_manager,
            &mut self.graph,
        ) {
            // Use the actual NodeId returned from create_node instead of unreliable HashMap iteration
            let viewed_nodes = self.get_viewed_nodes();
            if let Some(node) = viewed_nodes.get(&node_id) {
                // The node should already have its panel type set by the factory
                if let Some(panel_type) = node.get_panel_type() {
                    // Set appropriate stacking defaults based on panel type
                    // IMPORTANT: Keep viewport and parameter panels completely separate
                    match panel_type {
                        crate::nodes::interface::PanelType::Viewport => {
                            // Viewport panels float by default - never stack with parameter panels
                            self.panel_manager.interface_panel_manager_mut()
                                .set_panel_stacked(node_id, false);
                            println!("🔧 Editor: Set stacking to false for viewport node {}", node_id);
                        },
                        crate::nodes::interface::PanelType::Parameter => {
                            // Parameter panels stack by default - separate from viewport panels
                            self.panel_manager.interface_panel_manager_mut()
                                .set_panel_stacked(node_id, true);
                        },
                        _ => {
                            // Other panel types stack by default
                            self.panel_manager.interface_panel_manager_mut()
                                .set_panel_stacked(node_id, true);
                        }
                    }
                    
                    // Automatically open panels for newly created nodes
                    match panel_type {
                        crate::nodes::interface::PanelType::Parameter |
                        crate::nodes::interface::PanelType::Viewport => {
                            let panel_manager = self.panel_manager.interface_panel_manager_mut();
                            panel_manager.set_panel_visibility(node_id, true);
                            panel_manager.set_panel_open(node_id, true);
                            
                            // Debug: Track panel visibility setting
                            if panel_type == crate::nodes::interface::PanelType::Viewport {
                                println!("🔧 Editor: Set viewport panel visibility for node {} to TRUE", node_id);
                                println!("🔧 Editor: Set viewport panel open for node {} to TRUE", node_id);
                                if let Some(node) = viewed_nodes.get(&node_id) {
                                    println!("🔧 Editor: Node type: {:?}", node.node_type);
                                }
                            }
                        },
                        crate::nodes::interface::PanelType::Viewer |
                        crate::nodes::interface::PanelType::Editor |
                        crate::nodes::interface::PanelType::Inspector => {
                            // Other panel types could auto-open in the future
                        }
                    }
                }
            }
            
            self.mark_modified();
            // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
        }
    }

    /// Add benchmark nodes in a grid pattern for performance testing
    fn add_benchmark_nodes(&mut self, count: usize) {
        DebugToolsManager::add_benchmark_nodes(&mut self.graph, count);
    }

    /// Add a large number of nodes with many connections for serious performance stress testing
    fn add_performance_stress_test(&mut self, count: usize) {
        DebugToolsManager::add_performance_stress_test(&mut self.graph, count);
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

    // File operations
    
    /// Create a new empty graph
    /// Create a new file (reset graph state)
    pub fn new_file(&mut self) {
        self.graph = NodeGraph::new();
        self.navigation.set_root_view();
        self.navigation = NavigationManager::new();
        self.interaction.clear_selection();
        self.file_manager.new_file();
        // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
        // Reset context manager to root (no active context)
        self.workspace_manager.set_active_workspace_by_id(None);
    }
    
    /// Save the current graph to a specific file path
    pub fn save_to_file(&mut self, file_path: &Path) -> Result<(), String> {
        self.file_manager.save_to_file(file_path, &self.graph, &self.canvas)
    }
    
    /// Load a graph from a specific file path
    pub fn load_from_file(&mut self, file_path: &Path) -> Result<(), String> {
        match self.file_manager.load_from_file(file_path) {
            Ok((graph, canvas)) => {
                self.graph = graph;
                self.canvas = canvas;
                
                // Reset view state
                self.navigation.set_root_view();
                self.navigation = NavigationManager::new();
                self.interaction.clear_selection();
                // Reset context manager to root (no active context)
                self.workspace_manager.set_active_workspace_by_id(None);
                
                // Update port positions and rebuild GPU instances
                self.graph.update_all_port_positions();
                // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                
                Ok(())
            }
            Err(error) => Err(error)
        }
    }
    
    /// Get display name for the current file
    pub fn get_file_display_name(&self) -> String {
        self.file_manager.get_file_display_name()
    }
    
    /// Check if there are unsaved changes
    pub fn has_unsaved_changes(&self) -> bool {
        self.file_manager.has_unsaved_changes()
    }
    
    /// Mark the file as modified
    pub fn mark_modified(&mut self) {
        self.file_manager.mark_modified();
    }
    
    /// Open file dialog and load selected file
    pub fn open_file_dialog(&mut self) {
        match self.file_manager.open_file_dialog() {
            Ok(Some((graph, canvas))) => {
                self.graph = graph;
                self.canvas = canvas;
                
                // Reset view state
                self.navigation.set_root_view();
                self.navigation = NavigationManager::new();
                self.interaction.clear_selection();
                // Reset context manager to root (no active context)
                self.workspace_manager.set_active_workspace_by_id(None);
                
                // Update port positions and rebuild GPU instances
                self.graph.update_all_port_positions();
                // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
            }
            Ok(None) => {
                // User cancelled - do nothing
            }
            Err(error) => {
                eprintln!("Failed to load file: {}", error);
                // TODO: Show error dialog to user
            }
        }
    }
    
    /// Save to current file path, or prompt for new path if none exists
    pub fn save_file(&mut self) {
        match self.file_manager.save_file(&self.graph, &self.canvas) {
            Ok(()) => {
                // File saved successfully
            }
            Err(_) => {
                // No current path, use save as dialog
                self.save_as_file_dialog();
            }
        }
    }
    
    /// Save as dialog
    pub fn save_as_file_dialog(&mut self) {
        match self.file_manager.save_as_file_dialog(&self.graph, &self.canvas) {
            Ok(true) => {
                // File saved successfully
            }
            Ok(false) => {
                // User cancelled - do nothing
            }
            Err(error) => {
                eprintln!("Failed to save file: {}", error);
                // TODO: Show error dialog to user
            }
        }
    }
    


    /// Render interface panels for all nodes that have visibility enabled
    fn render_interface_panels(&mut self, ui: &mut egui::Ui, viewed_nodes: &HashMap<NodeId, Node>, menu_bar_height: f32) {
        // Store menu bar height in editor state for window constraints
        self.store_menu_bar_height(menu_bar_height);
        
        // Debug: Check viewed_nodes for viewport nodes
        let viewport_nodes: Vec<_> = viewed_nodes.iter()
            .filter(|(_, node)| node.get_panel_type() == Some(crate::nodes::interface::PanelType::Viewport))
            .collect();
        if !viewport_nodes.is_empty() {
            println!("🔧 Editor: render_interface_panels found {} viewport nodes", viewport_nodes.len());
            for (&id, node) in viewport_nodes {
                println!("  - Viewport node {} '{}' visible={}", id, node.title, node.visible);
            }
        }
        
        // Delegate to the panel manager
        self.panel_manager.render_interface_panels(
            ui, 
            viewed_nodes, 
            menu_bar_height, 
            self.navigation.current_view(), 
            &mut self.graph
        );
    }

    /// Check for USD LoadStage to Viewport connections and execute automatic data flow
    fn check_and_execute_connections(&mut self, viewed_nodes: &HashMap<NodeId, Node>) {
        // Find all USD_LoadStage nodes that have output connections to Viewport nodes
        let connections = self.graph.connections.clone(); // Clone to avoid borrow conflicts
        
        for connection in &connections {
            if let (Some(source_node), Some(target_node)) = (
                viewed_nodes.get(&connection.from_node),
                viewed_nodes.get(&connection.to_node)
            ) {
                // Check if this is a USD LoadStage -> Viewport connection
                if source_node.title.contains("Load Stage") && target_node.title.contains("Viewport") {
                    // First check the LoadStage interface panel for the file path
                    let file_path = if let Some(loadstage_panel) = self.panel_manager.get_loadstage_file_path(connection.from_node) {
                        Some(loadstage_panel)
                    } else if let Some(file_path_param) = source_node.parameters.get("file_path") {
                        // Fallback to checking node parameters
                        if let crate::nodes::interface::NodeData::String(file_path) = file_path_param {
                            Some(file_path.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    
                    if let Some(file_path) = file_path {
                        if !file_path.is_empty() && std::path::Path::new(&file_path).exists() {
                            // Create stage ID from file path
                            let stage_id = format!("file://{}", file_path);
                            
                            // Log the automatic execution
                            println!("Auto-executing: USD LoadStage {} -> Viewport {} with file: {}", 
                                connection.from_node, connection.to_node, file_path);
                            
                            // Use the panel manager to auto-load USD into the viewport
                            self.panel_manager.auto_load_usd_into_viewport(connection.to_node, &stage_id);
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for NodeEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request repaint
        ctx.request_repaint();

        // Track frame time for performance monitoring
        self.debug_tools.update_frame_time();
        
        // Set dark theme for window decorations
        ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(egui::SystemTheme::Dark));

        // Render top menu bar as TopBottomPanel to ensure it's always on top with solid background
        let menu_bar_height = egui::TopBottomPanel::top("top_menu_bar")
            .frame(egui::Frame::none().fill(Color32::from_rgb(28, 28, 28)).inner_margin(8.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(4.0); // Left padding
                
                // File menu - uses EXACT same shared menu function as context menu
                let file_button_response = ui.button("File");
                if file_button_response.clicked() {
                    self.show_file_menu = !self.show_file_menu;
                }
                
                // Render file menu using EXACT same shared function
                if self.show_file_menu {
                    let menu_pos = file_button_response.rect.left_bottom();
                    let menu_items = vec![("New", false), ("Open...", false), ("Save", false), ("Save As...", false)];
                    
                    let (selected_item, menu_response) = menus::render_shared_menu(
                        ui.ctx(),
                        "file_menu",
                        menu_pos,
                        menu_items,
                        |ui, items, menu_width| {
                            for (text, _) in items {
                                if menus::render_menu_item(ui, text, menu_width) {
                                    return Some(text.to_string());
                                }
                            }
                            None
                        }
                    );
                    
                    // Handle selected item
                    if let Some(item) = selected_item {
                        match item.as_str() {
                            "New" => self.new_file(),
                            "Open..." => self.open_file_dialog(),
                            "Save" => self.save_file(),
                            "Save As..." => self.save_as_file_dialog(),
                            _ => {}
                        }
                        self.show_file_menu = false;
                    }
                    
                    // Close menu if clicked outside
                    if ui.input(|i| i.pointer.any_click()) && !menu_response.clicked() && !file_button_response.clicked() {
                        self.show_file_menu = false;
                    }
                }
                
                ui.separator();
                
                // Navigation breadcrumb bar
                let nav_action = self.navigation.render_breadcrumb(ui);
                
                // Handle navigation actions
                match nav_action {
                    NavigationAction::NavigateTo(path) => {
                        let is_root = path.is_root();
                        self.navigation.navigate_to(path);
                        
                        // Update current view based on path
                        if is_root {
                            self.navigation.set_root_view();
                            // Clear workspace stack when going to root
                            self.navigation.workspace_stack.clear();
                        } else {
                            // If navigating to a workspace path, we might need to stay in current workspace view
                            // This handles breadcrumb navigation within workspace contexts
                        }
                        
                        // Synchronize context manager with navigation state
                        let workspace_id = self.navigation.current_path.current_workspace();
                        self.workspace_manager.set_active_workspace_by_id(workspace_id);
                        self.interaction.clear_selection();
                    }
                    // All removed NavigationAction variants have been cleaned up
                    NavigationAction::None => {}
                }
                
                ui.separator();
                
                // Show current file name
                let file_name = self.get_file_display_name();
                let file_display = if self.has_unsaved_changes() {
                    format!("{}*", file_name)
                } else {
                    file_name
                };
                ui.label(egui::RichText::new(file_display).color(Color32::LIGHT_BLUE));
                
                ui.separator();
                ui.label(format!("Zoom: {:.1}x", self.canvas.zoom));
                ui.label(format!(
                    "Pan: ({:.0}, {:.0})",
                    self.canvas.pan_offset.x, self.canvas.pan_offset.y
                ));
                
                    ui.add_space(4.0); // Right padding
                });
            })
            .response
            .rect
            .height();

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(22, 27, 34)))
            .show(ctx, |ui| {
            let response = ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());
            
            // Set cursor based on special modes  
            if self.input_state.is_cutting_mode() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair); // Use crosshair for cutting mode
            } else if self.input_state.is_connecting_mode() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair); // Use crosshair for connecting mode
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

            // Apply zoom and pan transforms using canvas
            let zoom = self.canvas.zoom;
            let pan_offset = self.canvas.pan_offset;

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
                self.canvas.pan(pan_delta);
            }

            // Handle zoom with mouse wheel using input state
            if self.input_state.has_scroll_input() {
                if let Some(mouse_pos) = response.hover_pos() {
                    self.zoom_at_point(mouse_pos, self.input_state.get_zoom_delta());
                }
            }

            // Get viewed nodes/connections for all interactions
            let viewed_nodes = self.get_viewed_nodes();

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
                        // Check if we clicked on a port first - use active graph for consistency
                        let active_graph = self.navigation.get_active_graph(&self.graph);
                        // Use smaller radius for precise clicks when not in connecting mode
                        let click_radius = if self.input_state.is_connecting_mode() { 80.0 } else { 8.0 };
                        if let Some((node_id, port_idx, is_input)) = self.input_state.find_clicked_port(active_graph, click_radius) {
                            // Handle connection logic
                            if self.input_state.is_connecting_active() {
                                // Try to complete connection
                                if let Some(connection) = self.input_state.complete_connection(node_id, port_idx) {
                                    // Check if target is an input port and already has a connection
                                    if is_input {
                                        if let Some((existing_idx, _, _)) = self.input_state.find_input_connection(active_graph, node_id, port_idx) {
                                            // Remove existing connection to input port
                                            self.remove_connection_from_active_graph(existing_idx);
                                            self.mark_modified();
                                        }
                                    }
                                    let _ = self.add_connection_to_active_graph(connection);
                                    self.mark_modified();
                                    // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                                } else {
                                    // Start new connection from this port
                                    self.input_state.start_connection(node_id, port_idx, is_input);
                                    // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                                }
                            } else {
                                // Not currently connecting - check if clicking on connected input port
                                if is_input {
                                    if let Some((conn_idx, from_node, from_port)) = self.input_state.find_input_connection(active_graph, node_id, port_idx) {
                                        // Disconnect and start new connection from original source
                                        self.remove_connection_from_active_graph(conn_idx);
                                        self.mark_modified();
                                        self.input_state.start_connection(from_node, from_port, false);
                                        // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                                        return; // Skip starting connection from input port
                                    }
                                }
                                // Start new connection from this port
                                self.input_state.start_connection(node_id, port_idx, is_input);
                                // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                            }
                        } else if let Some(node_id) = self.input_state.find_node_under_mouse(&self.build_temp_graph(&viewed_nodes)) {
                            // Check for button clicks first
                            let mouse_pos = self.input_state.mouse_world_pos.unwrap_or_default();
                            let mut handled_button_click = false;
                            
                            // Get the correct graph for button interaction
                            match self.navigation.current_view() {
                                GraphView::Root => {
                                    if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                                        if node.is_point_in_left_button(mouse_pos) {
                                            node.toggle_left_button();
                                            self.mark_modified();
                                            // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                                            // Force immediate instance update instead of waiting for next frame
                                            let viewed_nodes = self.get_viewed_nodes();
                                            let mut all_selected_nodes = self.interaction.selected_nodes.clone();
                                            ui.ctx().request_repaint(); // Force immediate visual update
                                            handled_button_click = true;
                                        } else if node.is_point_in_right_button(mouse_pos) {
                                            node.toggle_right_button();
                                            self.mark_modified();
                                            // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                                            // Force immediate instance update instead of waiting for next frame
                                            let viewed_nodes = self.get_viewed_nodes();
                                            let mut all_selected_nodes = self.interaction.selected_nodes.clone();
                                            ui.ctx().request_repaint(); // Force immediate visual update
                                            handled_button_click = true;
                                        } else if node.is_point_in_visibility_flag(mouse_pos) {
                                            node.toggle_visibility();
                                            // If toggling visibility ON, make panel visible and open
                                            if node.visible {
                                                let panel_manager = self.panel_manager.interface_panel_manager_mut();
                                                panel_manager.set_panel_visibility(node_id, true);
                                                panel_manager.set_panel_open(node_id, true);
                                            }
                                            self.mark_modified();
                                            // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                                            // Force immediate instance update instead of waiting for next frame
                                            let viewed_nodes = self.get_viewed_nodes();
                                            let mut all_selected_nodes = self.interaction.selected_nodes.clone();
                                            ui.ctx().request_repaint(); // Force immediate visual update
                                            handled_button_click = true;
                                        }
                                    }
                                }
                                GraphView::WorkspaceNode(workspace_node_id) => {
                                    if let Some(workspace_node) = self.graph.nodes.get_mut(&workspace_node_id) {
                                        if let Some(internal_graph) = workspace_node.get_internal_graph_mut() {
                                            if let Some(node) = internal_graph.nodes.get_mut(&node_id) {
                                                if node.is_point_in_left_button(mouse_pos) {
                                                    node.toggle_left_button();
                                                    self.mark_modified();
                                                    // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                                                    // Force immediate instance update for context nodes
                                                    let viewed_nodes = self.get_viewed_nodes();
                                                    let mut all_selected_nodes = self.interaction.selected_nodes.clone();
                                                    ui.ctx().request_repaint(); // Force immediate visual update
                                                    handled_button_click = true;
                                                } else if node.is_point_in_right_button(mouse_pos) {
                                                    node.toggle_right_button();
                                                    self.mark_modified();
                                                    // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                                                    // Force immediate instance update for context nodes
                                                    let viewed_nodes = self.get_viewed_nodes();
                                                    let mut all_selected_nodes = self.interaction.selected_nodes.clone();
                                                    ui.ctx().request_repaint(); // Force immediate visual update
                                                    handled_button_click = true;
                                                } else if node.is_point_in_visibility_flag(mouse_pos) {
                                                    node.toggle_visibility();
                                                    // If toggling visibility ON, make panel visible and open
                                                    if node.visible {
                                                        let panel_manager = self.panel_manager.interface_panel_manager_mut();
                                                        panel_manager.set_panel_visibility(node_id, true);
                                                        panel_manager.set_panel_open(node_id, true);
                                                    }
                                                    self.mark_modified();
                                                    // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                                                    // Force immediate instance update for context nodes
                                                    let viewed_nodes = self.get_viewed_nodes();
                                                    let mut all_selected_nodes = self.interaction.selected_nodes.clone();
                                                    ui.ctx().request_repaint(); // Force immediate visual update
                                                    handled_button_click = true;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            
                            // Only handle node selection if we didn't click a button
                            if !handled_button_click {
                                // Handle node selection and double-click
                                self.interaction.select_node(node_id, self.input_state.is_multi_select());
                                
                                // Check for double-click on workspace nodes
                                if self.interaction.check_double_click(node_id) {
                                    // Check if the node exists in the active graph and is a workspace node
                                    let is_workspace_node = match self.navigation.current_view() {
                                        GraphView::Root => {
                                            self.graph.nodes.get(&node_id).map(|n| n.is_workspace()).unwrap_or(false)
                                        }
                                        GraphView::WorkspaceNode(workspace_node_id) => {
                                            if let Some(workspace_node) = self.graph.nodes.get(&workspace_node_id) {
                                                if let Some(internal_graph) = workspace_node.get_internal_graph() {
                                                    internal_graph.nodes.get(&node_id).map(|n| n.is_workspace()).unwrap_or(false)
                                                } else {
                                                    false
                                                }
                                            } else {
                                                false
                                            }
                                        }
                                    };
                                    
                                    if is_workspace_node {
                                        // Get workspace type from the node
                                        let workspace_type = match self.navigation.current_view() {
                                            GraphView::Root => {
                                                self.graph.nodes.get(&node_id).and_then(|n| n.get_workspace_type())
                                            }
                                            GraphView::WorkspaceNode(workspace_node_id) => {
                                                if let Some(workspace_node) = self.graph.nodes.get(&workspace_node_id) {
                                                    if let Some(internal_graph) = workspace_node.get_internal_graph() {
                                                        internal_graph.nodes.get(&node_id).and_then(|n| n.get_workspace_type())
                                                    } else {
                                                        None
                                                    }
                                                } else {
                                                    None
                                                }
                                            }
                                        };
                                        
                                        if let Some(workspace_type) = workspace_type {
                                                self.navigation.enter_workspace_node(node_id, workspace_type);
                                                self.navigation.set_workspace_view(node_id);
                                                // Clear selections when entering a new graph
                                                self.interaction.clear_selection();
                                                // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                                                // Synchronize workspace manager with the node's workspace type
                                                // Map workspace type to workspace ID (3D -> 3d, MaterialX -> materialx)
                                                let workspace_id = match workspace_type {
                                                    "3D" => Some("3d"),
                                                    "MaterialX" => Some("materialx"),
                                                    _ => None,
                                                };
                                                self.workspace_manager.set_active_workspace_by_id(workspace_id);
                                        }
                                    }
                                }
                                
                                // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                            }
                        } else if let Some(connection_idx) = self.input_state.find_clicked_connection(&self.build_temp_graph(&viewed_nodes), 8.0, self.canvas.zoom) {
                            // Handle connection selection with multi-select support
                            self.interaction.select_connection_multi(connection_idx, self.input_state.is_multi_select());
                            // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                        } else {
                            // Clicked on empty space - deselect all and cancel connections
                            self.interaction.clear_selection();
                            self.input_state.cancel_connection();
                            // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                        }
                    }

                    // Handle drag start for connections, node movement and box selection
                    if self.input_state.drag_started_this_frame {
                        // Check if we're starting to drag from a port for connections - use active graph for consistency
                        let active_graph = self.navigation.get_active_graph(&self.graph);
                        // Use smaller radius for precise clicks when not in connecting mode
                        let click_radius = if self.input_state.is_connecting_mode() { 80.0 } else { 8.0 };
                        if let Some((node_id, port_idx, is_input)) = self.input_state.find_clicked_port(active_graph, click_radius) {
                            // Handle input port disconnection on drag
                            if is_input {
                                if let Some((conn_idx, from_node, from_port)) = self.input_state.find_input_connection(active_graph, node_id, port_idx) {
                                    // Disconnect and start new connection from original source
                                    self.remove_connection_from_active_graph(conn_idx);
                                    self.mark_modified();
                                    self.input_state.start_connection(from_node, from_port, false);
                                    // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                                } else {
                                    // No existing connection, start from input port
                                    self.input_state.start_connection(node_id, port_idx, is_input);
                                    // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                                }
                            } else {
                                // Output port - start connection normally
                                self.input_state.start_connection(node_id, port_idx, is_input);
                                // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                            }
                        } else {
                            // Check if we're starting to drag a selected node
                            let mut dragging_selected = false;
                            let current_graph = match self.navigation.current_view() {
                                GraphView::Root => &self.graph,
                                GraphView::WorkspaceNode(node_id) => {
                                    if let Some(node) = self.graph.nodes.get(&node_id) {
                                        if let Some(internal_graph) = node.get_internal_graph() {
                                            internal_graph
                                        } else {
                                            &self.graph
                                        }
                                    } else {
                                        &self.graph
                                    }
                                }
                            };
                            
                            for &node_id in &self.interaction.selected_nodes {
                                if let Some(node) = current_graph.nodes.get(&node_id) {
                                    if node.get_rect().contains(pos) {
                                        // Start dragging selected nodes
                                        self.interaction.start_drag(pos, current_graph);
                                        dragging_selected = true;
                                        break;
                                    }
                                }
                            }
                            
                            // If not dragging selected nodes, check for clicking on any node
                            if !dragging_selected {
                                if let Some(node_id) = self.input_state.find_node_under_mouse(&self.build_temp_graph(&viewed_nodes)) {
                                    // Select the node and start dragging it
                                    self.interaction.select_node(node_id, false);
                                    self.interaction.start_drag(pos, current_graph);
                                } else {
                                    // Start box selection if not on any node and using left mouse button
                                    if self.input_state.is_primary_down(ui) {
                                        self.interaction.start_box_selection(pos);
                                        // Force GPU rebuild for immediate visual feedback
                                        // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                                    }
                                }
                            }
                        }
                    }

                    // Handle dragging
                    if response.dragged() {
                        if !self.interaction.drag_offsets.is_empty() {
                            // Drag all selected nodes - use correct graph based on current view
                            match self.navigation.current_view() {
                                GraphView::Root => {
                                    self.interaction.update_drag(pos, &mut self.graph);
                                }
                                GraphView::WorkspaceNode(node_id) => {
                                    if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                                        if let Some(internal_graph) = node.get_internal_graph_mut() {
                                            self.interaction.update_drag(pos, internal_graph);
                                        }
                                    }
                                }
                            }
                            // Force GPU instance manager to rebuild when nodes are moved
                            // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                        } else if self.interaction.box_selection_start.is_some() {
                            // Update box selection
                            self.interaction.update_box_selection(pos);
                            // Force GPU rebuild for immediate visual feedback
                            // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                        }
                    }

                    // Handle connection completion
                    if self.input_state.drag_stopped_this_frame {
                        if self.input_state.is_connecting_active() {
                            // Check if we released on a port to complete connection - use active graph for consistency
                            let active_graph = self.navigation.get_active_graph(&self.graph);
                            // Use smaller radius for precise clicks when not in connecting mode
                            let click_radius = if self.input_state.is_connecting_mode() { 80.0 } else { 8.0 };
                            if let Some((node_id, port_idx, _)) = self.input_state.find_clicked_port(active_graph, click_radius) {
                                if let Some(connection) = self.input_state.complete_connection(node_id, port_idx) {
                                    let _ = self.add_connection_to_active_graph(connection);
                                    self.mark_modified();
                                }
                            } else {
                                // Cancel connection if we didn't release on a port
                                self.input_state.cancel_connection();
                            }
                            // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                        }
                    }
                }

                if self.input_state.drag_stopped_this_frame {
                    // Ensure final positions are updated in GPU
                    if self.use_gpu_rendering {
                        // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                    }

                    // Complete box selection
                    if self.interaction.box_selection_start.is_some() {
                        match self.navigation.current_view() {
                            GraphView::Root => {
                                self.interaction.complete_box_selection(&self.graph, self.input_state.is_multi_select());
                            }
                            GraphView::WorkspaceNode(node_id) => {
                                if let Some(node) = self.graph.nodes.get(&node_id) {
                                    if let Some(internal_graph) = node.get_internal_graph() {
                                        self.interaction.complete_box_selection(internal_graph, self.input_state.is_multi_select());
                                    }
                                }
                            }
                        }
                        // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                    }
                    
                    // End any dragging operations
                    self.interaction.end_drag();
                }
            }

            // Handle keyboard input using input state
            if self.input_state.delete_pressed(ui) {
                if !self.interaction.selected_nodes.is_empty() {
                    // Delete all selected nodes from the correct graph
                    match self.navigation.current_view() {
                        GraphView::Root => {
                            self.interaction.delete_selected(&mut self.graph);
                        }
                        GraphView::WorkspaceNode(node_id) => {
                            if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                                if let Some(internal_graph) = node.get_internal_graph_mut() {
                                    self.interaction.delete_selected(internal_graph);
                                }
                            }
                        }
                    }
                    self.mark_modified();
                    // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                } else if !self.interaction.selected_connections.is_empty() {
                    // Delete all selected connections (in reverse order to maintain indices)
                    let mut connection_indices: Vec<usize> = self.interaction.selected_connections.iter().copied().collect();
                    connection_indices.sort_by(|a, b| b.cmp(a)); // Sort in reverse order
                    
                    match self.navigation.current_view() {
                        GraphView::Root => {
                            for conn_idx in connection_indices {
                                self.graph.remove_connection(conn_idx);
                                self.mark_modified();
                            }
                        }
                        GraphView::WorkspaceNode(node_id) => {
                            if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                                if let Some(internal_graph) = node.get_internal_graph_mut() {
                                    for conn_idx in connection_indices {
                                        internal_graph.remove_connection(conn_idx);
                                    }
                                }
                            }
                            self.mark_modified();
                        }
                    }
                    
                    self.interaction.clear_connection_selection();
                    // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                }
            }

            // Handle ESC key to cancel connections
            if self.input_state.escape_pressed(ui) {
                self.input_state.cancel_connection();
                // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
            }

            // Update port positions BEFORE connection handling
            self.graph.update_all_port_positions();
            
            // Handle connection cutting when X key is released
            if !self.input_state.is_cutting_mode() && (!self.input_state.get_cut_paths().is_empty() || !self.input_state.get_current_cut_path().is_empty()) {
                // X key was just released - apply cuts
                let cut_connections = {
                    let active_graph = self.navigation.get_active_graph(&self.graph);
                    self.input_state.find_cut_connections(active_graph, self.canvas.zoom)
                };
                
                if !cut_connections.is_empty() {
                    // Sort in reverse order to maintain indices during deletion
                    let mut sorted_cuts = cut_connections;
                    sorted_cuts.sort_by(|a, b| b.cmp(a));
                    
                    for conn_idx in sorted_cuts {
                        self.remove_connection_from_active_graph(conn_idx);
                        self.mark_modified();
                    }
                    
                    // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                }
                
                // Clear cut paths after applying
                self.input_state.clear_cut_paths();
            }

            // Handle connection drawing when C key is released
            if !self.input_state.is_connecting_mode() && (!self.input_state.get_connect_paths().is_empty() || !self.input_state.get_current_connect_path().is_empty()) {
                // C key was just released - create connections from drawn paths
                let (new_connections, connections_to_remove) = {
                    let active_graph = self.navigation.get_active_graph(&self.graph);
                    let new_connections = self.input_state.create_connections_from_paths(active_graph);
                    
                    // Process connections to find existing ones to remove
                    let mut connections_to_remove = Vec::new();
                    for connection in &new_connections {
                        if let Some((existing_idx, _, _)) = self.input_state.find_input_connection(active_graph, connection.to_node, connection.to_port) {
                            connections_to_remove.push(existing_idx);
                        }
                    }
                    
                    (new_connections, connections_to_remove)
                };
                
                if !new_connections.is_empty() {
                    // Remove existing connections
                    for existing_idx in connections_to_remove {
                        self.remove_connection_from_active_graph(existing_idx);
                        self.mark_modified();
                    }
                    
                    // Add new connections
                    for connection in new_connections {
                        let _ = self.add_connection_to_active_graph(connection);
                        self.mark_modified();
                    }
                    
                    // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                }
                
                // Clear connect paths after applying
                self.input_state.clear_connect_paths();
            }

            // Handle F1 to toggle performance info
            if self.input_state.f1_pressed(ui) {
                self.debug_tools.toggle_performance_info();
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
                // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
            }

            // Handle F6 to toggle GPU/CPU rendering
            if self.input_state.f6_pressed(ui) {
                self.use_gpu_rendering = !self.use_gpu_rendering;
            }

            // Handle right-click for context menu first (before other input handling)
            if self.input_state.right_clicked_this_frame {
                if let Some(node_id) = self.input_state.find_node_under_mouse(&self.build_temp_graph(&viewed_nodes)) {
                    // Right-clicked on a node - select it
                    self.interaction.select_node(node_id, false);
                } else {
                    // Right-clicked on empty space - context menu is handled in InputState update
                    // (context_menu_pos is automatically set)
                }
            }



            // Draw nodes - GPU vs CPU rendering
            if self.use_gpu_rendering && !viewed_nodes.is_empty() {
                    // Calculate viewport bounds for GPU callback
                    let viewport_rect = response.rect;
                    
                    // Create GPU callback for node body rendering  
                    // Screen size in logical coordinates using full screen height
                    let screen_size = Vec2::new(
                        ui.ctx().screen_rect().width(),
                        ui.ctx().screen_rect().height()
                    );
                    
                    // Get current graph for box selection preview
                    let current_graph = self.navigation.get_active_graph(&self.graph);
                    
                    // Combine selected nodes with box selection preview for immediate highlighting
                    let mut all_selected_nodes = self.interaction.selected_nodes.clone();
                    let box_preview_nodes = self.interaction.get_box_selection_preview(current_graph);
                    for node_id in box_preview_nodes {
                        all_selected_nodes.insert(node_id);
                    }
                    
                    // Use persistent instance manager for optimal performance
                    let (node_instances, port_instances, button_instances, flag_instances) = self.gpu_instance_manager.update_instances(
                        &viewed_nodes,
                        &all_selected_nodes,
                        self.input_state.get_connecting_from(),
                        &self.input_state,
                        &self.build_temp_graph(&viewed_nodes),
                    );
                    
                    let gpu_callback = NodeRenderCallback::from_instances(
                        node_instances,
                        port_instances,
                        button_instances,
                        flag_instances,
                        self.canvas.get_gpu_pan_offset(self.current_menu_bar_height),
                        self.canvas.zoom,
                        screen_size,
                    );
                    
                    // Add the GPU paint callback using egui_wgpu::Callback - this will trigger prepare() and paint() methods
                    painter.add(egui_wgpu::Callback::new_paint_callback(
                        viewport_rect,
                        gpu_callback,
                    ));
                    
                    // Render node titles using CPU (GPU handles node bodies and ports)
                    for (_node_id, node) in &viewed_nodes {
                    // Node titles (CPU-rendered text)
                    painter.text(
                        transform_pos(node.position + Vec2::new(node.size.x / 2.0, 15.0)),
                        egui::Align2::CENTER_CENTER,
                        &node.title,
                        egui::FontId::proportional(12.0 * self.canvas.zoom),
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
                                    egui::FontId::proportional(10.0 * self.canvas.zoom),
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
                                    egui::FontId::proportional(10.0 * self.canvas.zoom),
                                    Color32::WHITE,
                                );
                            }
                        }
                    }
                }
                
                // Visibility flags are now rendered by GPU shader
                
            } else if !viewed_nodes.is_empty() {
                // CPU rendering path - fallback mode using MeshRenderer
                
                // Get current graph for box selection preview
                let current_graph = self.navigation.get_active_graph(&self.graph);
                
                // Get box selection preview nodes for immediate highlighting
                let box_preview_nodes = self.interaction.get_box_selection_preview(current_graph);
                
                for (node_id, node) in &viewed_nodes {
                    let is_selected = self.interaction.selected_nodes.contains(&node_id) || 
                                    box_preview_nodes.contains(&node_id);
                    
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
                            // Check for start port preview (before drawing begins)
                            if self.input_state.get_current_connect_path().is_empty() {
                                if let Some((start_node, start_port, start_is_input)) = self.input_state.get_connection_start_preview(&self.build_temp_graph(&viewed_nodes)) {
                                    if start_node == *node_id && start_port == port_idx && start_is_input {
                                        is_connecting_port = true;
                                    }
                                }
                            } else {
                                // Check for completed connection preview (while drawing)
                                if let Some(((start_node, start_port, start_is_input), (end_node, end_port, end_is_input))) = self.input_state.get_connection_preview(&self.build_temp_graph(&viewed_nodes)) {
                                    if (start_node == *node_id && start_port == port_idx && start_is_input) ||
                                       (end_node == *node_id && end_port == port_idx && end_is_input) {
                                        is_connecting_port = true;
                                    }
                                }
                                // Also check for end port preview (current mouse position)
                                if !is_connecting_port {
                                    if let Some((end_node, end_port, end_is_input)) = self.input_state.get_connection_end_preview(&self.build_temp_graph(&viewed_nodes)) {
                                        if end_node == *node_id && end_port == port_idx && end_is_input {
                                            is_connecting_port = true;
                                        }
                                    }
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
                            // Check for start port preview (before drawing begins)
                            if self.input_state.get_current_connect_path().is_empty() {
                                if let Some((start_node, start_port, start_is_input)) = self.input_state.get_connection_start_preview(&self.build_temp_graph(&viewed_nodes)) {
                                    if start_node == *node_id && start_port == port_idx && !start_is_input {
                                        is_connecting_port = true;
                                    }
                                }
                            } else {
                                // Check for completed connection preview (while drawing)
                                if let Some(((start_node, start_port, start_is_input), (end_node, end_port, end_is_input))) = self.input_state.get_connection_preview(&self.build_temp_graph(&viewed_nodes)) {
                                    if (start_node == *node_id && start_port == port_idx && !start_is_input) ||
                                       (end_node == *node_id && end_port == port_idx && !end_is_input) {
                                        is_connecting_port = true;
                                    }
                                }
                                // Also check for end port preview (current mouse position)
                                if !is_connecting_port {
                                    if let Some((end_node, end_port, end_is_input)) = self.input_state.get_connection_end_preview(&self.build_temp_graph(&viewed_nodes)) {
                                        if end_node == *node_id && end_port == port_idx && !end_is_input {
                                            is_connecting_port = true;
                                        }
                                    }
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
                // Render visibility toggle outlines and dots (CPU mode)
                for (_node_id, node) in &viewed_nodes {
                    let flag_pos = transform_pos(node.get_flag_position());
                    
                    // Draw border outline (outer layer) - blue if enabled, grey if disabled
                    let border_color = if node.visible {
                        Color32::from_rgb(100, 150, 255) // Blue selection color when enabled
                    } else {
                        Color32::from_rgb(64, 64, 64) // Grey when disabled
                    };
                    
                    let border_radius = 5.0 * zoom;
                    painter.circle_stroke(
                        flag_pos,
                        border_radius,
                        Stroke::new(1.0 * zoom, border_color),
                    );
                    
                    // Draw bevel outline (inner layer) - 1px smaller than border
                    let bevel_radius = 4.0 * zoom;
                    painter.circle_stroke(
                        flag_pos,
                        bevel_radius,
                        Stroke::new(1.0 * zoom, Color32::from_rgb(38, 38, 38)), // Bevel outline
                    );
                    
                    // Add bigger dot for visible nodes only
                    if node.visible {
                        let dot_radius = 3.5 * zoom; // Bigger solid dot
                        painter.circle_filled(
                            flag_pos,
                            dot_radius,
                            Color32::from_rgb(100, 150, 255), // Same blue as border highlight
                        );
                    }
                }
            } // End of CPU rendering mode

            // Draw connections
            let viewed_connections = self.get_viewed_connections();
            for (idx, connection) in viewed_connections.iter().enumerate() {
                if let (Some(from_node), Some(to_node)) = (
                    viewed_nodes.get(&connection.from_node),
                    viewed_nodes.get(&connection.to_node),
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

                        // Draw bezier curve with handle length proportional to total distance
                        let total_distance = (transformed_to - transformed_from).length();
                        let control_offset = total_distance.sqrt() * 4.0;

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
                    if let Some(node) = viewed_nodes.get(&from_node) {
                        let from_pos = if from_is_input {
                            node.inputs[from_port].position
                        } else {
                            node.outputs[from_port].position
                        };

                        let transformed_from = transform_pos(from_pos);
                        let transformed_to = mouse_pos;

                        // Draw bezier curve for connection preview with proportional handle length
                        let total_distance = (transformed_to - transformed_from).length();
                        let control_offset = total_distance.sqrt() * 4.0;

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

            // Interface panel rendering - render panels for nodes that have them
            self.render_interface_panels(ui, &viewed_nodes, menu_bar_height);

            // Connection-based execution - check for USD LoadStage to Viewport connections
            self.check_and_execute_connections(&viewed_nodes);

            // Performance info overlay
            self.debug_tools.render_performance_info(ui, self.use_gpu_rendering, self.graph.nodes.len(), self.current_menu_bar_height);
        });
    }

}