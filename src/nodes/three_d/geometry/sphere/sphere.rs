//! 3D Sphere geometry node - bare minimum initialization and metadata

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// 3D Sphere geometry node - main entry point
#[derive(Default)]
pub struct SphereNode3D;

impl NodeFactory for SphereNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "3D_Sphere",
            "Sphere",
            NodeCategory::new(&["3D", "Geometry"]),
            "Creates a 3D sphere primitive with customizable parameters"
        )
        .with_color(Color32::from_rgb(100, 150, 200)) // Blue tint for geometry
        .with_icon("🔵")
        .with_inputs(vec![
            PortDefinition::optional("Transform", DataType::Any)
                .with_description("Optional transform matrix to position the sphere"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Geometry", DataType::Any)
                .with_description("Generated sphere geometry"),
        ])
        .with_tags(vec!["3d", "geometry", "primitive", "sphere"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Medium)
        .with_workspace_compatibility(vec!["3D", "USD", "MaterialX"])
    }
}