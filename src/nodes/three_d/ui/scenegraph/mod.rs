//! Scenegraph node for displaying USD scene hierarchy in a tree view

mod parameters;
pub mod logic;

pub use parameters::ScenegraphNode;
pub use logic::ScenegraphLogic;

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Scenegraph node factory that displays USD scene hierarchy in a tree view
#[derive(Default)]
pub struct ScenegraphNodeFactory;

impl NodeFactory for ScenegraphNodeFactory {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Scenegraph",
            "Scenegraph", 
            NodeCategory::new(&["UI"]),
            "Display USD scene hierarchy in a tree view"
        )
        .with_color(Color32::from_rgb(80, 140, 100))
        .with_icon("🌳")
        .with_panel_type(crate::nodes::interface::PanelType::Tree)
        .with_inputs(vec![
            PortDefinition::required("USD Scene", DataType::Any)
                .with_description("USD scene data to visualize in tree view"),
        ])
        .with_outputs(vec![])
        .with_tags(vec!["output", "usd", "scene", "hierarchy", "tree", "viewer"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Minimal)
        .with_workspace_compatibility(vec!["USD", "3D", "General"])
    }
}