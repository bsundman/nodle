//! Multiply node - bare minimum initialization and metadata

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Multiply node - main entry point
#[derive(Default)]
pub struct MultiplyNode;

impl NodeFactory for MultiplyNode {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Math_Multiply",
            "Multiply",
            NodeCategory::new(&["Math", "Arithmetic"]),
            "Multiplies two input values together"
        )
        .with_color(Color32::from_rgb(100, 100, 200)) // Blue for math
        .with_icon("×")
        .with_inputs(vec![
            PortDefinition::required("A", DataType::Float)
                .with_description("First multiplicand"),
            PortDefinition::required("B", DataType::Float)
                .with_description("Second multiplicand"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Result", DataType::Float)
                .with_description("Product (A × B)"),
        ])
        .with_tags(vec!["math", "arithmetic", "multiply", "product"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Low)
        .with_workspace_compatibility(vec!["Math", "General"])
    }
}