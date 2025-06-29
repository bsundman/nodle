//! USD Create Stage node - creates a new USD stage

use egui::Color32;
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};
use super::usd_engine::with_usd_engine;

/// Creates a new USD stage for scene assembly
#[derive(Default)]
pub struct USDCreateStage;

impl USDCreateStage {
    /// Execute the USD Create Stage operation
    pub fn execute(node: &Node) -> Result<String, String> {
        // Generate a stage identifier based on node ID for now
        let identifier = format!("stage_{}", node.id);
        
        // Create USD stage using the engine
        with_usd_engine(|engine| {
            match engine.create_stage(&identifier) {
                Ok(stage) => {
                    println!("✓ Created USD stage: {} at {}", stage.identifier, stage.path);
                    Ok(stage.identifier)
                }
                Err(e) => {
                    eprintln!("✗ Failed to create USD stage: {}", e);
                    Err(e)
                }
            }
        })
    }
}

impl NodeFactory for USDCreateStage {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "USD_CreateStage",
            "Create Stage",
            NodeCategory::new(&["3D", "USD", "Stage"]),
            "Creates a new USD stage for scene assembly"
        )
        .with_color(Color32::from_rgb(200, 150, 100)) // Orange-brown for USD nodes
        .with_icon("🎬")
        .with_inputs(vec![
            PortDefinition::optional("Identifier", DataType::String)
                .with_description("Optional stage identifier"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Stage", DataType::Any)
                .with_description("USD Stage reference"),
        ])
        .with_tags(vec!["3d", "usd", "stage", "create", "pixar"])
        .with_processing_cost(crate::nodes::ProcessingCost::Medium)
        .with_workspace_compatibility(vec!["3d", "usd", "pipeline"])
    }
}