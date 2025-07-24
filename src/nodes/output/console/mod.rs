//! Console node implementation
//!
//! Console node for displaying text output and executable messages

mod logic;
pub mod parameters;
pub mod viewer;

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

pub use logic::ConsoleLogic;
pub use viewer::ConsoleViewer;

/// Console node factory that provides text output display
#[derive(Default)]
pub struct ConsoleNodeFactory;

impl NodeFactory for ConsoleNodeFactory {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Console",
            "Console",
            NodeCategory::output(),
            "Display text output and executable messages in a terminal-like console window"
        )
        .with_color(Color32::from_rgb(0, 0, 0)) // Black like a terminal
        .with_icon("🖥️")
        .with_panel_type(crate::nodes::interface::PanelType::Viewer)
        .with_inputs(vec![
            PortDefinition::optional("Text", DataType::String)
                .with_description("Text to display in console"),
        ])
        .with_outputs(vec![])
        .with_workspace_compatibility(vec!["2D", "3D", "USD", "MaterialX", "General"])
        .with_tags(vec!["output", "console", "text", "debug", "terminal"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Low)
    }
}