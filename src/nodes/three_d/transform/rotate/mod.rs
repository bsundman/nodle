//! Rotate node module - modular structure with separated concerns

pub mod rotate;
pub mod logic;
pub mod parameters;

pub use rotate::RotateNode3D;
pub use logic::RotateLogic;
pub use parameters::RotateNode;

use crate::nodes::NodeFactory;

impl NodeFactory for parameters::RotateNode {
    fn metadata() -> crate::nodes::NodeMetadata {
        // Create new metadata specifically for the Pattern A interface version
        crate::nodes::NodeMetadata::new(
            "3D_Rotate",
            "Rotate",
            crate::nodes::NodeCategory::new(&["3D", "Transform"]),
            "Rotates 3D geometry with Pattern A interface controls"
        )
        .with_color(egui::Color32::from_rgb(120, 160, 200))
        .with_icon("🔄")
        .with_inputs(vec![
            crate::nodes::PortDefinition::optional("Transform", crate::nodes::DataType::Any)
                .with_description("Optional transform matrix to combine with rotation"),
        ])
        .with_outputs(vec![
            crate::nodes::PortDefinition::required("Transform", crate::nodes::DataType::Any)
                .with_description("Rotation transform matrix"),
        ])
        .with_tags(vec!["3d", "transform", "rotate", "spin", "interface", "pattern_a"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Medium)
        .with_workspace_compatibility(vec!["3D", "USD", "MaterialX"])
    }
}