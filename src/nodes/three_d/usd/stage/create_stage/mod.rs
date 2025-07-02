//! Create Stage node module - modular structure with separated concerns

pub mod logic;
pub mod parameters;

pub use logic::CreateStageLogic;
pub use parameters::CreateStageNode;

use crate::nodes::NodeFactory;

impl NodeFactory for parameters::CreateStageNode {
    fn metadata() -> crate::nodes::NodeMetadata {
        crate::nodes::NodeMetadata::new(
            "USD_Stage_Create",
            "Create Stage",
            crate::nodes::NodeCategory::new(&["3D", "USD", "Stage"]),
            "Creates a new USD stage for scene composition"
        )
        .with_color(egui::Color32::from_rgb(180, 120, 80))
        .with_icon("📄")
        .with_inputs(vec![
            crate::nodes::PortDefinition::optional("Identifier", crate::nodes::DataType::String)
                .with_description("Stage identifier (defaults to 'default')"),
        ])
        .with_outputs(vec![
            crate::nodes::PortDefinition::required("Stage", crate::nodes::DataType::Any)
                .with_description("USD Stage reference"),
            crate::nodes::PortDefinition::required("Root Path", crate::nodes::DataType::String)
                .with_description("Root prim path (/)")
        ])
        .with_tags(vec!["usd", "stage", "create", "scene"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Low)
        .with_workspace_compatibility(vec!["3D", "USD"])
        .with_panel_type(crate::nodes::interface::PanelType::Parameter)
    }
}