//! USD Load Stage node - loads a USD file

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Loads a USD stage from a file
#[derive(Default)]
pub struct USDLoadStage;

impl NodeFactory for USDLoadStage {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "USD_LoadStage",
            "Load Stage",
            NodeCategory::new(&["3D", "USD", "Stage"]),
            "Loads a USD stage from a .usd, .usda, or .usdc file"
        )
        .with_color(Color32::from_rgb(200, 150, 100)) // Orange-brown for USD nodes
        .with_icon("📂")
        .with_inputs(vec![
            PortDefinition::required("File Path", DataType::String)
                .with_description("Path to USD file"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Stage", DataType::Any)
                .with_description("Loaded USD Stage"),
        ])
        .with_tags(vec!["3d", "usd", "stage", "load", "import", "pixar"])
        .with_processing_cost(crate::nodes::ProcessingCost::High)
        .with_workspace_compatibility(vec!["3d", "usd", "pipeline"])
    }
}