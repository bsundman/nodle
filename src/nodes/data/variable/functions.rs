//! Variable node computation functions

use crate::nodes::interface::NodeData;

/// Process function for the Variable node
pub fn process(params: &std::collections::HashMap<String, NodeData>, inputs: Vec<NodeData>) -> Vec<NodeData> {
    // Check if read-only
    let is_read_only = params.get("read_only")
        .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
        .unwrap_or(false);
    
    // Get current value
    let mut current_value = params.get("value").cloned().unwrap_or(NodeData::Float(0.0));
    
    // If we have an input and the variable is not read-only, update the value
    if !is_read_only && !inputs.is_empty() {
        // Take the first input as the new value
        current_value = inputs[0].clone();
        
        // Note: In a real implementation, we would need to persist this change
        // back to the node's parameters through a mutable reference or state management
        
        // If logging is enabled, we would track the change here
        let log_changes = params.get("log_changes")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
        
        if log_changes {
            // In a real implementation, we would increment the change counter
            // and store the change history
        }
    }
    
    // Always output the current value
    vec![current_value]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_variable_float_output() {
        let mut params = HashMap::new();
        params.insert("value_type".to_string(), NodeData::String("Float".to_string()));
        params.insert("value".to_string(), NodeData::Float(3.14));
        params.insert("read_only".to_string(), NodeData::Boolean(false));
        
        let result = process(&params, vec![]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], NodeData::Float(3.14));
    }
    
    #[test]
    fn test_variable_set_value() {
        let mut params = HashMap::new();
        params.insert("value_type".to_string(), NodeData::String("Float".to_string()));
        params.insert("value".to_string(), NodeData::Float(3.14));
        params.insert("read_only".to_string(), NodeData::Boolean(false));
        
        // Should return the input value since it's not read-only
        let result = process(&params, vec![NodeData::Float(2.71)]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], NodeData::Float(2.71));
    }
    
    #[test]
    fn test_variable_read_only() {
        let mut params = HashMap::new();
        params.insert("value_type".to_string(), NodeData::String("Float".to_string()));
        params.insert("value".to_string(), NodeData::Float(3.14));
        params.insert("read_only".to_string(), NodeData::Boolean(true));
        
        // Should ignore input and return stored value since it's read-only
        let result = process(&params, vec![NodeData::Float(2.71)]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], NodeData::Float(3.14));
    }
    
    #[test]
    fn test_variable_integer() {
        let mut params = HashMap::new();
        params.insert("value_type".to_string(), NodeData::String("Integer".to_string()));
        params.insert("value".to_string(), NodeData::Integer(42));
        params.insert("read_only".to_string(), NodeData::Boolean(false));
        
        let result = process(&params, vec![]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], NodeData::Integer(42));
    }
    
    #[test]
    fn test_variable_boolean() {
        let mut params = HashMap::new();
        params.insert("value_type".to_string(), NodeData::String("Boolean".to_string()));
        params.insert("value".to_string(), NodeData::Boolean(true));
        params.insert("read_only".to_string(), NodeData::Boolean(false));
        
        let result = process(&params, vec![]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], NodeData::Boolean(true));
    }
    
    #[test]
    fn test_variable_string() {
        let mut params = HashMap::new();
        params.insert("value_type".to_string(), NodeData::String("Text".to_string()));
        params.insert("value".to_string(), NodeData::String("Hello".to_string()));
        params.insert("read_only".to_string(), NodeData::Boolean(false));
        
        let result = process(&params, vec![]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], NodeData::String("Hello".to_string()));
    }
    
    #[test]
    fn test_variable_no_value() {
        let params = HashMap::new();
        
        let result = process(&params, vec![]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], NodeData::Float(0.0)); // Default
    }
}