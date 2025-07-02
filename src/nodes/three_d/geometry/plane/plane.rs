//! 3D Plane geometry node - bare minimum initialization and metadata

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// 3D Plane geometry node - main entry point
#[derive(Default)]
pub struct PlaneNode3D;

impl NodeFactory for PlaneNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "3D_Plane",
            "Plane",
            NodeCategory::new(&["3D", "Geometry"]),
            "Creates a 3D plane primitive with customizable parameters"
        )
        .with_color(Color32::from_rgb(100, 150, 200)) // Blue tint for geometry
        .with_icon("▬")
        .with_inputs(vec![
            PortDefinition::optional("Transform", DataType::Any)
                .with_description("Optional transform matrix to position the plane"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Geometry", DataType::Any)
                .with_description("Generated plane geometry"),
        ])
        .with_tags(vec!["3d", "geometry", "primitive", "plane"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Medium)
        .with_workspace_compatibility(vec!["3D", "USD", "MaterialX"])
    }
}