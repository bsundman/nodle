//! USD Cylinder node module - modular structure with separated concerns

pub mod logic;
pub mod parameters;

pub use logic::USDCylinderLogic;
pub use parameters::USDCylinderNode;

use crate::nodes::NodeFactory;

impl NodeFactory for parameters::USDCylinderNode {
    fn metadata() -> crate::nodes::NodeMetadata {
        crate::nodes::NodeMetadata::new(
            "USD_Geometry_Cylinder",
            "USD Cylinder",
            crate::nodes::NodeCategory::new(&["3D", "USD", "Geometry", "Primitives"]),
            "Creates a USD cylinder primitive"
        )
        .with_color(egui::Color32::from_rgb(200, 150, 100))
        .with_icon("🥤")
        .with_inputs(vec![
            crate::nodes::PortDefinition::required("Stage", crate::nodes::DataType::Any)
                .with_description("USD Stage reference"),
            crate::nodes::PortDefinition::required("Parent Path", crate::nodes::DataType::String)
                .with_description("Parent prim path"),
            crate::nodes::PortDefinition::optional("Name", crate::nodes::DataType::String)
                .with_description("Prim name (auto-generated if empty)"),
            crate::nodes::PortDefinition::optional("Radius", crate::nodes::DataType::Float)
                .with_description("Cylinder radius"),
            crate::nodes::PortDefinition::optional("Height", crate::nodes::DataType::Float)
                .with_description("Cylinder height"),
        ])
        .with_outputs(vec![
            crate::nodes::PortDefinition::required("Prim Path", crate::nodes::DataType::String)
                .with_description("Created prim path"),
            crate::nodes::PortDefinition::required("Prim", crate::nodes::DataType::Any)
                .with_description("USD Prim reference"),
        ])
        .with_tags(vec!["usd", "geometry", "cylinder", "primitive"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Low)
        .with_workspace_compatibility(vec!["3D", "USD"])
        .with_panel_type(crate::nodes::interface::PanelType::Parameter)
    }
}