//! Variable node functional operations - variable storage logic

use crate::nodes::interface::NodeData;
use super::super::constant::logic::ConstantValue;

/// Core variable data and functionality
#[derive(Debug, Clone)]
pub struct VariableLogic {
    /// The current variable value
    pub value: ConstantValue,
    /// Variable name for identification
    pub name: String,
    /// Whether the variable is read-only
    pub read_only: bool,
    /// Whether to log value changes
    pub log_changes: bool,
    /// Change history (limited to last N changes)
    pub change_history: Vec<ChangeRecord>,
    /// Maximum history size
    pub max_history_size: usize,
}

#[derive(Debug, Clone)]
pub struct ChangeRecord {
    /// Previous value
    pub old_value: ConstantValue,
    /// New value
    pub new_value: ConstantValue,
    /// Timestamp (simplified as string for now)
    pub timestamp: String,
}

impl Default for VariableLogic {
    fn default() -> Self {
        Self {
            value: ConstantValue::Float(0.0),
            name: "variable".to_string(),
            read_only: false,
            log_changes: false,
            change_history: Vec::new(),
            max_history_size: 10,
        }
    }
}

impl VariableLogic {
    /// Process input data and update/return the variable value
    pub fn process(&mut self, inputs: Vec<NodeData>) -> Vec<NodeData> {
        // If there's a "Set" input and we're not read-only, update the value
        if !self.read_only && inputs.len() >= 1 {
            let new_value = match inputs[0] {
                NodeData::Float(f) => ConstantValue::Float(f),
                NodeData::Boolean(b) => ConstantValue::Boolean(b),
                NodeData::String(ref s) => ConstantValue::Text(s.clone()),
                NodeData::Vector3(v) => ConstantValue::Vector3(v),
                NodeData::Color(c) => ConstantValue::Color(c),
                _ => ConstantValue::Float(0.0), // Default fallback
            };
            
            // Record the change if logging is enabled
            if self.log_changes {
                self.record_change(new_value.clone());
            }
            
            self.value = new_value;
        }
        
        // Always return the current value
        vec![self.get_node_data()]
    }
    
    /// Get the variable value as NodeData
    pub fn get_node_data(&self) -> NodeData {
        match &self.value {
            ConstantValue::Float(val) => NodeData::Float(*val),
            ConstantValue::Integer(val) => NodeData::Float(*val as f32),
            ConstantValue::Boolean(val) => NodeData::Boolean(*val),
            ConstantValue::Text(val) => NodeData::String(val.clone()),
            ConstantValue::Vector3(val) => NodeData::Vector3(*val),
            ConstantValue::Color(val) => NodeData::Color(*val),
        }
    }
    
    /// Record a value change in the history
    fn record_change(&mut self, new_value: ConstantValue) {
        let change = ChangeRecord {
            old_value: self.value.clone(),
            new_value: new_value.clone(),
            timestamp: "now".to_string(), // Simplified timestamp
        };
        
        self.change_history.push(change);
        
        // Keep history size limited
        if self.change_history.len() > self.max_history_size {
            self.change_history.remove(0);
        }
    }
    
    /// Get the value type name for display
    pub fn get_type_name(&self) -> &'static str {
        match self.value {
            ConstantValue::Float(_) => "Float",
            ConstantValue::Integer(_) => "Integer",
            ConstantValue::Boolean(_) => "Boolean",
            ConstantValue::Text(_) => "Text",
            ConstantValue::Vector3(_) => "Vector3",
            ConstantValue::Color(_) => "Color",
        }
    }
    
    /// Get a formatted string representation of the value
    pub fn get_formatted_value(&self) -> String {
        match &self.value {
            ConstantValue::Float(val) => format!("{:.3}", val),
            ConstantValue::Integer(val) => val.to_string(),
            ConstantValue::Boolean(val) => val.to_string(),
            ConstantValue::Text(val) => val.clone(),
            ConstantValue::Vector3(val) => format!("[{:.2}, {:.2}, {:.2}]", val[0], val[1], val[2]),
            ConstantValue::Color(val) => format!("rgba({:.2}, {:.2}, {:.2}, {:.2})", val[0], val[1], val[2], val[3]),
        }
    }
    
    /// Reset the variable to its default value
    pub fn reset(&mut self) {
        let default_value = match self.value {
            ConstantValue::Float(_) => ConstantValue::Float(0.0),
            ConstantValue::Integer(_) => ConstantValue::Integer(0),
            ConstantValue::Boolean(_) => ConstantValue::Boolean(false),
            ConstantValue::Text(_) => ConstantValue::Text(String::new()),
            ConstantValue::Vector3(_) => ConstantValue::Vector3([0.0, 0.0, 0.0]),
            ConstantValue::Color(_) => ConstantValue::Color([0.0, 0.0, 0.0, 1.0]),
        };
        
        if self.log_changes {
            self.record_change(default_value.clone());
        }
        
        self.value = default_value;
    }
    
    /// Get the number of changes recorded
    pub fn get_change_count(&self) -> usize {
        self.change_history.len()
    }
}