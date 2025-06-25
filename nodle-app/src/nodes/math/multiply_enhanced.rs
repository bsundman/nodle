//! Enhanced multiplication node implementation using new factory system

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Multiplication node that takes two numeric inputs and produces their product
#[derive(Default)]
pub struct MultiplyNodeEnhanced;

impl NodeFactory for MultiplyNodeEnhanced {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "Multiply",
            display_name: "Multiply",
            category: NodeCategory::math(),
            description: "Multiplies two numeric values together",
            color: Color32::from_rgb(45, 55, 65),
            inputs: vec![
                PortDefinition::required("A", DataType::Float)
                    .with_description("First factor"),
                PortDefinition::required("B", DataType::Float)
                    .with_description("Second factor"),
            ],
            outputs: vec![
                PortDefinition::required("Result", DataType::Float)
                    .with_description("Product (A * B)"),
            ],
        }
    }
}

// Also implement the old trait for backward compatibility during transition
impl crate::NodeFactory for MultiplyNodeEnhanced {
    fn node_type() -> &'static str {
        "Multiply"
    }
    
    fn display_name() -> &'static str {
        "Multiply"
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