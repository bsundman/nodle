//! Core computation logic for subtraction node

use crate::nodes::interface::NodeData;

/// Process subtraction operation on input data
pub fn process_subtract(inputs: Vec<NodeData>) -> Vec<NodeData> {
    if inputs.len() >= 2 {
        match (&inputs[0], &inputs[1]) {
            (NodeData::Float(a), NodeData::Float(b)) => {
                vec![NodeData::Float(a - b)]
            },
            _ => {
                // Try to convert other types to float for subtraction
                let a = extract_float(&inputs[0]).unwrap_or(0.0);
                let b = extract_float(&inputs[1]).unwrap_or(0.0);
                vec![NodeData::Float(a - b)]
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

/// Validate input types for subtraction
pub fn validate_subtract_inputs(inputs: &[NodeData]) -> bool {
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

/// Get default values for subtraction inputs
pub fn get_default_subtract_inputs() -> Vec<NodeData> {
    vec![
        NodeData::Float(0.0),
        NodeData::Float(0.0),
    ]
}

/// Perform componentwise vector subtraction
pub fn subtract_vectors(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Perform componentwise color subtraction with clamping
pub fn subtract_colors(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [
        (a[0] - b[0]).max(0.0),
        (a[1] - b[1]).max(0.0),
        (a[2] - b[2]).max(0.0),
        (a[3] - b[3]).clamp(0.0, 1.0), // Alpha should stay in valid range
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_subtract_floats() {
        let inputs = vec![NodeData::Float(5.0), NodeData::Float(3.0)];
        let result = process_subtract(inputs);
        assert_eq!(result.len(), 1);
        if let NodeData::Float(value) = &result[0] {
            assert!((value - 2.0).abs() < f32::EPSILON);
        } else {
            panic!("Expected Float result");
        }
    }

    #[test]
    fn test_process_subtract_mixed_types() {
        let inputs = vec![NodeData::Float(5.0), NodeData::Boolean(true)];
        let result = process_subtract(inputs);
        assert_eq!(result.len(), 1);
        if let NodeData::Float(value) = &result[0] {
            assert!((value - 4.0).abs() < f32::EPSILON);
        } else {
            panic!("Expected Float result");
        }
    }

    #[test]
    fn test_subtract_vectors() {
        let a = [5.0, 3.0, 1.0];
        let b = [2.0, 1.0, 0.5];
        let result = subtract_vectors(a, b);
        assert_eq!(result, [3.0, 2.0, 0.5]);
    }

    #[test]
    fn test_subtract_colors() {
        let a = [0.8, 0.6, 0.4, 1.0];
        let b = [0.3, 0.1, 0.2, 0.2];
        let result = subtract_colors(a, b);
        assert_eq!(result, [0.5, 0.5, 0.2, 0.8]);
    }

    #[test]
    fn test_subtract_colors_clamping() {
        let a = [0.3, 0.2, 0.1, 0.5];
        let b = [0.5, 0.4, 0.3, 0.8];
        let result = subtract_colors(a, b);
        assert_eq!(result, [0.0, 0.0, 0.0, 0.0]); // Negative values clamped to 0
    }

    #[test]
    fn test_validate_subtract_inputs() {
        assert!(validate_subtract_inputs(&[NodeData::Float(1.0), NodeData::Float(2.0)]));
        assert!(validate_subtract_inputs(&[NodeData::Float(1.0), NodeData::Boolean(true)]));
        assert!(!validate_subtract_inputs(&[NodeData::Float(1.0)])); // Not enough inputs
        assert!(!validate_subtract_inputs(&[NodeData::Float(1.0), NodeData::String("hello".to_string())]));
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