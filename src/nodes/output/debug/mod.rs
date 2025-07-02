//! Debug node implementation
//!
//! Uses Pattern A: build_interface method
//! - mod.rs: Base node metadata and factory implementation
//! - logic.rs: Core debug functionality
//! - parameters.rs: Pattern A interface with build_interface method

mod logic;
pub mod parameters;

pub use logic::*;
pub use parameters::*;

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Debug node factory that provides debugging output with configurable parameters
#[derive(Default)]
pub struct DebugNodeFactory;

impl NodeFactory for DebugNodeFactory {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Debug",
            "Debug",
            NodeCategory::output(),
            "Debug node with configurable output format, verbosity, timestamps, and debug categories"
        )
        .with_color(Color32::from_rgb(120, 60, 60))
        .with_icon("🐛")
        .with_inputs(vec![
            PortDefinition::required("Value", DataType::Any)
                .with_description("Value to debug and display"),
            PortDefinition::optional("Label", DataType::String)
                .with_description("Optional custom label for debug output"),
        ])
        .with_outputs(vec![
            PortDefinition::required("PassThrough", DataType::Any)
                .with_description("Input value passed through unchanged for chaining"),
        ])
        .with_tags(vec!["output", "debug", "diagnostics", "logging"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Low)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Pos2;

    #[test]
    fn test_debug_node_metadata() {
        let metadata = DebugNodeFactory::metadata();
        assert_eq!(metadata.node_type, "Debug");
        assert_eq!(metadata.display_name, "Debug");
        assert_eq!(metadata.inputs.len(), 2);
        assert_eq!(metadata.outputs.len(), 1);
        
        // Test input metadata
        assert_eq!(metadata.inputs[0].name, "Value");
        assert_eq!(metadata.inputs[0].data_type, DataType::Any);
        assert!(!metadata.inputs[0].optional);
        
        assert_eq!(metadata.inputs[1].name, "Label");
        assert_eq!(metadata.inputs[1].data_type, DataType::String);
        assert!(metadata.inputs[1].optional);
        
        // Test output metadata
        assert_eq!(metadata.outputs[0].name, "PassThrough");
        assert_eq!(metadata.outputs[0].data_type, DataType::Any);
    }
}