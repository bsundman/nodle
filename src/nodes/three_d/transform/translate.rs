//! 3D Translation transform node

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// 3D Translation transform node
#[derive(Default)]
pub struct TranslateNode3D;

impl NodeFactory for TranslateNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "3D_Translate",
            "Translate",
            NodeCategory::new(&["3D", "Transform"]),
            "Translates 3D geometry by a vector"
        )
        .with_color(Color32::from_rgb(120, 160, 200)) // Blue-ish for transforms
        .with_icon("↔️")
        .with_inputs(vec![
            PortDefinition::required("Input", DataType::Any)
                .with_description("Geometry input"),
            PortDefinition::required("Vector", DataType::Vector3)
                .with_description("Translation vector (x, y, z)"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Output", DataType::Any)
                .with_description("Transformed geometry"),
        ])
        .with_tags(vec!["3d", "transform", "translate", "move"])
        .with_processing_cost(crate::nodes::ProcessingCost::Medium)
        .with_workspace_compatibility(vec!["3d", "modeling"])
    }
}