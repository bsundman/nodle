//! AND node implementation
//!
//! Uses Pattern A: build_interface method
//! - mod.rs: Base node metadata and factory implementation
//! - logic.rs: Core computation logic
//! - parameters.rs: Pattern A interface with build_interface method

pub mod logic;
pub mod parameters;

// Wildcard imports removed - unused

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// AND logic node that performs boolean AND operation
#[derive(Default)]
pub struct AndNodeFactory;

impl NodeFactory for AndNodeFactory {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Logic_And",
            "AND",
            NodeCategory::new(&["Logic", "Boolean"]),
            "Logical AND operation with configurable options"
        )
        .with_color(Color32::from_rgb(40, 50, 70))
        .with_icon("&")
        .with_inputs(vec![
            PortDefinition::optional("A", DataType::Boolean)
                .with_description("First boolean input"),
            PortDefinition::optional("B", DataType::Boolean)
                .with_description("Second boolean input"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Result", DataType::Boolean)
                .with_description("A AND B (or NAND if inverted)"),
        ])
        .with_tags(vec!["logic", "boolean", "and", "gate", "nand"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Minimal)
        .with_workspace_compatibility(vec!["Logic", "Programming", "General"])
    }
}