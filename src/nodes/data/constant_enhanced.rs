//! Enhanced constant data node implementation using new factory system

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Constant node that provides a configurable constant value
#[derive(Default)]
pub struct ConstantNodeEnhanced;

impl NodeFactory for ConstantNodeEnhanced {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Constant",
            "Constant",
            NodeCategory::data(),
            "Provides a configurable constant value"
        )
        .with_color(Color32::from_rgb(65, 45, 65)) // Purple tint
        .with_icon("🔢")
        .with_inputs(vec![]) // No inputs - generates constant value
        .with_outputs(vec![
            PortDefinition::required("Value", DataType::Float)
                .with_description("Constant output value"),
        ])
        .with_tags(vec!["data", "constant", "value", "generator"])
        .with_processing_cost(crate::nodes::ProcessingCost::Minimal)
    }
}

// Also implement the old trait for backward compatibility during transition
impl crate::NodeFactory for ConstantNodeEnhanced {
    fn node_type() -> &'static str {
        "Constant"
    }
    
    fn display_name() -> &'static str {
        "Constant"
    }
    
    fn category() -> crate::NodeCategory {
        crate::NodeCategory::Data
    }
    
    fn color() -> Color32 {
        Color32::from_rgb(65, 45, 65)
    }
    
    fn create(position: Pos2) -> Node {
        <Self as NodeFactory>::create(position)
    }
}