//! Enhanced subtraction node implementation using new factory system

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Subtraction node that takes two numeric inputs and produces their difference
#[derive(Default)]
pub struct SubtractNodeEnhanced;

impl NodeFactory for SubtractNodeEnhanced {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "Subtract",
            display_name: "Subtract",
            category: NodeCategory::math(),
            description: "Subtracts the second input from the first",
            color: Color32::from_rgb(45, 55, 65),
            inputs: vec![
                PortDefinition::required("A", DataType::Float)
                    .with_description("Minuend (value to subtract from)"),
                PortDefinition::required("B", DataType::Float)
                    .with_description("Subtrahend (value to subtract)"),
            ],
            outputs: vec![
                PortDefinition::required("Result", DataType::Float)
                    .with_description("Difference (A - B)"),
            ],
        }
    }
}

// Also implement the old trait for backward compatibility during transition
impl crate::NodeFactory for SubtractNodeEnhanced {
    fn node_type() -> &'static str {
        "Subtract"
    }
    
    fn display_name() -> &'static str {
        "Subtract"
    }
    
    fn category() -> crate::NodeCategory {
        crate::NodeCategory::Math
    }
    
    fn color() -> Color32 {
        Color32::from_rgb(45, 55, 65)
    }
    
    fn create(position: Pos2) -> Node {
        <Self as NodeFactory>::create(position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Pos2;

    #[test]
    fn test_enhanced_subtract_node_metadata() {
        let metadata = SubtractNodeEnhanced::metadata();
        assert_eq!(metadata.node_type, "Subtract");
        assert_eq!(metadata.display_name, "Subtract");
        assert_eq!(metadata.description, "Subtracts the second input from the first");
    }

    #[test]
    fn test_enhanced_subtract_node_creation() {
        let node = SubtractNodeEnhanced::create(Pos2::new(100.0, 100.0));
        assert_eq!(node.title, "Subtract");
        assert_eq!(node.inputs.len(), 2);
        assert_eq!(node.outputs.len(), 1);
    }
}