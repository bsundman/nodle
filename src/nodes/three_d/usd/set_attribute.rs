//! USD Set Attribute node - sets attributes on USD prims

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Sets an attribute value on a USD prim
#[derive(Default)]
pub struct USDSetAttribute;

impl NodeFactory for USDSetAttribute {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "USD_SetAttribute",
            display_name: "Set Attribute",
            category: NodeCategory::new(&["3D", "USD", "Attributes"]),
            description: "Sets an attribute value on a USD prim",
            color: Color32::from_rgb(200, 150, 100), // Orange-brown for USD nodes
            inputs: vec![
                PortDefinition::required("Prim", DataType::Any)
                    .with_description("USD Prim to modify"),
                PortDefinition::required("Attribute", DataType::String)
                    .with_description("Attribute name (e.g., 'xformOp:translate')"),
                PortDefinition::required("Value", DataType::Any)
                    .with_description("Attribute value"),
                PortDefinition::optional("Time", DataType::Float)
                    .with_description("Time code for animated attributes"),
            ],
            outputs: vec![
                PortDefinition::required("Prim", DataType::Any)
                    .with_description("Modified USD Prim"),
            ],
        }
    }
}