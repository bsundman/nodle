//! Tree panel implementation
//! 
//! Handles tree-type interface panels for hierarchical visualization (e.g., USD scene graphs)

use egui::{Context, Color32, Pos2, ScrollArea};
use crate::nodes::{Node, NodeId, InterfacePanelManager};
use crate::nodes::interface::{PanelType, NodeData};
use crate::editor::panels::PanelAction;
use crate::workspaces::three_d::usd::usd_engine::{USDMeshGeometry, USDScenegraphMetadata, USDMeshMetadata};
use std::collections::HashMap;
use log::{debug, info};

// Import USD types
// Note: We use the workspaces USD types since that's what NodeData uses

/// Cached render data to avoid recreating UI elements every frame
#[derive(Clone)]
struct CachedRenderData {
    stage_label: String,
    total_stats_label: String,
    meshes_header: String,
    lights_header: String,
    materials_header: String,
    mesh_names: Vec<String>,
    light_names: Vec<(String, String)>, // (icon, name)
    material_names: Vec<String>,
}

/// Tree panel renderer for hierarchical data visualization
pub struct TreePanel {
    /// Default tree panel size
    default_size: [f32; 2],
    /// Selected tab for each stacked tree window
    selected_tabs: HashMap<String, usize>,
    /// Expanded state for tree nodes
    expanded_nodes: HashMap<NodeId, HashMap<String, bool>>,
    /// Cached USD data for each node to avoid constant re-fetching
    cached_data: HashMap<NodeId, (NodeData, u64)>,
    /// Cache versions to track when data changes
    last_cache_versions: HashMap<NodeId, u64>,
    /// Pagination state for large lists (NodeId -> (category, page_index))
    pagination_state: HashMap<NodeId, HashMap<String, usize>>,
    /// Frame counter for throttling cache updates (NodeId -> last_check_frame)
    last_cache_check_frame: HashMap<NodeId, u64>,
    /// Global frame counter
    current_frame: u64,
    /// Cached render data to avoid string allocations every frame
    cached_render_data: HashMap<NodeId, CachedRenderData>,
}

impl TreePanel {
    pub fn new() -> Self {
        Self {
            default_size: [400.0, 600.0], // Default size for tree panels
            selected_tabs: HashMap::new(),
            expanded_nodes: HashMap::new(),
            cached_data: HashMap::new(),
            last_cache_versions: HashMap::new(),
            pagination_state: HashMap::new(),
            last_cache_check_frame: HashMap::new(),
            current_frame: 0,
            cached_render_data: HashMap::new(),
        }
    }

    /// Render tree panels (handles both tabbed stacking and individual floating in same window)
    pub fn render(
        &mut self,
        ctx: &Context,
        node_id: NodeId,
        node: &Node,
        panel_manager: &mut InterfacePanelManager,
        menu_bar_height: f32,
        viewed_nodes: &HashMap<NodeId, Node>,
        graph: &mut crate::nodes::NodeGraph,
        execution_engine: &mut crate::nodes::NodeGraphEngine,
    ) -> PanelAction {
        // Check if this panel should be stacked
        if panel_manager.is_panel_stacked(node_id) {
            // For stacked panels, only render the shared window from the first stacked node
            let stacked_tree_nodes = panel_manager.get_stacked_panels_by_type(
                PanelType::Tree, 
                viewed_nodes
            );
            
            if let Some(&first_node_id) = stacked_tree_nodes.first() {
                if node_id == first_node_id {
                    // Render as tabbed stack (even if only one panel)
                    self.render_tree_window(ctx, first_node_id, node, panel_manager, menu_bar_height, viewed_nodes, graph, execution_engine, true, &stacked_tree_nodes)
                } else {
                    // This is not the first node, don't render a window (already handled by first node)
                    PanelAction::None
                }
            } else {
                PanelAction::None
            }
        } else {
            // Render as individual floating window
            self.render_tree_window(ctx, node_id, node, panel_manager, menu_bar_height, viewed_nodes, graph, execution_engine, false, &[node_id])
        }
    }

