//! 3D Point Light node

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// 3D Point Light node
#[derive(Default)]
pub struct PointLightNode3D;

impl NodeFactory for PointLightNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "3D_PointLight",
            display_name: "Point Light",
            category: NodeCategory::new(&["3D", "Lighting"]),
            description: "Creates a point light that emits in all directions",
            color: Color32::from_rgb(255, 255, 150), // Yellow-ish for lights
            inputs: vec![
                PortDefinition::required("Position", DataType::Vector3)
                    .with_description("World position of the light"),
                PortDefinition::required("Color", DataType::Color)
                    .with_description("Light color"),
                PortDefinition::required("Intensity", DataType::Float)
                    .with_description("Light intensity/brightness"),
            ],
            outputs: vec![
                PortDefinition::required("Light", DataType::Any)
                    .with_description("Light output for scene"),
            ],
        }
    }
}