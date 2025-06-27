//! 3D Rotation transform node

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// 3D Rotation transform node
#[derive(Default)]
pub struct RotateNode3D;

impl NodeFactory for RotateNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "3D_Rotate",
            display_name: "Rotate",
            category: NodeCategory::new(&["3D", "Transform"]),
            description: "Rotates 3D geometry by euler angles",
            color: Color32::from_rgb(120, 160, 200), // Blue-ish for transforms
            inputs: vec![
                PortDefinition::required("Input", DataType::Any)
                    .with_description("Geometry input"),
                PortDefinition::required("Vector", DataType::Vector3)
                    .with_description("Rotation angles (x, y, z) in degrees"),
            ],
            outputs: vec![
                PortDefinition::required("Output", DataType::Any)
                    .with_description("Transformed geometry"),
            ],
        }
    }
}