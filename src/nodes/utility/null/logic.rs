//! Null node functional operations - passthrough logic

use crate::nodes::interface::NodeData;

/// Core null data and functionality
#[derive(Debug, Clone)]
pub struct NullLogic {
    /// Optional label for organization
    pub label: String,
    /// Whether the null is enabled (passes data through)
    pub enabled: bool,
    /// Description for documentation
    pub description: String,
}

impl Default for NullLogic {
    fn default() -> Self {
        Self {
            label: "Null".to_string(),
            enabled: true,
            description: "Passthrough node for organization".to_string(),
        }
    }
}

impl NullLogic {
    /// Process input data - simple passthrough
    pub fn process(&self, inputs: Vec<NodeData>) -> Vec<NodeData> {
        if self.enabled {
            // Pass through the first input if available
            if !inputs.is_empty() {
                vec![inputs[0].clone()]
            } else {
                // If no input, output None
                vec![NodeData::None]
            }
        } else {
            // If disabled, output None regardless of input
            vec![NodeData::None]
        }
    }
    
    /// Get the current label
    pub fn get_label(&self) -> &str {
        &self.label
    }
    
    /// Set a new label
    pub fn set_label(&mut self, label: String) {
        self.label = label;
    }
    
    /// Check if the null is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Enable or disable the null
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Get the description
    pub fn get_description(&self) -> &str {
        &self.description
    }
    
    /// Set a new description
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }
}