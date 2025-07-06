//! Cube node module - enhanced NodeFactory system

pub mod logic;
pub mod parameters;

// logic types removed - only used internally
pub use parameters::CubeNode;

use crate::nodes::NodeFactory;

impl NodeFactory for parameters::CubeNode {
    fn metadata() -> crate::nodes::NodeMetadata {
        // Create new metadata specifically for the interface version
        crate::nodes::NodeMetadata::new(
            "3D_Cube",
            "Cube",
            crate::nodes::NodeCategory::new(&["3D", "Geometry"]),
            "Creates a cube primitive with interface panel controls"
        )
        .with_color(egui::Color32::from_rgb(100, 150, 200))
        .with_icon("🟫")
        .with_inputs(vec![
            crate::nodes::PortDefinition::optional("Transform", crate::nodes::DataType::Any)
                .with_description("Optional transform matrix to position the cube"),
        ])
        .with_outputs(vec![
            crate::nodes::PortDefinition::required("Geometry", crate::nodes::DataType::Any)
                .with_description("Generated cube geometry"),
        ])
        .with_tags(vec!["3d", "geometry", "primitive", "cube", "interface"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Medium)
        .with_workspace_compatibility(vec!["3D", "USD", "MaterialX"])
    }
}