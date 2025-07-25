//! NOT node functional operations - logical NOT logic

use crate::nodes::interface::NodeData;

/// Core NOT data and functionality
#[derive(Debug, Clone)]
pub struct NotLogic {
    /// Input value
    pub input: bool,
    /// Whether to double-invert (effectively a buffer)
    pub double_invert: bool,
    /// Whether to use tri-state logic (for null/undefined inputs)
    pub tri_state: bool,
}

impl Default for NotLogic {
    fn default() -> Self {
        Self {
            input: false,
            double_invert: false,
            tri_state: false,
        }
    }
}

impl NotLogic {
    /// Process input data and perform NOT operation
    pub fn process(&self, inputs: Vec<NodeData>) -> Vec<NodeData> {
        let mut input = self.input;
        
        // Extract value from input if provided
        if inputs.len() >= 1 {
            if let NodeData::Boolean(val) = inputs[0] {
                input = val;
            }
        }
        
        // Perform NOT operation
        let result = !input;
        
        // Apply double inversion if enabled (acts as a buffer)
        let final_result = if self.double_invert {
            !result  // Double negation returns original
        } else {
            result
        };
        
        vec![NodeData::Boolean(final_result)]
    }
    
    /// Perform NOT operation with current value
    pub fn not(&self) -> bool {
        let result = !self.input;
        if self.double_invert {
            !result
        } else {
            result
        }
    }
    
    /// Get truth table for current settings
    pub fn get_truth_table(&self) -> Vec<(bool, bool)> {
        let mut table = Vec::new();
        for input in [false, true] {
            let result = !input;
            let final_result = if self.double_invert {
                !result
            } else {
                result
            };
            table.push((input, final_result));
        }
        table
    }
    
    /// Get the operation name for display
    pub fn get_operation_name(&self) -> &'static str {
        if self.double_invert {
            "BUFFER" // Double NOT acts as a buffer
        } else {
            "NOT"
        }
    }
}