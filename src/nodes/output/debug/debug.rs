//! Debug node - bare minimum initialization and metadata

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Debug node - main entry point
#[derive(Default)]
pub struct DebugNode;

impl NodeFactory for DebugNode {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Output_Debug",
            "Debug",
            NodeCategory::new(&["Output", "Debug", "Diagnostics"]),
            "Advanced debugging output with detailed information"
        )
        .with_color(Color32::from_rgb(75, 35, 35)) // Darker red for debug nodes
        .with_icon("🐛")
        .with_inputs(vec![
            PortDefinition::required("Value", DataType::Any)
                .with_description("Value to debug"),
            PortDefinition::optional("Label", DataType::String)
                .with_description("Optional debug label"),
        ])
        .with_outputs(vec![
            PortDefinition::required("PassThrough", DataType::Any)
                .with_description("Input value passed through unchanged"),
        ])
        .with_tags(vec!["output", "debug", "diagnostics", "inspect"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Low)
        .with_workspace_compatibility(vec!["General", "Debug", "Testing", "Development"])
    }
}