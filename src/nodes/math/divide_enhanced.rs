//! Enhanced division node implementation using new factory system

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Division node that takes two numeric inputs and produces their quotient
#[derive(Default)]
pub struct DivideNodeEnhanced;

impl NodeFactory for DivideNodeEnhanced {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Divide",
            "Divide",
            NodeCategory::math(),
            "Divides the first input by the second"
        )
        .with_color(Color32::from_rgb(45, 55, 65))
        .with_icon("➗")
        .with_inputs(vec![
            PortDefinition::required("A", DataType::Float)
                .with_description("Dividend (value to be divided)"),
            PortDefinition::required("B", DataType::Float)
                .with_description("Divisor (value to divide by)"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Result", DataType::Float)
                .with_description("Quotient (A / B)"),
        ])
        .with_tags(vec!["math", "arithmetic", "divide"])
        .with_processing_cost(crate::nodes::ProcessingCost::Minimal)
    }
}

// Also implement the old trait for backward compatibility during transition
impl crate::NodeFactory for DivideNodeEnhanced {
    fn node_type() -> &'static str {
        "Divide"
    }
    
    fn display_name() -> &'static str {
        "Divide"
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