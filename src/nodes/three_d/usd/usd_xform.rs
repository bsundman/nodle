//! USD Xform node - creates a transform primitive

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Creates a USD Xform (transform) primitive
#[derive(Default)]
pub struct USDXform;

impl NodeFactory for USDXform {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "USD_Xform",
            display_name: "USD Xform",
            category: NodeCategory::new(&["3D", "USD", "Primitives"]),
            description: "Creates a USD transform primitive for hierarchical scene organization",
            color: Color32::from_rgb(200, 150, 100), // Orange-brown for USD nodes
            inputs: vec![
                PortDefinition::required("Stage", DataType::Any)
                    .with_description("USD Stage reference"),
                PortDefinition::required("Path", DataType::String)
                    .with_description("Prim path (e.g., /World/MyTransform)"),
                PortDefinition::optional("Parent", DataType::Any)
                    .with_description("Parent prim (optional)"),
            ],
            outputs: vec![
                PortDefinition::required("Prim", DataType::Any)
                    .with_description("USD Xform prim"),
                PortDefinition::required("Stage", DataType::Any)
                    .with_description("Pass-through stage reference"),
            ],
        }
    }
}