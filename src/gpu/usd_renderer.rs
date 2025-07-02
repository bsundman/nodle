//! USD-native 3D renderer using PyO3 bindings
//! 
//! This module implements a 3D renderer that directly reads USD stages
//! and renders USD geometry, materials, and lights using wgpu.

use wgpu::{Device, Queue, Buffer, BufferUsages, CommandEncoder, RenderPass};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Vec4};
use std::collections::HashMap;
use crate::gpu::renderer3d::{Renderer3D, Camera3D, Vertex3D, Uniforms3D};
use crate::nodes::three_d::usd::usd_engine::{USDStage, USDPrim, with_usd_engine};

#[cfg(feature = "usd")]
use pyo3::prelude::*;

/// USD Geometry data extracted from USD prims
#[derive(Debug, Clone)]
pub struct USDGeometry {
    pub prim_path: String,
    pub prim_type: String,
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
    pub transform: Mat4,
    pub material_path: Option<String>,
    pub visibility: bool,
}

/// USD Light data extracted from UsdLux lights
#[derive(Debug, Clone)]
pub struct USDLight {
    pub prim_path: String,
    pub light_type: String, // "distant", "rect", "sphere", etc.
    pub intensity: f32,
    pub color: Vec3,
    pub transform: Mat4,
    pub exposure: f32,
    pub cone_angle: Option<f32>, // For spot lights
    pub cone_softness: Option<f32>,
}

/// USD Material data extracted from UsdShade materials
#[derive(Debug, Clone)]
pub struct USDMaterial {
    pub prim_path: String,
    pub diffuse_color: Vec3,
    pub metallic: f32,
    pub roughness: f32,
    pub opacity: f32,
    pub emission_color: Vec3,
}

/// USD Camera data extracted from UsdGeom cameras
#[derive(Debug, Clone)]
pub struct USDCamera {
    pub prim_path: String,
    pub transform: Mat4,
    pub focal_length: f32,
    pub horizontal_aperture: f32,
    pub vertical_aperture: f32,
    pub clipping_range: (f32, f32),
}

/// USD Scene representation
#[derive(Debug, Clone)]
pub struct USDScene {
    pub stage_id: String,
    pub geometries: Vec<USDGeometry>,
    pub lights: Vec<USDLight>,
    pub materials: HashMap<String, USDMaterial>,
    pub cameras: Vec<USDCamera>,
    pub time_code: f64,
}

impl Default for USDScene {
    fn default() -> Self {
        Self {
            stage_id: String::new(),
            geometries: Vec::new(),
            lights: Vec::new(),
            materials: HashMap::new(),
            cameras: Vec::new(),
            time_code: 0.0,
        }
    }
}

/// USD-native 3D renderer
pub struct USDRenderer {
    /// Base 3D renderer
    pub base_renderer: Renderer3D,
    /// Current USD scene
    pub current_scene: USDScene,
    /// Geometry buffers for USD prims
    pub geometry_buffers: HashMap<String, (Buffer, Buffer, u32)>, // vertex, index, index_count
    /// USD render settings
    pub render_settings: USDRenderSettings,
    /// Selected USD prims
    pub selected_prims: Vec<String>,
    /// Viewport camera or USD camera mode
    pub camera_mode: CameraMode,
}

