//! 3D Directional Light node

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// 3D Directional Light node
#[derive(Default)]
pub struct DirectionalLightNode3D;

impl NodeFactory for DirectionalLightNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "3D_DirectionalLight",
            display_name: "Directional Light",
            category: NodeCategory::new(&["3D", "Lighting"]),
            description: "Creates a directional light (like sunlight)",
            color: Color32::from_rgb(255, 255, 150), // Yellow-ish for lights
            inputs: vec![
                PortDefinition::required("Position", DataType::Vector3)
                    .with_description("Light position"),
                PortDefinition::required("Direction", DataType::Vector3)
                    .with_description("Light direction vector"),
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