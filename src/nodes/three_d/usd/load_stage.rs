//! USD Load Stage node - loads a USD file

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Loads a USD stage from a file
#[derive(Default)]
pub struct USDLoadStage;

impl NodeFactory for USDLoadStage {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "USD_LoadStage",
            display_name: "Load Stage",
            category: NodeCategory::new(&["3D", "USD", "Stage"]),
            description: "Loads a USD stage from a .usd, .usda, or .usdc file",
            color: Color32::from_rgb(200, 150, 100), // Orange-brown for USD nodes
            inputs: vec![
                PortDefinition::required("File Path", DataType::String)
                    .with_description("Path to USD file"),
            ],
            outputs: vec![
                PortDefinition::required("Stage", DataType::Any)
                    .with_description("Loaded USD Stage"),
            ],
        }
    }
}