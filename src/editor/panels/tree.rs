//! Tree panel implementation
//! 
//! Handles tree-type interface panels for hierarchical visualization (e.g., USD scene graphs)

use egui::{Context, Pos2, ScrollArea};
use crate::nodes::{Node, NodeId, InterfacePanelManager};
use crate::nodes::interface::{PanelType, NodeData};
use crate::editor::panels::PanelAction;
use crate::workspaces::three_d::usd::usd_engine::{USDMeshGeometry, USDMeshMetadata};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;
use log::debug;

// Import USD types
// Note: We use the workspaces USD types since that's what NodeData uses

// Adaptive throttling constants for cache checks
const ADAPTIVE_CHECK_BASE: u64 = 30;      // Base interval for cache checks
const ADAPTIVE_CHECK_MIN: u64 = 5;        // Minimum interval for high-frequency updates
const ADAPTIVE_CHECK_MAX: u64 = 120;      // Maximum interval for stable data
const ADAPTIVE_THRESHOLD: u64 = 10;       // Threshold for adaptive adjustment

/// Simple string interner for UI labels to reduce memory allocations
#[derive(Default)]
struct StringInterner {
    strings: HashMap<String, Arc<str>>,
}

impl StringInterner {
    fn intern(&mut self, s: &str) -> Arc<str> {
        if let Some(interned) = self.strings.get(s) {
            interned.clone()
        } else {
            let interned: Arc<str> = s.into();
            self.strings.insert(s.to_string(), interned.clone());
            interned
        }
    }
    
    fn clear(&mut self) {
        self.strings.clear();
    }
}

/// Global string interner for UI labels
static STRING_INTERNER: Lazy<Arc<Mutex<StringInterner>>> = Lazy::new(|| {
    Arc::new(Mutex::new(StringInterner::default()))
});

/// Performance metrics for scenegraph operations
#[derive(Debug, Clone)]
struct ScenegraphMetrics {
    cache_hit_rate: f64,
    avg_lookup_time: Duration,
    memory_usage: usize,
    cache_updates: u64,
    cache_misses: u64,
    cache_hits: u64,
    partial_updates: u64,
    last_updated: Instant,
}

impl Default for ScenegraphMetrics {
    fn default() -> Self {
        Self {
            cache_hit_rate: 0.0,
            avg_lookup_time: Duration::from_nanos(0),
            memory_usage: 0,
            cache_updates: 0,
            cache_misses: 0,
            cache_hits: 0,
            partial_updates: 0,
            last_updated: Instant::now(),
        }
    }
}

/// Pre-computed mesh statistics to avoid recalculating every frame
#[derive(Clone)]
struct MeshStats {
    display_name: Arc<str>,
    vertex_count: usize,
    triangle_count: usize,
    path_display: Arc<str>,
    // Pre-formatted strings for common display
    vertex_label: Arc<str>,
    triangle_label: Arc<str>,
}

/// Page information for dynamic complexity-based pagination
#[derive(Clone)]
struct PageInfo {
    start_idx: usize,
    end_idx: usize,
    item_count: usize,
    complexity_score: usize, // Total vertex count for this page
}

/// Tracks what parts of the render cache have changed to enable incremental updates
#[derive(Default, Clone)]
struct CacheChangeTracker {
    /// Hash of the stage path to detect stage changes
    stage_hash: u64,
    /// Hash of mesh data to detect mesh changes
    meshes_hash: u64,
    /// Hash of light data to detect light changes
    lights_hash: u64,
    /// Hash of material data to detect material changes
    materials_hash: u64,
    /// Individual mesh hashes for granular change detection
    mesh_hashes: Vec<u64>,
    /// Light hashes for granular change detection
    light_hashes: Vec<u64>,
    /// Material hashes for granular change detection
    material_hashes: Vec<u64>,
}

