//! 3D Scale transform node

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// 3D Scale transform node
#[derive(Default)]
pub struct ScaleNode3D;

impl NodeFactory for ScaleNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "3D_Scale",
            display_name: "Scale",
            category: NodeCategory::new(&["3D", "Transform"]),
            description: "Scales 3D geometry by a factor",
            color: Color32::from_rgb(120, 160, 200), // Blue-ish for transforms
            inputs: vec![
                PortDefinition::required("Input", DataType::Any)
                    .with_description("Geometry input"),
                PortDefinition::required("Vector", DataType::Vector3)
                    .with_description("Scale factors (x, y, z)"),
            ],
            outputs: vec![
                PortDefinition::required("Output", DataType::Any)
                    .with_description("Transformed geometry"),
            ],
        }
    }
}