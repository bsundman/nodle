//! Translate node module - modular structure with separated concerns

pub mod translate;
pub mod logic;
pub mod parameters;

pub use translate::TranslateNode3D;
pub use logic::TranslateLogic;
pub use parameters::TranslateNode;

use crate::nodes::NodeFactory;

impl NodeFactory for parameters::TranslateNode {
    fn metadata() -> crate::nodes::NodeMetadata {
        // Create new metadata specifically for the Pattern A interface version
        crate::nodes::NodeMetadata::new(
            "3D_Translate",
            "Translate",
            crate::nodes::NodeCategory::new(&["3D", "Transform"]),
            "Translates 3D geometry with Pattern A interface controls"
        )
        .with_color(egui::Color32::from_rgb(120, 160, 200))
        .with_icon("↔️")
        .with_inputs(vec![
            crate::nodes::PortDefinition::optional("Transform", crate::nodes::DataType::Any)
                .with_description("Optional transform matrix to combine with translation"),
        ])
        .with_outputs(vec![
            crate::nodes::PortDefinition::required("Transform", crate::nodes::DataType::Any)
                .with_description("Translation transform matrix"),
        ])
        .with_tags(vec!["3d", "transform", "translate", "move", "interface", "pattern_a"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Medium)
        .with_workspace_compatibility(vec!["3D", "USD", "MaterialX"])
    }
}