//! Test node logic

use crate::nodes::interface::NodeData;
use std::collections::HashMap;

/// Test node logic - processes inputs and returns outputs
#[derive(Debug, Clone)]
pub struct TestLogic {
    pub enabled: bool,
    pub multiplier: f32,
    pub operation_mode: String,
}

impl Default for TestLogic {
    fn default() -> Self {
        Self {
            enabled: true,
            multiplier: 1.0,
            operation_mode: "passthrough".to_string(),
        }
    }
}

impl TestLogic {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Process inputs and return outputs based on current settings
    pub fn process(&self, inputs: &HashMap<String, NodeData>) -> HashMap<String, NodeData> {
        let mut outputs = HashMap::new();
        
        if !self.enabled {
            return outputs;
        }
        
        if let Some(input_data) = inputs.get("Input") {
            let output_data = match &self.operation_mode[..] {
                "passthrough" => input_data.clone(),
                "multiply" => {
                    if let NodeData::Float(f) = input_data {
                        NodeData::Float(f * self.multiplier)
                    } else {
                        input_data.clone()
                    }
                },
                "debug" => {
                    println!("Test node processing: {:?}", input_data);
                    input_data.clone()
                },
                _ => input_data.clone(),
            };
            
            outputs.insert("Output".to_string(), output_data);
        }
        
        outputs
    }
}