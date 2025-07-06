//! Subtraction node implementation
//!
//! Uses Pattern A: build_interface method
//! - mod.rs: Base node metadata and factory implementation
//! - functions.rs: Core computation logic
//! - parameters.rs: Pattern A interface with build_interface method

mod functions;
pub mod parameters;

// Re-exports removed - these were unused

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Subtraction node that takes two numeric inputs and produces their difference
#[derive(Default)]
pub struct SubtractNodeFactory;

impl NodeFactory for SubtractNodeFactory {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Subtract",
            "Subtract",
            NodeCategory::math(),
            "Subtracts the second input from the first"
        )
        .with_color(Color32::from_rgb(45, 55, 65))
        .with_icon("➖")
        .with_inputs(vec![
            PortDefinition::required("A", DataType::Float)
                .with_description("Minuend (value to subtract from)"),
            PortDefinition::required("B", DataType::Float)
                .with_description("Subtrahend (value to subtract)"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Result", DataType::Float)
                .with_description("Difference (A - B)"),
        ])
        .with_tags(vec!["math", "arithmetic", "subtract"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Minimal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Pos2;

    #[test]
    fn test_subtract_node_metadata() {
        let metadata = SubtractNodeFactory::metadata();
        assert_eq!(metadata.node_type, "Subtract");
        assert_eq!(metadata.display_name, "Subtract");
        assert_eq!(metadata.description, "Subtracts the second input from the first");
        assert_eq!(metadata.inputs.len(), 2);
        assert_eq!(metadata.outputs.len(), 1);
    }

    #[test]
    fn test_subtract_node_creation() {
        let node = SubtractNodeFactory::create(Pos2::new(100.0, 100.0));
        assert_eq!(node.title, "Subtract");
        assert_eq!(node.inputs.len(), 2);
        assert_eq!(node.outputs.len(), 1);
        assert_eq!(node.inputs[0].name, "A");
        assert_eq!(node.inputs[1].name, "B");
        assert_eq!(node.outputs[0].name, "Result");
    }
}