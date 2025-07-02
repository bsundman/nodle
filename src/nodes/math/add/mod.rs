//! Addition node implementation
//!
//! Uses Pattern A: build_interface method
//! - mod.rs: Base node metadata and factory implementation
//! - functions.rs: Core computation logic
//! - parameters.rs: Pattern A interface with build_interface method

mod functions;
pub mod parameters;

pub use functions::*;
pub use parameters::*;

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Addition node that takes two numeric inputs and produces their sum
#[derive(Default)]
pub struct AddNodeFactory;

impl NodeFactory for AddNodeFactory {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Add",
            "Add",
            NodeCategory::math(),
            "Adds two numeric values together"
        )
        .with_color(Color32::from_rgb(45, 55, 65))
        .with_icon("➕")
        .with_inputs(vec![
            PortDefinition::required("A", DataType::Float)
                .with_description("First input value"),
            PortDefinition::required("B", DataType::Float)
                .with_description("Second input value"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Result", DataType::Float)
                .with_description("Sum of A and B"),
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
    fn test_add_node_metadata() {
        let metadata = AddNodeFactory::metadata();
        assert_eq!(metadata.node_type, "Add");
        assert_eq!(metadata.display_name, "Add");
        assert_eq!(metadata.description, "Adds two numeric values together");
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
    fn test_add_node_creation() {
        let node = AddNodeFactory::create(Pos2::new(100.0, 100.0));
        assert_eq!(node.title, "Add");
        assert_eq!(node.inputs.len(), 2);
        assert_eq!(node.outputs.len(), 1);
        assert_eq!(node.inputs[0].name, "A");
        assert_eq!(node.inputs[1].name, "B");
        assert_eq!(node.outputs[0].name, "Result");
    }
}