#[derive(Debug, Clone)]
pub struct USDRenderSettings {
    pub shading_mode: ShadingMode,
    pub show_guides: bool,
    pub show_render: bool,
    pub show_proxy: bool,
    pub show_purposes: Vec<String>, // "default", "render", "proxy", "guide"
    pub complexity: ComplexityLevel,
    pub enable_lighting: bool,
    pub ambient_occlusion: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShadingMode {
    Wireframe,
    WireframeOnSurface,
    FlatShaded,
    SmoothShaded,
    MaterialPreview,
    Rendered,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComplexityLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CameraMode {
    Viewport,
    USDCamera(String), // USD camera prim path
}

impl Default for USDRenderSettings {
    fn default() -> Self {
        Self {
            shading_mode: ShadingMode::SmoothShaded,
            show_guides: false,
            show_render: true,
            show_proxy: false,
            show_purposes: vec!["default".to_string(), "render".to_string()],
            complexity: ComplexityLevel::Medium,
            enable_lighting: true,
            ambient_occlusion: false,
        }
    }
}

impl Clone for USDRenderer {
    fn clone(&self) -> Self {
        Self {
            base_renderer: Renderer3D::new(), // Create new renderer since it can't be cloned
            current_scene: self.current_scene.clone(),
            geometry_buffers: HashMap::new(), // Buffers can't be cloned, create new
            render_settings: self.render_settings.clone(),
            selected_prims: self.selected_prims.clone(),
            camera_mode: self.camera_mode.clone(),
        }
    }
}

impl std::fmt::Debug for USDRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("USDRenderer")
            .field("current_scene", &self.current_scene.stage_id)
            .field("geometry_count", &self.current_scene.geometries.len())
            .field("light_count", &self.current_scene.lights.len())
            .field("material_count", &self.current_scene.materials.len())
            .field("camera_mode", &self.camera_mode)
            .field("render_settings", &self.render_settings)
            .finish()
    }
}

impl Default for USDRenderer {
    fn default() -> Self {
        Self {
            base_renderer: Renderer3D::new(),
            current_scene: USDScene::default(),
            geometry_buffers: HashMap::new(),
            render_settings: USDRenderSettings::default(),
            selected_prims: Vec::new(),
            camera_mode: CameraMode::Viewport,
        }
    }
}

impl USDRenderer {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Initialize the USD renderer with wgpu device and queue
    pub fn initialize(&mut self, device: Device, queue: Queue) {
        self.base_renderer.initialize(device, queue);
    }
    
    /// Load a USD stage and populate the scene
    pub fn load_stage(&mut self, stage_id: &str) -> Result<(), String> {
        println!("Loading USD stage: {}", stage_id);
        
        // Clear previous scene
        self.current_scene = USDScene {
            stage_id: stage_id.to_string(),
            ..Default::default()
        };
        self.geometry_buffers.clear();
        
        #[cfg(feature = "usd")]
        {
            // Try to extract real USD stage data
            if let Err(_) = self.extract_stage_data(stage_id) {
                // If extraction fails, create mock scene instead
                self.create_mock_scene(stage_id);
            }
            
            // If no geometries were loaded, create mock scene as fallback
            if self.current_scene.geometries.is_empty() {
                self.create_mock_scene(stage_id);
            }
        }
        
        #[cfg(not(feature = "usd"))]
        {
            // Create mock scene for testing without USD
            self.create_mock_scene(stage_id);
        }
        
        self.upload_geometry_buffers()?;
        
        println!("✓ Loaded USD stage: {} geometries, {} lights, {} materials", 
                 self.current_scene.geometries.len(),
                 self.current_scene.lights.len(),
                 self.current_scene.materials.len());
        
        Ok(())
    }
    
    #[cfg(feature = "usd")]
    fn extract_stage_data(&mut self, stage_id: &str) -> Result<(), String> {
        with_usd_engine(|engine| {
            // Get stage reference (this would need to be added to USDEngine)
            if let Some(stage) = engine.get_stage(stage_id) {
                let result = Python::with_gil(|py| -> Result<(), String> {
                    let usd = py.import("pxr.Usd").map_err(|e| format!("Failed to import USD: {}", e))?;
                    let usd_geom = py.import("pxr.UsdGeom").map_err(|e| format!("Failed to import UsdGeom: {}", e))?;
                    let usd_lux = py.import("pxr.UsdLux").map_err(|e| format!("Failed to import UsdLux: {}", e))?;
                    let usd_shade = py.import("pxr.UsdShade").map_err(|e| format!("Failed to import UsdShade: {}", e))?;
                    
                    // TODO: Get actual stage object from engine
                    // For now, this is a framework for USD data extraction
                    
                    // Extract geometry prims
                    self.extract_geometry_prims(py, usd_geom, stage_id)?;
                    
                    // Extract light prims  
                    self.extract_light_prims(py, usd_lux, stage_id)?;
                    
                    // Extract material prims
                    self.extract_material_prims(py, usd_shade, stage_id)?;
                    
                    // Extract camera prims
                    self.extract_camera_prims(py, usd_geom, stage_id)?;
                    
                    Ok(())
                });
                
                if let Err(e) = result {
                    eprintln!("Error extracting USD stage data: {}", e);
                }
            }
        });
        
        Ok(())
    }
    
