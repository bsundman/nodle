//! Spot light node module - enhanced NodeFactory system

pub mod logic;
pub mod parameters;

pub use logic::SpotLightLogic;
pub use parameters::SpotLightNode;

use crate::nodes::NodeFactory;

impl NodeFactory for parameters::SpotLightNode {
    fn metadata() -> crate::nodes::NodeMetadata {
        crate::nodes::NodeMetadata::new(
            "3D_SpotLight",
            "Spot Light",
            crate::nodes::NodeCategory::new(&["3D", "Lighting"]),
            "Creates a spot light with interface panel controls"
        )
        .with_color(egui::Color32::from_rgb(255, 255, 150))
        .with_icon("🔦")
        .with_inputs(vec![
            crate::nodes::PortDefinition::optional("Transform", crate::nodes::DataType::Any)
                .with_description("Optional transform matrix to position the light"),
        ])
        .with_outputs(vec![
            crate::nodes::PortDefinition::required("Light", crate::nodes::DataType::Any)
                .with_description("Light output for scene"),
        ])
        .with_tags(vec!["3d", "lighting", "spot", "cone", "directional", "interface"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Low)
        .with_workspace_compatibility(vec!["3D", "USD", "MaterialX"])
    }
}