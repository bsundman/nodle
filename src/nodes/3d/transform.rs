//! 3D transform node implementations

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// 3D Translation transform node
#[derive(Default)]
pub struct TranslateNode3D;

impl NodeFactory for TranslateNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "3D_Translate",
            display_name: "Translate",
            category: NodeCategory::new(&["3D", "Transform"]),
            description: "Translates 3D geometry by a vector",
            color: Color32::from_rgb(120, 160, 200), // Blue-ish for transforms
            inputs: vec![
                PortDefinition::required("Input", DataType::Any)
                    .with_description("Geometry input"),
                PortDefinition::required("Vector", DataType::Vector3)
                    .with_description("Translation vector (x, y, z)"),
            ],
            outputs: vec![
                PortDefinition::required("Output", DataType::Any)
                    .with_description("Transformed geometry"),
            ],
        }
    }
}

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