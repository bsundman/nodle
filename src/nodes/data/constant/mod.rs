//! Constant node implementation
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

/// Constant node that outputs a fixed value
#[derive(Default)]
pub struct ConstantNodeFactory;

impl NodeFactory for ConstantNodeFactory {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Data_Constant",
            "Constant",
            NodeCategory::new(&["Data", "Source"]),
            "Outputs a constant value with interface panel controls"
        )
        .with_color(Color32::from_rgb(55, 45, 65))
        .with_icon("C")
        .with_inputs(vec![])
        .with_outputs(vec![
            PortDefinition::required("Value", DataType::Any)
                .with_description("The constant output value"),
        ])
        .with_tags(vec!["data", "constant", "source", "value", "interface"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Minimal)
        .with_workspace_compatibility(vec!["General", "Data", "Math"])
    }
}