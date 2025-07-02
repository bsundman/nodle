//! Viewport node module - modular structure with separated concerns

pub mod viewport;
pub mod logic;
pub mod viewport_interface;

pub use viewport::ViewportNode3D;
pub use logic::ViewportLogic;
pub use viewport_interface::ViewportNode;

use crate::nodes::NodeFactory;
use crate::nodes::interface::{ParameterChange, InterfaceParameter};

impl NodeFactory for viewport_interface::ViewportNode {
    fn metadata() -> crate::nodes::NodeMetadata {
        // Create metadata for Pattern A viewport interface
        crate::nodes::NodeMetadata::viewport(
            "3D_Viewport",
            "3D Viewport",
            crate::nodes::NodeCategory::new(&["3D", "Output"]),
            "3D viewport with Pattern A interface for scene visualization and rendering"
        )
        .with_inputs(vec![
            crate::nodes::PortDefinition::required("Scene", crate::nodes::DataType::Any)
                .with_description("Complete scene data to render in viewport"),
        ])
        .with_outputs(vec![
            crate::nodes::PortDefinition::optional("Rendered Image", crate::nodes::DataType::Any)
                .with_description("Captured viewport image"),
            crate::nodes::PortDefinition::optional("Depth Buffer", crate::nodes::DataType::Any)
                .with_description("Depth information from render"),
        ])
        .with_size_hint(egui::Vec2::new(160.0, 120.0))
        .with_workspace_compatibility(vec!["3D", "USD", "MaterialX"])
        .with_tags(vec!["3d", "viewport", "output", "render", "wgpu", "pattern-a"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::High)
        .with_version("2.1") // Increment version to indicate Pattern A conversion
    }
}