    /// Unified tree window renderer (handles both individual and stacked modes)
    fn render_tree_window(
        &mut self,
        ctx: &Context,
        primary_node_id: NodeId,
        primary_node: &Node,
        panel_manager: &mut InterfacePanelManager,
        menu_bar_height: f32,
        viewed_nodes: &HashMap<NodeId, Node>,
        graph: &mut crate::nodes::NodeGraph,
        execution_engine: &mut crate::nodes::NodeGraphEngine,
        is_stacked: bool,
        node_ids: &[NodeId],
    ) -> PanelAction {
        // Check if panel is marked as visible
        if !panel_manager.is_panel_visible(primary_node_id) {
            return PanelAction::None;
        }
        
        // Simple window ID logic
        let panel_id = if is_stacked {
            egui::Id::new("stacked_tree_panels")
        } else {
            egui::Id::new(format!("tree_panel_{}", primary_node_id))
        };
        
        let mut panel_action = PanelAction::None;
        
        // Determine window title
        let title = if is_stacked {
            format!("🌳 Scene Graph Tree ({})", node_ids.len())
        } else {
            let custom_name = panel_manager.get_node_name(primary_node_id);
            format!("🌳 {}", custom_name.unwrap_or(&primary_node.title))
        };
        
        // Get panel open state reference
        let mut is_open = panel_manager.is_panel_open(primary_node_id);
        
        // Create window with constraints
        let mut window = egui::Window::new(title)
            .id(panel_id)
            .open(&mut is_open)
            .default_size(self.default_size)
            .min_size([300.0, 400.0])
            .resizable(true)
            .collapsible(true)
            .constrain_to(egui::Rect::from_min_size(
                Pos2::new(0.0, menu_bar_height),
                egui::Vec2::new(ctx.screen_rect().width(), ctx.screen_rect().height() - menu_bar_height)
            ));
        
        // Only set default position if not stacked (stacked windows manage their own position)
        if !is_stacked {
            // Position tree panel to the right of the node
            let node_pos = primary_node.position;
            window = window.default_pos(node_pos + egui::vec2(200.0, 0.0));
        }
        
        let window_response = window.show(ctx, |ui| {
            if is_stacked && node_ids.len() > 1 {
                // Render tabs for multiple tree panels
                ui.horizontal(|ui| {
                    for (idx, &tree_node_id) in node_ids.iter().enumerate() {
                        if let Some(tree_node) = viewed_nodes.get(&tree_node_id) {
                            let custom_name = panel_manager.get_node_name(tree_node_id);
                            let tab_name = custom_name.unwrap_or(&tree_node.title);
                            
                            let is_selected = self.selected_tabs.get("tree").copied().unwrap_or(0) == idx;
                            if ui.selectable_label(is_selected, tab_name).clicked() {
                                self.selected_tabs.insert("tree".to_string(), idx);
                            }
                        }
                    }
                });
                ui.separator();
                
                // Render the selected tab's content
                let selected_idx = self.selected_tabs.get("tree").copied().unwrap_or(0);
                if let Some(&selected_node_id) = node_ids.get(selected_idx) {
                    if let Some(selected_node) = viewed_nodes.get(&selected_node_id) {
                        self.render_tree_content(ui, selected_node_id, selected_node, graph, execution_engine);
                    }
                }
            } else {
                // Single tree panel
                self.render_tree_content(ui, primary_node_id, primary_node, graph, execution_engine);
            }
            
            // Window controls
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("📌 Pin").on_hover_text("Pin window to prevent automatic repositioning").clicked() {
                    panel_action = PanelAction::TogglePin;
                }
                
                if ui.button(if is_stacked { "📤 Unstack" } else { "📥 Stack" })
                    .on_hover_text(if is_stacked { "Separate into individual window" } else { "Stack with other tree panels" })
                    .clicked() 
                {
                    panel_action = PanelAction::ToggleStack;
                }
                
                if ui.button("❌ Close").clicked() {
                    panel_action = if is_stacked { PanelAction::CloseAll } else { PanelAction::Close };
                }
            });
        });
        
        // Update open state in panel manager
        panel_manager.set_panel_open(primary_node_id, is_open);
        
        // Handle window close via 'X' button
        if !is_open {
            debug!("Tree window closed via X button for node {}", primary_node_id);
            panel_action = if is_stacked { PanelAction::CloseAll } else { PanelAction::Close };
        }
        
        panel_action
    }

    /// Render the tree content for a specific node
    fn render_tree_content(
        &mut self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        node: &Node,
        graph: &crate::nodes::NodeGraph,
        execution_engine: &mut crate::nodes::NodeGraphEngine,
    ) {
        // For responsive disconnection detection, always check cache when we have data
        // For performance, throttle checks when we have no data  
        let has_local_cache = self.cached_data.contains_key(&node_id);
        
        if has_local_cache {
            // If we have cached data, check every frame for disconnection
            self.update_cache_if_needed(node_id);
        } else {
            // If we have no cached data, only check periodically
            const CACHE_CHECK_INTERVAL: u64 = 30;
            let last_check = self.last_cache_check_frame.get(&node_id).copied().unwrap_or(0);
            let should_check_cache = self.current_frame.wrapping_sub(last_check) >= CACHE_CHECK_INTERVAL;
            
            if should_check_cache {
                self.current_frame = self.current_frame.wrapping_add(1);
                self.update_cache_if_needed(node_id);
                self.last_cache_check_frame.insert(node_id, self.current_frame);
            }
        }
        
        // Check what type of data we have
        if let Some((data, _)) = self.cached_data.get(&node_id) {
            match data {
                NodeData::USDScenegraphMetadata(_) => {
                    // Render with lightweight metadata - much faster!
                    self.render_usd_metadata_optimized(ui, node_id);
                }
                NodeData::USDSceneData(_) => {
                    // Legacy fallback - render with full geometry data
                    self.render_usd_scene_data_optimized(ui, node_id);
                }
                NodeData::String(path) => {
                    ScrollArea::vertical().show(ui, |ui| {
                        ui.label(format!("USD File: {}", path));
                        ui.label("(No scene data loaded yet)");
                    });
                }
                _ => {
                    ScrollArea::vertical().show(ui, |ui| {
                        ui.label("No USD data connected");
                        ui.label("Connect a USD source node to view the scene hierarchy");
                    });
                }
            }
        } else {
            ScrollArea::vertical().show(ui, |ui| {
                ui.label("No USD data connected");
                ui.label("Connect a USD source node to view the scene hierarchy");
            });
        }
    }
    
    /// Super-optimized render method using lightweight metadata (no geometry data)
    fn render_usd_metadata_optimized(&mut self, ui: &mut egui::Ui, node_id: NodeId) {
        // Get the lightweight metadata and clone it to avoid borrowing issues
        let metadata = match self.cached_data.get(&node_id) {
            Some((NodeData::USDScenegraphMetadata(data), _)) => data.clone(),
            _ => return,
        };
        
        // Ensure we have cached render data for this metadata
        if !self.cached_render_data.contains_key(&node_id) {
            self.update_metadata_render_cache(node_id, &metadata);
        }
        
        // Use cached render data to avoid expensive string operations every frame
        let render_data = match self.cached_render_data.get(&node_id) {
            Some(data) => data,
            None => return, // This shouldn't happen, but safety first
        };
        
        ScrollArea::vertical().show(ui, |ui| {
            // Stage info - use cached strings
            ui.label(&render_data.stage_label);
            ui.label(&render_data.total_stats_label);
            ui.separator();
            
            // Render meshes with metadata only
            if !metadata.meshes.is_empty() {
                ui.collapsing(&render_data.meshes_header, |ui| {
                    const ITEMS_PER_PAGE: usize = 50;
                    let total_pages = (metadata.meshes.len() + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE;
                    
                    if total_pages > 1 {
                        // Get current page
                        let current_page = self.pagination_state
                            .get(&node_id)
                            .and_then(|p| p.get("meshes"))
                            .copied()
                            .unwrap_or(0);
                        
                        // Show pagination controls
                        ui.horizontal(|ui| {
                            ui.label(format!("Page {} of {}", current_page + 1, total_pages));
                        });
                        ui.separator();
                        
                        // Render current page items using cached names
                        let start_idx = current_page * ITEMS_PER_PAGE;
                        let end_idx = (start_idx + ITEMS_PER_PAGE).min(metadata.meshes.len());
                        
                        for idx in start_idx..end_idx {
                            if let (Some(mesh), Some(cached_name)) = (metadata.meshes.get(idx), render_data.mesh_names.get(idx)) {
                                Self::render_mesh_metadata_cached(ui, mesh, cached_name);
                            }
                        }
                    } else {
                        // Render all items using cached names
                        for (mesh, cached_name) in metadata.meshes.iter().zip(render_data.mesh_names.iter()) {
                            Self::render_mesh_metadata_cached(ui, mesh, cached_name);
                        }
                    }
                });
            }
            
            // Render lights with metadata only - use cached data
            if !metadata.lights.is_empty() {
                ui.collapsing(&render_data.lights_header, |ui| {
                    for (light, (icon, name)) in metadata.lights.iter().zip(render_data.light_names.iter()) {
                        ui.collapsing(format!("{} {}", icon, name), |ui| {
                            ui.horizontal(|ui| {
                                ui.label("  📍");
                                ui.label(format!("Path: {}", light.prim_path));
                            });
                            ui.horizontal(|ui| {
                                ui.label("  🏷️");
                                ui.label(format!("Type: {}", light.light_type));
                            });
                            ui.horizontal(|ui| {
                                ui.label("  ⚡");
                                ui.label(format!("Intensity: {:.2}", light.intensity));
                            });
                        });
                    }
                });
            }
            
            // Render materials with metadata only - use cached data
            if !metadata.materials.is_empty() {
                ui.collapsing(&render_data.materials_header, |ui| {
                    for (material, cached_name) in metadata.materials.iter().zip(render_data.material_names.iter()) {
                        ui.collapsing(format!("🔸 {}", cached_name), |ui| {
                            ui.horizontal(|ui| {
                                ui.label("  📍");
                                ui.label(format!("Path: {}", material.prim_path));
                            });
                            ui.horizontal(|ui| {
                                ui.label("  🖼️");
                                ui.label(format!("Diffuse: {}, Normal: {}, PBR: {}", 
                                    if material.has_diffuse_texture { "✓" } else { "✗" },
                                    if material.has_normal_texture { "✓" } else { "✗" },
                                    if material.has_metallic_roughness { "✓" } else { "✗" }
                                ));
                            });
                        });
                    }
                });
            }
        });
    }
    
    /// Render a single mesh metadata item using cached name
    fn render_mesh_metadata_cached(ui: &mut egui::Ui, mesh: &USDMeshMetadata, cached_name: &str) {
        ui.collapsing(format!("🔹 {}", cached_name), |ui| {
            ui.horizontal(|ui| {
                ui.label("  📍");
                ui.label(format!("Path: {}", mesh.prim_path));
            });
            ui.horizontal(|ui| {
                ui.label("  🔸");
                ui.label(format!("Vertices: {}", mesh.vertex_count));
            });
            ui.horizontal(|ui| {
                ui.label("  🔺");
                ui.label(format!("Triangles: {}", mesh.triangle_count));
            });
            ui.horizontal(|ui| {
                ui.label("  📊");
                ui.label(format!("Attributes: {}{}{}",
                    if mesh.has_normals { "N " } else { "" },
                    if mesh.has_uvs { "UV " } else { "" },
                    if mesh.has_colors { "C " } else { "" }
                ));
            });
            if let Some(material) = &mesh.material_binding {
                ui.horizontal(|ui| {
                    ui.label("  🎨");
                    ui.label(format!("Material: {}", material));
                });
            }
        });
    }

    /// Render a single mesh metadata item
    fn render_mesh_metadata(ui: &mut egui::Ui, mesh: &USDMeshMetadata) {
        let mesh_name = mesh.prim_path.split('/').last().unwrap_or("Mesh");
        Self::render_mesh_metadata_cached(ui, mesh, mesh_name);
    }
    
    /// Create or update cached render data from USD metadata
    fn update_metadata_render_cache(&mut self, node_id: NodeId, metadata: &crate::workspaces::three_d::usd::usd_engine::USDScenegraphMetadata) {
        let render_data = CachedRenderData {
            stage_label: format!("Stage: {}", metadata.stage_path),
            total_stats_label: format!("Total: {} vertices, {} triangles", metadata.total_vertices, metadata.total_triangles),
            meshes_header: format!("📦 Meshes ({})", metadata.meshes.len()),
            lights_header: format!("💡 Lights ({})", metadata.lights.len()),
            materials_header: format!("🎨 Materials ({})", metadata.materials.len()),
            mesh_names: metadata.meshes.iter().map(|mesh| {
                mesh.prim_path.split('/').last().unwrap_or("Mesh").to_string()
            }).collect(),
            light_names: metadata.lights.iter().map(|light| {
                let icon = match light.light_type.as_str() {
                    "distant" | "directional" => "☀️",
                    "point" => "💡",
                    "spot" => "🔦",
                    "rect" => "⬜",
                    "disk" => "⭕",
                    "sphere" => "🔮",
                    "cylinder" => "🛢️",
                    "dome" => "🌐",
                    _ => "💡",
                };
                let name = light.prim_path.split('/').last().unwrap_or("Light").to_string();
                (icon.to_string(), name)
            }).collect(),
            material_names: metadata.materials.iter().map(|material| {
                material.prim_path.split('/').last().unwrap_or("Material").to_string()
            }).collect(),
        };
        
        self.cached_render_data.insert(node_id, render_data);
    }

    /// Create or update cached render data from USD scene data
    fn update_render_cache(&mut self, node_id: NodeId, scene_data: &crate::workspaces::three_d::usd::usd_engine::USDSceneData) {
        let total_vertices: usize = scene_data.meshes.iter().map(|m| m.vertices.len()).sum();
        let total_triangles: usize = scene_data.meshes.iter().map(|m| m.indices.len() / 3).sum();
        
        let render_data = CachedRenderData {
            stage_label: format!("Stage: {}", scene_data.stage_path),
            total_stats_label: format!("Total: {} vertices, {} triangles", total_vertices, total_triangles),
            meshes_header: format!("📦 Meshes ({})", scene_data.meshes.len()),
            lights_header: format!("💡 Lights ({})", scene_data.lights.len()),
            materials_header: format!("🎨 Materials ({})", scene_data.materials.len()),
            mesh_names: scene_data.meshes.iter().map(|mesh| {
                let prim_parts: Vec<&str> = mesh.prim_path.split('/').filter(|s| !s.is_empty()).collect();
                prim_parts.last().map(|s| s.to_string()).unwrap_or_else(|| "Mesh".to_string())
            }).collect(),
            light_names: scene_data.lights.iter().map(|light| {
                let icon = match light.light_type.as_str() {
                    "distant" | "directional" => "☀️",
                    "point" => "💡",
                    "spot" => "🔦",
                    "rect" => "⬜",
                    "disk" => "⭕",
                    "sphere" => "🔮",
                    "cylinder" => "🛢️",
                    "dome" => "🌐",
                    _ => "💡",
                };
                let light_name = light.prim_path.split('/').last().unwrap_or("Light");
                (icon.to_string(), light_name.to_string())
            }).collect(),
            material_names: scene_data.materials.iter().map(|mat| {
                mat.prim_path.split('/').last().unwrap_or("Material").to_string()
            }).collect(),
        };
        
        self.cached_render_data.insert(node_id, render_data);
    }
    
    /// Optimized render method using cached data
    fn render_usd_scene_data_optimized(&mut self, ui: &mut egui::Ui, node_id: NodeId) {
        // Check if render cache needs updating
        if !self.cached_render_data.contains_key(&node_id) {
            // Clone the data to avoid borrow checker issues
            if let Some((data, _)) = self.cached_data.get(&node_id).cloned() {
                if let NodeData::USDSceneData(scene_data) = &data {
                    self.update_render_cache(node_id, scene_data);
                }
            } else {
                return; // No data to render
            }
        }
        
        // Clone the data we need to avoid borrow checker issues
        let (render_data, scene_data) = {
            let render_data = match self.cached_render_data.get(&node_id) {
                Some(data) => data.clone(),
                None => return,
            };
            
            let scene_data = match self.cached_data.get(&node_id) {
                Some((NodeData::USDSceneData(data), _)) => data.clone(),
                _ => return,
            };
            
            (render_data, scene_data)
        };
        
        // Pre-calculate pagination state
        let current_mesh_page = self.pagination_state
            .get(&node_id)
            .and_then(|p| p.get("meshes"))
            .copied()
            .unwrap_or(0);
        
        ScrollArea::vertical().show(ui, |ui| {
            // Use cached strings
            ui.label(&render_data.stage_label);
            ui.separator();
            
            // Render meshes with cached header
            if !scene_data.meshes.is_empty() {
                ui.collapsing(&render_data.meshes_header, |ui| {
                    // Simple inline pagination for now
                    const ITEMS_PER_PAGE: usize = 50;
                    let total_pages = (scene_data.meshes.len() + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE;
                    
                    if total_pages > 1 {
                        // Show pagination controls
                        ui.horizontal(|ui| {
                            ui.label(format!("Page {} of {}", current_mesh_page + 1, total_pages));
                        });
                        ui.separator();
                        
                        // Render current page items
                        let start_idx = current_mesh_page * ITEMS_PER_PAGE;
                        let end_idx = (start_idx + ITEMS_PER_PAGE).min(scene_data.meshes.len());
                        
                        for idx in start_idx..end_idx {
                            if let (Some(mesh), Some(name)) = (scene_data.meshes.get(idx), render_data.mesh_names.get(idx)) {
                                Self::render_single_mesh_optimized(ui, idx, mesh, name);
                            }
                        }
                    } else {
                        // Render all items
                        for (idx, (mesh, name)) in scene_data.meshes.iter().zip(render_data.mesh_names.iter()).enumerate() {
                            Self::render_single_mesh_optimized(ui, idx, mesh, name);
                        }
                    }
                });
            }
            
            // Render lights with cached data
            if !scene_data.lights.is_empty() {
                ui.collapsing(&render_data.lights_header, |ui| {
                    for (idx, light) in scene_data.lights.iter().enumerate() {
                        if let Some((icon, name)) = render_data.light_names.get(idx) {
                            ui.collapsing(format!("{} {}", icon, name), |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("  📍");
                                    ui.label(format!("Path: {}", light.prim_path));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("  🏷️");
                                    ui.label(format!("Type: {}", light.light_type));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("  ⚡");
                                    ui.label(format!("Intensity: {:.2}", light.intensity));
                                });
                            });
                        }
                    }
                });
            }
            
            // Render materials with cached names
            if !scene_data.materials.is_empty() {
                ui.collapsing(&render_data.materials_header, |ui| {
                    for (idx, mat) in scene_data.materials.iter().enumerate() {
                        if let Some(name) = render_data.material_names.get(idx) {
                            ui.collapsing(format!("🔸 {}", name), |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("  📍");
                                    ui.label(format!("Path: {}", mat.prim_path));
                                });
                            });
                        }
                    }
                });
            }
        });
        
        // Handle pagination updates outside of closures
        // This is a simplified approach - pagination buttons would need to be handled differently
    }
    
    /// Render USD scene data with pagination (old method, kept for compatibility)
    fn render_usd_scene_data(&mut self, ui: &mut egui::Ui, node_id: NodeId, scene_data: &crate::workspaces::three_d::usd::usd_engine::USDSceneData) {
        // Clone the meshes data to avoid borrowing issues
        let meshes = scene_data.meshes.clone();
        
        ScrollArea::vertical().show(ui, |ui| {
            // Render USD scene hierarchy
            ui.label(format!("Stage: {}", scene_data.stage_path));
            ui.separator();
            
            // Render mesh hierarchy - paginated for performance
            if !meshes.is_empty() {
                ui.collapsing(format!("📦 Meshes ({})", meshes.len()), |ui| {
                    self.render_paginated_meshes(ui, node_id, &meshes);
                });
            }
            
            // Render lights hierarchy - collapsed by default for performance  
            if !scene_data.lights.is_empty() {
                ui.collapsing(format!("💡 Lights ({})", scene_data.lights.len()), |ui| {
                    for (_idx, light) in scene_data.lights.iter().enumerate() {
                        let icon = match light.light_type.as_str() {
                            "distant" | "directional" => "☀️",
                            "point" => "💡",
                            "spot" => "🔦",
                            "rect" => "⬜",
                            "disk" => "⭕",
                            "sphere" => "🔮",
                            "cylinder" => "🛢️",
                            "dome" => "🌐",
                            _ => "💡", // Default to point light icon
                        };
                        let light_name = light.prim_path.split('/').last().unwrap_or("Light");
                        
                        // Each light is collapsed by default
                        ui.collapsing(format!("{} {}", icon, light_name), |ui| {
                            ui.horizontal(|ui| {
                                ui.label("  📍");
                                ui.label(format!("Path: {}", light.prim_path));
                            });
                            ui.horizontal(|ui| {
                                ui.label("  🏷️");
                                ui.label(format!("Type: {}", light.light_type));
                            });
                            ui.horizontal(|ui| {
                                ui.label("  ⚡");
                                ui.label(format!("Intensity: {:.2}", light.intensity));
                            });
                        });
                    }
                });
            }
            
            // Render materials hierarchy - collapsed by default for performance
            if !scene_data.materials.is_empty() {
                ui.collapsing(format!("🎨 Materials ({})", scene_data.materials.len()), |ui| {
                    for (_idx, mat) in scene_data.materials.iter().enumerate() {
                        let material_name = mat.prim_path.split('/').last().unwrap_or("Material");
                        
                        // Each material is collapsed by default
                        ui.collapsing(format!("🔸 {}", material_name), |ui| {
                            ui.horizontal(|ui| {
                                ui.label("  📍");
                                ui.label(format!("Path: {}", mat.prim_path));
                            });
                        });
                    }
                });
            }
        });
    }

    /// Render a collapsible tree node
    fn render_tree_node<F>(
        &self,
        ui: &mut egui::Ui,
        expanded_state: &mut HashMap<String, bool>,
        label: &str,
        contents: F,
    ) where
        F: FnOnce(&mut egui::Ui, &mut bool),
    {
        let expanded = expanded_state.entry(label.to_string()).or_insert(true);
        
        ui.horizontal(|ui| {
            if ui.button(if *expanded { "▼" } else { "▶" }).clicked() {
                *expanded = !*expanded;
            }
            ui.label(label);
        });
        
        if *expanded {
            ui.indent(label, |ui| {
                contents(ui, expanded);
            });
        }
    }

    /// Optimized render paginated meshes using cached names
    fn render_paginated_meshes_optimized(&mut self, ui: &mut egui::Ui, node_id: NodeId, meshes: &[USDMeshGeometry], cached_names: &[String]) {
        const ITEMS_PER_PAGE: usize = 50;
        
        let total_meshes = meshes.len();
        let total_pages = (total_meshes + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE;
        
        if total_pages <= 1 {
            // Small list, render all items with cached names
            for (idx, (mesh, name)) in meshes.iter().zip(cached_names.iter()).enumerate() {
                Self::render_single_mesh_optimized(ui, idx, mesh, name);
            }
            return;
        }
        
        // Get current page and handle pagination updates
        let node_pagination = self.pagination_state.entry(node_id).or_insert_with(HashMap::new);
        let mut current_page = *node_pagination.get("meshes").unwrap_or(&0);
        let mut page_changed = false;
        
        // Show pagination controls
        ui.horizontal(|ui| {
            if ui.button("◀ Prev").clicked() && current_page > 0 {
                current_page -= 1;
                page_changed = true;
            }
            
            ui.label(format!("Page {} of {} ({} items)", current_page + 1, total_pages, total_meshes));
            
            if ui.button("Next ▶").clicked() && current_page + 1 < total_pages {
                current_page += 1;
                page_changed = true;
            }
        });
        
        // Update pagination state if changed
        if page_changed {
            self.pagination_state.get_mut(&node_id).unwrap().insert("meshes".to_string(), current_page);
        }
        
        ui.separator();
        
        // Calculate range for current page
        let start_idx = current_page * ITEMS_PER_PAGE;
        let end_idx = (start_idx + ITEMS_PER_PAGE).min(total_meshes);
        
        // Render only the items for this page with cached names
        for idx in start_idx..end_idx {
            if let (Some(mesh), Some(name)) = (meshes.get(idx), cached_names.get(idx)) {
                Self::render_single_mesh_optimized(ui, idx, mesh, name);
            }
        }
        
        // Show pagination info at bottom
        ui.separator();
        ui.label(format!("Showing {} - {} of {} meshes", start_idx + 1, end_idx, total_meshes));
    }
    
    /// Render single mesh with pre-cached name
    fn render_single_mesh_optimized(ui: &mut egui::Ui, idx: usize, mesh: &USDMeshGeometry, cached_name: &str) {
        ui.collapsing(format!("🔹 {}", cached_name), |ui| {
            ui.horizontal(|ui| {
                ui.label("  📍");
                ui.label(format!("Path: {}", mesh.prim_path));
            });
            ui.horizontal(|ui| {
                ui.label("  🔸");
                ui.label(format!("Vertices: {}", mesh.vertices.len()));
            });
            ui.horizontal(|ui| {
                ui.label("  🔺");
                ui.label(format!("Triangles: {}", mesh.indices.len() / 3));
            });
        });
    }
    
    /// Render paginated meshes for performance
    fn render_paginated_meshes(&mut self, ui: &mut egui::Ui, node_id: NodeId, meshes: &[USDMeshGeometry]) {
        const ITEMS_PER_PAGE: usize = 50; // Limit to 50 meshes per page for performance
        
        let total_meshes = meshes.len();
        let total_pages = (total_meshes + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE;
        
        if total_pages <= 1 {
            // Small list, render all items
            for (idx, mesh) in meshes.iter().enumerate() {
                Self::render_single_mesh_static(ui, idx, mesh);
            }
            return;
        }
        
        // Get current page for this node's meshes
        let pagination = self.pagination_state.entry(node_id).or_insert_with(HashMap::new);
        let current_page = *pagination.get("meshes").unwrap_or(&0);
        
        // Show pagination controls
        ui.horizontal(|ui| {
            if ui.button("◀ Prev").clicked() && current_page > 0 {
                pagination.insert("meshes".to_string(), current_page - 1);
            }
            
            ui.label(format!("Page {} of {} ({} items)", current_page + 1, total_pages, total_meshes));
            
            if ui.button("Next ▶").clicked() && current_page + 1 < total_pages {
                pagination.insert("meshes".to_string(), current_page + 1);
            }
        });
        
        ui.separator();
        
        // Calculate range for current page
        let start_idx = current_page * ITEMS_PER_PAGE;
        let end_idx = (start_idx + ITEMS_PER_PAGE).min(total_meshes);
        
        // Render only the items for this page
        for (idx, mesh) in meshes[start_idx..end_idx].iter().enumerate() {
            Self::render_single_mesh_static(ui, start_idx + idx, mesh);
        }
        
        // Show pagination info at bottom
        ui.separator();
        ui.label(format!("Showing {} - {} of {} meshes", start_idx + 1, end_idx, total_meshes));
    }
    
    /// Render a single mesh item (static version to avoid borrowing issues)
    fn render_single_mesh_static(ui: &mut egui::Ui, idx: usize, mesh: &USDMeshGeometry) {
        // Show the full prim path as a hierarchy
        let prim_parts: Vec<&str> = mesh.prim_path.split('/').filter(|s| !s.is_empty()).collect();
        let mesh_name = prim_parts.last().map(|s| s.to_string()).unwrap_or_else(|| format!("Mesh_{}", idx));
        
        // Each mesh is collapsed by default
        ui.collapsing(format!("🔹 {}", mesh_name), |ui| {
            ui.horizontal(|ui| {
                ui.label("  📍");
                ui.label(format!("Path: {}", mesh.prim_path));
            });
            ui.horizontal(|ui| {
                ui.label("  🔸");
                ui.label(format!("Vertices: {}", mesh.vertices.len()));
            });
            ui.horizontal(|ui| {
                ui.label("  🔺");
                ui.label(format!("Triangles: {}", mesh.indices.len() / 3));
            });
        });
    }

    /// Update cache if needed - separate method to avoid borrowing conflicts
    fn update_cache_if_needed(&mut self, node_id: NodeId) {
        // Check if we need to update our local cache
        if let Ok(global_cache) = crate::nodes::three_d::ui::scenegraph::logic::SCENEGRAPH_INPUT_CACHE.lock() {
            if let Some(cached_scenegraph_data) = global_cache.get(&node_id) {
                let current_version = cached_scenegraph_data.version;
                let last_known_version = self.last_cache_versions.get(&node_id).copied().unwrap_or(0);
                
                // Only update local cache if the global cache has newer data
                if current_version > last_known_version {
                    debug!("🌳 Tree panel updating cache for node {} (version {} -> {})", 
                           node_id, last_known_version, current_version);
                    self.cached_data.insert(node_id, (cached_scenegraph_data.data.clone(), current_version));
                    self.last_cache_versions.insert(node_id, current_version);
                    
                    // Clear render cache so it gets regenerated with new data
                    self.cached_render_data.remove(&node_id);
                }
            } else {
                // Clear local cache if global cache no longer has data
                if self.cached_data.contains_key(&node_id) {
                    debug!("🌳 Tree panel clearing cache for node {} (no global data)", node_id);
                    self.cached_data.remove(&node_id);
                    self.last_cache_versions.remove(&node_id);
                    self.cached_render_data.remove(&node_id);
                }
            }
        }
    }
    
    /// Force cache update for a specific node (used when connections change)
    pub fn force_cache_update(&mut self, node_id: NodeId) {
        self.update_cache_if_needed(node_id);
        self.last_cache_check_frame.insert(node_id, self.current_frame);
        debug!("🌳 Tree panel forced cache update for node {}", node_id);
    }

    /// Clean up panel caches for a deleted node
    pub fn cleanup_deleted_node(&mut self, node_id: NodeId) {
        self.expanded_nodes.remove(&node_id);
        self.cached_data.remove(&node_id);
        self.last_cache_versions.remove(&node_id);
        self.pagination_state.remove(&node_id);
        self.last_cache_check_frame.remove(&node_id);
        self.cached_render_data.remove(&node_id);
        debug!("🧹 Tree panel cleanup completed for deleted node: {}", node_id);
    }
}

impl Default for TreePanel {
    fn default() -> Self {
        Self::new()
    }
}