//! Variable node - bare minimum initialization and metadata

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Variable node - main entry point
#[derive(Default)]
pub struct VariableNode;

impl NodeFactory for VariableNode {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Data_Variable",
            "Variable",
            NodeCategory::new(&["Data", "Storage"]),
            "Stores and outputs a variable value that can be modified"
        )
        .with_color(Color32::from_rgb(65, 55, 75)) // Slightly different purple for variables
        .with_icon("V")
        .with_inputs(vec![
            PortDefinition::optional("Set", DataType::Any)
                .with_description("Input to set the variable value"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Value", DataType::Any)
                .with_description("The current variable value"),
        ])
        .with_tags(vec!["data", "variable", "storage", "mutable"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Minimal)
        .with_workspace_compatibility(vec!["General", "Data", "Programming"])
    }
}