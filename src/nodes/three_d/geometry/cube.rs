//! 3D Cube geometry node

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// 3D Cube geometry node
#[derive(Default)]
pub struct CubeNode3D;

impl NodeFactory for CubeNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "3D_Cube",
            "Cube",
            NodeCategory::new(&["3D", "Geometry"]),
            "Creates a 3D cube primitive"
        )
        .with_color(Color32::from_rgb(160, 120, 200)) // Purple-ish for geometry
        .with_icon("🟫")
        .with_inputs(vec![
            PortDefinition::required("Size", DataType::Float)
                .with_description("Size of the cube"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Geometry", DataType::Any)
                .with_description("Generated cube geometry"),
        ])
        .with_tags(vec!["3d", "geometry", "primitive", "cube"])
        .with_processing_cost(crate::nodes::ProcessingCost::Medium)
        .with_workspace_compatibility(vec!["3d", "modeling"])
    }
}