//! USD Set Attribute node - sets attributes on USD prims

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition, ProcessingCost};

/// Sets an attribute value on a USD prim
#[derive(Default)]
pub struct USDSetAttribute;

impl NodeFactory for USDSetAttribute {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "USD_SetAttribute",
            "Set Attribute",
            NodeCategory::new(&["3D", "USD", "Attributes"]),
            "Sets an attribute value on a USD prim"
        )
        .with_color(Color32::from_rgb(200, 150, 100))
        .with_icon("⚙️")
        .with_inputs(vec![
            PortDefinition::required("Prim", DataType::Any)
                .with_description("USD Prim to modify"),
            PortDefinition::required("Attribute", DataType::String)
                .with_description("Attribute name (e.g., 'xformOp:translate')"),
            PortDefinition::required("Value", DataType::Any)
                .with_description("Attribute value"),
            PortDefinition::optional("Time", DataType::Float)
                .with_description("Time code for animated attributes"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Prim", DataType::Any)
                .with_description("Modified USD Prim"),
        ])
        .with_workspace_compatibility(vec!["3D", "USD"])
        .with_tags(vec!["usd", "3d", "attribute", "modify"])
        .with_processing_cost(ProcessingCost::Low)
    }
}