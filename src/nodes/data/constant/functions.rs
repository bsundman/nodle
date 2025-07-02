//! Constant node computation functions

use crate::nodes::interface::NodeData;

/// Process function for the Constant node
pub fn process(params: &std::collections::HashMap<String, NodeData>, _inputs: Vec<NodeData>) -> Vec<NodeData> {
    // Constants ignore inputs and return the stored value
    
    // Get the value from parameters
    if let Some(value) = params.get("value") {
        vec![value.clone()]
    } else {
        // Default to 0.0 if no value is set
        vec![NodeData::Float(0.0)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_constant_float() {
        let mut params = HashMap::new();
        params.insert("value_type".to_string(), NodeData::String("Float".to_string()));
        params.insert("value".to_string(), NodeData::Float(3.14));
        
        let result = process(&params, vec![]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], NodeData::Float(3.14));
    }
    
    #[test]
    fn test_constant_integer() {
        let mut params = HashMap::new();
        params.insert("value_type".to_string(), NodeData::String("Integer".to_string()));
        params.insert("value".to_string(), NodeData::Integer(42));
        
        let result = process(&params, vec![]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], NodeData::Integer(42));
    }
    
    #[test]
    fn test_constant_boolean() {
        let mut params = HashMap::new();
        params.insert("value_type".to_string(), NodeData::String("Boolean".to_string()));
        params.insert("value".to_string(), NodeData::Boolean(true));
        
        let result = process(&params, vec![]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], NodeData::Boolean(true));
    }
    
    #[test]
    fn test_constant_string() {
        let mut params = HashMap::new();
        params.insert("value_type".to_string(), NodeData::String("Text".to_string()));
        params.insert("value".to_string(), NodeData::String("Hello".to_string()));
        
        let result = process(&params, vec![]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], NodeData::String("Hello".to_string()));
    }
    
    #[test]
    fn test_constant_vector3() {
        let mut params = HashMap::new();
        params.insert("value_type".to_string(), NodeData::String("Vector3".to_string()));
        params.insert("value".to_string(), NodeData::Vector3([1.0, 2.0, 3.0]));
        
        let result = process(&params, vec![]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], NodeData::Vector3([1.0, 2.0, 3.0]));
    }
    
    #[test]
    fn test_constant_color() {
        let mut params = HashMap::new();
        params.insert("value_type".to_string(), NodeData::String("Color".to_string()));
        params.insert("value".to_string(), NodeData::Color([1.0, 0.5, 0.0, 1.0]));
        
        let result = process(&params, vec![]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], NodeData::Color([1.0, 0.5, 0.0, 1.0]));
    }
    
    #[test]
    fn test_constant_no_value() {
        let params = HashMap::new();
        
        let result = process(&params, vec![]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], NodeData::Float(0.0)); // Default
    }
}