//! USD Get Attribute node - reads attributes from USD prims

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Gets an attribute value from a USD prim
#[derive(Default)]
pub struct USDGetAttribute;

impl NodeFactory for USDGetAttribute {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "USD_GetAttribute",
            display_name: "Get Attribute",
            category: NodeCategory::new(&["3D", "USD", "Attributes"]),
            description: "Gets an attribute value from a USD prim",
            color: Color32::from_rgb(200, 150, 100), // Orange-brown for USD nodes
            inputs: vec![
                PortDefinition::required("Prim", DataType::Any)
                    .with_description("USD Prim to read from"),
                PortDefinition::required("Attribute", DataType::String)
                    .with_description("Attribute name (e.g., 'xformOp:translate')"),
                PortDefinition::optional("Time", DataType::Float)
                    .with_description("Time code for animated attributes"),
            ],
            outputs: vec![
                PortDefinition::required("Value", DataType::Any)
                    .with_description("Attribute value"),
                PortDefinition::required("Type", DataType::String)
                    .with_description("Attribute type name"),
            ],
        }
    }
}