//! Multiply node functional operations - multiplication logic

use crate::nodes::interface::NodeData;

/// Core multiply data and functionality
#[derive(Debug, Clone)]
pub struct MultiplyLogic {
    /// First input value
    pub a: f32,
    /// Second input value
    pub b: f32,
    /// Clamping settings
    pub clamp_result: bool,
    pub min_result: f32,
    pub max_result: f32,
}

impl Default for MultiplyLogic {
    fn default() -> Self {
        Self {
            a: 1.0,
            b: 1.0,
            clamp_result: false,
            min_result: -1000.0,
            max_result: 1000.0,
        }
    }
}

impl MultiplyLogic {
    /// Process input data and perform multiplication
    pub fn process(&self, inputs: Vec<NodeData>) -> Vec<NodeData> {
        let mut a = self.a;
        let mut b = self.b;
        
        // Extract values from inputs if provided
        if inputs.len() >= 1 {
            if let NodeData::Float(val) = inputs[0] {
                a = val;
            }
        }
        if inputs.len() >= 2 {
            if let NodeData::Float(val) = inputs[1] {
                b = val;
            }
        }
        
        // Perform multiplication
        let mut result = a * b;
        
        // Apply clamping if enabled
        if self.clamp_result {
            result = result.clamp(self.min_result, self.max_result);
        }
        
        vec![NodeData::Float(result)]
    }
    
    /// Perform multiplication with current values
    pub fn multiply(&self) -> f32 {
        let result = self.a * self.b;
        if self.clamp_result {
            result.clamp(self.min_result, self.max_result)
        } else {
            result
        }
    }
}