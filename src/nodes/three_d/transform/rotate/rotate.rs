//! Rotate node - bare minimum initialization and metadata

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// 3D Rotation transform node - main entry point
#[derive(Default)]
pub struct RotateNode3D;

impl NodeFactory for RotateNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "3D_Rotate",
            "Rotate",
            NodeCategory::new(&["3D", "Transform"]),
            "Rotates 3D geometry by euler angles"
        )
        .with_color(Color32::from_rgb(120, 160, 200)) // Blue-ish for transforms
        .with_icon("🔄")
        .with_inputs(vec![
            PortDefinition::required("Input", DataType::Any)
                .with_description("Geometry input"),
            PortDefinition::required("Vector", DataType::Vector3)
                .with_description("Rotation angles (x, y, z) in degrees"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Output", DataType::Any)
                .with_description("Transformed geometry"),
        ])
        .with_tags(vec!["3d", "transform", "rotate", "spin"])
        .with_processing_cost(crate::nodes::ProcessingCost::Medium)
        .with_workspace_compatibility(vec!["3d", "modeling"])
    }
}