//! Viewport node implementation with complete USD viewport functionality
//! Copied from USD plugin and adapted for core integration

use crate::nodes::interface::{NodeData, ParameterChange, PanelType};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory};
use egui::Ui;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use crate::viewport::*;
use super::logic::USDViewportLogic;
use super::usd_rendering::USDRenderer;
use glam::{Mat4, Vec3};

/// Global cache for viewport input data to bridge process_node and get_viewport_data
pub static VIEWPORT_INPUT_CACHE: Lazy<Arc<Mutex<HashMap<crate::nodes::NodeId, crate::workspaces::three_d::usd::usd_engine::USDSceneData>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

/// Cache for converted viewport data to avoid expensive conversion every frame
pub static VIEWPORT_DATA_CACHE: Lazy<Arc<Mutex<HashMap<crate::nodes::NodeId, (ViewportData, u64)>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

/// Global flag to force viewport cache bypass when parameters change
pub static FORCE_VIEWPORT_REFRESH: Lazy<Arc<Mutex<HashSet<crate::nodes::NodeId>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashSet::new()))
});

/// Calculate hash for USD scene data to detect changes - includes source node ID
fn calculate_usd_scene_hash(usd_scene_data: &crate::workspaces::three_d::usd::usd_engine::USDSceneData, source_node_id: crate::nodes::NodeId) -> u64 {
    let mut hasher = DefaultHasher::new();
    
    // CRITICAL: Include source node ID in hash so data from different nodes is always different
    source_node_id.hash(&mut hasher);
    
    usd_scene_data.stage_path.hash(&mut hasher);
    usd_scene_data.meshes.len().hash(&mut hasher);
    
    // Hash first few vertices for more detailed change detection
    for (i, mesh) in usd_scene_data.meshes.iter().enumerate().take(5) {
        mesh.prim_path.hash(&mut hasher);
        mesh.vertices.len().hash(&mut hasher);
        for vertex in mesh.vertices.iter().take(10) {
            vertex.x.to_bits().hash(&mut hasher);
            vertex.y.to_bits().hash(&mut hasher);
            vertex.z.to_bits().hash(&mut hasher);
        }
        if i >= 5 { break; }
    }
    
    hasher.finish()
}

/// Cache for USD renderers to avoid reloading stages every frame
pub struct USDRendererCache {
    pub renderers: HashMap<String, (USDRenderer, ViewportData)>,
    pub scene_bounds: HashMap<String, Option<(Vec3, Vec3)>>,  // Cache for scene bounds
}

impl USDRendererCache {
    fn new() -> Self {
        Self {
            renderers: HashMap::new(),
            scene_bounds: HashMap::new(),
        }
    }
    
    /// Calculate scene bounds from USD geometries - no transforms, with caching
    fn calculate_scene_bounds(&mut self, usd_renderer: &USDRenderer, stage_path: &str) -> Option<(Vec3, Vec3)> {
        // Check if bounds are already cached
        if let Some(cached_bounds) = self.scene_bounds.get(stage_path) {
            return *cached_bounds;
        }
        
        if usd_renderer.current_scene.geometries.is_empty() {
            self.scene_bounds.insert(stage_path.to_string(), None);
            return None;
        }
        
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        
        // Calculate bounds using raw USD coordinates - no transforms at all
        for geometry in &usd_renderer.current_scene.geometries {
            for vertex in &geometry.vertices {
                // Use raw vertex position - no transforms
                let vertex_pos: Vec3 = vertex.position.into();
                min = min.min(vertex_pos);
                max = max.max(vertex_pos);
            }
        }
        
        let bounds = Some((min, max));
        self.scene_bounds.insert(stage_path.to_string(), bounds);
        bounds
    }
    
    
    
    
    fn get_or_load(&mut self, stage_path: &str) -> &ViewportData {
        if !self.renderers.contains_key(stage_path) {
            // println!("📦 Cache miss: Loading USD stage '{}'", stage_path); // Removed: called during loading
            
            // Create and load USD renderer
            let mut usd_renderer = USDRenderer::new();
            if let Err(e) = usd_renderer.load_stage(stage_path) {
                println!("Failed to load USD stage: {}", e);
            }
            
            // Convert to viewport data
            let viewport_data = self.convert_to_viewport_data(&usd_renderer, stage_path);
            
            self.renderers.insert(stage_path.to_string(), (usd_renderer, viewport_data));
        } else {
            // println!("✅ Cache hit: Using cached USD stage '{}'", stage_path); // Removed: called frequently
        }
        
        &self.renderers.get(stage_path).unwrap().1
    }
    
    /// Calculate optimal camera position based on scene bounds
    fn calculate_camera_position(bounds: (Vec3, Vec3)) -> (Vec3, Vec3, f32) {
        let (min, max) = bounds;
        let center = (min + max) * 0.5;
        let size = max - min;
        let max_dimension = size.x.max(size.y).max(size.z);
        
        // println!("📐 Scene center: {:?}, size: {:?}, max_dim: {}", center, size, max_dimension); // Removed: called every frame
        
        // Position camera to see the whole scene
        let camera_distance = max_dimension * 1.5;
        let camera_position = center + Vec3::new(
            camera_distance * 0.7,  // Slight angle
            camera_distance * 0.7,  // Above the scene
            camera_distance * 0.7,  // Away from scene
        );
        
        // Calculate far plane based on scene size
        let far_plane = max_dimension * 3.0; // Ensure we can see the whole scene
        
        // println!("📷 Camera position: {:?}, target: {:?}", camera_position, center); // Removed: called every frame
        (camera_position, center, far_plane)
    }

