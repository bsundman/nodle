//! USD Viewport node - visualizes USD stages in a 3D viewport

pub mod properties;
pub mod logic;
pub mod usd_rendering;
pub mod camera;

pub use logic::USDViewportLogic;
pub use usd_rendering::USDRenderer;
pub use camera::Camera3D;

// USDViewport struct is defined below and automatically exported

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// USD Viewport node for 3D scene visualization
#[derive(Debug, Clone)]
pub struct USDViewport {
    pub viewport_width: i32,
    pub viewport_height: i32,
    pub background_color: [f32; 4],
    pub enable_wireframe: bool,
    pub enable_lighting: bool,
    pub enable_grid: bool,
    pub samples: i32,
}

impl Default for USDViewport {
    fn default() -> Self {
        Self {
            viewport_width: 1920,
            viewport_height: 1080,
            background_color: [0.2, 0.2, 0.2, 1.0],
            enable_wireframe: false,
            enable_lighting: true,
            enable_grid: true,
            samples: 4,
        }
    }
}

impl NodeFactory for USDViewport {
    fn metadata() -> NodeMetadata {
        NodeMetadata::viewport(
            "USD_Viewport",
            "USD Viewport", 
            NodeCategory::new(&["3D", "USD", "Output"]),
            "3D viewport for visualizing and rendering USD stages with real-time navigation"
        )
        .with_color(Color32::from_rgb(100, 200, 100))
        .with_icon("🎥")
        .with_inputs(vec![
            PortDefinition::required("Stage", DataType::Any)
                .with_description("USD Stage to visualize"),
            PortDefinition::optional("Camera", DataType::Any)
                .with_description("Camera prim for viewport (optional)"),
            PortDefinition::optional("Selection", DataType::Any)
                .with_description("Selected prims to highlight"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Rendered Image", DataType::Any)
                .with_description("Viewport render output"),
            PortDefinition::optional("Camera Data", DataType::Any)
                .with_description("Current camera state"),
            PortDefinition::optional("Selection Data", DataType::Any)
                .with_description("Current selection state"),
        ])
        .with_tags(vec!["3d", "usd", "viewport", "visualization", "real-time", "pixar"])
        .with_processing_cost(crate::nodes::ProcessingCost::High)
        .with_workspace_compatibility(vec!["3D", "USD", "Rendering"])
        .with_panel_type(crate::nodes::interface::PanelType::Viewport)
    }
}