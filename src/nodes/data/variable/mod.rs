//! Variable node implementation
//!
//! Uses Pattern A: build_interface method
//! - mod.rs: Base node metadata and factory implementation
//! - functions.rs: Core computation logic
//! - parameters.rs: Pattern A interface with build_interface method
//! - logic.rs: Legacy logic (kept for compatibility)

pub mod logic;
mod functions;
pub mod parameters;

pub use functions::*;
pub use parameters::*;

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Variable node that stores and outputs a mutable value
#[derive(Default)]
pub struct VariableNodeFactory;

impl NodeFactory for VariableNodeFactory {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Data_Variable",
            "Variable",
            NodeCategory::new(&["Data", "Storage"]),
            "Stores and outputs a variable value with interface panel controls"
        )
        .with_color(Color32::from_rgb(65, 55, 75))
        .with_icon("V")
        .with_inputs(vec![
            PortDefinition::optional("Set", DataType::Any)
                .with_description("Input to set the variable value"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Value", DataType::Any)
                .with_description("The current variable value"),
        ])
        .with_tags(vec!["data", "variable", "storage", "mutable", "interface"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Minimal)
        .with_workspace_compatibility(vec!["General", "Data", "Programming"])
    }
}