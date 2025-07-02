//! USD Material node module - modular structure with separated concerns

pub mod logic;
pub mod parameters;

pub use logic::USDMaterialLogic;
pub use parameters::USDMaterialNode;

use crate::nodes::NodeFactory;

impl NodeFactory for parameters::USDMaterialNode {
    fn metadata() -> crate::nodes::NodeMetadata {
        crate::nodes::NodeMetadata::new(
            "USD_Shading_Material",
            "USD Material",
            crate::nodes::NodeCategory::new(&["3D", "USD", "Shading", "Materials"]),
            "Creates a USD material for surface shading"
        )
        .with_color(egui::Color32::from_rgb(150, 100, 200))
        .with_icon("🎨")
        .with_inputs(vec![
            crate::nodes::PortDefinition::required("Stage", crate::nodes::DataType::Any)
                .with_description("USD Stage reference"),
            crate::nodes::PortDefinition::required("Parent Path", crate::nodes::DataType::String)
                .with_description("Parent prim path"),
            crate::nodes::PortDefinition::optional("Name", crate::nodes::DataType::String)
                .with_description("Material name (auto-generated if empty)"),
            crate::nodes::PortDefinition::optional("Surface Shader", crate::nodes::DataType::Any)
                .with_description("Surface shader input"),
        ])
        .with_outputs(vec![
            crate::nodes::PortDefinition::required("Material Path", crate::nodes::DataType::String)
                .with_description("Created material path"),
            crate::nodes::PortDefinition::required("Material", crate::nodes::DataType::Any)
                .with_description("USD Material reference"),
            crate::nodes::PortDefinition::required("Surface Output", crate::nodes::DataType::Any)
                .with_description("Surface shader output"),
        ])
        .with_tags(vec!["usd", "shading", "material", "surface"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Low)
        .with_workspace_compatibility(vec!["3D", "USD"])
        .with_panel_type(crate::nodes::interface::PanelType::Parameter)
    }
}