    #[cfg(feature = "usd")]
    fn extract_geometry_prims(&mut self, py: Python, usd_geom: &PyAny, stage_id: &str) -> Result<(), String> {
        // This would iterate through all geometry prims and extract mesh data
        // For now, create a placeholder cube
        let cube_geometry = self.create_cube_geometry("/World/Cube", Mat4::IDENTITY);
        self.current_scene.geometries.push(cube_geometry);
        
        Ok(())
    }
    
    #[cfg(feature = "usd")]
    fn extract_light_prims(&mut self, py: Python, usd_lux: &PyAny, stage_id: &str) -> Result<(), String> {
        // Extract USD lights
        let default_light = USDLight {
            prim_path: "/World/DefaultLight".to_string(),
            light_type: "distant".to_string(),
            intensity: 1.0,
            color: Vec3::new(1.0, 1.0, 1.0),
            transform: Mat4::IDENTITY,
            exposure: 0.0,
            cone_angle: None,
            cone_softness: None,
        };
        self.current_scene.lights.push(default_light);
        
        Ok(())
    }
    
    #[cfg(feature = "usd")]
    fn extract_material_prims(&mut self, py: Python, usd_shade: &PyAny, stage_id: &str) -> Result<(), String> {
        // Extract USD materials
        let default_material = USDMaterial {
            prim_path: "/World/DefaultMaterial".to_string(),
            diffuse_color: Vec3::new(0.7, 0.7, 0.8),
            metallic: 0.0,
            roughness: 0.5,
            opacity: 1.0,
            emission_color: Vec3::ZERO,
        };
        self.current_scene.materials.insert("/World/DefaultMaterial".to_string(), default_material);
        
        Ok(())
    }
    
    #[cfg(feature = "usd")]
    fn extract_camera_prims(&mut self, py: Python, usd_geom: &PyAny, stage_id: &str) -> Result<(), String> {
        // Extract USD cameras
        Ok(())
    }
    
    pub fn create_mock_scene(&mut self, stage_id: &str) {
        // Create a mock scene for testing without USD
        
        // Add some test geometry
        let cube = self.create_cube_geometry("/World/Cube", Mat4::from_translation(Vec3::new(-2.0, 0.0, 0.0)));
        let sphere = self.create_sphere_geometry("/World/Sphere", Mat4::from_translation(Vec3::new(2.0, 0.0, 0.0)));
        let plane = self.create_plane_geometry("/World/Plane", Mat4::from_translation(Vec3::new(0.0, -1.0, 0.0)));
        
        self.current_scene.geometries.push(cube);
        self.current_scene.geometries.push(sphere);
        self.current_scene.geometries.push(plane);
        
        // Add a default light
        let light = USDLight {
            prim_path: "/World/DefaultLight".to_string(),
            light_type: "distant".to_string(),
            intensity: 1.0,
            color: Vec3::new(1.0, 1.0, 0.9),
            transform: Mat4::from_rotation_x(-45_f32.to_radians()),
            exposure: 0.0,
            cone_angle: None,
            cone_softness: None,
        };
        self.current_scene.lights.push(light);
        
        // Add a default material
        let material = USDMaterial {
            prim_path: "/World/DefaultMaterial".to_string(),
            diffuse_color: Vec3::new(0.6, 0.7, 0.8),
            metallic: 0.1,
            roughness: 0.4,
            opacity: 1.0,
            emission_color: Vec3::ZERO,
        };
        self.current_scene.materials.insert("/World/DefaultMaterial".to_string(), material);
    }
    
