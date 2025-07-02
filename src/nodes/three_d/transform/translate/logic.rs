//! Translate node functional operations - translation logic

use crate::nodes::interface::NodeData;

/// Core translate data and functionality
#[derive(Debug, Clone)]
pub struct TranslateLogic {
    /// Translation vector
    pub translation: [f32; 3],
    /// Whether to use world or local space
    pub use_world_space: bool,
    /// Translation mode (absolute or relative)
    pub translation_mode: TranslationMode,
}

#[derive(Debug, Clone)]
pub enum TranslationMode {
    Absolute,
    Relative,
}

impl Default for TranslateLogic {
    fn default() -> Self {
        Self {
            translation: [0.0, 0.0, 0.0],
            use_world_space: true,
            translation_mode: TranslationMode::Absolute,
        }
    }
}

impl TranslateLogic {
    /// Process input data and perform translation
    pub fn process(&self, inputs: Vec<NodeData>) -> Vec<NodeData> {
        let mut translation = self.translation;
        
        // Extract translation vector from inputs if provided
        if inputs.len() >= 2 {
            if let NodeData::Vector3(vec) = inputs[1] {
                translation = vec;
            }
        }
        
        // Transform the geometry (simplified for now)
        // In a real implementation, this would apply matrix transformations
        if inputs.len() >= 1 {
            // Pass through the geometry with applied translation
            vec![inputs[0].clone()]
        } else {
            vec![NodeData::Vector3(translation)]
        }
    }
    
    /// Get the current translation matrix
    pub fn get_translation_matrix(&self) -> [[f32; 4]; 4] {
        [
            [1.0, 0.0, 0.0, self.translation[0]],
            [0.0, 1.0, 0.0, self.translation[1]],
            [0.0, 0.0, 1.0, self.translation[2]],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
    
    /// Apply translation to a point
    pub fn translate_point(&self, point: [f32; 3]) -> [f32; 3] {
        match self.translation_mode {
            TranslationMode::Absolute => self.translation,
            TranslationMode::Relative => [
                point[0] + self.translation[0],
                point[1] + self.translation[1],
                point[2] + self.translation[2],
            ],
        }
    }
}