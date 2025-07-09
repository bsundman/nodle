//! Viewport node implementation with complete USD viewport functionality
//! Copied from USD plugin and adapted for core integration

use crate::nodes::interface::{NodeData, ParameterChange, PanelType};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory};
use egui::Ui;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use nodle_plugin_sdk::viewport::*;
use super::logic::USDViewportLogic;
use super::usd_rendering::USDRenderer;
use glam::{Mat4, Vec3};

/// Cache for USD renderers to avoid reloading stages every frame
struct USDRendererCache {
    renderers: HashMap<String, (USDRenderer, ViewportData)>,
}

impl USDRendererCache {
    fn new() -> Self {
        Self {
            renderers: HashMap::new(),
        }
    }
    
    /// Calculate scene bounds from USD geometries - no transforms
    fn calculate_scene_bounds(usd_renderer: &USDRenderer) -> Option<(Vec3, Vec3)> {
        if usd_renderer.current_scene.geometries.is_empty() {
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
        
        println!("📐 RAW USD Scene bounds: min={:?}, max={:?}", min, max);
        Some((min, max))
    }
    
    
    
    
    fn get_or_load(&mut self, stage_path: &str) -> &ViewportData {
        if !self.renderers.contains_key(stage_path) {
            println!("📦 Cache miss: Loading USD stage '{}'", stage_path);
            
            // Create and load USD renderer
            let mut usd_renderer = USDRenderer::new();
            if let Err(e) = usd_renderer.load_stage(stage_path) {
                println!("Failed to load USD stage: {}", e);
            }
            
            // Convert to viewport data
            let viewport_data = Self::convert_to_viewport_data(&usd_renderer, stage_path);
            
            self.renderers.insert(stage_path.to_string(), (usd_renderer, viewport_data));
        } else {
            println!("✅ Cache hit: Using cached USD stage '{}'", stage_path);
        }
        
        &self.renderers.get(stage_path).unwrap().1
    }
    
    /// Calculate optimal camera position based on scene bounds
    fn calculate_camera_position(bounds: (Vec3, Vec3)) -> (Vec3, Vec3, f32) {
        let (min, max) = bounds;
        let center = (min + max) * 0.5;
        let size = max - min;
        let max_dimension = size.x.max(size.y).max(size.z);
        
        println!("📐 Scene center: {:?}, size: {:?}, max_dim: {}", center, size, max_dimension);
        
        // Position camera to see the whole scene
        let camera_distance = max_dimension * 1.5;
        let camera_position = center + Vec3::new(
            camera_distance * 0.7,  // Slight angle
            camera_distance * 0.7,  // Above the scene
            camera_distance * 0.7,  // Away from scene
        );
        
        // Calculate far plane based on scene size
        let far_plane = max_dimension * 3.0; // Ensure we can see the whole scene
        
        println!("📷 Camera position: {:?}, target: {:?}", camera_position, center);
        (camera_position, center, far_plane)
    }

    fn convert_to_viewport_data(usd_renderer: &USDRenderer, stage_path: &str) -> ViewportData {
        // Convert USD scene to SDK ViewportData format
        let mut scene = SceneData::default();
        scene.name = format!("USD Stage: {}", stage_path);
        
        // Calculate scene bounds - no transforms
        let scene_bounds = Self::calculate_scene_bounds(usd_renderer);
        
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
static USD_RENDERER_CACHE: Lazy<Arc<Mutex<USDRendererCache>>> = Lazy::new(|| {
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
        let mut viewport_logic = USDViewportLogic::default();
        // Load test stage immediately to show something
        viewport_logic.load_test_stage();
        
        Self {
            current_stage: "./Kitchen_set.usd".to_string(),
            viewport_data: ViewportData::default(),
            camera_settings: CameraSettings::default(),
            viewport_logic,
        }
    }
}

impl ViewportNode {
    /// Update scene size in camera settings based on current USD stage
    pub fn update_scene_size(&mut self, node: &Node) {
        // Get current stage
        let current_stage = node.parameters.get("current_stage")
            .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
            .unwrap_or_else(|| "./Kitchen_set.usd".to_string());
        
        // Get scene size from cached data if available
        let mut cache = USD_RENDERER_CACHE.lock().unwrap();
        let cached_viewport_data = cache.get_or_load(&current_stage);
        
        // Extract scene size from scene name (encoded during creation)
        let scene_size = if let Some(size_start) = cached_viewport_data.scene.name.rfind("(size: ") {
            let size_part = &cached_viewport_data.scene.name[size_start + 7..];
            if let Some(size_end) = size_part.find(')') {
                size_part[..size_end].parse::<f32>().unwrap_or(10.0)
            } else { 10.0 }
        } else { 10.0 };
        
        // Update camera settings with scene size
        self.camera_settings.scene_size = scene_size;
        println!("📐 Updated scene size to: {:.1}", scene_size);
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
        
        println!("🎯 Adaptive sensitivity - scene_size: {:.1}, distance: {:.1}, scale_factor: {:.3}, distance_factor: {:.3}", 
                 scene_size, distance, scene_scale_factor, distance_factor);
        println!("🎯 Calculated sensitivity - orbit: {:.4}, pan: {:.4}, zoom: {:.4}", 
                 orbit_sensitivity, pan_sensitivity, zoom_sensitivity);
        
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
        
        // Stage Information
        ui.label("📁 Stage Information");
        
        let current_stage = node.parameters.get("current_stage")
            .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
            .unwrap_or_default();
        
        if current_stage.is_empty() {
            ui.label("No USD stage loaded");
        } else {
            ui.label(format!("Current Stage: {}", current_stage));
        }
        
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
        
        // USD stage parameters - start with a default stage to show something
        params.insert("current_stage".to_string(), NodeData::String("./Kitchen_set.usd".to_string()));
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
        let mut outputs = Vec::new();
        
        // Get current stage
        let current_stage = node.parameters.get("current_stage")
            .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
            .unwrap_or_default();
        
        // Process stage input if provided via connections
        let mut stage_from_input = None;
        for input in inputs {
            if let NodeData::String(stage_path) = input {
                if !stage_path.is_empty() {
                    stage_from_input = Some(stage_path.clone());
                    break;
                }
            }
        }
        
        // Use input stage if provided, otherwise use current parameter
        let active_stage = stage_from_input.unwrap_or(current_stage.clone());
        
        // Return appropriate output
        if !active_stage.is_empty() {
            outputs.push(NodeData::String(format!("USD Viewport: {}", active_stage)));
            outputs.push(NodeData::Boolean(true)); // Stage loaded indicator
        } else {
            outputs.push(NodeData::String("No USD stage loaded".to_string()));
            outputs.push(NodeData::Boolean(false)); // No stage loaded
        }
        
        outputs
    }
    
    /// Get viewport data for 3D rendering - uses cached data when possible
    /// This is called by the viewport panel system to get scene data
    pub fn get_viewport_data(node: &Node) -> Option<ViewportData> {
        println!("🔍 ViewportNode::get_viewport_data called");
        
        // Get the current stage path from node parameters
        let current_stage = node.parameters.get("current_stage")
            .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
            .unwrap_or_else(|| "./Kitchen_set.usd".to_string());
        
        // Use the cached renderer if available
        let mut cache = USD_RENDERER_CACHE.lock().unwrap();
        let cached_viewport_data = cache.get_or_load(&current_stage);
        
        // Clone the viewport data but update settings from node parameters
        let mut viewport_data = cached_viewport_data.clone();
        
        // Extract scene size from scene name (encoded during creation)
        let scene_size = if let Some(size_start) = viewport_data.scene.name.rfind("(size: ") {
            let size_part = &viewport_data.scene.name[size_start + 7..];
            if let Some(size_end) = size_part.find(')') {
                size_part[..size_end].parse::<f32>().unwrap_or(10.0)
            } else { 10.0 }
        } else { 10.0 };
        
        // Update viewport settings from node parameters
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
        
        // Store scene size in a temporary way (we'll need to improve this later)
        println!("📏 Scene size extracted: {:.1}", scene_size);
        
        println!("🔍 Returning viewport data with {} meshes, {} materials, {} lights", 
                 viewport_data.scene.meshes.len(),
                 viewport_data.scene.materials.len(),
                 viewport_data.scene.lights.len());
        
        Some(viewport_data)
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
            crate::nodes::PortDefinition::optional("Stage", crate::nodes::DataType::String)
                .with_description("USD Stage to visualize"),
            crate::nodes::PortDefinition::optional("Camera", crate::nodes::DataType::String)
                .with_description("Camera prim for viewport (optional)"),
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
        let mut node = Node::new(0, meta.node_type, position);
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