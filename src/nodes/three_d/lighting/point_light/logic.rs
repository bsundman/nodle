//! Point light node functional operations - light calculation logic

use crate::nodes::interface::{NodeData, LightData, LightType};

/// Core point light data and functionality
#[derive(Debug, Clone)]
pub struct PointLightLogic {
    /// Light position in world space
    pub position: [f32; 3],
    /// Light color (RGB)
    pub color: [f32; 3],
    /// Light intensity/brightness
    pub intensity: f32,
    /// Attenuation settings
    pub constant_attenuation: f32,
    pub linear_attenuation: f32,
    pub quadratic_attenuation: f32,
    /// Whether light casts shadows
    pub cast_shadows: bool,
}

impl Default for PointLightLogic {
    fn default() -> Self {
        Self {
            position: [0.0, 2.0, 0.0], // Above origin
            color: [1.0, 1.0, 1.0], // White light
            intensity: 1.0,
            constant_attenuation: 1.0,
            linear_attenuation: 0.09,
            quadratic_attenuation: 0.032,
            cast_shadows: true,
        }
    }
}

impl PointLightLogic {
    /// Process input data and generate light data
    pub fn process(&self, inputs: Vec<NodeData>, node_id: usize) -> Vec<NodeData> {
        // Generate light data
        let light_data = LightData {
            id: format!("point_light_{}", node_id),
            light_type: LightType::Point,
            position: self.position,
            color: self.color,
            intensity: self.intensity,
        };
        
        // If there's a transform input, we would apply it here
        let _ = inputs; // Suppress unused warning
        vec![NodeData::Light(light_data)]
    }
    
    /// Calculate light attenuation at given distance
    pub fn calculate_attenuation(&self, distance: f32) -> f32 {
        1.0 / (self.constant_attenuation + 
               self.linear_attenuation * distance + 
               self.quadratic_attenuation * distance * distance)
    }
    
    /// Get effective light radius (where attenuation drops to 1%)
    pub fn get_effective_radius(&self) -> f32 {
        // Solve quadratic equation for 1% intensity
        let target_attenuation = 0.01;
        let a = self.quadratic_attenuation;
        let b = self.linear_attenuation;
        let c = self.constant_attenuation - (1.0 / target_attenuation);
        
        if a != 0.0 {
            let discriminant = b * b - 4.0 * a * c;
            if discriminant >= 0.0 {
                (-b + discriminant.sqrt()) / (2.0 * a)
            } else {
                100.0 // Default large radius
            }
        } else if b != 0.0 {
            -c / b
        } else {
            100.0 // Default large radius
        }
    }
}