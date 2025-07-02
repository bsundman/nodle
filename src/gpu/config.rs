//! Centralized graphics configuration for consistent wgpu settings

use wgpu;

/// Global graphics configuration
pub struct GraphicsConfig {
    pub sample_count: u32,
    pub texture_format: wgpu::TextureFormat,
}

impl GraphicsConfig {
    /// Get the global graphics configuration
    pub fn global() -> Self {
        Self {
            sample_count: 1, // Disable multisampling for better compatibility
            texture_format: wgpu::TextureFormat::Bgra8Unorm,
        }
    }

    /// Create multisample state from config
    pub fn multisample_state(&self) -> wgpu::MultisampleState {
        wgpu::MultisampleState {
            count: self.sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        }
    }

    /// Create color target state from config
    pub fn color_target_state(&self) -> wgpu::ColorTargetState {
        wgpu::ColorTargetState {
            format: self.texture_format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        }
    }
}

/// Get the global sample count (for eframe configuration)
pub fn global_sample_count() -> u32 {
    GraphicsConfig::global().sample_count
}