    fn convert_to_viewport_data(&mut self, usd_renderer: &USDRenderer, stage_path: &str) -> ViewportData {
        // Convert USD scene to SDK ViewportData format
        let mut scene = SceneData::default();
        scene.name = format!("USD Stage: {}", stage_path);
        
        // Calculate scene bounds - no transforms, with caching
        let scene_bounds = self.calculate_scene_bounds(usd_renderer, stage_path);
        
        // Calculate camera position based on scene bounds
        let (camera_position, camera_target, far_plane) = if let Some(bounds) = scene_bounds {
            Self::calculate_camera_position(bounds)
        } else {
            (Vec3::new(5.0, 5.0, 5.0), Vec3::ZERO, 100.0)
        };
        
        // Calculate scene size for adaptive sensitivity
        let scene_size = if let Some(bounds) = scene_bounds {
            let size = bounds.1 - bounds.0;
            size.x.max(size.y).max(size.z)
        } else {
            10.0 // Default reference size
        };
        
        scene.camera = CameraData {
            position: camera_position.into(),
            target: camera_target.into(),
            up: [0.0, 1.0, 0.0],  // Default Y-up viewport
            fov: 45.0_f32.to_radians(),
            near: 0.1,
            far: far_plane,  // Calculated based on scene size
            aspect: 800.0 / 600.0,
        };
        
        // Convert USD geometries to SDK mesh format - no transforms
        for usd_geometry in &usd_renderer.current_scene.geometries {
            // Use identity transform - no scaling, no coordinate conversion
            let final_transform = Mat4::IDENTITY;
            
            let mesh_data = MeshData {
                id: usd_geometry.prim_path.clone(),
                vertices: usd_geometry.vertices.iter().flat_map(|v| v.position.iter().cloned()).collect(),
                normals: usd_geometry.vertices.iter().flat_map(|v| v.normal.iter().cloned()).collect(),
                uvs: usd_geometry.vertices.iter().flat_map(|v| v.uv.iter().cloned()).collect(),
                indices: usd_geometry.indices.clone(),
                vertex_colors: Some(usd_geometry.vertices.iter().flat_map(|v| v.color.iter().cloned()).collect()),
                material_id: usd_geometry.material_path.clone(),
                transform: final_transform.to_cols_array_2d(),
            };
            scene.meshes.push(mesh_data);
        }
        
        // Convert USD materials to SDK format
        for (material_id, usd_material) in &usd_renderer.current_scene.materials {
            let material_data = MaterialData {
                id: material_id.clone(),
                name: usd_material.prim_path.clone(),
                base_color: [usd_material.diffuse_color.x, usd_material.diffuse_color.y, usd_material.diffuse_color.z, usd_material.opacity],
                metallic: usd_material.metallic,
                roughness: usd_material.roughness,
                emission: [usd_material.emission_color.x, usd_material.emission_color.y, usd_material.emission_color.z],
                diffuse_texture: None,
                normal_texture: None,
                roughness_texture: None,
                metallic_texture: None,
            };
            scene.materials.push(material_data);
        }
        
        // Convert USD lights to SDK format
        for usd_light in &usd_renderer.current_scene.lights {
            let light_data = LightData {
                id: usd_light.prim_path.clone(),
                light_type: match usd_light.light_type.as_str() {
                    "distant" => LightType::Directional,
                    "sphere" => LightType::Point,
                    _ => LightType::Directional,
                },
                position: [
                    usd_light.transform.w_axis.x,
                    usd_light.transform.w_axis.y,
                    usd_light.transform.w_axis.z
                ],
                direction: [
                    -usd_light.transform.z_axis.x,
                    -usd_light.transform.z_axis.y,
                    -usd_light.transform.z_axis.z
                ],
                color: [usd_light.color.x, usd_light.color.y, usd_light.color.z],
                intensity: usd_light.intensity,
                range: 100.0,
                spot_angle: 0.0,
            };
            scene.lights.push(light_data);
        }
        
        // Set scene bounding box based on raw USD coordinates
        if !scene.meshes.is_empty() {
            if let Some(bounds) = scene_bounds {
                // Use raw USD bounds - no transforms
                scene.bounding_box = Some((bounds.0.into(), bounds.1.into()));
            } else {
                scene.bounding_box = Some(([-5.0, -5.0, -5.0], [5.0, 5.0, 5.0]));
            }
        }
        
        // Create viewport data with default settings
        let mut viewport_data = ViewportData {
            scene,
            dimensions: (800, 600),
            scene_dirty: true,
            settings: ViewportSettings {
                background_color: [0.2, 0.2, 0.2, 1.0],
                wireframe: false,
                lighting: true,
                show_grid: true,
                show_ground_plane: false,
                aa_samples: 4,
                shading_mode: ShadingMode::Smooth,
            },
            settings_dirty: false,
        };
        
        // Store scene size for adaptive sensitivity in scene metadata
        // We'll use the scene name to encode the scene size for now
        viewport_data.scene.name = format!("USD Stage: {} (size: {:.1})", stage_path, scene_size);
        
        viewport_data
    }
}

// Global cache for USD renderers
pub static USD_RENDERER_CACHE: Lazy<Arc<Mutex<USDRendererCache>>> = Lazy::new(|| {
    Arc::new(Mutex::new(USDRendererCache::new()))
});

/// Core viewport instance that holds state and provides 3D rendering
#[derive(Debug)]
pub struct ViewportInstance {
    pub current_stage: String,
    pub viewport_data: ViewportData,
    pub camera_settings: CameraSettings,
    pub viewport_logic: USDViewportLogic,
}

impl Clone for ViewportInstance {
    fn clone(&self) -> Self {
        Self {
            current_stage: self.current_stage.clone(),
            viewport_data: self.viewport_data.clone(),
            camera_settings: self.camera_settings.clone(),
            viewport_logic: self.viewport_logic.clone(),
        }
    }
}

/// Core viewport node struct - stores scene and camera data
#[derive(Debug)]
pub struct ViewportNode {
    pub current_stage: String,
    pub viewport_data: ViewportData,
    pub camera_settings: CameraSettings,
    pub viewport_logic: USDViewportLogic,
}

impl Clone for ViewportNode {
    fn clone(&self) -> Self {
        Self {
            current_stage: self.current_stage.clone(),
            viewport_data: self.viewport_data.clone(),
            camera_settings: self.camera_settings.clone(),
            viewport_logic: self.viewport_logic.clone(),
        }
    }
}

/// Camera settings for viewport navigation
#[derive(Debug, Clone)]
pub struct CameraSettings {
    pub orbit_sensitivity: f32,
    pub pan_sensitivity: f32,
    pub zoom_sensitivity: f32,
    pub scene_size: f32,  // For adaptive sensitivity calculation
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            orbit_sensitivity: 0.5,
            pan_sensitivity: 1.0,
            zoom_sensitivity: 1.0,
            scene_size: 10.0,  // Default reference size
        }
    }
}

