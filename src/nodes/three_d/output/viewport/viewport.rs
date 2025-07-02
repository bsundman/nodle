//! 3D Viewport output node - bare minimum initialization and metadata

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// 3D Viewport output node - main entry point
#[derive(Default)]
pub struct ViewportNode3D;

impl NodeFactory for ViewportNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata::viewport(
            "3D_Viewport",
            "3D Viewport",
            NodeCategory::new(&["3D", "Output"]),
            "Fully functional 3D viewport with wgpu rendering and camera controls"
        )
        .with_inputs(vec![
            PortDefinition::required("Scene", DataType::Any)
                .with_description("Complete scene data to render in viewport"),
        ])
        .with_outputs(vec![
            PortDefinition::optional("Rendered Image", DataType::Any)
                .with_description("Captured viewport image"),
            PortDefinition::optional("Depth Buffer", DataType::Any)
                .with_description("Depth information from render"),
        ])
        .with_size_hint(egui::Vec2::new(160.0, 120.0))
        .with_workspace_compatibility(vec!["3D", "USD", "MaterialX"])
        .with_tags(vec!["3d", "viewport", "output", "render", "wgpu"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::High)
        .with_version("2.0")
    }
}