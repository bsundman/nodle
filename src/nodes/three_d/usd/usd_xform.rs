//! USD Xform node - creates a transform primitive

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition, ProcessingCost};

/// Creates a USD Xform (transform) primitive
#[derive(Default)]
pub struct USDXform;

impl NodeFactory for USDXform {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "USD_Xform",
            "USD Xform",
            NodeCategory::new(&["3D", "USD", "Primitives"]),
            "Creates a USD transform primitive for hierarchical scene organization"
        )
        .with_color(Color32::from_rgb(200, 150, 100))
        .with_icon("🔧")
        .with_inputs(vec![
            PortDefinition::required("Stage", DataType::Any)
                .with_description("USD Stage reference"),
            PortDefinition::required("Path", DataType::String)
                .with_description("Prim path (e.g., /World/MyTransform)"),
            PortDefinition::optional("Parent", DataType::Any)
                .with_description("Parent prim (optional)"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Prim", DataType::Any)
                .with_description("USD Xform prim"),
            PortDefinition::required("Stage", DataType::Any)
                .with_description("Pass-through stage reference"),
        ])
        .with_workspace_compatibility(vec!["3D", "USD"])
        .with_tags(vec!["usd", "3d", "transform", "hierarchy"])
        .with_processing_cost(ProcessingCost::Low)
    }
}