/// Cached render data to avoid recreating UI elements every frame
/// Uses Arc<str> for interned strings to reduce memory allocations
#[derive(Clone)]
struct CachedRenderData {
    stage_label: Arc<str>,
    total_stats_label: Arc<str>,
    meshes_header: Arc<str>,
    lights_header: Arc<str>,
    materials_header: Arc<str>,
    mesh_names: Vec<Arc<str>>,
    light_names: Vec<(Arc<str>, Arc<str>)>, // (icon, name)
    material_names: Vec<Arc<str>>,
    mesh_stats: Vec<MeshStats>, // Pre-computed mesh statistics
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
    /// Adaptive throttling intervals (NodeId -> current_interval)
    adaptive_intervals: HashMap<NodeId, u64>,
    /// Change frequency tracking (NodeId -> change_count)
    change_frequency: HashMap<NodeId, u64>,
    /// Performance metrics
    metrics: ScenegraphMetrics,
    /// Tracks changes to enable incremental cache updates
    change_trackers: HashMap<NodeId, CacheChangeTracker>,
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
            adaptive_intervals: HashMap::new(),
            change_frequency: HashMap::new(),
            metrics: ScenegraphMetrics::default(),
            change_trackers: HashMap::new(),
        }
    }

    /// Intern a string to reduce memory allocations
    fn intern_string(s: &str) -> Arc<str> {
        if let Ok(mut interner) = STRING_INTERNER.lock() {
            interner.intern(s)
        } else {
            // Fallback if interner is locked
            Arc::from(s)
        }
    }

    /// Calculate adaptive throttling interval based on change frequency
    fn calculate_adaptive_interval(&self, node_id: NodeId) -> u64 {
        let change_count = self.change_frequency.get(&node_id).copied().unwrap_or(0);
        let current_interval = self.adaptive_intervals.get(&node_id).copied().unwrap_or(ADAPTIVE_CHECK_BASE);
        
        if change_count > ADAPTIVE_THRESHOLD {
            // High frequency changes - reduce interval
            (current_interval * 2 / 3).max(ADAPTIVE_CHECK_MIN)
        } else if change_count < ADAPTIVE_THRESHOLD / 2 {
            // Low frequency changes - increase interval
            (current_interval * 4 / 3).min(ADAPTIVE_CHECK_MAX)
        } else {
            // Moderate frequency - keep current interval
            current_interval
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
            // If we have no cached data, only check periodically with adaptive throttling
            let adaptive_interval = self.calculate_adaptive_interval(node_id);
            let last_check = self.last_cache_check_frame.get(&node_id).copied().unwrap_or(0);
            let should_check_cache = self.current_frame.wrapping_sub(last_check) >= adaptive_interval;
            
            if should_check_cache {
                self.current_frame = self.current_frame.wrapping_add(1);
                self.update_cache_if_needed(node_id);
                self.last_cache_check_frame.insert(node_id, self.current_frame);
                
                // Update adaptive interval based on calculated value
                self.adaptive_intervals.insert(node_id, adaptive_interval);
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
            ui.label(&*render_data.stage_label);
            ui.label(&*render_data.total_stats_label);
            ui.separator();
            
            // Render meshes with metadata only
            if !metadata.meshes.is_empty() {
                ui.collapsing(&*render_data.meshes_header, |ui| {
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
                ui.collapsing(&*render_data.lights_header, |ui| {
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
                ui.collapsing(&*render_data.materials_header, |ui| {
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
    fn render_mesh_metadata_cached(ui: &mut egui::Ui, mesh: &USDMeshMetadata, cached_name: &Arc<str>) {
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
        let interned_name = Self::intern_string(mesh_name);
        Self::render_mesh_metadata_cached(ui, mesh, &interned_name);
    }
    
    /// Create or update cached render data from USD metadata
    fn update_metadata_render_cache(&mut self, node_id: NodeId, metadata: &crate::workspaces::three_d::usd::usd_engine::USDScenegraphMetadata) {
        // Pre-compute mesh statistics
        let mesh_stats = if let Ok(mut interner) = STRING_INTERNER.lock() {
            metadata.meshes.iter().map(|mesh| {
                let name = mesh.prim_path.split('/').last().unwrap_or("Mesh");
                let display_name = interner.intern(name);
                let path_display = interner.intern(&mesh.prim_path);
                
                // Pre-format commonly used strings
                let vertex_label = interner.intern(&format!("Vertices: {}", mesh.vertex_count));
                let triangle_label = interner.intern(&format!("Triangles: {}", mesh.triangle_count));
                
                MeshStats {
                    display_name,
                    vertex_count: mesh.vertex_count,
                    triangle_count: mesh.triangle_count,
                    path_display,
                    vertex_label,
                    triangle_label,
                }
            }).collect()
        } else {
            Vec::new()
        };
        
        // Use interned strings to reduce memory allocations
        let render_data = CachedRenderData {
            stage_label: Self::intern_string(&format!("Stage: {}", metadata.stage_path)),
            total_stats_label: Self::intern_string(&format!("Total: {} vertices, {} triangles", metadata.total_vertices, metadata.total_triangles)),
            meshes_header: Self::intern_string(&format!("📦 Meshes ({})", metadata.meshes.len())),
            lights_header: Self::intern_string(&format!("💡 Lights ({})", metadata.lights.len())),
            materials_header: Self::intern_string(&format!("🎨 Materials ({})", metadata.materials.len())),
            mesh_names: metadata.meshes.iter().map(|mesh| {
                let name = mesh.prim_path.split('/').last().unwrap_or("Mesh");
                Self::intern_string(name)
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
                let name = light.prim_path.split('/').last().unwrap_or("Light");
                (Self::intern_string(icon), Self::intern_string(name))
            }).collect(),
            material_names: metadata.materials.iter().map(|material| {
                let name = material.prim_path.split('/').last().unwrap_or("Material");
                Self::intern_string(name)
            }).collect(),
            mesh_stats,
        };
        
        self.cached_render_data.insert(node_id, render_data);
    }

    /// Create or update cached render data from USD scene data
    fn update_render_cache(&mut self, node_id: NodeId, scene_data: &crate::workspaces::three_d::usd::usd_engine::USDSceneData) {
        // Calculate current hashes for change detection
        let current_stage_hash = Self::calculate_hash(&scene_data.stage_path);
        let current_meshes_hash = Self::calculate_hash(&scene_data.meshes.len());
        let current_lights_hash = Self::calculate_hash(&scene_data.lights.len());
        let current_materials_hash = Self::calculate_hash(&scene_data.materials.len());
        
        let current_mesh_hashes: Vec<u64> = scene_data.meshes.iter().map(Self::calculate_mesh_hash).collect();
        let current_light_hashes: Vec<u64> = scene_data.lights.iter().map(Self::calculate_light_hash).collect();
        let current_material_hashes: Vec<u64> = scene_data.materials.iter().map(Self::calculate_material_hash).collect();
        
        // Get or create change tracker
        let change_tracker = self.change_trackers.entry(node_id).or_default();
        
        // Check if we need any updates
        let stage_changed = change_tracker.stage_hash != current_stage_hash;
        let meshes_changed = change_tracker.meshes_hash != current_meshes_hash || 
                            change_tracker.mesh_hashes != current_mesh_hashes;
        let lights_changed = change_tracker.lights_hash != current_lights_hash ||
                            change_tracker.light_hashes != current_light_hashes;
        let materials_changed = change_tracker.materials_hash != current_materials_hash ||
                               change_tracker.material_hashes != current_material_hashes;
        
        // If nothing changed, skip update
        if !stage_changed && !meshes_changed && !lights_changed && !materials_changed {
            self.metrics.cache_hits += 1;
            return;
        }
        
        // Get existing render data or create new one
        let mut render_data = self.cached_render_data.get(&node_id).cloned().unwrap_or_else(|| {
            CachedRenderData {
                stage_label: Self::intern_string(""),
                total_stats_label: Self::intern_string(""),
                meshes_header: Self::intern_string(""),
                lights_header: Self::intern_string(""),
                materials_header: Self::intern_string(""),
                mesh_names: Vec::new(),
                light_names: Vec::new(),
                material_names: Vec::new(),
                mesh_stats: Vec::new(),
            }
        });
        
        // Update only changed sections
        if stage_changed {
            render_data.stage_label = Self::intern_string(&format!("Stage: {}", scene_data.stage_path));
        }
        
        if meshes_changed {
            let total_vertices: usize = scene_data.meshes.iter().map(|m| m.vertices.len()).sum();
            let total_triangles: usize = scene_data.meshes.iter().map(|m| m.indices.len() / 3).sum();
            
            render_data.total_stats_label = Self::intern_string(&format!("Total: {} vertices, {} triangles", total_vertices, total_triangles));
            render_data.meshes_header = Self::intern_string(&format!("📦 Meshes ({})", scene_data.meshes.len()));
            render_data.mesh_names = scene_data.meshes.iter().map(|mesh| {
                let prim_parts: Vec<&str> = mesh.prim_path.split('/').filter(|s| !s.is_empty()).collect();
                let name = prim_parts.last().unwrap_or(&"Mesh");
                Self::intern_string(name)
            }).collect();
            // We need to compute mesh stats separately to avoid borrowing issue
            let mesh_stats = Self::compute_mesh_stats_static(&scene_data.meshes);
            render_data.mesh_stats = mesh_stats;
        }
        
        if lights_changed {
            render_data.lights_header = Self::intern_string(&format!("💡 Lights ({})", scene_data.lights.len()));
            render_data.light_names = scene_data.lights.iter().map(|light| {
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
                (Self::intern_string(icon), Self::intern_string(light_name))
            }).collect();
        }
        
        if materials_changed {
            render_data.materials_header = Self::intern_string(&format!("🎨 Materials ({})", scene_data.materials.len()));
            render_data.material_names = scene_data.materials.iter().map(|mat| {
                let name = mat.prim_path.split('/').last().unwrap_or("Material");
                Self::intern_string(name)
            }).collect();
        }
        
        // Update change tracker
        change_tracker.stage_hash = current_stage_hash;
        change_tracker.meshes_hash = current_meshes_hash;
        change_tracker.lights_hash = current_lights_hash;
        change_tracker.materials_hash = current_materials_hash;
        change_tracker.mesh_hashes = current_mesh_hashes;
        change_tracker.light_hashes = current_light_hashes;
        change_tracker.material_hashes = current_material_hashes;
        
        // Store updated render data
        self.cached_render_data.insert(node_id, render_data);
        
        // Update performance metrics
        self.metrics.partial_updates += 1;
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
            ui.label(&*render_data.stage_label);
            ui.separator();
            
            // Render meshes with cached header using dynamic pagination
            if !scene_data.meshes.is_empty() {
                ui.collapsing(&*render_data.meshes_header, |ui| {
                    // Use dynamic complexity-based pagination
                    self.render_paginated_meshes_optimized(ui, node_id, &scene_data.meshes, &render_data.mesh_names);
                });
            }
            
            // Render lights with cached data
            if !scene_data.lights.is_empty() {
                ui.collapsing(&*render_data.lights_header, |ui| {
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
                ui.collapsing(&*render_data.materials_header, |ui| {
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

    /// Optimized render paginated meshes using cached names with dynamic complexity-based pagination
    fn render_paginated_meshes_optimized(&mut self, ui: &mut egui::Ui, node_id: NodeId, meshes: &[USDMeshGeometry], cached_names: &[Arc<str>]) {
        let total_meshes = meshes.len();
        
        // Calculate dynamic pagination based on mesh complexity
        let pages_data = self.calculate_dynamic_pagination(meshes);
        let total_pages = pages_data.len();
        
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
        
        // Ensure current page is within bounds
        current_page = current_page.min(total_pages - 1);
        
        // Get current page info
        let page_info = &pages_data[current_page];
        
        // Show pagination controls with complexity info
        ui.horizontal(|ui| {
            if ui.button("◀ Prev").clicked() && current_page > 0 {
                current_page -= 1;
                page_changed = true;
            }
            
            ui.label(format!("Page {} of {} ({} items, ~{}K verts)", 
                current_page + 1, total_pages, page_info.item_count, page_info.complexity_score / 1000));
            
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
        
        // Render items for current page
        for idx in page_info.start_idx..page_info.end_idx {
            if let (Some(mesh), Some(name)) = (meshes.get(idx), cached_names.get(idx)) {
                Self::render_single_mesh_optimized(ui, idx, mesh, name);
            }
        }
        
        // Show pagination info at bottom
        ui.separator();
        ui.label(format!("Showing {} - {} of {} meshes (complexity: {}K vertices)", 
            page_info.start_idx + 1, page_info.end_idx, total_meshes, page_info.complexity_score / 1000));
    }
    
    /// Render single mesh with pre-cached name
    fn render_single_mesh_optimized(ui: &mut egui::Ui, idx: usize, mesh: &USDMeshGeometry, cached_name: &Arc<str>) {
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
    
    /// Render meshes with virtual scrolling for better performance
    fn render_virtualized_meshes(&mut self, ui: &mut egui::Ui, node_id: NodeId, meshes: &[USDMeshGeometry]) {
        const ITEM_HEIGHT: f32 = 60.0; // Estimated height per mesh item
        const BUFFER_ITEMS: usize = 5; // Extra items to render above/below visible area
        
        let total_meshes = meshes.len();
        
        if total_meshes == 0 {
            ui.label("No meshes in scene");
            return;
        }
        
        // Check if we have pre-computed statistics in cache
        let use_cached_stats = self.cached_render_data.get(&node_id)
            .map(|cache| cache.mesh_stats.len() == total_meshes)
            .unwrap_or(false);
        
        // For small lists, render all items normally
        if total_meshes <= 100 {
            if use_cached_stats {
                let mesh_stats = &self.cached_render_data.get(&node_id).unwrap().mesh_stats;
                for (idx, stats) in mesh_stats.iter().enumerate() {
                    Self::render_single_mesh_with_stats(ui, idx, stats);
                }
            } else {
                for (idx, mesh) in meshes.iter().enumerate() {
                    Self::render_single_mesh_fallback(ui, idx, mesh);
                }
            }
            return;
        }
        
        // Virtual scrolling for large lists
        let available_height = ui.available_height();
        let visible_items = (available_height / ITEM_HEIGHT).ceil() as usize;
        
        // Use ScrollArea with virtual scrolling
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show_viewport(ui, |ui, viewport| {
                // Calculate which items are visible
                let scroll_top = viewport.min.y;
                let scroll_bottom = viewport.max.y;
                
                let start_idx = ((scroll_top / ITEM_HEIGHT) as usize).saturating_sub(BUFFER_ITEMS);
                let end_idx = ((scroll_bottom / ITEM_HEIGHT) as usize + BUFFER_ITEMS + 1).min(total_meshes);
                
                // Add spacing for items above visible area
                if start_idx > 0 {
                    ui.add_space(start_idx as f32 * ITEM_HEIGHT);
                }
                
                // Render only visible items with optimal method
                if use_cached_stats {
                    let mesh_stats = &self.cached_render_data.get(&node_id).unwrap().mesh_stats;
                    for idx in start_idx..end_idx {
                        if idx < mesh_stats.len() {
                            Self::render_single_mesh_with_stats(ui, idx, &mesh_stats[idx]);
                        }
                    }
                } else {
                    for idx in start_idx..end_idx {
                        if idx < meshes.len() {
                            Self::render_single_mesh_fallback(ui, idx, &meshes[idx]);
                        }
                    }
                }
                
                // Add spacing for items below visible area
                let remaining_items = total_meshes.saturating_sub(end_idx);
                if remaining_items > 0 {
                    ui.add_space(remaining_items as f32 * ITEM_HEIGHT);
                }
            });
        
        // Show total count at bottom
        ui.separator();
        ui.label(format!("Total meshes: {}", total_meshes));
    }
    
    /// Render paginated meshes for performance (fallback method)
    fn render_paginated_meshes(&mut self, ui: &mut egui::Ui, node_id: NodeId, meshes: &[USDMeshGeometry]) {
        // Use virtual scrolling instead of pagination for better performance
        self.render_virtualized_meshes(ui, node_id, meshes);
    }
    
    /// Render a single mesh item with pre-computed statistics and optimized string formatting
    fn render_single_mesh_with_stats(ui: &mut egui::Ui, idx: usize, mesh_stats: &MeshStats) {
        // Use pre-computed display name
        let header_text = format!("🔹 {}", mesh_stats.display_name);
        ui.collapsing(header_text, |ui| {
            // Use pre-formatted strings - no runtime string formatting needed
            ui.horizontal(|ui| {
                ui.label("  📍");
                ui.label(&*mesh_stats.path_display);
            });
            ui.horizontal(|ui| {
                ui.label("  🔸");
                ui.label(&*mesh_stats.vertex_label);
            });
            ui.horizontal(|ui| {
                ui.label("  🔺");
                ui.label(&*mesh_stats.triangle_label);
            });
        });
    }
    
    /// Render a single mesh item with optimized string handling (fallback)
    fn render_single_mesh_fallback(ui: &mut egui::Ui, idx: usize, mesh: &USDMeshGeometry) {
        // Use pre-computed mesh name if available, otherwise compute once
        let mesh_name = Self::get_mesh_display_name(mesh, idx);
        
        // Lazy evaluation - only compute expensive stats when expanded
        ui.collapsing(format!("🔹 {}", mesh_name), |ui| {
            // Only compute expensive operations when actually expanded
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
    
    /// Pre-compute mesh statistics for efficient rendering
    fn compute_mesh_stats(&mut self, meshes: &[USDMeshGeometry]) -> Vec<MeshStats> {
        Self::compute_mesh_stats_static(meshes)
    }
    
    /// Static version of compute_mesh_stats to avoid borrowing issues
    fn compute_mesh_stats_static(meshes: &[USDMeshGeometry]) -> Vec<MeshStats> {
        let mut stats = Vec::with_capacity(meshes.len());
        
        if let Ok(mut interner) = STRING_INTERNER.lock() {
            for (idx, mesh) in meshes.iter().enumerate() {
                let display_name = Self::get_mesh_display_name(mesh, idx);
                let display_name_arc = interner.intern(&display_name);
                let path_display_arc = interner.intern(&mesh.prim_path);
                
                let vertex_count = mesh.vertices.len();
                let triangle_count = mesh.indices.len() / 3;
                
                // Pre-format commonly used strings to avoid formatting during rendering
                let vertex_label = interner.intern(&format!("Vertices: {}", vertex_count));
                let triangle_label = interner.intern(&format!("Triangles: {}", triangle_count));
                
                stats.push(MeshStats {
                    display_name: display_name_arc,
                    vertex_count,
                    triangle_count,
                    path_display: path_display_arc,
                    vertex_label,
                    triangle_label,
                });
            }
        }
        
        stats
    }
    
    /// Calculate dynamic pagination based on mesh complexity
    /// Groups meshes into pages based on total vertex count rather than fixed item count
    fn calculate_dynamic_pagination(&self, meshes: &[USDMeshGeometry]) -> Vec<PageInfo> {
        if meshes.is_empty() {
            return vec![];
        }
        
        // Define complexity thresholds
        const MAX_COMPLEXITY_PER_PAGE: usize = 50_000; // Maximum vertices per page
        const MIN_ITEMS_PER_PAGE: usize = 5; // Minimum items per page (prevent single huge mesh pages)
        const MAX_ITEMS_PER_PAGE: usize = 100; // Maximum items per page (prevent too many small meshes)
        
        let mut pages = Vec::new();
        let mut current_page_start = 0;
        let mut current_complexity = 0;
        let mut current_item_count = 0;
        
        for (idx, mesh) in meshes.iter().enumerate() {
            let mesh_complexity = mesh.vertices.len();
            let would_exceed_complexity = current_complexity + mesh_complexity > MAX_COMPLEXITY_PER_PAGE;
            let would_exceed_max_items = current_item_count >= MAX_ITEMS_PER_PAGE;
            let has_minimum_items = current_item_count >= MIN_ITEMS_PER_PAGE;
            
            // Start new page if:
            // 1. Adding this mesh would exceed complexity budget AND we have minimum items
            // 2. We've reached maximum items per page
            if (would_exceed_complexity && has_minimum_items) || would_exceed_max_items {
                // Close current page
                pages.push(PageInfo {
                    start_idx: current_page_start,
                    end_idx: idx,
                    item_count: current_item_count,
                    complexity_score: current_complexity,
                });
                
                // Start new page
                current_page_start = idx;
                current_complexity = mesh_complexity;
                current_item_count = 1;
            } else {
                // Add to current page
                current_complexity += mesh_complexity;
                current_item_count += 1;
            }
        }
        
        // Add final page
        if current_item_count > 0 {
            pages.push(PageInfo {
                start_idx: current_page_start,
                end_idx: meshes.len(),
                item_count: current_item_count,
                complexity_score: current_complexity,
            });
        }
        
        // Fallback to single page if no pages created
        if pages.is_empty() {
            pages.push(PageInfo {
                start_idx: 0,
                end_idx: meshes.len(),
                item_count: meshes.len(),
                complexity_score: meshes.iter().map(|m| m.vertices.len()).sum(),
            });
        }
        
        pages
    }
    
    /// Calculate hash for change detection
    fn calculate_hash<T: std::hash::Hash>(data: &T) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Calculate hash for mesh data
    fn calculate_mesh_hash(mesh: &USDMeshGeometry) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        mesh.prim_path.hash(&mut hasher);
        mesh.vertices.len().hash(&mut hasher);
        mesh.indices.len().hash(&mut hasher);
        // Hash first few vertices for content change detection (using component values)
        if !mesh.vertices.is_empty() {
            let sample_count = mesh.vertices.len().min(10);
            for vertex in &mesh.vertices[..sample_count] {
                vertex.x.to_bits().hash(&mut hasher);
                vertex.y.to_bits().hash(&mut hasher);
                vertex.z.to_bits().hash(&mut hasher);
            }
        }
        // Hash some indices for additional content validation
        if !mesh.indices.is_empty() {
            let sample_count = mesh.indices.len().min(20);
            for index in &mesh.indices[..sample_count] {
                index.hash(&mut hasher);
            }
        }
        hasher.finish()
    }
    
    /// Calculate hash for light data
    fn calculate_light_hash(light: &crate::workspaces::three_d::usd::usd_engine::USDLightData) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        light.prim_path.hash(&mut hasher);
        light.light_type.hash(&mut hasher);
        light.intensity.to_bits().hash(&mut hasher);
        hasher.finish()
    }
    
    /// Calculate hash for material data
    fn calculate_material_hash(material: &crate::workspaces::three_d::usd::usd_engine::USDMaterialData) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        material.prim_path.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Get optimized mesh display name (could be cached in future)
    fn get_mesh_display_name(mesh: &USDMeshGeometry, idx: usize) -> String {
        // Extract mesh name from path efficiently
        if let Some(last_slash) = mesh.prim_path.rfind('/') {
            if last_slash + 1 < mesh.prim_path.len() {
                return mesh.prim_path[last_slash + 1..].to_string();
            }
        }
        format!("Mesh_{}", idx)
    }
    
    /// Render a single mesh item (static version to avoid borrowing issues)
    fn render_single_mesh_static(ui: &mut egui::Ui, idx: usize, mesh: &USDMeshGeometry) {
        // Delegate to fallback version
        Self::render_single_mesh_fallback(ui, idx, mesh);
    }

    /// Update cache if needed - separate method to avoid borrowing conflicts
    fn update_cache_if_needed(&mut self, node_id: NodeId) {
        let start_time = Instant::now();
        
        // Check if we need to update our local cache
        if let Ok(global_cache) = crate::nodes::three_d::ui::scenegraph::logic::SCENEGRAPH_INPUT_CACHE.read() {
            if let Some(cached_scenegraph_data) = global_cache.get(&node_id) {
                let current_version = cached_scenegraph_data.version;
                let last_known_version = self.last_cache_versions.get(&node_id).copied().unwrap_or(0);
                
                // Only update local cache if the global cache has newer data
                if current_version > last_known_version {
                    debug!("🌳 Tree panel updating cache for node {} (version {} -> {})", 
                           node_id, last_known_version, current_version);
                    self.cached_data.insert(node_id, (cached_scenegraph_data.data.clone(), current_version));
                    self.last_cache_versions.insert(node_id, current_version);
                    
                    // Track change frequency for adaptive throttling
                    let change_count = self.change_frequency.entry(node_id).or_insert(0);
                    *change_count += 1;
                    
                    // Update metrics
                    self.metrics.cache_updates += 1;
                    self.metrics.cache_misses += 1;
                    
                    // Clear render cache so it gets regenerated with new data
                    self.cached_render_data.remove(&node_id);
                } else {
                    // Cache hit
                    self.metrics.cache_hits += 1;
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
        
        // Update timing metrics
        let lookup_time = start_time.elapsed();
        self.metrics.avg_lookup_time = 
            (self.metrics.avg_lookup_time + lookup_time) / 2;
        
        // Update cache hit rate
        let total_ops = self.metrics.cache_hits + self.metrics.cache_misses;
        if total_ops > 0 {
            self.metrics.cache_hit_rate = self.metrics.cache_hits as f64 / total_ops as f64;
        }
        self.metrics.last_updated = Instant::now();
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
        self.adaptive_intervals.remove(&node_id);
        self.change_frequency.remove(&node_id);
        debug!("🧹 Tree panel cleanup completed for deleted node: {}", node_id);
    }
    
    /// Get performance metrics for debugging and optimization
    pub fn get_metrics(&self) -> &ScenegraphMetrics {
        &self.metrics
    }
    
    /// Reset performance metrics
    pub fn reset_metrics(&mut self) {
        self.metrics = ScenegraphMetrics::default();
    }
}

impl Default for TreePanel {
    fn default() -> Self {
        Self::new()
    }
}