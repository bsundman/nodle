//! USD Viewport node - visualizes USD stages in a 3D viewport

use egui::Color32;
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};
use super::usd_engine::with_usd_engine;

/// Creates a USD Viewport for scene visualization and rendering
#[derive(Default)]
pub struct USDViewport;

impl USDViewport {
    /// Execute the USD Viewport operation (renders/displays the stage)
    pub fn execute(node: &Node) -> Result<String, String> {
        let stage_id = "default_stage";
        let viewport_name = format!("viewport_{}", node.id);
        let render_width = 1920;
        let render_height = 1080;
        let camera_path = "/main_camera"; // Default camera
        
        with_usd_engine(|engine| {
            match engine.render_stage(stage_id, &viewport_name, camera_path, render_width, render_height) {
                Ok(render_info) => {
                    println!("✓ Rendered USD stage '{}' in viewport '{}': {}", stage_id, viewport_name, render_info);
                    Ok(render_info)
                }
                Err(e) => {
                    eprintln!("✗ Failed to render USD stage: {}", e);
                    Err(e)
                }
            }
        })
    }
}

impl NodeFactory for USDViewport {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "USD_Viewport",
            display_name: "USD Viewport",
            category: NodeCategory::new(&["3D", "USD", "Output"]),
            description: "Viewport for visualizing and rendering USD stages",
            color: Color32::from_rgb(100, 200, 100), // Green for viewport/output
            inputs: vec![
                PortDefinition::required("Stage", DataType::Any)
                    .with_description("USD Stage to render"),
                PortDefinition::optional("Camera", DataType::Any)
                    .with_description("Camera prim for rendering (optional)"),
                PortDefinition::optional("Width", DataType::Float)
                    .with_description("Render width in pixels (default: 1920)"),
                PortDefinition::optional("Height", DataType::Float)
                    .with_description("Render height in pixels (default: 1080)"),
                PortDefinition::optional("Samples", DataType::Float)
                    .with_description("Anti-aliasing samples (default: 4)"),
                PortDefinition::optional("Max Depth", DataType::Float)
                    .with_description("Maximum ray tracing depth (default: 8)"),
            ],
            outputs: vec![
                PortDefinition::required("Image", DataType::Any)
                    .with_description("Rendered image output"),
                PortDefinition::required("Render Stats", DataType::String)
                    .with_description("Rendering statistics and info"),
            ],
        }
    }
}