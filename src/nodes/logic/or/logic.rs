//! OR node functional operations - logical OR logic

use crate::nodes::interface::NodeData;

/// Core OR data and functionality
#[derive(Debug, Clone)]
pub struct OrLogic {
    /// First input value
    pub a: bool,
    /// Second input value
    pub b: bool,
    /// Whether to use short-circuit evaluation
    pub short_circuit: bool,
    /// Whether to invert the result
    pub invert_result: bool,
    /// Whether to use exclusive OR (XOR)
    pub exclusive: bool,
}

impl Default for OrLogic {
    fn default() -> Self {
        Self {
            a: false,
            b: false,
            short_circuit: true,
            invert_result: false,
            exclusive: false,
        }
    }
}

impl OrLogic {
    /// Process input data and perform OR operation
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
        
        // Perform OR operation
        let result = if self.exclusive {
            a ^ b  // XOR operation
        } else if self.short_circuit {
            a || b  // Short-circuit evaluation
        } else {
            a | b   // Bitwise OR (no short-circuit)
        };
        
        // Apply inversion if enabled
        let final_result = if self.invert_result {
            !result
        } else {
            result
        };
        
        vec![NodeData::Boolean(final_result)]
    }
    
    /// Perform OR operation with current values
    pub fn or(&self) -> bool {
        let result = if self.exclusive {
            self.a ^ self.b
        } else if self.short_circuit {
            self.a || self.b
        } else {
            self.a | self.b
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
                let result = if self.exclusive {
                    a ^ b
                } else if self.short_circuit {
                    a || b
                } else {
                    a | b
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
    
    /// Get the operation name for display
    pub fn get_operation_name(&self) -> &'static str {
        match (self.exclusive, self.invert_result) {
            (true, false) => "XOR",
            (true, true) => "XNOR",
            (false, false) => "OR",
            (false, true) => "NOR",
        }
    }
}