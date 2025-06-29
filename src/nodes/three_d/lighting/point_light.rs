//! 3D Point Light node

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// 3D Point Light node
#[derive(Default)]
pub struct PointLightNode3D;

impl NodeFactory for PointLightNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "3D_PointLight",
            "Point Light",
            NodeCategory::new(&["3D", "Lighting"]),
            "Creates a point light that emits in all directions"
        )
        .with_color(Color32::from_rgb(255, 255, 150)) // Yellow-ish for lights
        .with_icon("💡")
        .with_inputs(vec![
            PortDefinition::required("Position", DataType::Vector3)
                .with_description("World position of the light"),
            PortDefinition::required("Color", DataType::Color)
                .with_description("Light color"),
            PortDefinition::required("Intensity", DataType::Float)
                .with_description("Light intensity/brightness"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Light", DataType::Any)
                .with_description("Light output for scene"),
        ])
        .with_tags(vec!["3d", "lighting", "point", "omnidirectional"])
        .with_processing_cost(crate::nodes::ProcessingCost::Low)
        .with_workspace_compatibility(vec!["3d", "rendering"])
    }
}