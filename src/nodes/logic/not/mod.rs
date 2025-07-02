//! NOT node implementation
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

/// NOT logic node that performs boolean inversion
#[derive(Default)]
pub struct NotNodeFactory;

impl NodeFactory for NotNodeFactory {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Logic_Not",
            "NOT",
            NodeCategory::new(&["Logic", "Boolean"]),
            "Logical NOT operation with buffer and tri-state options"
        )
        .with_color(Color32::from_rgb(40, 50, 70))
        .with_icon("!")
        .with_inputs(vec![
            PortDefinition::optional("Input", DataType::Boolean)
                .with_description("Boolean input to invert"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Result", DataType::Boolean)
                .with_description("NOT Input (or buffered if double invert enabled)"),
        ])
        .with_tags(vec!["logic", "boolean", "not", "invert", "buffer", "gate"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Minimal)
        .with_workspace_compatibility(vec!["Logic", "Programming", "General"])
    }
}