//! 3D viewport rendering callback for wgpu integration with egui
//! 
//! This module provides the egui paint callback for 3D viewport rendering,
//! allowing plugins to provide viewport data while the core handles all wgpu rendering.

use egui_wgpu::CallbackTrait;
use std::sync::{Arc, Mutex};
use super::viewport_3d_rendering::{Renderer3D, Camera3D};
use nodle_plugin_sdk::viewport::ViewportData;
use once_cell::sync::Lazy;

// Global shared renderer instance for all viewports
static SHARED_RENDERER: Lazy<Arc<Mutex<Renderer3D>>> = Lazy::new(|| {
    Arc::new(Mutex::new(Renderer3D::new()))
});

/// 3D viewport rendering callback that integrates with egui's wgpu renderer
#[derive(Clone)]
pub struct ViewportRenderCallback {
    renderer: Arc<Mutex<Renderer3D>>,
    camera: Camera3D,
    viewport_data: Option<ViewportData>,
    viewport_size: (u32, u32),
}

impl ViewportRenderCallback {
    pub fn new() -> Self {
        Self {
            renderer: SHARED_RENDERER.clone(),
            camera: Camera3D::default(),
            viewport_data: None,
            viewport_size: (800, 600),
        }
    }
    
    /// Update the viewport data from plugins
    pub fn update_viewport_data(&mut self, data: ViewportData) {
        // Extract scene size from viewport data for adaptive sensitivity
        let scene_size = if let Some(bounding_box) = &data.scene.bounding_box {
            let min = glam::Vec3::new(bounding_box.0[0], bounding_box.0[1], bounding_box.0[2]);
            let max = glam::Vec3::new(bounding_box.1[0], bounding_box.1[1], bounding_box.1[2]);
            let size = max - min;
            size.x.max(size.y).max(size.z)
        } else {
            10.0 // Default reference size
        };
        
        // Update camera scene size for adaptive sensitivity
        self.camera.set_scene_size(scene_size);
        
        // Update camera state from viewport data to maintain persistence
        if let Some(ref viewport_data) = self.viewport_data {
            // Only update if camera has changed to avoid overwriting local manipulations
            let current_camera = &viewport_data.scene.camera;
            let new_camera = &data.scene.camera;
            
            // Check if camera data is different before updating
            if current_camera.position != new_camera.position ||
               current_camera.target != new_camera.target ||
               current_camera.up != new_camera.up ||
               current_camera.fov != new_camera.fov {
                // Sync camera state from viewport data
                self.camera.position = glam::Vec3::new(new_camera.position[0], new_camera.position[1], new_camera.position[2]);
                self.camera.target = glam::Vec3::new(new_camera.target[0], new_camera.target[1], new_camera.target[2]);
                self.camera.up = glam::Vec3::new(new_camera.up[0], new_camera.up[1], new_camera.up[2]);
                self.camera.fov = new_camera.fov;
                self.camera.near = new_camera.near;
                self.camera.far = new_camera.far;
                self.camera.aspect = new_camera.aspect;
            }
        } else {
            // First time - sync camera from viewport data
            let camera_data = &data.scene.camera;
            self.camera.position = glam::Vec3::new(camera_data.position[0], camera_data.position[1], camera_data.position[2]);
            self.camera.target = glam::Vec3::new(camera_data.target[0], camera_data.target[1], camera_data.target[2]);
            self.camera.up = glam::Vec3::new(camera_data.up[0], camera_data.up[1], camera_data.up[2]);
            self.camera.fov = camera_data.fov;
            self.camera.near = camera_data.near;
            self.camera.far = camera_data.far;
            self.camera.aspect = camera_data.aspect;
        }
        
        self.viewport_data = Some(data);
    }
    
    /// Update viewport size
    pub fn update_viewport_size(&mut self, width: u32, height: u32) {
        self.viewport_size = (width, height);
        self.camera.set_aspect(width as f32 / height as f32);
    }
    
    /// Handle camera manipulation
    pub fn handle_camera_manipulation(&mut self, delta_x: f32, delta_y: f32, manipulation_type: CameraManipulationType) {
        match manipulation_type {
            CameraManipulationType::Orbit => {
                self.camera.orbit(delta_x, delta_y);
            }
            CameraManipulationType::Pan => {
                self.camera.pan(delta_x, delta_y);
            }
            CameraManipulationType::Zoom => {
                self.camera.zoom(delta_x); // Use delta_x as zoom amount
            }
        }
    }
    
