//! 3D Translation transform node

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

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