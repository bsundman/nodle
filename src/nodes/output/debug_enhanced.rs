//! Enhanced debug output node implementation using new factory system

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Debug node that outputs values with detailed debugging information
#[derive(Default)]
pub struct DebugNodeEnhanced;

impl NodeFactory for DebugNodeEnhanced {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Debug",
            "Debug",
            NodeCategory::output(),
            "Outputs values with detailed debugging information"
        )
        .with_color(Color32::from_rgb(65, 55, 45)) // Red-orange tint
        .with_icon("🐛")
        .with_inputs(vec![
            PortDefinition::required("Value", DataType::Any)
                .with_description("Value to debug output"),
        ])
        .with_outputs(vec![]) // No outputs - terminal node
        .with_tags(vec!["output", "debug", "inspect", "terminal"])
        .with_processing_cost(crate::nodes::ProcessingCost::Low)
    }
}

// Also implement the old trait for backward compatibility during transition
impl crate::NodeFactory for DebugNodeEnhanced {
    fn node_type() -> &'static str {
        "Debug"
    }
    
    fn display_name() -> &'static str {
        "Debug"
    }
    
    fn category() -> crate::NodeCategory {
        crate::NodeCategory::Output
    }
    
    fn color() -> Color32 {
        Color32::from_rgb(65, 55, 45)
    }
    
    fn create(position: Pos2) -> Node {
        <Self as NodeFactory>::create(position)
    }
}