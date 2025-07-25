//! Spot light node functional operations - light calculation logic

use crate::nodes::interface::{NodeData, LightData, LightType};

/// Core spot light data and functionality
#[derive(Debug, Clone)]
pub struct SpotLightLogic {
    /// Light position in world space
    pub position: [f32; 3],
    /// Light direction (normalized vector)
    pub direction: [f32; 3],
    /// Light color (RGB)
    pub color: [f32; 3],
    /// Light intensity/brightness
    pub intensity: f32,
    /// Cone angle in radians
    pub cone_angle: f32,
    /// Inner cone angle for soft falloff
    pub inner_cone_angle: f32,
    /// Attenuation settings
    pub constant_attenuation: f32,
    pub linear_attenuation: f32,
    pub quadratic_attenuation: f32,
    /// Whether light casts shadows
    pub cast_shadows: bool,
}

impl Default for SpotLightLogic {
    fn default() -> Self {
        Self {
            position: [0.0, 2.0, 0.0], // Above origin
            direction: [0.0, -1.0, 0.0], // Pointing down
            color: [1.0, 1.0, 1.0], // White light
            intensity: 1.0,
            cone_angle: std::f32::consts::PI * 0.25, // 45 degrees
            inner_cone_angle: std::f32::consts::PI * 0.15, // 27 degrees
            constant_attenuation: 1.0,
            linear_attenuation: 0.09,
            quadratic_attenuation: 0.032,
            cast_shadows: true,
        }
    }
}

impl SpotLightLogic {
    /// Process input data and generate light data
    pub fn process(&self, inputs: Vec<NodeData>, node_id: usize) -> Vec<NodeData> {
        // Generate light data
        let light_data = LightData {
            id: format!("spot_light_{}", node_id),
            light_type: LightType::Spot { 
                direction: self.direction, 
                cone_angle: self.cone_angle 
            },
            position: self.position,
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
    
    /// Calculate spot light falloff at given angle from center
    pub fn calculate_spot_falloff(&self, angle_from_center: f32) -> f32 {
        if angle_from_center > self.cone_angle {
            0.0 // Outside cone
        } else if angle_from_center < self.inner_cone_angle {
            1.0 // Full intensity
        } else {
            // Smooth falloff between inner and outer cone
            let t = (self.cone_angle - angle_from_center) / (self.cone_angle - self.inner_cone_angle);
            t * t // Quadratic falloff
        }
    }
    
    /// Get effective light range
    pub fn get_effective_range(&self) -> f32 {
        // Similar to point light calculation
        let target_attenuation = 0.01;
        let a = self.quadratic_attenuation;
        let b = self.linear_attenuation;
        let c = self.constant_attenuation - (1.0 / target_attenuation);
        
        if a != 0.0 {
            let discriminant = b * b - 4.0 * a * c;
            if discriminant >= 0.0 {
                (-b + discriminant.sqrt()) / (2.0 * a)
            } else {
                50.0 // Default range
            }
        } else if b != 0.0 {
            -c / b
        } else {
            50.0 // Default range
        }
    }
}