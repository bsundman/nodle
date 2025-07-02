//! 3D Viewport rendering callback for egui integration
//!
//! This module provides a callback system for rendering 3D viewport content
//! using wgpu within egui interfaces.

use wgpu::{Device, Queue, CommandEncoder, RenderPass, Texture, TextureView, TextureDescriptor, TextureFormat, TextureUsages, Extent3d, TextureDimension};
use egui::{Rect, Color32};
use crate::gpu::{USDRenderer, Camera3D};
use std::sync::{Arc, Mutex};

/// 3D Viewport rendering callback for egui
pub struct ViewportRenderCallback {
    pub usd_renderer: Arc<Mutex<USDRenderer>>,
    pub viewport_rect: Rect,
    pub background_color: [f32; 4],
    pub camera: Camera3D,
}

impl ViewportRenderCallback {
    pub fn new(usd_renderer: USDRenderer, viewport_rect: Rect, background_color: [f32; 4], camera: Camera3D) -> Self {
        Self {
            usd_renderer: Arc::new(Mutex::new(usd_renderer)),
            viewport_rect,
            background_color,
            camera,
        }
    }
}

impl egui_wgpu::CallbackTrait for ViewportRenderCallback {
    fn prepare(
        &self,
        device: &Device,
        queue: &Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut CommandEncoder,
        _callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        // Initialize renderer if needed (check for pipeline existence since device isn't stored)
        let mut renderer = self.usd_renderer.lock().unwrap();
        if renderer.base_renderer.mesh_pipeline.is_none() {
            // We need to create a new renderer with the device and queue
            // Since we can't clone Device/Queue, we'll create a new base renderer
            renderer.base_renderer = crate::gpu::renderer3d::Renderer3D::new();
            renderer.base_renderer.initialize_from_refs(device, queue);
            
            // Scene will be empty by default
        }
        
        // Always update camera with current viewport camera state (this is the correct camera from navigation)
        renderer.base_renderer.camera = self.camera.clone();
        
        // Update camera uniforms with the updated camera
        renderer.base_renderer.update_camera_uniforms(queue);
        
        Vec::new()
    }
    
    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut RenderPass<'static>,
        _callback_resources: &egui_wgpu::CallbackResources,
    ) {
        // Get render target dimensions from the paint callback info
        let render_target_width = info.screen_size_px[0] as f32;
        let render_target_height = info.screen_size_px[1] as f32;
        
        // Validate and clamp viewport rect to ensure it's within render target bounds
        let viewport_x = self.viewport_rect.min.x.max(0.0);
        let viewport_y = self.viewport_rect.min.y.max(0.0);
        let max_width = (render_target_width - viewport_x).max(1.0);
        let max_height = (render_target_height - viewport_y).max(1.0);
        let viewport_width = self.viewport_rect.width().max(1.0).min(max_width);
        let viewport_height = self.viewport_rect.height().max(1.0).min(max_height);
        
        // Skip rendering if viewport would be invalid
        if viewport_x >= render_target_width || viewport_y >= render_target_height || 
           viewport_width <= 0.0 || viewport_height <= 0.0 {
            return;
        }
        
        render_pass.set_viewport(
            viewport_x,
            viewport_y,
            viewport_width,
            viewport_height,
            0.0,
            1.0,
        );
        
        // Set scissor rect to clip rendering to viewport bounds (using validated dimensions)
        render_pass.set_scissor_rect(
            viewport_x as u32,
            viewport_y as u32,
            viewport_width as u32,
            viewport_height as u32,
        );
        
        // Check if base renderer is initialized (by checking for pipelines)
        let renderer = self.usd_renderer.lock().unwrap();
        if renderer.base_renderer.mesh_pipeline.is_some() && renderer.base_renderer.uniform_bind_group.is_some() {
            // Renderer is initialized, render the scene
            renderer.render_to_pass(render_pass);
        } else {
            // Renderer not initialized - this should not happen now that we initialize in prepare()
        }
    }
}

/// Trait for rendering USD scenes to render passes
pub trait USDRenderPass {
    fn render_to_pass(&self, render_pass: &mut RenderPass);
}