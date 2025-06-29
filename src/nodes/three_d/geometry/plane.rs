//! 3D Plane geometry node

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// 3D Plane geometry node
#[derive(Default)]
pub struct PlaneNode3D;

impl NodeFactory for PlaneNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "3D_Plane",
            "Plane",
            NodeCategory::new(&["3D", "Geometry"]),
            "Creates a 3D plane primitive"
        )
        .with_color(Color32::from_rgb(160, 120, 200)) // Purple-ish for geometry
        .with_icon("▬")
        .with_inputs(vec![
            PortDefinition::required("Width", DataType::Float)
                .with_description("Width of the plane"),
            PortDefinition::required("Height", DataType::Float)
                .with_description("Height of the plane"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Geometry", DataType::Any)
                .with_description("Generated plane geometry"),
        ])
        .with_tags(vec!["3d", "geometry", "primitive", "plane"])
        .with_processing_cost(crate::nodes::ProcessingCost::Medium)
        .with_workspace_compatibility(vec!["3d", "modeling"])
    }
}