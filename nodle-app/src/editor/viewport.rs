//! Viewport management for pan/zoom operations

use egui::{Pos2, Vec2};

/// Manages viewport state including pan and zoom
#[derive(Debug, Clone)]
pub struct Viewport {
    pub pan_offset: Vec2,
    pub zoom: f32,
}

impl Viewport {
    /// Creates a new viewport with default settings
    pub fn new() -> Self {
        Self {
            pan_offset: Vec2::ZERO,
            zoom: 1.0,
        }
    }

    /// Zoom at a specific screen point
    pub fn zoom_at_point(&mut self, screen_point: Pos2, zoom_delta: f32) {
        let old_zoom = self.zoom;
        self.zoom = (self.zoom * zoom_delta).clamp(0.1, 5.0);
        
        // Adjust pan to keep the zoom point stationary
        let zoom_factor = self.zoom / old_zoom;
        let screen_point_vec = screen_point.to_vec2();
        self.pan_offset = screen_point_vec + (self.pan_offset - screen_point_vec) * zoom_factor;
    }

    /// Apply pan offset
    pub fn pan(&mut self, delta: Vec2) {
        self.pan_offset += delta;
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}