//! USD Rect Light node module - modular structure with separated concerns

pub mod logic;
pub mod parameters;

pub use logic::USDRectLightLogic;
pub use parameters::USDRectLightNode;

use crate::nodes::NodeFactory;

impl NodeFactory for parameters::USDRectLightNode {
    fn metadata() -> crate::nodes::NodeMetadata {
        crate::nodes::NodeMetadata::new(
            "USD_Lighting_RectLight",
            "USD Rect Light",
            crate::nodes::NodeCategory::new(&["3D", "USD", "Lighting"]),
            "Creates a USD rectangular area light"
        )
        .with_color(egui::Color32::from_rgb(255, 200, 100))
        .with_icon("💡")
        .with_inputs(vec![
            crate::nodes::PortDefinition::required("Stage", crate::nodes::DataType::Any)
                .with_description("USD Stage reference"),
            crate::nodes::PortDefinition::required("Parent Path", crate::nodes::DataType::String)
                .with_description("Parent prim path"),
            crate::nodes::PortDefinition::optional("Name", crate::nodes::DataType::String)
                .with_description("Light name (auto-generated if empty)"),
        ])
        .with_outputs(vec![
            crate::nodes::PortDefinition::required("Light Path", crate::nodes::DataType::String)
                .with_description("Created light path"),
            crate::nodes::PortDefinition::required("Light", crate::nodes::DataType::Any)
                .with_description("USD Light reference"),
        ])
        .with_tags(vec!["usd", "lighting", "rect", "area"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Low)
        .with_workspace_compatibility(vec!["3D", "USD"])
        .with_panel_type(crate::nodes::interface::PanelType::Parameter)
    }
}