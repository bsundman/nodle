//! USD Save Stage node - saves a USD stage to file

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Saves a USD stage to a file
#[derive(Default)]
pub struct USDSaveStage;

impl NodeFactory for USDSaveStage {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "USD_SaveStage",
            display_name: "Save Stage",
            category: NodeCategory::new(&["3D", "USD", "Stage"]),
            description: "Saves a USD stage to a .usd, .usda, or .usdc file",
            color: Color32::from_rgb(200, 150, 100), // Orange-brown for USD nodes
            inputs: vec![
                PortDefinition::required("Stage", DataType::Any)
                    .with_description("USD Stage to save"),
                PortDefinition::required("File Path", DataType::String)
                    .with_description("Output file path"),
                PortDefinition::optional("Format", DataType::String)
                    .with_description("File format: usda (ASCII) or usdc (Crate)"),
            ],
            outputs: vec![
                PortDefinition::required("Success", DataType::Boolean)
                    .with_description("True if save succeeded"),
            ],
        }
    }
}