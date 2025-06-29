//! USD Cube node - creates a cube primitive

use egui::Color32;
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition, ProcessingCost};
use super::usd_engine::with_usd_engine;

/// Creates a USD Cube primitive
#[derive(Default)]
pub struct USDCube;

impl USDCube {
    /// Execute the USD Cube creation operation
    pub fn execute(node: &Node) -> Result<String, String> {
        // For now, use default values - in the future we'll get these from input ports
        let stage_id = "default_stage";
        let prim_path = format!("/cube_{}", node.id);
        let size = 1.0;
        
        // Create USD cube using the engine
        with_usd_engine(|engine| {
            match engine.create_cube(stage_id, &prim_path, size) {
                Ok(prim) => {
                    println!("✓ Created USD cube: {} in stage {}", prim.path, prim.stage_id);
                    Ok(prim.path)
                }
                Err(e) => {
                    eprintln!("✗ Failed to create USD cube: {}", e);
                    Err(e)
                }
            }
        })
    }
}

impl NodeFactory for USDCube {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "USD_Cube",
            "USD Cube",
            NodeCategory::new(&["3D", "USD", "Primitives"]),
            "Creates a USD cube primitive"
        )
        .with_color(Color32::from_rgb(200, 150, 100))
        .with_icon("📦")
        .with_inputs(vec![
            PortDefinition::required("Stage", DataType::Any)
                .with_description("USD Stage reference"),
            PortDefinition::required("Path", DataType::String)
                .with_description("Prim path (e.g., /World/MyCube)"),
            PortDefinition::optional("Size", DataType::Float)
                .with_description("Cube size (default: 1.0)"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Prim", DataType::Any)
                .with_description("USD Cube prim"),
            PortDefinition::required("Stage", DataType::Any)
                .with_description("Pass-through stage reference"),
        ])
        .with_workspace_compatibility(vec!["3D", "USD"])
        .with_tags(vec!["usd", "3d", "geometry", "primitive"])
        .with_processing_cost(ProcessingCost::Low)
    }
}