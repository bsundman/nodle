//! Constant node functional operations - constant value logic

use crate::nodes::interface::NodeData;

/// Core constant data and functionality
#[derive(Debug, Clone)]
pub struct ConstantLogic {
    /// The constant value
    pub value: ConstantValue,
    /// Whether to format the output
    pub format_output: bool,
    /// Number of decimal places for numeric values
    pub decimal_places: u8,
}

#[derive(Debug, Clone)]
pub enum ConstantValue {
    Float(f32),
    Integer(i32),
    Boolean(bool),
    Text(String),
    Vector3([f32; 3]),
    Color([f32; 4]), // RGBA
}

impl Default for ConstantLogic {
    fn default() -> Self {
        Self {
            value: ConstantValue::Float(0.0),
            format_output: false,
            decimal_places: 2,
        }
    }
}

impl ConstantLogic {
    /// Process input data and return the constant value
    pub fn process(&self, _inputs: Vec<NodeData>) -> Vec<NodeData> {
        // Constants ignore inputs and always return the same value
        vec![self.get_node_data()]
    }
    
    /// Get the constant value as NodeData
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
            ConstantValue::Float(val) => {
                if self.format_output {
                    format!("{:.1$}", val, self.decimal_places as usize)
                } else {
                    val.to_string()
                }
            },
            ConstantValue::Integer(val) => val.to_string(),
            ConstantValue::Boolean(val) => val.to_string(),
            ConstantValue::Text(val) => val.clone(),
            ConstantValue::Vector3(val) => format!("[{:.2}, {:.2}, {:.2}]", val[0], val[1], val[2]),
            ConstantValue::Color(val) => format!("rgba({:.2}, {:.2}, {:.2}, {:.2})", val[0], val[1], val[2], val[3]),
        }
    }
    
    /// Set the value from a string
    pub fn set_value_from_string(&mut self, s: &str, value_type: &str) {
        match value_type {
            "Float" => {
                if let Ok(val) = s.parse::<f32>() {
                    self.value = ConstantValue::Float(val);
                }
            },
            "Integer" => {
                if let Ok(val) = s.parse::<i32>() {
                    self.value = ConstantValue::Integer(val);
                }
            },
            "Boolean" => {
                self.value = ConstantValue::Boolean(
                    s.to_lowercase() == "true" || s == "1"
                );
            },
            "Text" => {
                self.value = ConstantValue::Text(s.to_string());
            },
            _ => {} // Vector3 and Color would need more complex parsing
        }
    }
}