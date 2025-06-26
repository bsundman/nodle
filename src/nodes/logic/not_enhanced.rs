//! Enhanced NOT logic node implementation using new factory system

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// NOT logic node that takes a boolean input and produces its logical negation
#[derive(Default)]
pub struct NotNodeEnhanced;

impl NodeFactory for NotNodeEnhanced {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "NOT",
            display_name: "NOT",
            category: NodeCategory::logic(),
            description: "Logical NOT operation (inverts boolean input)",
            color: Color32::from_rgb(55, 45, 65), // Blue-purple tint
            inputs: vec![
                PortDefinition::required("Input", DataType::Boolean)
                    .with_description("Boolean input to invert"),
            ],
            outputs: vec![
                PortDefinition::required("Result", DataType::Boolean)
                    .with_description("Logical NOT result (!Input)"),
            ],
        }
    }
}

// Also implement the old trait for backward compatibility during transition
impl crate::NodeFactory for NotNodeEnhanced {
    fn node_type() -> &'static str {
        "NOT"
    }
    
    fn display_name() -> &'static str {
        "NOT"
    }
    
    fn category() -> crate::NodeCategory {
        crate::NodeCategory::Logic
    }
    
    fn color() -> Color32 {
        Color32::from_rgb(55, 45, 65)
    }
    
    fn create(position: Pos2) -> Node {
        <Self as NodeFactory>::create(position)
    }
}