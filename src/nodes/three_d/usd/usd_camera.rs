//! USD Camera node - creates a camera primitive

use egui::Color32;
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition, ProcessingCost};
use super::usd_engine::with_usd_engine;

/// Creates a USD Camera primitive
#[derive(Default)]
pub struct USDCamera;

impl USDCamera {
    /// Execute the USD Camera creation operation
    pub fn execute(node: &Node) -> Result<String, String> {
        // For now, use default values - in the future we'll get these from input ports
        let stage_id = "default_stage";
        let prim_path = format!("/camera_{}", node.id);
        let focal_length = 50.0; // mm
        let near_clip = 0.1;
        let far_clip = 1000.0;
        
        // Create USD camera using the engine
        with_usd_engine(|engine| {
            match engine.create_camera(stage_id, &prim_path, focal_length, near_clip, far_clip) {
                Ok(prim) => {
                    println!("✓ Created USD camera: {} in stage {}", prim.path, prim.stage_id);
                    Ok(prim.path)
                }
                Err(e) => {
                    eprintln!("✗ Failed to create USD camera: {}", e);
                    Err(e)
                }
            }
        })
    }
}

impl NodeFactory for USDCamera {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "USD_Camera",
            "USD Camera",
            NodeCategory::new(&["3D", "USD", "Primitives"]),
            "Creates a USD camera primitive with lens parameters"
        )
        .with_color(Color32::from_rgb(200, 150, 100))
        .with_icon("🎥")
        .with_inputs(vec![
            PortDefinition::required("Stage", DataType::Any)
                .with_description("USD Stage reference"),
            PortDefinition::required("Path", DataType::String)
                .with_description("Prim path (e.g., /World/MainCamera)"),
            PortDefinition::optional("Focal Length", DataType::Float)
                .with_description("Camera focal length in mm (default: 50.0)"),
            PortDefinition::optional("Near Clip", DataType::Float)
                .with_description("Near clipping plane (default: 0.1)"),
            PortDefinition::optional("Far Clip", DataType::Float)
                .with_description("Far clipping plane (default: 1000.0)"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Prim", DataType::Any)
                .with_description("USD Camera prim"),
            PortDefinition::required("Stage", DataType::Any)
                .with_description("Pass-through stage reference"),
        ])
        .with_workspace_compatibility(vec!["3D", "USD"])
        .with_tags(vec!["usd", "3d", "camera", "lens"])
        .with_processing_cost(ProcessingCost::Low)
    }
}