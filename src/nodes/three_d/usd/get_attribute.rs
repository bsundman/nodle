//! USD Get Attribute node - reads attributes from USD prims

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition, ProcessingCost};

/// Gets an attribute value from a USD prim
#[derive(Default)]
pub struct USDGetAttribute;

impl NodeFactory for USDGetAttribute {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "USD_GetAttribute",
            "Get Attribute",
            NodeCategory::new(&["3D", "USD", "Attributes"]),
            "Gets an attribute value from a USD prim"
        )
        .with_color(Color32::from_rgb(200, 150, 100))
        .with_icon("📋")
        .with_inputs(vec![
            PortDefinition::required("Prim", DataType::Any)
                .with_description("USD Prim to read from"),
            PortDefinition::required("Attribute", DataType::String)
                .with_description("Attribute name (e.g., 'xformOp:translate')"),
            PortDefinition::optional("Time", DataType::Float)
                .with_description("Time code for animated attributes"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Value", DataType::Any)
                .with_description("Attribute value"),
            PortDefinition::required("Type", DataType::String)
                .with_description("Attribute type name"),
        ])
        .with_workspace_compatibility(vec!["3D", "USD"])
        .with_tags(vec!["usd", "3d", "attribute", "read"])
        .with_processing_cost(ProcessingCost::Low)
    }
}