//! Scale node module - enhanced NodeFactory system

pub mod logic;
pub mod parameters;

// logic::ScaleLogic removed - only used internally
pub use parameters::ScaleNode;

use crate::nodes::NodeFactory;

impl NodeFactory for parameters::ScaleNode {
    fn metadata() -> crate::nodes::NodeMetadata {
        // Create new metadata specifically for the Pattern A interface version
        crate::nodes::NodeMetadata::new(
            "3D_Scale",
            "Scale",
            crate::nodes::NodeCategory::new(&["3D", "Transform"]),
            "Scales 3D geometry with Pattern A interface controls"
        )
        .with_color(egui::Color32::from_rgb(120, 160, 200))
        .with_icon("🔍")
        .with_inputs(vec![
            crate::nodes::PortDefinition::optional("Transform", crate::nodes::DataType::Any)
                .with_description("Optional transform matrix to combine with scaling"),
        ])
        .with_outputs(vec![
            crate::nodes::PortDefinition::required("Transform", crate::nodes::DataType::Any)
                .with_description("Scale transform matrix"),
        ])
        .with_tags(vec!["3d", "transform", "scale", "resize", "interface", "pattern_a"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Medium)
        .with_workspace_compatibility(vec!["3D", "USD", "MaterialX"])
    }
}