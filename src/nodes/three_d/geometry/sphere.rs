//! 3D Sphere geometry node

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// 3D Sphere geometry node
#[derive(Default)]
pub struct SphereNode3D;

impl NodeFactory for SphereNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "3D_Sphere",
            "Sphere",
            NodeCategory::new(&["3D", "Geometry"]),
            "Creates a 3D sphere primitive"
        )
        .with_color(Color32::from_rgb(160, 120, 200)) // Purple-ish for geometry
        .with_icon("🔵")
        .with_inputs(vec![
            PortDefinition::required("Radius", DataType::Float)
                .with_description("Radius of the sphere"),
            PortDefinition::optional("Subdivisions", DataType::Float)
                .with_description("Number of subdivisions for smoothness"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Geometry", DataType::Any)
                .with_description("Generated sphere geometry"),
        ])
        .with_tags(vec!["3d", "geometry", "primitive", "sphere"])
        .with_processing_cost(crate::nodes::ProcessingCost::Medium)
        .with_workspace_compatibility(vec!["3d", "modeling"])
    }
}