    fn create_cube_geometry(&self, prim_path: &str, transform: Mat4) -> USDGeometry {
        // Create cube vertices
        let vertices = vec![
            // Front face
            Vertex3D { position: [-1.0, -1.0,  1.0], normal: [ 0.0,  0.0,  1.0], uv: [0.0, 0.0] },
            Vertex3D { position: [ 1.0, -1.0,  1.0], normal: [ 0.0,  0.0,  1.0], uv: [1.0, 0.0] },
            Vertex3D { position: [ 1.0,  1.0,  1.0], normal: [ 0.0,  0.0,  1.0], uv: [1.0, 1.0] },
            Vertex3D { position: [-1.0,  1.0,  1.0], normal: [ 0.0,  0.0,  1.0], uv: [0.0, 1.0] },
            
            // Back face
            Vertex3D { position: [-1.0, -1.0, -1.0], normal: [ 0.0,  0.0, -1.0], uv: [1.0, 0.0] },
            Vertex3D { position: [-1.0,  1.0, -1.0], normal: [ 0.0,  0.0, -1.0], uv: [1.0, 1.0] },
            Vertex3D { position: [ 1.0,  1.0, -1.0], normal: [ 0.0,  0.0, -1.0], uv: [0.0, 1.0] },
            Vertex3D { position: [ 1.0, -1.0, -1.0], normal: [ 0.0,  0.0, -1.0], uv: [0.0, 0.0] },
            
            // Top face
            Vertex3D { position: [-1.0,  1.0, -1.0], normal: [ 0.0,  1.0,  0.0], uv: [0.0, 1.0] },
            Vertex3D { position: [-1.0,  1.0,  1.0], normal: [ 0.0,  1.0,  0.0], uv: [0.0, 0.0] },
            Vertex3D { position: [ 1.0,  1.0,  1.0], normal: [ 0.0,  1.0,  0.0], uv: [1.0, 0.0] },
            Vertex3D { position: [ 1.0,  1.0, -1.0], normal: [ 0.0,  1.0,  0.0], uv: [1.0, 1.0] },
            
            // Bottom face
            Vertex3D { position: [-1.0, -1.0, -1.0], normal: [ 0.0, -1.0,  0.0], uv: [1.0, 1.0] },
            Vertex3D { position: [ 1.0, -1.0, -1.0], normal: [ 0.0, -1.0,  0.0], uv: [0.0, 1.0] },
            Vertex3D { position: [ 1.0, -1.0,  1.0], normal: [ 0.0, -1.0,  0.0], uv: [0.0, 0.0] },
            Vertex3D { position: [-1.0, -1.0,  1.0], normal: [ 0.0, -1.0,  0.0], uv: [1.0, 0.0] },
            
            // Right face
            Vertex3D { position: [ 1.0, -1.0, -1.0], normal: [ 1.0,  0.0,  0.0], uv: [1.0, 0.0] },
            Vertex3D { position: [ 1.0,  1.0, -1.0], normal: [ 1.0,  0.0,  0.0], uv: [1.0, 1.0] },
            Vertex3D { position: [ 1.0,  1.0,  1.0], normal: [ 1.0,  0.0,  0.0], uv: [0.0, 1.0] },
            Vertex3D { position: [ 1.0, -1.0,  1.0], normal: [ 1.0,  0.0,  0.0], uv: [0.0, 0.0] },
            
            // Left face
            Vertex3D { position: [-1.0, -1.0, -1.0], normal: [-1.0,  0.0,  0.0], uv: [0.0, 0.0] },
            Vertex3D { position: [-1.0, -1.0,  1.0], normal: [-1.0,  0.0,  0.0], uv: [1.0, 0.0] },
            Vertex3D { position: [-1.0,  1.0,  1.0], normal: [-1.0,  0.0,  0.0], uv: [1.0, 1.0] },
            Vertex3D { position: [-1.0,  1.0, -1.0], normal: [-1.0,  0.0,  0.0], uv: [0.0, 1.0] },
        ];
        
        let indices = vec![
            0,  1,  2,   0,  2,  3,    // front
            4,  5,  6,   4,  6,  7,    // back
            8,  9,  10,  8,  10, 11,   // top
            12, 13, 14,  12, 14, 15,   // bottom
            16, 17, 18,  16, 18, 19,   // right
            20, 21, 22,  20, 22, 23,   // left
        ];
        
        USDGeometry {
            prim_path: prim_path.to_string(),
            prim_type: "Cube".to_string(),
            vertices,
            indices,
            transform,
            material_path: Some("/World/DefaultMaterial".to_string()),
            visibility: true,
        }
    }
    
