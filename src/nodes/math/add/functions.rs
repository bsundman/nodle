//! Core computation logic for addition node

use crate::nodes::interface::NodeData;

/// Process addition operation on input data
pub fn process_add(inputs: Vec<NodeData>) -> Vec<NodeData> {
    if inputs.len() >= 2 {
        match (&inputs[0], &inputs[1]) {
            (NodeData::Float(a), NodeData::Float(b)) => {
                vec![NodeData::Float(a + b)]
            },
            _ => {
                // Try to convert other types to float for addition
                let a = extract_float(&inputs[0]).unwrap_or(0.0);
                let b = extract_float(&inputs[1]).unwrap_or(0.0);
                vec![NodeData::Float(a + b)]
            }
        }
    } else {
        vec![NodeData::Float(0.0)]
    }
}

/// Extract float value from any NodeData type
fn extract_float(data: &NodeData) -> Option<f32> {
    match data {
        NodeData::Float(f) => Some(*f),
        NodeData::Vector3(v) => Some(v[0]), // Use X component
        NodeData::Color(c) => Some(c[0]),   // Use R component
        NodeData::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
        _ => None,
    }
}

/// Validate input types for addition
pub fn validate_add_inputs(inputs: &[NodeData]) -> bool {
    inputs.len() >= 2 && 
    inputs.iter().take(2).all(|data| {
        matches!(data, 
            NodeData::Float(_) | 
            NodeData::Vector3(_) | 
            NodeData::Color(_) | 
            NodeData::Boolean(_)
        )
    })
}

/// Get default values for addition inputs
pub fn get_default_add_inputs() -> Vec<NodeData> {
    vec![
        NodeData::Float(0.0),
        NodeData::Float(0.0),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_add_floats() {
        let inputs = vec![NodeData::Float(2.5), NodeData::Float(3.7)];
        let result = process_add(inputs);
        assert_eq!(result.len(), 1);
        if let NodeData::Float(value) = &result[0] {
            assert!((value - 6.2).abs() < f32::EPSILON);
        } else {
            panic!("Expected Float result");
        }
    }

    #[test]
    fn test_process_add_mixed_types() {
        let inputs = vec![NodeData::Float(2.0), NodeData::Boolean(true)];
        let result = process_add(inputs);
        assert_eq!(result.len(), 1);
        if let NodeData::Float(value) = &result[0] {
            assert!((value - 3.0).abs() < f32::EPSILON);
        } else {
            panic!("Expected Float result");
        }
    }

    #[test]
    fn test_validate_add_inputs() {
        assert!(validate_add_inputs(&[NodeData::Float(1.0), NodeData::Float(2.0)]));
        assert!(validate_add_inputs(&[NodeData::Float(1.0), NodeData::Boolean(true)]));
        assert!(!validate_add_inputs(&[NodeData::Float(1.0)])); // Not enough inputs
        assert!(!validate_add_inputs(&[NodeData::Float(1.0), NodeData::String("hello".to_string())]));
    }

    #[test]
    fn test_extract_float() {
        assert_eq!(extract_float(&NodeData::Float(3.14)), Some(3.14));
        assert_eq!(extract_float(&NodeData::Boolean(true)), Some(1.0));
        assert_eq!(extract_float(&NodeData::Boolean(false)), Some(0.0));
        assert_eq!(extract_float(&NodeData::Vector3([1.0, 2.0, 3.0])), Some(1.0));
        assert_eq!(extract_float(&NodeData::String("test".to_string())), None);
    }
}