    /// Reset camera to default position
    pub fn reset_camera(&mut self) {
        self.camera = Camera3D::default();
        self.camera.set_aspect(self.viewport_size.0 as f32 / self.viewport_size.1 as f32);
    }
    
    /// Get current camera data for plugins
    pub fn get_camera_data(&self) -> nodle_plugin_sdk::viewport::CameraData {
        nodle_plugin_sdk::viewport::CameraData {
            position: [self.camera.position.x, self.camera.position.y, self.camera.position.z],
            target: [self.camera.target.x, self.camera.target.y, self.camera.target.z],
            up: [self.camera.up.x, self.camera.up.y, self.camera.up.z],
            fov: self.camera.fov,
            near: self.camera.near,
            far: self.camera.far,
            aspect: self.camera.aspect,
        }
    }
}

pub enum CameraManipulationType {
    Orbit,
    Pan,
    Zoom,
}

impl CallbackTrait for ViewportRenderCallback {
    fn prepare(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut eframe::wgpu::CommandEncoder,
        _callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<eframe::wgpu::CommandBuffer> {
        println!("🔧 ViewportCallback::prepare - Starting preparation");
        
        // Initialize renderer if not already done
        match self.renderer.lock() {
            Ok(mut renderer) => {
                if renderer.device.is_none() {
                    println!("🔧 ViewportCallback::prepare - Initializing renderer");
                    renderer.initialize_from_refs(device, queue);
                    
                    // Verify initialization succeeded
                    if renderer.device.is_some() {
                        println!("✅ ViewportCallback::prepare - Renderer initialized successfully");
                    } else {
                        println!("❌ ViewportCallback::prepare - Renderer initialization failed");
                    }
                } else {
                    println!("✅ ViewportCallback::prepare - Renderer already initialized");
                }
                
                // Update camera in renderer
                renderer.set_camera(&self.camera);
                
                // Update camera uniforms
                renderer.update_camera_uniforms(queue);
                
                println!("✅ ViewportCallback::prepare - Preparation complete");
            }
            Err(e) => {
                println!("❌ ViewportCallback::prepare - Failed to lock renderer: {:?}", e);
            }
        }
        
        Vec::new()
    }
    
    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut eframe::wgpu::RenderPass<'static>,
        _callback_resources: &egui_wgpu::CallbackResources,
    ) {
        println!("🎨 ViewportCallback::paint - Starting render");
        
        // Render the 3D viewport with proper error handling
        match self.renderer.lock() {
            Ok(mut renderer) => {
                // Validate renderer is properly initialized
                if renderer.device.is_none() {
                    println!("❌ ViewportCallback::paint - Renderer not initialized (device is None)");
                    return;
                }
                
                if renderer.mesh_pipeline.is_none() {
                    println!("❌ ViewportCallback::paint - Renderer pipelines not created");
                    return;
                }
                
                println!("✅ ViewportCallback::paint - Renderer validation passed");
                
                // Update camera in renderer
                renderer.set_camera(&self.camera);
                
                // Render the scene
                if let Some(ref viewport_data) = self.viewport_data {
                    println!("🎨 ViewportCallback::paint - Rendering scene with {} meshes", 
                             viewport_data.scene.meshes.len());
                    // Convert plugin viewport data to renderer format and render
                    renderer.render_scene(render_pass, viewport_data, self.viewport_size);
                } else {
                    println!("🎨 ViewportCallback::paint - Rendering basic scene (no viewport data)");
                    // Render basic grid and axes when no scene data
                    renderer.render_basic_scene(render_pass, self.viewport_size);
                }
                
                println!("✅ ViewportCallback::paint - Render complete");
            }
            Err(e) => {
                println!("❌ ViewportCallback::paint - Failed to lock renderer: {:?}", e);
            }
        }
    }
}

/// Create a new viewport render callback with shared state
pub fn create_viewport_callback() -> ViewportRenderCallback {
    ViewportRenderCallback::new()
}