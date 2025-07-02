//! USD Sphere node module - modular structure with separated concerns

pub mod logic;
pub mod parameters;

pub use logic::USDSphereLogic;
pub use parameters::USDSphereNode;

use crate::nodes::NodeFactory;

impl NodeFactory for parameters::USDSphereNode {
    fn metadata() -> crate::nodes::NodeMetadata {
        crate::nodes::NodeMetadata::new(
            "USD_Geometry_Sphere",
            "USD Sphere",
            crate::nodes::NodeCategory::new(&["3D", "USD", "Geometry", "Primitives"]),
            "Creates a USD sphere primitive"
        )
        .with_color(egui::Color32::from_rgb(200, 150, 100))
        .with_icon("🔵")
        .with_inputs(vec![
            crate::nodes::PortDefinition::required("Stage", crate::nodes::DataType::Any)
                .with_description("USD Stage reference"),
            crate::nodes::PortDefinition::required("Parent Path", crate::nodes::DataType::String)
                .with_description("Parent prim path"),
            crate::nodes::PortDefinition::optional("Name", crate::nodes::DataType::String)
                .with_description("Prim name (auto-generated if empty)"),
            crate::nodes::PortDefinition::optional("Radius", crate::nodes::DataType::Float)
                .with_description("Sphere radius"),
            crate::nodes::PortDefinition::optional("Transform", crate::nodes::DataType::Any)
                .with_description("Transform matrix"),
        ])
        .with_outputs(vec![
            crate::nodes::PortDefinition::required("Prim Path", crate::nodes::DataType::String)
                .with_description("Created prim path"),
            crate::nodes::PortDefinition::required("Prim", crate::nodes::DataType::Any)
                .with_description("USD Prim reference"),
        ])
        .with_tags(vec!["usd", "geometry", "sphere", "primitive"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Low)
        .with_workspace_compatibility(vec!["3D", "USD"])
        .with_panel_type(crate::nodes::interface::PanelType::Parameter)
    }
}