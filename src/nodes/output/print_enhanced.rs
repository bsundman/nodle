//! Enhanced print output node implementation using new factory system

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Print node that outputs values to console or debug output
#[derive(Default)]
pub struct PrintNodeEnhanced;

impl NodeFactory for PrintNodeEnhanced {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Print",
            "Print",
            NodeCategory::output(),
            "Prints input values to console output"
        )
        .with_color(Color32::from_rgb(65, 55, 45)) // Red-orange tint
        .with_icon("🖨️")
        .with_inputs(vec![
            PortDefinition::required("Value", DataType::Any)
                .with_description("Value to print to console"),
        ])
        .with_outputs(vec![]) // No outputs - terminal node
        .with_tags(vec!["output", "print", "console", "terminal"])
        .with_processing_cost(crate::nodes::ProcessingCost::Low)
    }
}

// Also implement the old trait for backward compatibility during transition
impl crate::NodeFactory for PrintNodeEnhanced {
    fn node_type() -> &'static str {
        "Print"
    }
    
    fn display_name() -> &'static str {
        "Print"
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