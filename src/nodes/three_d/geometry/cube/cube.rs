//! 3D Cube geometry node - bare minimum initialization and metadata

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// 3D Cube geometry node - main entry point
#[derive(Default)]
pub struct CubeNode3D;

impl NodeFactory for CubeNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "3D_Cube",
            "Cube",
            NodeCategory::new(&["3D", "Geometry"]),
            "Creates a 3D cube primitive with customizable parameters"
        )
        .with_color(Color32::from_rgb(100, 150, 200)) // Blue tint for geometry
        .with_icon("🟫")
        .with_inputs(vec![
            PortDefinition::optional("Transform", DataType::Any)
                .with_description("Optional transform matrix to position the cube"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Geometry", DataType::Any)
                .with_description("Generated cube geometry"),
        ])
        .with_tags(vec!["3d", "geometry", "primitive", "cube"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Medium)
        .with_workspace_compatibility(vec!["3D", "USD", "MaterialX"])
    }
}