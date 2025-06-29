//! 3D Scale transform node

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// 3D Scale transform node
#[derive(Default)]
pub struct ScaleNode3D;

impl NodeFactory for ScaleNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "3D_Scale",
            "Scale",
            NodeCategory::new(&["3D", "Transform"]),
            "Scales 3D geometry by a factor"
        )
        .with_color(Color32::from_rgb(120, 160, 200)) // Blue-ish for transforms
        .with_icon("🔍")
        .with_inputs(vec![
            PortDefinition::required("Input", DataType::Any)
                .with_description("Geometry input"),
            PortDefinition::required("Vector", DataType::Vector3)
                .with_description("Scale factors (x, y, z)"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Output", DataType::Any)
                .with_description("Transformed geometry"),
        ])
        .with_tags(vec!["3d", "transform", "scale", "resize"])
        .with_processing_cost(crate::nodes::ProcessingCost::Medium)
        .with_workspace_compatibility(vec!["3d", "modeling"])
    }
}