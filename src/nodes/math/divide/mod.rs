//! Division node implementation
//!
//! Uses Pattern A: build_interface method
//! - mod.rs: Base node metadata and factory implementation
//! - logic.rs: Core computation logic
//! - parameters.rs: Pattern A interface with build_interface method

pub mod functions;
mod logic;
pub mod parameters;

// DivideNode is exported for legacy compatibility if needed
pub use parameters::DivideNode;

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Division node that takes two numeric inputs and produces their quotient
#[derive(Default)]
pub struct DivideNodeFactory;

impl NodeFactory for DivideNodeFactory {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Divide",
            "Divide",
            NodeCategory::math(),
            "Divides the first input by the second input"
        )
        .with_color(Color32::from_rgb(45, 55, 65))
        .with_icon("÷")
        .with_inputs(vec![
            PortDefinition::required("A", DataType::Float)
                .with_description("Dividend (number to be divided)"),
            PortDefinition::required("B", DataType::Float)
                .with_description("Divisor (number to divide by)"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Result", DataType::Float)
                .with_description("Quotient (A ÷ B)"),
        ])
        .with_tags(vec!["math", "arithmetic", "basic"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Minimal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Pos2;

    #[test]
    fn test_divide_node_metadata() {
        let metadata = DivideNodeFactory::metadata();
        assert_eq!(metadata.node_type, "Divide");
        assert_eq!(metadata.display_name, "Divide");
        assert_eq!(metadata.description, "Divides the first input by the second input");
        assert_eq!(metadata.inputs.len(), 2);
        assert_eq!(metadata.outputs.len(), 1);
        
        // Test input metadata
        assert_eq!(metadata.inputs[0].name, "A");
        assert_eq!(metadata.inputs[0].data_type, DataType::Float);
        assert!(!metadata.inputs[0].optional);
        
        assert_eq!(metadata.inputs[1].name, "B");
        assert_eq!(metadata.inputs[1].data_type, DataType::Float);
        assert!(!metadata.inputs[1].optional);
        
        // Test output metadata
        assert_eq!(metadata.outputs[0].name, "Result");
        assert_eq!(metadata.outputs[0].data_type, DataType::Float);
    }

    #[test]
    fn test_divide_node_creation() {
        let node = DivideNodeFactory::create(Pos2::new(100.0, 100.0));
        assert_eq!(node.title, "Divide");
        assert_eq!(node.inputs.len(), 2);
        assert_eq!(node.outputs.len(), 1);
        assert_eq!(node.inputs[0].name, "A");
        assert_eq!(node.inputs[1].name, "B");
        assert_eq!(node.outputs[0].name, "Result");
    }
}