    fn create_sphere_geometry(&self, prim_path: &str, transform: Mat4) -> USDGeometry {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        let radius = 1.0;
        let segments = 32;
        let rings = 16;
        
        // Generate sphere vertices
        for ring in 0..=rings {
            let phi = std::f32::consts::PI * ring as f32 / rings as f32;
            let y = phi.cos();
            let ring_radius = phi.sin();
            
            for segment in 0..=segments {
                let theta = 2.0 * std::f32::consts::PI * segment as f32 / segments as f32;
                let x = ring_radius * theta.cos();
                let z = ring_radius * theta.sin();
                
                vertices.push(Vertex3D {
                    position: [x * radius, y * radius, z * radius],
                    normal: [x, y, z],
                    uv: [segment as f32 / segments as f32, ring as f32 / rings as f32],
                });
            }
        }
        
        // Generate sphere indices
        for ring in 0..rings {
            for segment in 0..segments {
                let current = ring * (segments + 1) + segment;
                let next = current + segments + 1;
                
                indices.push(current);
                indices.push(next);
                indices.push(current + 1);
                
                indices.push(current + 1);
                indices.push(next);
                indices.push(next + 1);
            }
        }
        
        USDGeometry {
            prim_path: prim_path.to_string(),
            prim_type: "Sphere".to_string(),
            vertices,
            indices,
            transform,
            material_path: Some("/World/DefaultMaterial".to_string()),
            visibility: true,
        }
    }
    
    fn create_plane_geometry(&self, prim_path: &str, transform: Mat4) -> USDGeometry {
        let size = 5.0;
        let vertices = vec![
            Vertex3D { position: [-size, 0.0, -size], normal: [0.0, 1.0, 0.0], uv: [0.0, 0.0] },
            Vertex3D { position: [ size, 0.0, -size], normal: [0.0, 1.0, 0.0], uv: [1.0, 0.0] },
            Vertex3D { position: [ size, 0.0,  size], normal: [0.0, 1.0, 0.0], uv: [1.0, 1.0] },
            Vertex3D { position: [-size, 0.0,  size], normal: [0.0, 1.0, 0.0], uv: [0.0, 1.0] },
        ];
        
        let indices = vec![0, 1, 2, 0, 2, 3];
        
        USDGeometry {
            prim_path: prim_path.to_string(),
            prim_type: "Plane".to_string(),
            vertices,
            indices,
            transform,
            material_path: Some("/World/DefaultMaterial".to_string()),
            visibility: true,
        }
    }
    
