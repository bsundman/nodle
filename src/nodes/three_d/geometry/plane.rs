//! 3D Plane geometry node

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// 3D Plane geometry node
#[derive(Default)]
pub struct PlaneNode3D;

impl NodeFactory for PlaneNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "3D_Plane",
            display_name: "Plane",
            category: NodeCategory::new(&["3D", "Geometry"]),
            description: "Creates a 3D plane primitive",
            color: Color32::from_rgb(160, 120, 200), // Purple-ish for geometry
            inputs: vec![
                PortDefinition::required("Width", DataType::Float)
                    .with_description("Width of the plane"),
                PortDefinition::required("Height", DataType::Float)
                    .with_description("Height of the plane"),
            ],
            outputs: vec![
                PortDefinition::required("Geometry", DataType::Any)
                    .with_description("Generated plane geometry"),
            ],
        }
    }
}