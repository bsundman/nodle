//! Enhanced AND logic node implementation using new factory system

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// AND logic node that takes two boolean inputs and produces their logical AND
#[derive(Default)]
pub struct AndNodeEnhanced;

impl NodeFactory for AndNodeEnhanced {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "AND",
            display_name: "AND",
            category: NodeCategory::logic(),
            description: "Logical AND operation (true if both inputs are true)",
            color: Color32::from_rgb(55, 45, 65), // Blue-purple tint
            inputs: vec![
                PortDefinition::required("A", DataType::Boolean)
                    .with_description("First boolean input"),
                PortDefinition::required("B", DataType::Boolean)
                    .with_description("Second boolean input"),
            ],
            outputs: vec![
                PortDefinition::required("Result", DataType::Boolean)
                    .with_description("Logical AND result (A && B)"),
            ],
        }
    }
}

// Also implement the old trait for backward compatibility during transition
impl crate::NodeFactory for AndNodeEnhanced {
    fn node_type() -> &'static str {
        "AND"
    }
    
    fn display_name() -> &'static str {
        "AND"
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