    fn upload_geometry_buffers(&mut self) -> Result<(), String> {
        if let Some(device) = &self.base_renderer.device {
            self.geometry_buffers.clear();
            
            for geometry in &self.current_scene.geometries {
                // Create vertex buffer
                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{}_vertices", geometry.prim_path)),
                    contents: bytemuck::cast_slice(&geometry.vertices),
                    usage: BufferUsages::VERTEX,
                });
                
                // Create index buffer
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{}_indices", geometry.prim_path)),
                    contents: bytemuck::cast_slice(&geometry.indices),
                    usage: BufferUsages::INDEX,
                });
                
                self.geometry_buffers.insert(
                    geometry.prim_path.clone(),
                    (vertex_buffer, index_buffer, geometry.indices.len() as u32)
                );
            }
        }
        
        Ok(())
    }
    
    /// Upload geometry buffers using device reference (for callback system)
    pub fn upload_geometry_buffers_from_refs(&mut self, device: &wgpu::Device) -> Result<(), String> {
        self.geometry_buffers.clear();
        
        for geometry in &self.current_scene.geometries {
            // Create vertex buffer
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{}_vertices", geometry.prim_path)),
                contents: bytemuck::cast_slice(&geometry.vertices),
                usage: BufferUsages::VERTEX,
            });
            
            // Create index buffer
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{}_indices", geometry.prim_path)),
                contents: bytemuck::cast_slice(&geometry.indices),
                usage: BufferUsages::INDEX,
            });
            
            self.geometry_buffers.insert(
                geometry.prim_path.clone(),
                (vertex_buffer, index_buffer, geometry.indices.len() as u32)
            );
        }
        
        Ok(())
    }
    
    /// Select USD prim by path
    pub fn select_prim(&mut self, prim_path: &str) {
        if !self.selected_prims.contains(&prim_path.to_string()) {
            self.selected_prims.push(prim_path.to_string());
        }
    }
    
    /// Deselect USD prim by path
    pub fn deselect_prim(&mut self, prim_path: &str) {
        self.selected_prims.retain(|p| p != prim_path);
    }
    
    /// Clear all selections
    pub fn clear_selection(&mut self) {
        self.selected_prims.clear();
    }
    
    /// Set render mode
    pub fn set_shading_mode(&mut self, mode: ShadingMode) {
        self.render_settings.shading_mode = mode;
    }
    
    /// Set camera mode
    pub fn set_camera_mode(&mut self, mode: CameraMode) {
        self.camera_mode = mode;
    }
    
    /// Get active camera for rendering
    pub fn get_active_camera(&self) -> Camera3D {
        match &self.camera_mode {
            CameraMode::Viewport => self.base_renderer.camera.clone(),
            CameraMode::USDCamera(path) => {
                // Find USD camera and convert to Camera3D
                if let Some(usd_camera) = self.current_scene.cameras.iter().find(|c| &c.prim_path == path) {
                    self.usd_camera_to_camera3d(usd_camera)
                } else {
                    self.base_renderer.camera.clone()
                }
            }
        }
    }
    
    fn usd_camera_to_camera3d(&self, usd_camera: &USDCamera) -> Camera3D {
        // Convert USD camera to viewport camera
        let mut camera = self.base_renderer.camera.clone();
        
        // Extract position and target from transform matrix
        let position = usd_camera.transform.transform_point3(Vec3::ZERO);
        let forward = -usd_camera.transform.transform_vector3(Vec3::Z);
        let target = position + forward;
        
        camera.position = position;
        camera.target = target;
        
        // Convert focal length to FOV
        let fov_degrees = 2.0 * (usd_camera.horizontal_aperture / (2.0 * usd_camera.focal_length)).atan().to_degrees();
        camera.fov = fov_degrees.to_radians();
        
        camera.near = usd_camera.clipping_range.0;
        camera.far = usd_camera.clipping_range.1;
        
        camera
    }
}

impl USDRenderPass for USDRenderer {
    fn render_to_pass(&self, render_pass: &mut wgpu::RenderPass) {
        // Camera uniforms are already updated in the callback's prepare method
        
        // Render all geometry based on shading mode
        for geometry in &self.current_scene.geometries {
            if !geometry.visibility {
                continue;
            }
            
            if let Some((vertex_buffer, index_buffer, index_count)) = self.geometry_buffers.get(&geometry.prim_path) {
                match self.render_settings.shading_mode {
                    ShadingMode::Wireframe | ShadingMode::WireframeOnSurface => {
                        self.base_renderer.render_wireframe(render_pass, vertex_buffer, index_buffer, *index_count);
                    }
                    _ => {
                        self.base_renderer.render_mesh(render_pass, vertex_buffer, index_buffer, *index_count);
                    }
                }
            }
        }
        
        // Render grid if enabled
        if self.render_settings.enable_lighting { // Using lighting toggle for grid for now
            self.base_renderer.render_grid(render_pass);
        }
        
        // Always render axis gizmo
        self.base_renderer.render_axis_gizmo(render_pass);
    }
}

// Need to add wgpu::util for buffer creation
use wgpu::util::DeviceExt;
use crate::gpu::viewport_callback::USDRenderPass;