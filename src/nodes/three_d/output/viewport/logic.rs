//! Viewport node functional operations - rendering and camera logic

use crate::nodes::interface::NodeData;
use crate::gpu::{Renderer3D, Camera3D, USDRenderer, ShadingMode, CameraMode};
use glam::{Vec3, Mat4};

/// Core viewport data and functionality
#[derive(Debug)]
pub struct ViewportLogic {
    /// 3D Camera with Maya-style navigation
    pub camera: Camera3D,
    
    /// Rendering settings
    pub background_color: [f32; 4],
    pub enable_wireframe: bool,
    pub enable_lighting: bool,
    pub enable_grid: bool,
    pub samples: i32,
    
    /// Viewport size
    pub viewport_width: i32,
    pub viewport_height: i32,
    
    /// USD-native 3D Renderer instance
    pub usd_renderer: USDRenderer,
    
    /// Current USD stage reference
    pub current_stage: Option<String>,
}

impl Default for ViewportLogic {
    fn default() -> Self {
        let usd_renderer = USDRenderer::new();
        
        Self {
            camera: Camera3D::default(),
            background_color: [0.2, 0.2, 0.2, 1.0], // Dark gray
            enable_wireframe: false,
            enable_lighting: true,
            enable_grid: true,
            samples: 4,
            viewport_width: 100, // Start with smaller default
            viewport_height: 100,
            usd_renderer,
            current_stage: None,
        }
    }
}

// Implement Clone manually since Renderer3D doesn't implement Clone
impl Clone for ViewportLogic {
    fn clone(&self) -> Self {
        Self {
            camera: self.camera.clone(),
            background_color: self.background_color,
            enable_wireframe: self.enable_wireframe,
            enable_lighting: self.enable_lighting,
            enable_grid: self.enable_grid,
            samples: self.samples,
            viewport_width: self.viewport_width,
            viewport_height: self.viewport_height,
            usd_renderer: USDRenderer::new(), // Create new renderer instance
            current_stage: self.current_stage.clone(),
        }
    }
}

impl ViewportLogic {
    /// Process input scene data and render to viewport
    pub fn process(&self, inputs: Vec<NodeData>) -> Vec<NodeData> {
        // Process scene data and render to viewport
        // For now, just return empty data as this is an endpoint
        let _ = inputs; // Suppress unused warning
        vec![]
    }
    
    /// Reset camera to default position
    pub fn reset_camera(&mut self) {
        self.camera = Camera3D::default();
    }
    
    /// Set camera to top view
    pub fn set_top_view(&mut self) {
        self.camera.position = Vec3::new(0.0, 10.0, 0.0);
        self.camera.target = Vec3::ZERO;
    }
    
    /// Set camera to front view
    pub fn set_front_view(&mut self) {
        self.camera.position = Vec3::new(0.0, 0.0, 10.0);
        self.camera.target = Vec3::ZERO;
    }
    
    /// Maya-style orbit camera around target (Alt + LMB)
    pub fn orbit_camera(&mut self, delta_x: f32, delta_y: f32) {
        self.camera.orbit(delta_x, -delta_y); // Invert Y for Maya-style
    }
    
    /// Orbit camera around mouse position with smart pivot selection
    pub fn orbit_camera_at_mouse(&mut self, delta_x: f32, delta_y: f32, mouse_x: f32, mouse_y: f32) {
        // Use smart pivot selection based on scene geometry
        let pivot_point = self.camera.find_orbit_pivot(mouse_x, mouse_y, &self.usd_renderer.current_scene.geometries);
        
        self.camera.orbit_around_point(pivot_point, delta_x, -delta_y);
    }
    
    /// Maya-style pan camera target (Alt + MMB) with 1:1 screen movement
    pub fn pan_camera(&mut self, delta_x: f32, delta_y: f32, viewport_height: f32) {
        let pan_delta = self.camera.screen_to_world_pan(delta_x, delta_y, viewport_height);
        self.camera.position += pan_delta;
        self.camera.target += pan_delta;
    }
    
    /// Maya-style zoom camera (Alt + RMB or scroll wheel)
    pub fn zoom_camera(&mut self, delta: f32) {
        self.camera.zoom(delta);
    }
    
    /// Zoom camera towards mouse position with smart target selection
    pub fn zoom_camera_to_mouse(&mut self, delta: f32, mouse_x: f32, mouse_y: f32) {
        // Use smart pivot selection based on scene geometry
        let zoom_point = self.camera.find_orbit_pivot(mouse_x, mouse_y, &self.usd_renderer.current_scene.geometries);
        
        self.camera.zoom_to_point(zoom_point, delta);
    }
    
    /// Initialize the USD renderer  
    pub fn initialize_renderer(&mut self, device: wgpu::Device, queue: wgpu::Queue) {
        self.usd_renderer.initialize(device, queue);
        
        // Load default test stage if no stage is set
        if self.current_stage.is_none() {
            self.load_test_stage();
        }
    }
    
    /// Update viewport size and camera aspect ratio
    pub fn resize_viewport(&mut self, width: u32, height: u32) {
        self.viewport_width = width as i32;
        self.viewport_height = height as i32;
        self.camera.set_aspect(width as f32 / height as f32);
    }
    
    /// Load a USD stage into the viewport
    pub fn load_stage(&mut self, stage_id: &str) -> Result<(), String> {
        self.current_stage = Some(stage_id.to_string());
        self.usd_renderer.load_stage(stage_id)
    }
    
    /// Load test stage with sample geometry
    pub fn load_test_stage(&mut self) {
        let stage_id = "test_stage";
        self.current_stage = Some(stage_id.to_string());
        if let Err(e) = self.usd_renderer.load_stage(stage_id) {
            eprintln!("Failed to load test stage: {}", e);
        }
    }
    
    /// Set shading mode for the viewport
    pub fn set_shading_mode(&mut self, mode: ShadingMode) {
        self.usd_renderer.set_shading_mode(mode);
    }
    
    /// Select USD prim by path
    pub fn select_prim(&mut self, prim_path: &str) {
        self.usd_renderer.select_prim(prim_path);
    }
    
    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.usd_renderer.clear_selection();
    }
    
    /// Get current USD scene
    pub fn get_scene(&self) -> &crate::gpu::USDScene {
        &self.usd_renderer.current_scene
    }
    
    /// Get selected prims
    pub fn get_selected_prims(&self) -> &Vec<String> {
        &self.usd_renderer.selected_prims
    }
}