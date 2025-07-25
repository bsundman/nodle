//! Scale node functional operations - scaling logic

use crate::nodes::interface::NodeData;

/// Core scale data and functionality
#[derive(Debug, Clone)]
pub struct ScaleLogic {
    /// Scale factors
    pub scale: [f32; 3],
    /// Whether to maintain aspect ratio
    pub uniform_scale: bool,
    /// Whether to use world or local space
    pub use_world_space: bool,
    /// Scale mode (multiply or absolute)
    pub scale_mode: ScaleMode,
}

#[derive(Debug, Clone)]
pub enum ScaleMode {
    Multiply,
    Absolute,
}

impl Default for ScaleLogic {
    fn default() -> Self {
        Self {
            scale: [1.0, 1.0, 1.0],
            uniform_scale: false,
            use_world_space: true,
            scale_mode: ScaleMode::Multiply,
        }
    }
}

impl ScaleLogic {
    /// Process input data and perform scaling
    pub fn process(&self, inputs: Vec<NodeData>) -> Vec<NodeData> {
        let mut scale = self.scale;
        
        // Extract scale vector from inputs if provided
        if inputs.len() >= 2 {
            if let NodeData::Vector3(vec) = inputs[1] {
                scale = vec;
            }
        }
        
        // Apply uniform scaling if enabled
        if self.uniform_scale {
            let uniform_factor = scale[0]; // Use X as the uniform factor
            scale = [uniform_factor, uniform_factor, uniform_factor];
        }
        
        // Transform the geometry (simplified for now)
        // In a real implementation, this would apply scale matrices
        if inputs.len() >= 1 {
            // Pass through the geometry with applied scale
            vec![inputs[0].clone()]
        } else {
            vec![NodeData::Vector3(scale)]
        }
    }
    
    /// Get the current scale matrix
    pub fn get_scale_matrix(&self) -> [[f32; 4]; 4] {
        let scale = if self.uniform_scale {
            [self.scale[0], self.scale[0], self.scale[0]]
        } else {
            self.scale
        };
        
        [
            [scale[0], 0.0, 0.0, 0.0],
            [0.0, scale[1], 0.0, 0.0],
            [0.0, 0.0, scale[2], 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
    
    /// Apply scaling to a point
    pub fn scale_point(&self, point: [f32; 3]) -> [f32; 3] {
        let scale = if self.uniform_scale {
            [self.scale[0], self.scale[0], self.scale[0]]
        } else {
            self.scale
        };
        
        match self.scale_mode {
            ScaleMode::Multiply => [
                point[0] * scale[0],
                point[1] * scale[1],
                point[2] * scale[2],
            ],
            ScaleMode::Absolute => scale,
        }
    }
    
    /// Get the effective scale factors
    pub fn get_effective_scale(&self) -> [f32; 3] {
        if self.uniform_scale {
            [self.scale[0], self.scale[0], self.scale[0]]
        } else {
            self.scale
        }
    }
}