//! Point light node module - modular structure with separated concerns

pub mod point_light;
pub mod logic;
pub mod parameters;

pub use point_light::PointLightNode3D;
pub use logic::PointLightLogic;
pub use parameters::PointLightNode;

use crate::nodes::NodeFactory;

impl NodeFactory for parameters::PointLightNode {
    fn metadata() -> crate::nodes::NodeMetadata {
        // Create new metadata specifically for the interface version
        crate::nodes::NodeMetadata::new(
            "3D_PointLight",
            "Point Light",
            crate::nodes::NodeCategory::new(&["3D", "Lighting"]),
            "Creates a point light with interface panel controls"
        )
        .with_color(egui::Color32::from_rgb(255, 255, 150))
        .with_icon("💡")
        .with_inputs(vec![
            crate::nodes::PortDefinition::optional("Transform", crate::nodes::DataType::Any)
                .with_description("Optional transform matrix to position the light"),
        ])
        .with_outputs(vec![
            crate::nodes::PortDefinition::required("Light", crate::nodes::DataType::Any)
                .with_description("Light output for scene"),
        ])
        .with_tags(vec!["3d", "lighting", "point", "omnidirectional", "interface"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Low)
        .with_workspace_compatibility(vec!["3D", "USD", "MaterialX"])
    }
}