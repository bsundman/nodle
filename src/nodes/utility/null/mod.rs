//! Null node module - enhanced NodeFactory system

pub mod logic;
pub mod parameters;

pub use logic::NullLogic;
pub use parameters::NullNode;

use crate::nodes::NodeFactory;

impl NodeFactory for parameters::NullNode {
    fn metadata() -> crate::nodes::NodeMetadata {
        crate::nodes::NodeMetadata::new(
            "Null",
            "Null",
            crate::nodes::NodeCategory::new(&["Utility"]),
            "A simple passthrough node for organization and hierarchy"
        )
        .with_color(egui::Color32::from_rgb(100, 100, 100))
        .with_icon("⬜")
        .with_inputs(vec![
            crate::nodes::PortDefinition::optional("Input", crate::nodes::DataType::Any)
                .with_description("Any input data to pass through"),
        ])
        .with_outputs(vec![
            crate::nodes::PortDefinition::optional("Output", crate::nodes::DataType::Any)
                .with_description("Passthrough of input data"),
        ])
        .with_panel_type(crate::nodes::interface::PanelType::Parameter)
        .with_tags(vec!["utility", "null", "passthrough", "organization", "placeholder"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Low)
        .with_workspace_compatibility(vec!["3D", "General", "USD", "MaterialX"])
    }
}