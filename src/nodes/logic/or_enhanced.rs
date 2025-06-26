//! Enhanced OR logic node implementation using new factory system

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// OR logic node that takes two boolean inputs and produces their logical OR
#[derive(Default)]
pub struct OrNodeEnhanced;

impl NodeFactory for OrNodeEnhanced {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "OR",
            display_name: "OR", 
            category: NodeCategory::logic(),
            description: "Logical OR operation (true if either input is true)",
            color: Color32::from_rgb(55, 45, 65), // Blue-purple tint
            inputs: vec![
                PortDefinition::required("A", DataType::Boolean)
                    .with_description("First boolean input"),
                PortDefinition::required("B", DataType::Boolean)
                    .with_description("Second boolean input"),
            ],
            outputs: vec![
                PortDefinition::required("Result", DataType::Boolean)
                    .with_description("Logical OR result (A || B)"),
            ],
        }
    }
}

// Also implement the old trait for backward compatibility during transition
impl crate::NodeFactory for OrNodeEnhanced {
    fn node_type() -> &'static str {
        "OR"
    }
    
    fn display_name() -> &'static str {
        "OR"
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