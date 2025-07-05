//! 3D viewport rendering callback for wgpu integration with egui
//! 
//! This module provides the egui paint callback for 3D viewport rendering,
//! allowing plugins to provide viewport data while the core handles all wgpu rendering.

use egui_wgpu::CallbackTrait;
use wgpu;
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
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        _callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        // Initialize renderer if not already done
        if let Ok(mut renderer) = self.renderer.lock() {
            if renderer.device.is_none() {
                renderer.initialize_from_refs(device, queue);
            }
            
            // Update camera in renderer
            renderer.set_camera(&self.camera);
            
            // Update camera uniforms
            renderer.update_camera_uniforms(queue);
        }
        
        Vec::new()
    }
    
    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        _callback_resources: &egui_wgpu::CallbackResources,
    ) {
        // Render the 3D viewport
        if let Ok(mut renderer) = self.renderer.lock() {
            // Update camera in renderer
            renderer.set_camera(&self.camera);
            
            // Update camera uniforms (we need access to queue for this)
            // For now, skip uniform updates during paint - they should be done in prepare
            
            // Render the scene
            if let Some(ref viewport_data) = self.viewport_data {
                // Convert plugin viewport data to renderer format and render
                renderer.render_scene(render_pass, viewport_data, self.viewport_size);
            } else {
                // Render basic grid and axes when no scene data
                renderer.render_basic_scene(render_pass, self.viewport_size);
            }
        }
    }
}

/// Create a new viewport render callback with shared state
pub fn create_viewport_callback() -> ViewportRenderCallback {
    ViewportRenderCallback::new()
}