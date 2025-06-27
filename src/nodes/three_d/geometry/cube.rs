//! 3D Cube geometry node

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// 3D Cube geometry node
#[derive(Default)]
pub struct CubeNode3D;

impl NodeFactory for CubeNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "3D_Cube",
            display_name: "Cube",
            category: NodeCategory::new(&["3D", "Geometry"]),
            description: "Creates a 3D cube primitive",
            color: Color32::from_rgb(160, 120, 200), // Purple-ish for geometry
            inputs: vec![
                PortDefinition::required("Size", DataType::Float)
                    .with_description("Size of the cube"),
            ],
            outputs: vec![
                PortDefinition::required("Geometry", DataType::Any)
                    .with_description("Generated cube geometry"),
            ],
        }
    }
}