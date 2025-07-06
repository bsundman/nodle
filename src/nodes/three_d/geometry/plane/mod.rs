//! Plane node module - enhanced NodeFactory system

pub mod logic;
pub mod parameters;

// logic::PlaneGeometry removed - only used internally
pub use parameters::PlaneNode;

use crate::nodes::NodeFactory;

impl NodeFactory for parameters::PlaneNode {
    fn metadata() -> crate::nodes::NodeMetadata {
        // Create new metadata specifically for the interface version
        crate::nodes::NodeMetadata::new(
            "3D_Plane",
            "Plane",
            crate::nodes::NodeCategory::new(&["3D", "Geometry"]),
            "Creates a plane primitive with interface panel controls"
        )
        .with_color(egui::Color32::from_rgb(100, 150, 200))
        .with_icon("▬")
        .with_inputs(vec![
            crate::nodes::PortDefinition::optional("Transform", crate::nodes::DataType::Any)
                .with_description("Optional transform matrix to position the plane"),
        ])
        .with_outputs(vec![
            crate::nodes::PortDefinition::required("Geometry", crate::nodes::DataType::Any)
                .with_description("Generated plane geometry"),
        ])
        .with_tags(vec!["3d", "geometry", "primitive", "plane", "interface"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Medium)
        .with_workspace_compatibility(vec!["3D", "USD", "MaterialX"])
    }
}