impl Default for ViewportNode {
    fn default() -> Self {
        let viewport_logic = USDViewportLogic::default();
        // Don't load any test stage - wait for input connection
        
        Self {
            current_stage: "".to_string(), // No default stage
            viewport_data: ViewportData::default(),
            camera_settings: CameraSettings::default(),
            viewport_logic,
        }
    }
}

impl ViewportNode {
    /// Update scene size in camera settings based on current USD input data
    pub fn update_scene_size(&mut self, node: &Node) {
        // Scene size is now calculated from input data only
        // This method is kept for compatibility but does nothing
        // Scene size is automatically calculated during USD data conversion
    }
    
    /// Calculate adaptive sensitivity based on scene scale and camera distance
    fn calculate_adaptive_sensitivity(
        camera_pos: [f32; 3],
        target_pos: [f32; 3],
        scene_size: f32,
        base_orbit: f32,
        base_pan: f32,
        base_zoom: f32,
    ) -> (f32, f32, f32) {
        // Calculate distance from camera to target
        let distance = ((camera_pos[0] - target_pos[0]).powi(2) + 
                       (camera_pos[1] - target_pos[1]).powi(2) + 
                       (camera_pos[2] - target_pos[2]).powi(2)).sqrt();
        
        // Scene scale factor (how much bigger/smaller than reference 10-unit scene)
        let scene_scale_factor = scene_size / 10.0;
        
        // Distance factor (how far camera is relative to scene size)
        let distance_factor = distance / scene_size;
        
        // Adaptive sensitivity calculations
        let orbit_sensitivity = base_orbit * scene_scale_factor * distance_factor;
        let pan_sensitivity = base_pan * scene_scale_factor * distance_factor;
        let zoom_sensitivity = base_zoom * scene_scale_factor;
        
        // println!("🎯 Adaptive sensitivity - scene_size: {:.1}, distance: {:.1}, scale_factor: {:.3}, distance_factor: {:.3}", 
        //          scene_size, distance, scene_scale_factor, distance_factor);
        // println!("🎯 Calculated sensitivity - orbit: {:.4}, pan: {:.4}, zoom: {:.4}", 
        //          orbit_sensitivity, pan_sensitivity, zoom_sensitivity);
        // Removed: called every frame during camera movement
        
        (orbit_sensitivity, pan_sensitivity, zoom_sensitivity)
    }

