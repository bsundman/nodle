//! Print node - bare minimum initialization and metadata

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Print node - main entry point
#[derive(Default)]
pub struct PrintNode;

impl NodeFactory for PrintNode {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Output_Print",
            "Print",
            NodeCategory::new(&["Output", "Debug"]),
            "Prints input values to the console or output stream"
        )
        .with_color(Color32::from_rgb(65, 45, 45)) // Dark red-grey for output nodes
        .with_icon("📄")
        .with_inputs(vec![
            PortDefinition::required("Value", DataType::Any)
                .with_description("Value to print"),
        ])
        .with_outputs(vec![])
        .with_tags(vec!["output", "print", "console", "debug"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Minimal)
        .with_workspace_compatibility(vec!["General", "Debug", "Testing"])
    }
}