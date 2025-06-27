//! Node editor implementation

// Module declarations
pub mod viewport;
pub mod input;
pub mod interaction;
pub mod menus;
pub mod rendering;
pub mod navigation;

// Re-exports
pub use viewport::Viewport;
pub use input::InputState;
pub use interaction::InteractionManager;
pub use menus::MenuManager;
pub use rendering::MeshRenderer;
pub use navigation::{NavigationManager, WorkspacePath, NavigationAction};

use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2, Shadow};
use egui_wgpu;
use crate::nodes::{
    NodeGraph, Node, NodeId, Connection,
};
use std::collections::HashMap;
use crate::workspace::WorkspaceManager;
use crate::workspaces::WorkspaceRegistry;
use crate::gpu::{NodeRenderCallback, FlagInstanceData};
use crate::gpu::GpuInstanceManager;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Save file data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub version: String,
    pub metadata: SaveMetadata,
    pub viewport: ViewportData,
    pub root_graph: NodeGraph,
}

/// Metadata for save files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMetadata {
    pub created: String,    // ISO 8601 timestamp
    pub modified: String,   // ISO 8601 timestamp
    pub creator: String,    // "Nōdle 1.0"
    pub description: String,
}

/// Viewport state for save files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportData {
    pub pan_offset: [f32; 2],
    pub zoom: f32,
}

/// Main application state for the node editor
pub struct NodeEditor {
    graph: NodeGraph,
    viewport: Viewport,
    input_state: InputState,      // Centralized input handling
    interaction: InteractionManager, // Node selection and dragging
    menus: MenuManager,           // Context menu management
    navigation: NavigationManager, // Workspace navigation and breadcrumbs
    workspace_manager: WorkspaceManager,
    // Performance tracking
    show_performance_info: bool,
    frame_times: Vec<f32>,
    last_frame_time: std::time::Instant,
    // GPU rendering toggle
    use_gpu_rendering: bool,
    // Persistent GPU instance manager
    gpu_instance_manager: GpuInstanceManager,
    // Current view state - which graph we're looking at
    current_view: GraphView,
    // File management
    current_file_path: Option<std::path::PathBuf>,
    is_modified: bool,
    // Menu state
    show_file_menu: bool,
}

/// Tracks which graph we're currently viewing
#[derive(Debug, Clone)]
enum GraphView {
    /// Viewing the root graph
    Root,
    /// Viewing a workspace node's internal graph
    WorkspaceNode(NodeId),
}

impl NodeEditor {
    pub fn new() -> Self {
        // Use the workspace registry to create a manager with all available workspaces
        let workspace_manager = WorkspaceRegistry::create_workspace_manager();
        
        let editor = Self {
            graph: NodeGraph::new(),
            viewport: Viewport::new(),
            input_state: InputState::new(),
            interaction: InteractionManager::new(),
            menus: MenuManager::new(),
            navigation: NavigationManager::new(),
            workspace_manager,
            // Performance tracking
            show_performance_info: false,
            frame_times: Vec::new(),
            last_frame_time: std::time::Instant::now(),
            // GPU rendering
            use_gpu_rendering: true, // Start with GPU rendering enabled
            // Persistent GPU instance manager
            gpu_instance_manager: GpuInstanceManager::new(),
            // Start viewing the root graph
            current_view: GraphView::Root,
            // File management
            current_file_path: None,
            is_modified: false,
            // Menu state
            show_file_menu: false,
        };

        // Start with empty node graph - use F2/F3/F4 to add test nodes

        editor
    }
    
    /// Get the nodes to render based on current view
    fn get_viewed_nodes(&self) -> HashMap<NodeId, Node> {
        match &self.current_view {
            GraphView::Root => self.graph.nodes.clone(),
            GraphView::WorkspaceNode(node_id) => {
                if let Some(node) = self.graph.nodes.get(node_id) {
                    if let Some(internal_graph) = node.get_internal_graph() {
                        return internal_graph.nodes.clone();
                    }
                }
                // Fallback to empty if node not found
                HashMap::new()
            }
        }
    }
    
    /// Get the connections to render based on current view
    fn get_viewed_connections(&self) -> Vec<Connection> {
        match &self.current_view {
            GraphView::Root => self.graph.connections.clone(),
            GraphView::WorkspaceNode(node_id) => {
                if let Some(node) = self.graph.nodes.get(node_id) {
                    if let Some(internal_graph) = node.get_internal_graph() {
                        return internal_graph.connections.clone();
                    }
                }
                // Fallback to empty if node not found
                Vec::new()
            }
        }
    }
    
