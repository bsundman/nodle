//! Viewport node implementation with complete USD viewport functionality
//! Copied from USD plugin and adapted for core integration

use crate::nodes::interface::{NodeData, ParameterChange, PanelType};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory};
use egui::Ui;
use std::collections::HashMap;
use nodle_plugin_sdk::viewport::*;
use super::logic::USDViewportLogic;
use super::usd_rendering::USDRenderer;

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
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            orbit_sensitivity: 0.5,
            pan_sensitivity: 1.0,
            zoom_sensitivity: 1.0,
        }
    }
}

impl Default for ViewportNode {
    fn default() -> Self {
        let mut viewport_logic = USDViewportLogic::default();
        // Load test stage immediately to show something
        viewport_logic.load_test_stage();
        
        Self {
            current_stage: "default_scene".to_string(),
            viewport_data: ViewportData::default(),
            camera_settings: CameraSettings::default(),
            viewport_logic,
        }
    }
}

impl ViewportNode {
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
        
        match manipulation {
            CameraManipulation::Orbit { delta_x, delta_y } => {
                let radius = ((camera.position[0] - camera.target[0]).powi(2) + 
                             (camera.position[1] - camera.target[1]).powi(2) + 
                             (camera.position[2] - camera.target[2]).powi(2)).sqrt();
                
                // Convert to spherical coordinates
                let mut theta = (camera.position[2] - camera.target[2]).atan2(camera.position[0] - camera.target[0]);
                let mut phi = ((camera.position[1] - camera.target[1]) / radius).asin();
                
                // Apply orbit deltas
                theta += delta_x * self.camera_settings.orbit_sensitivity;
                phi += delta_y * self.camera_settings.orbit_sensitivity;
                
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
                
                // Pan both position and target
                let pan_x = delta_x * self.camera_settings.pan_sensitivity;
                let pan_y = delta_y * self.camera_settings.pan_sensitivity;
                
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
                
                let zoom_factor = delta * self.camera_settings.zoom_sensitivity;
                
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
        params.insert("current_stage".to_string(), NodeData::String("default_scene".to_string()));
        params.insert("stage_loaded".to_string(), NodeData::Boolean(true));
        params.insert("stage_dirty".to_string(), NodeData::Boolean(true));
        
        // Camera settings
        params.insert("orbit_sensitivity".to_string(), NodeData::Float(0.5));
        params.insert("pan_sensitivity".to_string(), NodeData::Float(1.0));
        params.insert("zoom_sensitivity".to_string(), NodeData::Float(1.0));
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
    
    /// Get viewport data for 3D rendering - ALWAYS provides data for rendering
    /// This is called by the viewport panel system to get scene data
    pub fn get_viewport_data(node: &Node) -> Option<ViewportData> {
        println!("🔍 ViewportNode::get_viewport_data called");
        
        // Always provide viewport data with a test scene 
        // This ensures the 3D viewport is always rendered
        let current_stage = node.parameters.get("current_stage")
            .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
            .unwrap_or_else(|| "default_scene".to_string()); // Always have a stage
        
        println!("🔍 Creating viewport data for stage: '{}'", current_stage);
        
        // Create a USD renderer instance to generate proper scene data
        let mut usd_renderer = USDRenderer::new();
        if let Err(e) = usd_renderer.load_stage(&current_stage) {
            println!("Failed to load USD stage, using fallback: {}", e);
        }
        
        // Convert USD scene to SDK ViewportData format
        let mut scene = SceneData::default();
        scene.name = format!("USD Stage: {}", current_stage);
        
        // Set up proper camera with default position
        scene.camera = CameraData {
            position: [5.0, 5.0, 5.0],
            target: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            fov: 45.0_f32.to_radians(),
            near: 0.1,
            far: 100.0,
            aspect: 800.0 / 600.0,
        };
        
        // Convert USD geometries to SDK mesh format
        for usd_geometry in &usd_renderer.current_scene.geometries {
            let mesh_data = MeshData {
                id: usd_geometry.prim_path.clone(),
                vertices: usd_geometry.vertices.iter().flat_map(|v| v.position.iter().cloned()).collect(),
                normals: usd_geometry.vertices.iter().flat_map(|v| v.normal.iter().cloned()).collect(),
                uvs: usd_geometry.vertices.iter().flat_map(|v| v.uv.iter().cloned()).collect(),
                indices: usd_geometry.indices.clone(),
                material_id: usd_geometry.material_path.clone(),
                transform: [
                    [usd_geometry.transform.x_axis.x, usd_geometry.transform.x_axis.y, usd_geometry.transform.x_axis.z, usd_geometry.transform.x_axis.w],
                    [usd_geometry.transform.y_axis.x, usd_geometry.transform.y_axis.y, usd_geometry.transform.y_axis.z, usd_geometry.transform.y_axis.w],
                    [usd_geometry.transform.z_axis.x, usd_geometry.transform.z_axis.y, usd_geometry.transform.z_axis.z, usd_geometry.transform.z_axis.w],
                    [usd_geometry.transform.w_axis.x, usd_geometry.transform.w_axis.y, usd_geometry.transform.w_axis.z, usd_geometry.transform.w_axis.w],
                ],
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
        
        // Set scene bounding box based on geometry
        if !scene.meshes.is_empty() {
            scene.bounding_box = Some(([-5.0, -5.0, -5.0], [5.0, 5.0, 5.0]));
        }
        
        // Create viewport data with settings from node parameters
        let viewport_data = ViewportData {
            scene,
            dimensions: (800, 600), // Default viewport size
            scene_dirty: true,
            settings: ViewportSettings {
                background_color: [0.2, 0.2, 0.2, 1.0],
                wireframe: node.parameters.get("wireframe")
                    .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                    .unwrap_or(false),
                lighting: node.parameters.get("lighting")
                    .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                    .unwrap_or(true),
                show_grid: node.parameters.get("show_grid")
                    .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                    .unwrap_or(true),
                show_ground_plane: node.parameters.get("show_ground_plane")
                    .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                    .unwrap_or(false),
                aa_samples: 4,
                shading_mode: ShadingMode::Smooth,
            },
            settings_dirty: false,
        };
        
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
}