    /// Load USD stage and convert to scene data
    pub fn load_stage(&mut self, stage_path: &str) {
        println!("Core Viewport: Loading stage: {}", stage_path);
        
        // TODO: Implement actual USD stage loading
        // For now, create a simple test scene
        let mut scene = SceneData::default();
        scene.name = format!("USD Stage: {}", stage_path);
        
        // Set up proper camera with default position matching USD viewport
        scene.camera = CameraData {
            position: [5.0, 5.0, 5.0],
            target: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            fov: 45.0_f32.to_radians(),
            near: 0.1,
            far: 100.0,
            aspect: 800.0 / 600.0,
        };
        
        // Create a simple cube mesh as placeholder
        let cube_mesh = MeshData {
            id: "cube".to_string(),
            vertices: vec![
                // Front face
                -1.0, -1.0,  1.0,
                 1.0, -1.0,  1.0,
                 1.0,  1.0,  1.0,
                -1.0,  1.0,  1.0,
                // Back face
                -1.0, -1.0, -1.0,
                -1.0,  1.0, -1.0,
                 1.0,  1.0, -1.0,
                 1.0, -1.0, -1.0,
            ],
            normals: vec![
                // Front face
                0.0, 0.0, 1.0,
                0.0, 0.0, 1.0,
                0.0, 0.0, 1.0,
                0.0, 0.0, 1.0,
                // Back face
                0.0, 0.0, -1.0,
                0.0, 0.0, -1.0,
                0.0, 0.0, -1.0,
                0.0, 0.0, -1.0,
            ],
            uvs: vec![
                0.0, 0.0,
                1.0, 0.0,
                1.0, 1.0,
                0.0, 1.0,
                0.0, 0.0,
                1.0, 0.0,
                1.0, 1.0,
                0.0, 1.0,
            ],
            indices: vec![
                // Front face
                0, 1, 2,   2, 3, 0,
                // Back face
                4, 5, 6,   6, 7, 4,
                // Top face
                3, 2, 6,   6, 5, 3,
                // Bottom face
                0, 4, 7,   7, 1, 0,
                // Right face
                1, 7, 6,   6, 2, 1,
                // Left face
                4, 0, 3,   3, 5, 4,
            ],
            vertex_colors: Some(vec![
                // Front face
                1.0, 0.0, 0.0,  0.0, 1.0, 0.0,  0.0, 0.0, 1.0,  1.0, 1.0, 0.0,
                // Back face
                0.0, 1.0, 1.0,  1.0, 0.0, 1.0,  0.5, 0.5, 0.5,  1.0, 0.5, 0.0,
            ]),
            material_id: Some("usd_material".to_string()),
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };
        
        scene.meshes.push(cube_mesh);
        
        // Create a simple material
        let material = MaterialData {
            id: "usd_material".to_string(),
            name: "USD Material".to_string(),
            base_color: [0.7, 0.7, 0.9, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            emission: [0.0, 0.0, 0.0],
            diffuse_texture: None,
            normal_texture: None,
            roughness_texture: None,
            metallic_texture: None,
        };
        
        scene.materials.push(material);
        
        // Add a simple directional light
        let light = LightData {
            id: "sun".to_string(),
            light_type: LightType::Directional,
            position: [0.0, 10.0, 5.0],
            direction: [-0.5, -1.0, -0.5],
            color: [1.0, 1.0, 0.9],
            intensity: 5.0,
            range: 100.0,
            spot_angle: 0.0,
        };
        
        scene.lights.push(light);
        
        // Set scene bounding box
        scene.bounding_box = Some(([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]));
        
        self.viewport_data.scene = scene;
        self.viewport_data.scene_dirty = true;
        self.current_stage = stage_path.to_string();
    }
    
    /// Handle viewport input events (mouse, keyboard, etc.) for core viewports
    pub fn handle_viewport_input(&mut self, ui: &egui::Ui, response: &egui::Response, callback: &mut crate::gpu::viewport_3d_callback::ViewportRenderCallback) {
        // Handle mouse interactions for camera control - Maya-style navigation
        if response.dragged() {
            let delta = response.drag_delta();
            let ctx = ui.ctx();
            let modifiers = ctx.input(|i| i.modifiers);
            let pointer_button = ctx.input(|i| {
                if i.pointer.primary_down() {
                    Some(egui::PointerButton::Primary)  // Left mouse button
                } else if i.pointer.middle_down() {
                    Some(egui::PointerButton::Middle)   // Middle mouse button  
                } else if i.pointer.secondary_down() {
                    Some(egui::PointerButton::Secondary) // Right mouse button
                } else {
                    None
                }
            });
            
            // Maya-style camera navigation: Alt + mouse button combinations
            if modifiers.alt {
                let manipulation = match pointer_button {
                    Some(egui::PointerButton::Primary) => {
                        // Alt + Left Mouse = Orbit (invert Y for natural feel)
                        Some(CameraManipulation::Orbit {
                            delta_x: delta.x * 0.01, // TODO: Use constants
                            delta_y: -delta.y * 0.01,
                        })
                    }
                    Some(egui::PointerButton::Middle) => {
                        // Alt + Middle Mouse = Pan (invert for natural feel)
                        Some(CameraManipulation::Pan {
                            delta_x: -delta.x * 0.01,
                            delta_y: delta.y * 0.01,
                        })
                    }
                    Some(egui::PointerButton::Secondary) => {
                        // Alt + Right Mouse = Zoom (invert for natural feel)
                        Some(CameraManipulation::Zoom {
                            delta: delta.y * 0.01,
                        })
                    }
                    _ => None,
                };
                
                if let Some(manip) = manipulation {
                    // Apply to viewport node camera
                    self.handle_camera_manipulation(manip.clone());
                    
                    // Also apply to callback for immediate rendering
                    match manip {
                        CameraManipulation::Orbit { delta_x, delta_y } => {
                            callback.handle_camera_manipulation(
                                delta_x, delta_y, 
                                crate::gpu::viewport_3d_callback::CameraManipulationType::Orbit
                            );
                        }
                        CameraManipulation::Pan { delta_x, delta_y } => {
                            callback.handle_camera_manipulation(
                                delta_x, delta_y, 
                                crate::gpu::viewport_3d_callback::CameraManipulationType::Pan
                            );
                        }
                        CameraManipulation::Zoom { delta } => {
                            callback.handle_camera_manipulation(
                                delta, 0.0, 
                                crate::gpu::viewport_3d_callback::CameraManipulationType::Zoom
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // Handle scroll for zoom
        if response.hovered() {
            let ctx = ui.ctx();
            ctx.input(|i| {
                if i.raw_scroll_delta.y != 0.0 {
                    let zoom_manip = CameraManipulation::Zoom {
                        delta: -i.raw_scroll_delta.y * 0.1, // TODO: Use constants
                    };
                    
                    // Apply to viewport node camera
                    self.handle_camera_manipulation(zoom_manip);
                    
                    // Also apply to callback for immediate rendering
                    callback.handle_camera_manipulation(
                        -i.raw_scroll_delta.y * 0.1, 0.0,
                        crate::gpu::viewport_3d_callback::CameraManipulationType::Zoom
                    );
                }
                
                // Handle F key for framing - only when viewport is focused
                if i.key_pressed(egui::Key::F) {
                    // TODO: In the future, check if geometry is selected and frame that instead
                    // let selected_bounds = self.get_selected_geometry_bounds(); // Future implementation
                    let selected_bounds = None; // No selection system yet
                    
                    // Frame the scene or selected geometry
                    callback.frame_scene(selected_bounds);
                    
                    if selected_bounds.is_some() {
                        println!("🎯 F key: Framed selected geometry");
                    } else {
                        println!("🎯 F key: Framed entire scene");
                    }
                }
            });
        }
    }

    /// Handle viewport input events for plugin viewports
    pub fn handle_plugin_viewport_input(&mut self, ui: &egui::Ui, response: &egui::Response, callback: &mut crate::gpu::viewport_3d_callback::ViewportRenderCallback, plugin_node: &mut dyn nodle_plugin_sdk::PluginNode) {
        // Handle mouse interactions for camera control - Maya-style navigation
        if response.dragged() {
            let delta = response.drag_delta();
            let ctx = ui.ctx();
            let modifiers = ctx.input(|i| i.modifiers);
            let pointer_button = ctx.input(|i| {
                if i.pointer.primary_down() {
                    Some(egui::PointerButton::Primary)  // Left mouse button
                } else if i.pointer.middle_down() {
                    Some(egui::PointerButton::Middle)   // Middle mouse button  
                } else if i.pointer.secondary_down() {
                    Some(egui::PointerButton::Secondary) // Right mouse button
                } else {
                    None
                }
            });
            
            // Maya-style camera navigation: Alt + mouse button combinations
            let manipulation = if modifiers.alt {
                match pointer_button {
                    Some(egui::PointerButton::Primary) => {
                        // Alt + Left Mouse = Orbit (invert Y for natural feel)
                        Some(CameraManipulation::Orbit {
                            delta_x: delta.x * crate::constants::camera::DEFAULT_DRAG_SENSITIVITY,
                            delta_y: -delta.y * crate::constants::camera::DEFAULT_DRAG_SENSITIVITY,
                        })
                    }
                    Some(egui::PointerButton::Middle) => {
                        // Alt + Middle Mouse = Pan (invert for natural feel)
                        Some(CameraManipulation::Pan {
                            delta_x: -delta.x * crate::constants::camera::DEFAULT_DRAG_SENSITIVITY,
                            delta_y: delta.y * crate::constants::camera::DEFAULT_DRAG_SENSITIVITY,
                        })
                    }
                    Some(egui::PointerButton::Secondary) => {
                        // Alt + Right Mouse = Zoom (invert for natural feel)
                        Some(CameraManipulation::Zoom {
                            delta: delta.y * crate::constants::camera::DEFAULT_DRAG_SENSITIVITY,
                        })
                    }
                    Some(egui::PointerButton::Extra1) | Some(egui::PointerButton::Extra2) => None,
                    None => None,
                }
            } else {
                // No navigation without Alt key
                None
            };
            
            // Only process manipulation if Alt was held
            if let Some(manipulation) = manipulation {
                // Send manipulation to plugin node to update its camera state
                // Convert core manipulation to plugin manipulation using conversion layer
                let plugin_manipulation: nodle_plugin_sdk::viewport::CameraManipulation = manipulation.clone().into();
                plugin_node.handle_viewport_camera(plugin_manipulation);
                
                // Also update the callback for immediate rendering
                match manipulation {
                    CameraManipulation::Orbit { delta_x, delta_y } => {
                        callback.handle_camera_manipulation(
                            delta_x, 
                            delta_y, 
                            crate::gpu::viewport_3d_callback::CameraManipulationType::Orbit
                        );
                    }
                    CameraManipulation::Pan { delta_x, delta_y } => {
                        callback.handle_camera_manipulation(
                            delta_x, 
                            delta_y, 
                            crate::gpu::viewport_3d_callback::CameraManipulationType::Pan
                        );
                    }
                    CameraManipulation::Zoom { delta } => {
                        callback.handle_camera_manipulation(
                            delta, 
                            0.0, 
                            crate::gpu::viewport_3d_callback::CameraManipulationType::Zoom
                        );
                    }
                    _ => {}
                }
            }
        }
        
        // Handle scroll for zoom
        if response.hovered() {
            let ctx = ui.ctx();
            ctx.input(|i| {
                if i.raw_scroll_delta.y != 0.0 {
                    // Create zoom manipulation for plugin (invert for natural feel)
                    let zoom_manipulation = CameraManipulation::Zoom {
                        delta: -i.raw_scroll_delta.y * crate::constants::camera::DEFAULT_SCROLL_SENSITIVITY,
                    };
                    
                    // Send to plugin node
                    // Convert core manipulation to plugin manipulation using conversion layer
                    let plugin_manipulation: nodle_plugin_sdk::viewport::CameraManipulation = zoom_manipulation.into();
                    plugin_node.handle_viewport_camera(plugin_manipulation);
                    
                    // Also update callback for immediate rendering (invert for natural feel)
                    callback.handle_camera_manipulation(
                        -i.raw_scroll_delta.y * crate::constants::camera::DEFAULT_SCROLL_SENSITIVITY, 
                        0.0, 
                        crate::gpu::viewport_3d_callback::CameraManipulationType::Zoom
                    );
                }
                
                // Handle F key for framing - only when viewport is focused
                if i.key_pressed(egui::Key::F) {
                    // TODO: In the future, check if geometry is selected and frame that instead
                    // let selected_bounds = get_selected_geometry_bounds(); // Future implementation
                    let selected_bounds = None; // No selection system yet
                    
                    // Frame the scene or selected geometry
                    callback.frame_scene(selected_bounds);
                    
                    if selected_bounds.is_some() {
                        println!("🎯 F key: Framed selected geometry");
                    } else {
                        println!("🎯 F key: Framed entire scene");
                    }
                }
            });
        }
    }
    
    /// Handle camera manipulation with USD-specific behavior
    pub fn handle_camera_manipulation(&mut self, manipulation: CameraManipulation) {
        let camera = &mut self.viewport_data.scene.camera;
        
        // Calculate adaptive sensitivity based on current camera state
        let (adaptive_orbit, adaptive_pan, adaptive_zoom) = Self::calculate_adaptive_sensitivity(
            camera.position,
            camera.target,
            self.camera_settings.scene_size,
            self.camera_settings.orbit_sensitivity,
            self.camera_settings.pan_sensitivity,
            self.camera_settings.zoom_sensitivity,
        );
        
        match manipulation {
            CameraManipulation::Orbit { delta_x, delta_y } => {
                let radius = ((camera.position[0] - camera.target[0]).powi(2) + 
                             (camera.position[1] - camera.target[1]).powi(2) + 
                             (camera.position[2] - camera.target[2]).powi(2)).sqrt();
                
                // Convert to spherical coordinates
                let mut theta = (camera.position[2] - camera.target[2]).atan2(camera.position[0] - camera.target[0]);
                let mut phi = ((camera.position[1] - camera.target[1]) / radius).asin();
                
                // Apply orbit deltas using adaptive sensitivity
                theta += delta_x * adaptive_orbit;
                phi += delta_y * adaptive_orbit;
                
                // Clamp phi to prevent gimbal lock
                phi = phi.clamp(-std::f32::consts::PI * 0.49, std::f32::consts::PI * 0.49);
                
                // Convert back to Cartesian
                camera.position[0] = camera.target[0] + radius * phi.cos() * theta.cos();
                camera.position[1] = camera.target[1] + radius * phi.sin();
                camera.position[2] = camera.target[2] + radius * phi.cos() * theta.sin();
            }
            CameraManipulation::Pan { delta_x, delta_y } => {
                // Calculate camera right and up vectors
                let forward = [
                    camera.target[0] - camera.position[0],
                    camera.target[1] - camera.position[1],
                    camera.target[2] - camera.position[2]
                ];
                let right = [
                    forward[1] * camera.up[2] - forward[2] * camera.up[1],
                    forward[2] * camera.up[0] - forward[0] * camera.up[2],
                    forward[0] * camera.up[1] - forward[1] * camera.up[0]
                ];
                
                // Pan both position and target using adaptive sensitivity
                let pan_x = delta_x * adaptive_pan;
                let pan_y = delta_y * adaptive_pan;
                
                for i in 0..3 {
                    camera.position[i] += right[i] * pan_x + camera.up[i] * pan_y;
                    camera.target[i] += right[i] * pan_x + camera.up[i] * pan_y;
                }
            }
            CameraManipulation::Zoom { delta } => {
                let direction = [
                    camera.target[0] - camera.position[0],
                    camera.target[1] - camera.position[1],
                    camera.target[2] - camera.position[2]
                ];
                
                let zoom_factor = delta * adaptive_zoom;
                
                for i in 0..3 {
                    camera.position[i] += direction[i] * zoom_factor;
                }
            }
            CameraManipulation::Reset => {
                *camera = CameraData::default();
            }
            CameraManipulation::SetPosition { position, target } => {
                camera.position = position;
                camera.target = target;
            }
        }
        
        self.viewport_data.scene_dirty = true;
    }

    /// Build the UI interface for the viewport node (parameter panel)
    pub fn build_interface(node: &mut Node, ui: &mut Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("USD Viewport Settings");
        ui.separator();
        
        // Connection Status
        ui.label("🔗 Connection Status");
        ui.label("Connect USD File Reader to input port");
        ui.separator();
        
        // Camera Settings - Collapsible
        let show_camera_settings = node.parameters.get("show_camera_settings")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
        
        let camera_header = if show_camera_settings { "🎥 Camera Settings ▼" } else { "🎥 Camera Settings ▶" };
        if ui.button(camera_header).clicked() {
            changes.push(ParameterChange {
                parameter: "show_camera_settings".to_string(),
                value: NodeData::Boolean(!show_camera_settings),
            });
        }
        
        if show_camera_settings {
            ui.indent("camera_settings", |ui| {
                let mut orbit_sensitivity = node.parameters.get("orbit_sensitivity")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(0.5);
                
                if ui.add(egui::Slider::new(&mut orbit_sensitivity, 0.1..=2.0).text("Orbit Sensitivity")).changed() {
                    changes.push(ParameterChange {
                        parameter: "orbit_sensitivity".to_string(),
                        value: NodeData::Float(orbit_sensitivity),
                    });
                }
                
                let mut pan_sensitivity = node.parameters.get("pan_sensitivity")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(1.0);
                
                if ui.add(egui::Slider::new(&mut pan_sensitivity, 0.1..=2.0).text("Pan Sensitivity")).changed() {
                    changes.push(ParameterChange {
                        parameter: "pan_sensitivity".to_string(),
                        value: NodeData::Float(pan_sensitivity),
                    });
                }
                
                let mut zoom_sensitivity = node.parameters.get("zoom_sensitivity")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(1.0);
                
                if ui.add(egui::Slider::new(&mut zoom_sensitivity, 0.1..=2.0).text("Zoom Sensitivity")).changed() {
                    changes.push(ParameterChange {
                        parameter: "zoom_sensitivity".to_string(),
                        value: NodeData::Float(zoom_sensitivity),
                    });
                }
                
                if ui.button("Reset Camera").clicked() {
                    changes.push(ParameterChange {
                        parameter: "camera_reset".to_string(),
                        value: NodeData::Boolean(true),
                    });
                }
            });
        }
        
        ui.separator();
        
        // Viewport Settings - Collapsible
        let show_viewport_settings = node.parameters.get("show_viewport_settings")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
        
        let viewport_header = if show_viewport_settings { "⚙️ Viewport Settings ▼" } else { "⚙️ Viewport Settings ▶" };
        if ui.button(viewport_header).clicked() {
            changes.push(ParameterChange {
                parameter: "show_viewport_settings".to_string(),
                value: NodeData::Boolean(!show_viewport_settings),
            });
        }
        
        if show_viewport_settings {
            ui.indent("viewport_settings", |ui| {
                let mut wireframe = node.parameters.get("wireframe")
                    .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                    .unwrap_or(false);
                
                if ui.checkbox(&mut wireframe, "Wireframe").changed() {
                    changes.push(ParameterChange {
                        parameter: "wireframe".to_string(),
                        value: NodeData::Boolean(wireframe),
                    });
                }
                
                let mut lighting = node.parameters.get("lighting")
                    .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                    .unwrap_or(true);
                
                if ui.checkbox(&mut lighting, "Lighting").changed() {
                    changes.push(ParameterChange {
                        parameter: "lighting".to_string(),
                        value: NodeData::Boolean(lighting),
                    });
                }
                
                let mut show_grid = node.parameters.get("show_grid")
                    .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                    .unwrap_or(true);
                
                if ui.checkbox(&mut show_grid, "Show Grid").changed() {
                    changes.push(ParameterChange {
                        parameter: "show_grid".to_string(),
                        value: NodeData::Boolean(show_grid),
                    });
                }
                
                let mut show_ground_plane = node.parameters.get("show_ground_plane")
                    .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                    .unwrap_or(false);
                
                if ui.checkbox(&mut show_ground_plane, "Show Ground Plane").changed() {
                    changes.push(ParameterChange {
                        parameter: "show_ground_plane".to_string(),
                        value: NodeData::Boolean(show_ground_plane),
                    });
                }
            });
        }
        
        ui.separator();
        ui.label("💡 Core USD Integration - Data-driven viewport rendering");
        
        changes
    }
    
    /// Initialize default parameters for a new viewport node
    pub fn initialize_parameters() -> HashMap<String, NodeData> {
        let mut params = HashMap::new();
        
        // Remove stage-related parameters - only use input connections
        params.insert("stage_loaded".to_string(), NodeData::Boolean(true));
        params.insert("stage_dirty".to_string(), NodeData::Boolean(true));
        
        // Camera settings
        params.insert("orbit_sensitivity".to_string(), NodeData::Float(0.5));
        params.insert("pan_sensitivity".to_string(), NodeData::Float(1.0));
        params.insert("zoom_sensitivity".to_string(), NodeData::Float(1.0));
        params.insert("scene_size".to_string(), NodeData::Float(10.0));
        params.insert("camera_reset".to_string(), NodeData::Boolean(false));
        
        // Viewport settings
        params.insert("wireframe".to_string(), NodeData::Boolean(false));
        params.insert("lighting".to_string(), NodeData::Boolean(true));
        params.insert("show_grid".to_string(), NodeData::Boolean(true));
        params.insert("show_ground_plane".to_string(), NodeData::Boolean(false));
        
        // UI state
        params.insert("show_camera_settings".to_string(), NodeData::Boolean(false));
        params.insert("show_viewport_settings".to_string(), NodeData::Boolean(false));
        
        params
    }
    
    /// Process the viewport node's logic (called during graph execution)
    pub fn process_node(node: &Node, inputs: &[NodeData]) -> Vec<NodeData> {
        println!("🎬 ViewportNode::process_node called for node '{}' (type_id: {}) with {} inputs", node.title, node.type_id, inputs.len());
        let mut outputs = Vec::new();
        
        // Debug input contents
        for (i, input) in inputs.iter().enumerate() {
            match input {
                NodeData::USDSceneData(scene) => {
                    println!("  🎬 Input {}: USDSceneData with {} meshes from '{}'", i, scene.meshes.len(), scene.stage_path);
                }
                NodeData::String(s) => {
                    println!("  🎬 Input {}: String('{}')", i, s);
                }
                _ => {
                    println!("  🎬 Input {}: {:?}", i, input);
                }
            }
        }
        
        // Check for USDSceneData input (first priority)
        let mut usd_scene_input = None;
        
        // Process USD Scene input only
        if inputs.len() > 0 {
            if let NodeData::USDSceneData(usd_scene_data) = &inputs[0] {
                usd_scene_input = Some(usd_scene_data.clone());
            }
        }
        
        // Handle USD Scene input or show empty scene
        if let Some(usd_scene_data) = usd_scene_input {
            // Store the USDSceneData in the global cache for get_viewport_data to access
            if let Ok(mut cache) = VIEWPORT_INPUT_CACHE.lock() {
                cache.insert(node.id, usd_scene_data.clone());
                println!("🎬 Viewport: Cached USDSceneData for node {} with {} meshes from stage: {}", 
                         node.id, usd_scene_data.meshes.len(), usd_scene_data.stage_path);
            }
            
            outputs.push(NodeData::String(format!("USD Viewport: {} (from input)", usd_scene_data.stage_path)));
            outputs.push(NodeData::Boolean(true)); // Scene loaded indicator
            
            println!("🎬 Viewport: Using USDSceneData input from: {}", usd_scene_data.stage_path);
        } else {
            // No input data - show empty scene
            outputs.push(NodeData::String("Empty Viewport - Connect USD File Reader".to_string()));
            outputs.push(NodeData::Boolean(false)); // No scene loaded
        }
        
        outputs
    }
    
    /// Get viewport data for 3D rendering - uses cached data when possible
    /// This is called by the viewport panel system to get scene data
    pub fn get_viewport_data(node: &Node) -> Option<ViewportData> {
        // Check if we need to force refresh for this node (parameter changes)
        let force_refresh = if let Ok(mut force_set) = FORCE_VIEWPORT_REFRESH.lock() {
            let should_force = force_set.contains(&node.id);
            if should_force {
                force_set.remove(&node.id);
                println!("🔄 ViewportNode::get_viewport_data: FORCING refresh for node {} due to parameter change", node.id);
            }
            should_force
        } else {
            false
        };
        
        // First check if we have input USDSceneData cached
        if let Ok(input_cache) = VIEWPORT_INPUT_CACHE.lock() {
            if let Some(usd_scene_data) = input_cache.get(&node.id) {
                println!("🎬 ViewportNode::get_viewport_data: Found cached USD data for node {} with {} meshes", node.id, usd_scene_data.meshes.len());
                // Check if we have a cached converted viewport data (but bypass if force refresh)
                let scene_hash = calculate_usd_scene_hash(usd_scene_data, node.id);
                if let Ok(mut viewport_cache) = VIEWPORT_DATA_CACHE.lock() {
                    if !force_refresh {
                        if let Some((cached_viewport_data, cached_hash)) = viewport_cache.get(&node.id) {
                            if *cached_hash == scene_hash {
                                // Cache hit - return cached viewport data with updated settings
                                let mut viewport_data = cached_viewport_data.clone();
                                Self::apply_viewport_settings(&mut viewport_data, node);
                                return Some(viewport_data);
                            }
                        }
                    }
                    
                    // Cache miss or force refresh - need to convert and cache the result
                    println!("🔄 ViewportNode::get_viewport_data: Converting fresh USD data for node {}", node.id);
                    let viewport_data = Self::convert_usd_scene_to_viewport_data(usd_scene_data, node);
                    viewport_cache.insert(node.id, (viewport_data.clone(), scene_hash));
                    return Some(viewport_data);
                }
            } else {
            }
        }
        
        
        // No input data - return empty scene
        Some(Self::create_empty_viewport_data(node))
    }
    
    /// Convert USDSceneData to ViewportData for rendering
    fn convert_usd_scene_to_viewport_data(usd_scene_data: &crate::workspaces::three_d::usd::usd_engine::USDSceneData, node: &Node) -> ViewportData {
        let mut scene = SceneData::default();
        scene.name = format!("USD Scene: {}", usd_scene_data.stage_path);
        
        // Convert USD meshes to viewport meshes
        for (mesh_idx, usd_mesh) in usd_scene_data.meshes.iter().enumerate() {
            // Convert Vec<Vec3> to Vec<f32> (flatten)
            let vertices: Vec<f32> = usd_mesh.vertices.iter()
                .flat_map(|v| [v.x, v.y, v.z])
                .collect();
            
            let normals: Vec<f32> = usd_mesh.normals.iter()
                .flat_map(|n| [n.x, n.y, n.z])
                .collect();
            
            let uvs: Vec<f32> = usd_mesh.uvs.iter()
                .flat_map(|uv| [uv.x, uv.y])
                .collect();
            
            // Extract vertex colors if available
            let vertex_colors = if let Some(ref colors) = usd_mesh.vertex_colors {
                Some(colors.iter().flat_map(|c| [c.x, c.y, c.z]).collect())
            } else {
                None
            };
            
            let mesh = MeshData {
                id: format!("mesh_{}", mesh_idx),
                vertices,
                normals,
                uvs,
                indices: usd_mesh.indices.clone(),
                vertex_colors,
                material_id: None, // USD mesh doesn't have material_id in this structure
                transform: [
                    [usd_mesh.transform.x_axis.x, usd_mesh.transform.x_axis.y, usd_mesh.transform.x_axis.z, usd_mesh.transform.x_axis.w],
                    [usd_mesh.transform.y_axis.x, usd_mesh.transform.y_axis.y, usd_mesh.transform.y_axis.z, usd_mesh.transform.y_axis.w],
                    [usd_mesh.transform.z_axis.x, usd_mesh.transform.z_axis.y, usd_mesh.transform.z_axis.z, usd_mesh.transform.z_axis.w],
                    [usd_mesh.transform.w_axis.x, usd_mesh.transform.w_axis.y, usd_mesh.transform.w_axis.z, usd_mesh.transform.w_axis.w],
                ],
            };
            scene.meshes.push(mesh);
        }
        
        // Convert USD materials to viewport materials
        for (mat_idx, usd_material) in usd_scene_data.materials.iter().enumerate() {
            let material = MaterialData {
                id: format!("material_{}", mat_idx),
                name: format!("USD Material: {}", usd_material.prim_path),
                base_color: [usd_material.diffuse_color.x, usd_material.diffuse_color.y, usd_material.diffuse_color.z, 1.0],
                metallic: usd_material.metallic,
                roughness: usd_material.roughness,
                emission: [0.0, 0.0, 0.0], // USD doesn't have emission in this format
                diffuse_texture: None,
                normal_texture: None,
                roughness_texture: None,
                metallic_texture: None,
            };
            scene.materials.push(material);
        }
        
        // Convert USD lights to viewport lights
        for (light_idx, usd_light) in usd_scene_data.lights.iter().enumerate() {
            let light = LightData {
                id: format!("light_{}", light_idx),
                light_type: LightType::Directional,
                position: [usd_light.transform.w_axis.x, usd_light.transform.w_axis.y, usd_light.transform.w_axis.z],
                direction: [-usd_light.transform.z_axis.x, -usd_light.transform.z_axis.y, -usd_light.transform.z_axis.z],
                color: [usd_light.color.x, usd_light.color.y, usd_light.color.z],
                intensity: usd_light.intensity,
                range: 100.0,
                spot_angle: 0.0,
            };
            scene.lights.push(light);
        }
        
        // Calculate scene bounds from vertices
        let mut min_pos = [f32::MAX; 3];
        let mut max_pos = [f32::MIN; 3];
        let mut has_bounds = false;
        
        for mesh in &scene.meshes {
            // vertices is Vec<f32> with x,y,z,x,y,z... format
            for vertex_chunk in mesh.vertices.chunks(3) {
                if vertex_chunk.len() == 3 {
                    for i in 0..3 {
                        min_pos[i] = min_pos[i].min(vertex_chunk[i]);
                        max_pos[i] = max_pos[i].max(vertex_chunk[i]);
                    }
                    has_bounds = true;
                }
            }
        }
        
        if has_bounds {
            scene.bounding_box = Some((min_pos, max_pos));
        } else {
            scene.bounding_box = Some(([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]));
        }
        
        // Create viewport data with settings from node parameters
        let mut viewport_data = ViewportData {
            scene,
            dimensions: (800, 600),
            scene_dirty: true,
            settings: ViewportSettings {
                background_color: [0.2, 0.2, 0.2, 1.0],
                wireframe: false,
                lighting: true,
                show_grid: true,
                show_ground_plane: false,
                aa_samples: 4,
                shading_mode: ShadingMode::Smooth,
            },
            settings_dirty: false,
        };
        
        // Apply settings from node parameters
        Self::apply_viewport_settings(&mut viewport_data, node);
        viewport_data
    }
    
    /// Apply viewport settings from node parameters to viewport data
    fn apply_viewport_settings(viewport_data: &mut ViewportData, node: &Node) {
        viewport_data.settings.wireframe = node.parameters.get("wireframe")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
        viewport_data.settings.lighting = node.parameters.get("lighting")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(true);
        viewport_data.settings.show_grid = node.parameters.get("show_grid")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(true);
        viewport_data.settings.show_ground_plane = node.parameters.get("show_ground_plane")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
    }
    
    /// Create empty viewport data when no input is available
    fn create_empty_viewport_data(node: &Node) -> ViewportData {
        let mut scene = SceneData::default();
        scene.name = "Empty Viewport - Connect USD File Reader".to_string();
        
        // Add a simple text mesh to indicate no data
        // For now, we'll just return an empty scene
        scene.bounding_box = Some(([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]));
        
        let mut viewport_data = ViewportData {
            scene,
            dimensions: (800, 600),
            scene_dirty: true,
            settings: ViewportSettings {
                background_color: [0.1, 0.1, 0.1, 1.0], // Darker background for empty state
                wireframe: false,
                lighting: true,
                show_grid: true,
                show_ground_plane: false,
                aa_samples: 4,
                shading_mode: ShadingMode::Smooth,
            },
            settings_dirty: false,
        };
        
        // Apply any settings from node parameters
        Self::apply_viewport_settings(&mut viewport_data, node);
        viewport_data
    }
}

impl NodeFactory for ViewportNode {
    fn metadata() -> NodeMetadata where Self: Sized {
        NodeMetadata::new(
            "Viewport",
            "Viewport",
            NodeCategory::new(&["UI"]),
            "3D viewport for visualizing USD stages with Maya-style navigation"
        )
        .with_color(egui::Color32::from_rgb(100, 200, 100))
        .with_icon("🎥")
        .with_inputs(vec![
            crate::nodes::PortDefinition::optional("USD Scene", crate::nodes::DataType::Any)
                .with_description("USD scene data from USD File Reader"),
        ])
        .with_outputs(vec![
            crate::nodes::PortDefinition::optional("Rendered Image", crate::nodes::DataType::String)
                .with_description("Viewport render output"),
        ])
        .with_size_hint(egui::Vec2::new(400.0, 300.0))
        .with_workspace_compatibility(vec!["3D"])
        .with_tags(vec!["3d", "viewport", "ui", "usd", "render"])
        .with_panel_type(PanelType::Viewport)
        .with_processing_cost(crate::nodes::factory::ProcessingCost::High)
        .with_version("3.0")
    }
    
    fn create(position: egui::Pos2) -> Node {
        let meta = Self::metadata();
        let mut node = Node::new(0, meta.display_name, position);
        node.set_type_id(meta.node_type); // Set the type_id correctly
        node.color = meta.color;
        
        // Add inputs
        for input in &meta.inputs {
            node.add_input(&input.name);
        }
        
        // Add outputs  
        for output in &meta.outputs {
            node.add_output(&output.name);
        }
        
        // Set panel type from metadata
        node.set_panel_type(meta.panel_type);
        
        // Initialize default parameters
        let params = ViewportNode::initialize_parameters();
        for (key, value) in params {
            node.parameters.insert(key, value);
        }
        
        // CRITICAL: Update port positions after adding ports
        node.update_port_positions();
        
        node
    }
}