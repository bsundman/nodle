//! Divide node functional operations - division logic

use crate::nodes::interface::NodeData;
use super::functions::{process_divide, process_safe_divide};

/// Core divide data and functionality - legacy support
#[derive(Debug, Clone)]
pub struct DivideLogic {
    /// First input value (dividend)
    pub a: f32,
    /// Second input value (divisor)  
    pub b: f32,
    /// Whether to handle division by zero
    pub safe_division: bool,
    /// Value to return when dividing by zero
    pub division_by_zero_result: f32,
}

impl Default for DivideLogic {
    fn default() -> Self {
        Self {
            a: 0.0,
            b: 1.0, // Default to 1 to avoid division by zero
            safe_division: true,
            division_by_zero_result: 0.0,
        }
    }
}

impl DivideLogic {
    /// Process input data and perform division
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
        
        // Use the new functions for processing
        let node_inputs = vec![NodeData::Float(a), NodeData::Float(b)];
        if self.safe_division {
            process_safe_divide(node_inputs, self.division_by_zero_result)
        } else {
            process_divide(node_inputs)
        }
    }
    
    /// Perform division with current values
    pub fn divide(&self) -> f32 {
        if self.safe_division && self.b == 0.0 {
            self.division_by_zero_result
        } else {
            self.a / self.b
        }
    }
}