//! Constant node - bare minimum initialization and metadata

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Constant value node - main entry point
#[derive(Default)]
pub struct ConstantNode;

impl NodeFactory for ConstantNode {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Data_Constant",
            "Constant",
            NodeCategory::new(&["Data", "Source"]),
            "Outputs a constant value that doesn't change"
        )
        .with_color(Color32::from_rgb(55, 45, 65)) // Dark purple-grey for data nodes
        .with_icon("C")
        .with_inputs(vec![])
        .with_outputs(vec![
            PortDefinition::required("Value", DataType::Any)
                .with_description("The constant output value"),
        ])
        .with_tags(vec!["data", "constant", "source", "value"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Minimal)
        .with_workspace_compatibility(vec!["General", "Data", "Math"])
    }
}