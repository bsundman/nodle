//! USD Sphere node - creates a sphere primitive

use egui::Color32;
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition, ProcessingCost};
use super::usd_engine::with_usd_engine;

/// Creates a USD Sphere primitive
#[derive(Default)]
pub struct USDSphere;

impl USDSphere {
    /// Execute the USD Sphere creation operation
    pub fn execute(node: &Node) -> Result<String, String> {
        // For now, use default values - in the future we'll get these from input ports
        let stage_id = "default_stage";
        let prim_path = format!("/sphere_{}", node.id);
        let radius = 1.0;
        
        // Create USD sphere using the engine
        with_usd_engine(|engine| {
            match engine.create_sphere(stage_id, &prim_path, radius) {
                Ok(prim) => {
                    println!("✓ Created USD sphere: {} in stage {}", prim.path, prim.stage_id);
                    Ok(prim.path)
                }
                Err(e) => {
                    eprintln!("✗ Failed to create USD sphere: {}", e);
                    Err(e)
                }
            }
        })
    }
}

impl NodeFactory for USDSphere {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "USD_Sphere",
            "USD Sphere",
            NodeCategory::new(&["3D", "USD", "Primitives"]),
            "Creates a USD sphere primitive"
        )
        .with_color(Color32::from_rgb(200, 150, 100))
        .with_icon("⚽")
        .with_inputs(vec![
            PortDefinition::required("Stage", DataType::Any)
                .with_description("USD Stage reference"),
            PortDefinition::required("Path", DataType::String)
                .with_description("Prim path (e.g., /World/MySphere)"),
            PortDefinition::optional("Radius", DataType::Float)
                .with_description("Sphere radius (default: 1.0)"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Prim", DataType::Any)
                .with_description("USD Sphere prim"),
            PortDefinition::required("Stage", DataType::Any)
                .with_description("Pass-through stage reference"),
        ])
        .with_workspace_compatibility(vec!["3D", "USD"])
        .with_tags(vec!["usd", "3d", "geometry", "primitive"])
        .with_processing_cost(ProcessingCost::Low)
    }
}