    /// Build a temporary graph for GPU processing
    fn build_temp_graph(&self, nodes: &HashMap<NodeId, Node>) -> NodeGraph {
        let mut temp_graph = NodeGraph::new();
        temp_graph.nodes = nodes.clone();
        temp_graph.connections = self.get_viewed_connections();
        temp_graph
    }
    
    /// Get mutable reference to a workspace node's internal graph
    fn get_workspace_graph_mut(&mut self, node_id: NodeId) -> Option<&mut NodeGraph> {
        if let Some(node) = self.graph.nodes.get_mut(&node_id) {
            node.get_internal_graph_mut()
        } else {
            None
        }
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
                self.menus.render_workspace_menu(ui, menu_screen_pos, &self.workspace_manager, &self.navigation);
            
            // Handle node creation or navigation if a node type was selected
            if let Some(node_type) = selected_node_type {
                if node_type.starts_with("SUBWORKSPACE:") {
                    // Handle subworkspace navigation
                    let workspace_name = node_type.strip_prefix("SUBWORKSPACE:").unwrap();
                    self.navigation.navigate_to_workspace(workspace_name);
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
        // Check if this is a workspace node creation
        if node_type == "WORKSPACE:2D" {
            let mut workspace_node = Node::new_workspace(0, "2D", position);
            
            // Add some sample nodes to the 2D workspace for demonstration
            self.populate_2d_workspace(&mut workspace_node);
            
            // Workspace nodes can only be created in the root graph
            if matches!(self.current_view, GraphView::Root) {
                self.graph.add_node(workspace_node);
                self.mark_modified();
            } else if let GraphView::WorkspaceNode(node_id) = self.current_view {
                // Add to the workspace node's internal graph
                if let Some(internal_graph) = self.get_workspace_graph_mut(node_id) {
                    internal_graph.add_node(workspace_node);
                    self.mark_modified();
                }
            }
            // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
            return;
        }
        
        if node_type == "WORKSPACE:3D" {
            let mut workspace_node = Node::new_workspace(0, "3D", position);
            
            // Add some sample nodes to the 3D workspace for demonstration
            self.populate_3d_workspace(&mut workspace_node);
            
            // Workspace nodes can only be created in the root graph
            if matches!(self.current_view, GraphView::Root) {
                self.graph.add_node(workspace_node);
                self.mark_modified();
            } else if let GraphView::WorkspaceNode(node_id) = self.current_view {
                // Add to the workspace node's internal graph
                if let Some(internal_graph) = self.get_workspace_graph_mut(node_id) {
                    internal_graph.add_node(workspace_node);
                    self.mark_modified();
                }
            }
            // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
            return;
        }
        
        if node_type == "WORKSPACE:MaterialX" {
            let mut workspace_node = Node::new_workspace(0, "MaterialX", position);
            
            // Add some sample nodes to the MaterialX workspace for demonstration
            self.populate_materialx_workspace(&mut workspace_node);
            
            // Workspace nodes can only be created in the root graph
            if matches!(self.current_view, GraphView::Root) {
                self.graph.add_node(workspace_node);
                self.mark_modified();
            } else if let GraphView::WorkspaceNode(node_id) = self.current_view {
                // Add to the workspace node's internal graph
                if let Some(internal_graph) = self.get_workspace_graph_mut(node_id) {
                    internal_graph.add_node(workspace_node);
                    self.mark_modified();
                }
            }
            // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
            return;
        }
        
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
        
        // Create the node
        let new_node = if let Some(workspace) = self.workspace_manager.get_active_workspace() {
            crate::NodeRegistry::create_workspace_node(workspace, internal_node_type, position)
        } else {
            None
        }.or_else(|| crate::NodeRegistry::create_node(internal_node_type, position));
        
        // Add the node to the appropriate graph
        if let Some(node) = new_node {
            match self.current_view {
                GraphView::Root => {
                    self.graph.add_node(node);
                    self.mark_modified();
                }
                GraphView::WorkspaceNode(node_id) => {
                    if let Some(internal_graph) = self.get_workspace_graph_mut(node_id) {
                        internal_graph.add_node(node);
                        self.mark_modified();
                    }
                }
            }
            // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
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
                // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
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
        // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
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
    pub fn new_file(&mut self) {
        self.graph = NodeGraph::new();
        self.current_view = GraphView::Root;
        self.navigation = NavigationManager::new();
        self.interaction.clear_selection();
        self.current_file_path = None;
        self.is_modified = false;
        // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
        // Reset context manager to root (no active context)
        self.workspace_manager.set_active_workspace_by_id(None);
    }
    
    /// Save the current graph to a file
    pub fn save_to_file(&mut self, file_path: &Path) -> Result<(), String> {
        // Create the save data structure
        let save_data = SaveData {
            version: "1.0".to_string(),
            metadata: SaveMetadata {
                created: chrono::Utc::now().to_rfc3339(),
                modified: chrono::Utc::now().to_rfc3339(),
                creator: "Nōdle 1.0".to_string(),
                description: "Node graph project".to_string(),
            },
            viewport: ViewportData {
                pan_offset: [self.viewport.pan_offset.x, self.viewport.pan_offset.y],
                zoom: self.viewport.zoom,
            },
            root_graph: self.graph.clone(),
        };
        
        // Serialize to JSON
        let json_data = serde_json::to_string_pretty(&save_data)
            .map_err(|e| format!("Failed to serialize graph: {}", e))?;
        
        // Write to file
        std::fs::write(file_path, json_data)
            .map_err(|e| format!("Failed to write file: {}", e))?;
        
        // Update state
        self.current_file_path = Some(file_path.to_path_buf());
        self.is_modified = false;
        
        Ok(())
    }
    
    /// Load a graph from a file
    pub fn load_from_file(&mut self, file_path: &Path) -> Result<(), String> {
        // Read file
        let file_content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        // Deserialize from JSON
        let save_data: SaveData = serde_json::from_str(&file_content)
            .map_err(|e| format!("Failed to parse file: {}", e))?;
        
        // Apply loaded data
        self.graph = save_data.root_graph;
        self.viewport.pan_offset = Vec2::new(
            save_data.viewport.pan_offset[0],
            save_data.viewport.pan_offset[1],
        );
        self.viewport.zoom = save_data.viewport.zoom;
        
        // Reset view state
        self.current_view = GraphView::Root;
        self.navigation = NavigationManager::new();
        self.interaction.clear_selection();
        // Reset context manager to root (no active context)
        self.workspace_manager.set_active_workspace_by_id(None);
        
        // Update file state
        self.current_file_path = Some(file_path.to_path_buf());
        self.is_modified = false;
        
        // Update port positions and rebuild GPU instances
        self.graph.update_all_port_positions();
        // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
        
        Ok(())
    }
    
    /// Get the current file name for display
    pub fn get_file_display_name(&self) -> String {
        if let Some(path) = &self.current_file_path {
            if let Some(name) = path.file_name() {
                return name.to_string_lossy().to_string();
            }
        }
        "Untitled".to_string()
    }
    
    /// Check if the current file has unsaved changes
    pub fn has_unsaved_changes(&self) -> bool {
        self.is_modified
    }
    
    /// Mark the current file as modified
    pub fn mark_modified(&mut self) {
        self.is_modified = true;
    }
    
    /// Open file dialog and load selected file
    pub fn open_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Nōdle files", &["nodle"])
            .add_filter("JSON files", &["json"])
            .pick_file()
        {
            if let Err(error) = self.load_from_file(&path) {
                eprintln!("Failed to load file: {}", error);
                // TODO: Show error dialog to user
            }
        }
    }
    
    /// Save current file, or open Save As dialog if no current file
    pub fn save_file(&mut self) {
        if let Some(current_path) = self.current_file_path.clone() {
            if let Err(error) = self.save_to_file(&current_path) {
                eprintln!("Failed to save file: {}", error);
                // TODO: Show error dialog to user
            }
        } else {
            self.save_as_file_dialog();
        }
    }
    
    /// Open Save As dialog and save to selected file
    pub fn save_as_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Nōdle files", &["nodle"])
            .add_filter("JSON files", &["json"])
            .set_file_name("untitled.nodle")
            .save_file()
        {
            if let Err(error) = self.save_to_file(&path) {
                eprintln!("Failed to save file: {}", error);
                // TODO: Show error dialog to user
            }
        }
    }
    
    /// Populate a 2D workspace node with sample nodes for demonstration
    fn populate_2d_workspace(&mut self, workspace_node: &mut Node) {
        if let Some(_internal_graph) = workspace_node.get_internal_graph_mut() {
            // 2D workspace starts empty - users can add nodes via context menu
        }
    }
    
    /// Populate a 3D workspace node with sample nodes for demonstration
    fn populate_3d_workspace(&mut self, workspace_node: &mut Node) {
        if let Some(_internal_graph) = workspace_node.get_internal_graph_mut() {
            // 3D context starts empty - users can add nodes via context menu
        }
    }
    
    /// Populate a MaterialX context node with sample nodes for demonstration
    fn populate_materialx_workspace(&mut self, workspace_node: &mut Node) {
        if let Some(internal_graph) = workspace_node.get_internal_graph_mut() {
            // Create sample MaterialX nodes
            let mut image_node = Node::new(1, "Image", Pos2::new(50.0, 100.0))
                .with_color(Color32::from_rgb(140, 180, 140));
            image_node.add_input("File");
            image_node.add_input("UV");
            image_node.add_output("Color");
            image_node.add_output("Alpha");
            
            let mut noise_node = Node::new(2, "Noise", Pos2::new(50.0, 200.0))
                .with_color(Color32::from_rgb(140, 180, 140));
            noise_node.add_input("UV");
            noise_node.add_input("Scale");
            noise_node.add_output("Color");
            
            let mut mix_node = Node::new(3, "Mix", Pos2::new(300.0, 150.0))
                .with_color(Color32::from_rgb(200, 150, 100));
            mix_node.add_input("Input A");
            mix_node.add_input("Input B");
            mix_node.add_input("Mix Factor");
            mix_node.add_output("Output");
            
            let mut surface_node = Node::new(4, "Standard Surface", Pos2::new(500.0, 150.0))
                .with_color(Color32::from_rgb(180, 140, 100));
            surface_node.add_input("Base Color");
            surface_node.add_input("Roughness");
            surface_node.add_input("Metallic");
            surface_node.add_output("Surface");
            
            // Add nodes to internal graph
            internal_graph.add_node(image_node);
            internal_graph.add_node(noise_node);
            internal_graph.add_node(mix_node);
            internal_graph.add_node(surface_node);
            
            // Create sample connections
            let connection1 = Connection::new(1, 0, 3, 0); // image color to mix input A
            let connection2 = Connection::new(2, 0, 3, 1); // noise color to mix input B
            let connection3 = Connection::new(3, 0, 4, 0); // mix output to surface base color
            
            let _ = internal_graph.add_connection(connection1);
            let _ = internal_graph.add_connection(connection2);
            let _ = internal_graph.add_connection(connection3);
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
                        self.navigation.navigate_to(path);
                        // Synchronize context manager with navigation state
                        let workspace_id = self.navigation.current_path.current_workspace();
                        self.workspace_manager.set_active_workspace_by_id(workspace_id);
                    }
                    NavigationAction::EnterWorkspace(workspace_name) => {
                        self.navigation.enter_workspace(&workspace_name);
                        // Synchronize context manager with navigation state
                        let workspace_id = self.navigation.current_path.current_workspace();
                        self.workspace_manager.set_active_workspace_by_id(workspace_id);
                    }
                    NavigationAction::GoUp => {
                        // Exit from context node view
                        self.current_view = GraphView::Root;
                        self.interaction.clear_selection();
                        // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                        // When going up, clear the active context (back to root)
                        self.workspace_manager.set_active_workspace_by_id(None);
                    }
                    NavigationAction::GoToRoot => {
                        self.navigation.go_to_root();
                        // Synchronize context manager with navigation state (root = no active context)
                        self.workspace_manager.set_active_workspace_by_id(None);
                    }
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
                        // Check if we clicked on a port first
                        if let Some((node_id, port_idx, is_input)) = self.input_state.find_clicked_port(&self.build_temp_graph(&viewed_nodes), 10.0) {
                            // Handle connection logic
                            if self.input_state.is_connecting_active() {
                                // Try to complete connection
                                if let Some(connection) = self.input_state.complete_connection(node_id, port_idx) {
                                    // Check if target is an input port and already has a connection
                                    if is_input {
                                        if let Some((existing_idx, _, _)) = self.input_state.find_input_connection(&self.graph, node_id, port_idx) {
                                            // Remove existing connection to input port
                                            self.graph.remove_connection(existing_idx);
                                            self.mark_modified();
                                        }
                                    }
                                    let _ = self.graph.add_connection(connection);
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
                                    if let Some((conn_idx, from_node, from_port)) = self.input_state.find_input_connection(&self.graph, node_id, port_idx) {
                                        // Disconnect and start new connection from original source
                                        self.graph.remove_connection(conn_idx);
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
                            match self.current_view {
                                GraphView::Root => {
                                    if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                                        if node.is_point_in_left_button(mouse_pos) {
                                            node.toggle_left_button();
                                            self.mark_modified();
                                            // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                                            // Force immediate instance update instead of waiting for next frame
                                            let viewed_nodes = match self.current_view {
                                                GraphView::Root => self.graph.nodes.clone(),
                                                GraphView::WorkspaceNode(workspace_node_id) => {
                                                    if let Some(workspace_node) = self.graph.nodes.get(&workspace_node_id) {
                                                        if let Some(internal_graph) = workspace_node.get_internal_graph() {
                                                            internal_graph.nodes.clone()
                                                        } else { HashMap::new() }
                                                    } else { HashMap::new() }
                                                }
                                            };
                                            let mut all_selected_nodes = self.interaction.selected_nodes.clone();
                                            ui.ctx().request_repaint(); // Force immediate visual update
                                            handled_button_click = true;
                                        } else if node.is_point_in_right_button(mouse_pos) {
                                            node.toggle_right_button();
                                            self.mark_modified();
                                            // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                                            // Force immediate instance update instead of waiting for next frame
                                            let viewed_nodes = match self.current_view {
                                                GraphView::Root => self.graph.nodes.clone(),
                                                GraphView::WorkspaceNode(workspace_node_id) => {
                                                    if let Some(workspace_node) = self.graph.nodes.get(&workspace_node_id) {
                                                        if let Some(internal_graph) = workspace_node.get_internal_graph() {
                                                            internal_graph.nodes.clone()
                                                        } else { HashMap::new() }
                                                    } else { HashMap::new() }
                                                }
                                            };
                                            let mut all_selected_nodes = self.interaction.selected_nodes.clone();
                                            ui.ctx().request_repaint(); // Force immediate visual update
                                            handled_button_click = true;
                                        } else if node.is_point_in_visibility_flag(mouse_pos) {
                                            node.toggle_visibility();
                                            self.mark_modified();
                                            // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                                            // Force immediate instance update instead of waiting for next frame
                                            let viewed_nodes = match self.current_view {
                                                GraphView::Root => self.graph.nodes.clone(),
                                                GraphView::WorkspaceNode(workspace_node_id) => {
                                                    if let Some(workspace_node) = self.graph.nodes.get(&workspace_node_id) {
                                                        if let Some(internal_graph) = workspace_node.get_internal_graph() {
                                                            internal_graph.nodes.clone()
                                                        } else { HashMap::new() }
                                                    } else { HashMap::new() }
                                                }
                                            };
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
                                                    let viewed_nodes = match self.current_view {
                                                        GraphView::Root => self.graph.nodes.clone(),
                                                        GraphView::WorkspaceNode(workspace_node_id) => {
                                                            if let Some(workspace_node) = self.graph.nodes.get(&workspace_node_id) {
                                                                if let Some(internal_graph) = workspace_node.get_internal_graph() {
                                                                    internal_graph.nodes.clone()
                                                                } else { HashMap::new() }
                                                            } else { HashMap::new() }
                                                        }
                                                    };
                                                    let mut all_selected_nodes = self.interaction.selected_nodes.clone();
                                                    ui.ctx().request_repaint(); // Force immediate visual update
                                                    handled_button_click = true;
                                                } else if node.is_point_in_right_button(mouse_pos) {
                                                    node.toggle_right_button();
                                                    self.mark_modified();
                                                    // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                                                    // Force immediate instance update for context nodes
                                                    let viewed_nodes = match self.current_view {
                                                        GraphView::Root => self.graph.nodes.clone(),
                                                        GraphView::WorkspaceNode(workspace_node_id) => {
                                                            if let Some(workspace_node) = self.graph.nodes.get(&workspace_node_id) {
                                                                if let Some(internal_graph) = workspace_node.get_internal_graph() {
                                                                    internal_graph.nodes.clone()
                                                                } else { HashMap::new() }
                                                            } else { HashMap::new() }
                                                        }
                                                    };
                                                    let mut all_selected_nodes = self.interaction.selected_nodes.clone();
                                                    ui.ctx().request_repaint(); // Force immediate visual update
                                                    handled_button_click = true;
                                                } else if node.is_point_in_visibility_flag(mouse_pos) {
                                                    node.toggle_visibility();
                                                    self.mark_modified();
                                                    // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                                                    // Force immediate instance update for context nodes
                                                    let viewed_nodes = match self.current_view {
                                                        GraphView::Root => self.graph.nodes.clone(),
                                                        GraphView::WorkspaceNode(workspace_node_id) => {
                                                            if let Some(workspace_node) = self.graph.nodes.get(&workspace_node_id) {
                                                                if let Some(internal_graph) = workspace_node.get_internal_graph() {
                                                                    internal_graph.nodes.clone()
                                                                } else { HashMap::new() }
                                                            } else { HashMap::new() }
                                                        }
                                                    };
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
                                    if let Some(node) = self.graph.nodes.get(&node_id) {
                                        if node.is_workspace() {
                                            // Navigate into the workspace node
                                            if let Some(workspace_type) = node.get_workspace_type() {
                                                self.navigation.enter_workspace_node(node_id, workspace_type);
                                                self.current_view = GraphView::WorkspaceNode(node_id);
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
                                }
                                
                                // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
                            }
                        } else if let Some(connection_idx) = self.input_state.find_clicked_connection(&self.build_temp_graph(&viewed_nodes), 8.0, self.viewport.zoom) {
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
                        // Check if we're starting to drag from a port for connections
                        if let Some((node_id, port_idx, is_input)) = self.input_state.find_clicked_port(&self.build_temp_graph(&viewed_nodes), 10.0) {
                            // Handle input port disconnection on drag
                            if is_input {
                                if let Some((conn_idx, from_node, from_port)) = self.input_state.find_input_connection(&self.graph, node_id, port_idx) {
                                    // Disconnect and start new connection from original source
                                    self.graph.remove_connection(conn_idx);
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
                            let current_graph = match self.current_view {
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
                            match self.current_view {
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
                            // Check if we released on a port to complete connection
                            if let Some((node_id, port_idx, _)) = self.input_state.find_clicked_port(&self.build_temp_graph(&viewed_nodes), 10.0) {
                                if let Some(connection) = self.input_state.complete_connection(node_id, port_idx) {
                                    let _ = self.graph.add_connection(connection);
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
                        match self.current_view {
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
                    match self.current_view {
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
                    
                    match self.current_view {
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
                let new_connections = self.input_state.create_connections_from_paths(&self.graph);
                
                if !new_connections.is_empty() {
                    for connection in new_connections {
                        // Check if target is an input port and already has a connection
                        if let Some((existing_idx, _, _)) = self.input_state.find_input_connection(&self.graph, connection.to_node, connection.to_port) {
                            // Remove existing connection to input port
                            self.graph.remove_connection(existing_idx);
                            self.mark_modified();
                        }
                        
                        let _ = self.graph.add_connection(connection);
                        self.mark_modified();
                    }
                    
                    // self.gpu_instance_manager.force_rebuild(); // DISABLED: rebuilding every frame now
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


            // Update port positions
            self.graph.update_all_port_positions();

            // Draw nodes - GPU vs CPU rendering
            if self.use_gpu_rendering && !viewed_nodes.is_empty() {
                    // Calculate viewport bounds for GPU callback
                    let viewport_rect = response.rect;
                    
                    // Create GPU callback for node body rendering  
                    // Use the full screen size, not just the response rect size, to match GPU viewport
                    let screen_size = Vec2::new(
                        ui.ctx().screen_rect().width(),
                        ui.ctx().screen_rect().height()
                    );
                    
                    // Get current graph for box selection preview
                    let current_graph = match self.current_view {
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
                    for (_node_id, node) in &viewed_nodes {
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
                
                // Visibility flags are now rendered by GPU shader
                
            } else if !viewed_nodes.is_empty() {
                // CPU rendering path - fallback mode using MeshRenderer
                
                // Get current graph for box selection preview
                let current_graph = match self.current_view {
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
                    if let Some(node) = viewed_nodes.get(&from_node) {
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