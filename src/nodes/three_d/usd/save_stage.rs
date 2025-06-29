//! USD Save Stage node - saves a USD stage to file

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Saves a USD stage to a file
#[derive(Default)]
pub struct USDSaveStage;

impl NodeFactory for USDSaveStage {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "USD_SaveStage",
            "Save Stage",
            NodeCategory::new(&["3D", "USD", "Stage"]),
            "Saves a USD stage to a .usd, .usda, or .usdc file"
        )
        .with_color(Color32::from_rgb(200, 150, 100)) // Orange-brown for USD nodes
        .with_icon("💾")
        .with_inputs(vec![
            PortDefinition::required("Stage", DataType::Any)
                .with_description("USD Stage to save"),
            PortDefinition::required("File Path", DataType::String)
                .with_description("Output file path"),
            PortDefinition::optional("Format", DataType::String)
                .with_description("File format: usda (ASCII) or usdc (Crate)"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Success", DataType::Boolean)
                .with_description("True if save succeeded"),
        ])
        .with_tags(vec!["3d", "usd", "stage", "save", "export", "pixar"])
        .with_processing_cost(crate::nodes::ProcessingCost::High)
        .with_workspace_compatibility(vec!["3d", "usd", "pipeline"])
    }
}