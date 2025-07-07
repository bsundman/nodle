//! Centralized graphics configuration for consistent eframe::wgpu settings

use eframe::wgpu;

/// Global graphics configuration
pub struct GraphicsConfig {
    pub sample_count: u32,
    pub texture_format: eframe::wgpu::TextureFormat,
}

impl GraphicsConfig {
    /// Get the global graphics configuration
    pub fn global() -> Self {
        Self {
            sample_count: 1, // Disable multisampling for better compatibility
            texture_format: eframe::wgpu::TextureFormat::Bgra8Unorm,
        }
    }

    /// Create multisample state from config
    pub fn multisample_state(&self) -> eframe::wgpu::MultisampleState {
        eframe::wgpu::MultisampleState {
            count: self.sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        }
    }

    /// Create color target state from config
    pub fn color_target_state(&self) -> eframe::wgpu::ColorTargetState {
        eframe::wgpu::ColorTargetState {
            format: self.texture_format,
            blend: Some(eframe::wgpu::BlendState::REPLACE),
            write_mask: eframe::wgpu::ColorWrites::ALL,
        }
    }
}

/// Get the global sample count (for eframe configuration)
pub fn global_sample_count() -> u32 {
    GraphicsConfig::global().sample_count
}