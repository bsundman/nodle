//! Directional light node functional operations - light calculation logic

use crate::nodes::interface::{NodeData, LightData, LightType};

/// Core directional light data and functionality
#[derive(Debug, Clone)]
pub struct DirectionalLightLogic {
    /// Light direction (normalized vector)
    pub direction: [f32; 3],
    /// Light color (RGB)
    pub color: [f32; 3],
    /// Light intensity/brightness
    pub intensity: f32,
    /// Whether light casts shadows
    pub cast_shadows: bool,
    /// Shadow map resolution
    pub shadow_resolution: i32,
}

impl Default for DirectionalLightLogic {
    fn default() -> Self {
        Self {
            direction: [0.0, -1.0, 0.0], // Pointing down
            color: [1.0, 1.0, 1.0], // White light
            intensity: 1.0,
            cast_shadows: true,
            shadow_resolution: 1024,
        }
    }
}

impl DirectionalLightLogic {
    /// Process input data and generate light data
    pub fn process(&self, inputs: Vec<NodeData>, node_id: usize) -> Vec<NodeData> {
        // Generate light data
        let light_data = LightData {
            id: format!("directional_light_{}", node_id),
            light_type: LightType::Directional { direction: self.direction },
            position: [0.0, 0.0, 0.0], // Position doesn't matter for directional lights
            color: self.color,
            intensity: self.intensity,
        };
        
        let _ = inputs; // Suppress unused warning
        vec![NodeData::Light(light_data)]
    }
    
    /// Normalize the direction vector
    pub fn normalize_direction(&mut self) {
        let length = (self.direction[0].powi(2) + self.direction[1].powi(2) + self.direction[2].powi(2)).sqrt();
        if length > 0.0 {
            self.direction[0] /= length;
            self.direction[1] /= length;
            self.direction[2] /= length;
        }
    }
    
    /// Set direction from spherical coordinates (azimuth, elevation)
    pub fn set_direction_from_angles(&mut self, azimuth: f32, elevation: f32) {
        let cos_elevation = elevation.cos();
        self.direction[0] = cos_elevation * azimuth.sin();
        self.direction[1] = -elevation.sin();
        self.direction[2] = cos_elevation * azimuth.cos();
    }
}