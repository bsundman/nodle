//! AND node functional operations - logical AND logic

use crate::nodes::interface::NodeData;

/// Core AND data and functionality
#[derive(Debug, Clone)]
pub struct AndLogic {
    /// First input value
    pub a: bool,
    /// Second input value
    pub b: bool,
    /// Whether to use short-circuit evaluation
    pub short_circuit: bool,
    /// Whether to invert the result
    pub invert_result: bool,
}

impl Default for AndLogic {
    fn default() -> Self {
        Self {
            a: false,
            b: false,
            short_circuit: true,
            invert_result: false,
        }
    }
}

impl AndLogic {
    /// Process input data and perform AND operation
    pub fn process(&self, inputs: Vec<NodeData>) -> Vec<NodeData> {
        let mut a = self.a;
        let mut b = self.b;
        
        // Extract values from inputs if provided
        if inputs.len() >= 1 {
            if let NodeData::Boolean(val) = inputs[0] {
                a = val;
            }
        }
        if inputs.len() >= 2 {
            if let NodeData::Boolean(val) = inputs[1] {
                b = val;
            }
        }
        
        // Perform AND operation
        let result = if self.short_circuit {
            a && b  // Short-circuit evaluation
        } else {
            a & b   // Bitwise AND (no short-circuit)
        };
        
        // Apply inversion if enabled
        let final_result = if self.invert_result {
            !result
        } else {
            result
        };
        
        vec![NodeData::Boolean(final_result)]
    }
    
    /// Perform AND operation with current values
    pub fn and(&self) -> bool {
        let result = if self.short_circuit {
            self.a && self.b
        } else {
            self.a & self.b
        };
        
        if self.invert_result {
            !result
        } else {
            result
        }
    }
    
    /// Get truth table for current settings
    pub fn get_truth_table(&self) -> Vec<(bool, bool, bool)> {
        let mut table = Vec::new();
        for a in [false, true] {
            for b in [false, true] {
                let result = if self.short_circuit {
                    a && b
                } else {
                    a & b
                };
                let final_result = if self.invert_result {
                    !result
                } else {
                    result
                };
                table.push((a, b, final_result));
            }
        }
        table
    }
}