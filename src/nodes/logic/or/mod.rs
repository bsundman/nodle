//! OR node implementation
//!
//! Uses Pattern A: build_interface method
//! - mod.rs: Base node metadata and factory implementation
//! - logic.rs: Core computation logic
//! - parameters.rs: Pattern A interface with build_interface method

pub mod logic;
pub mod parameters;

pub use logic::*;
pub use parameters::*;

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// OR logic node that performs boolean OR/XOR operations
#[derive(Default)]
pub struct OrNodeFactory;

impl NodeFactory for OrNodeFactory {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Logic_Or",
            "OR",
            NodeCategory::new(&["Logic", "Boolean"]),
            "Logical OR/XOR operation with configurable options"
        )
        .with_color(Color32::from_rgb(40, 50, 70))
        .with_icon("|")
        .with_inputs(vec![
            PortDefinition::optional("A", DataType::Boolean)
                .with_description("First boolean input"),
            PortDefinition::optional("B", DataType::Boolean)
                .with_description("Second boolean input"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Result", DataType::Boolean)
                .with_description("A OR B (or XOR/NOR/XNOR based on settings)"),
        ])
        .with_tags(vec!["logic", "boolean", "or", "xor", "nor", "xnor", "gate"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Minimal)
        .with_workspace_compatibility(vec!["Logic", "Programming", "General"])
    }
}