//! Print node implementation
//!
//! Uses Pattern A: build_interface method
//! - mod.rs: Base node metadata and factory implementation
//! - logic.rs: Core printing functionality
//! - parameters.rs: Pattern A interface with build_interface method

mod logic;
pub mod parameters;

pub use logic::*;
pub use parameters::*;

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Print node factory that provides formatted output with configurable parameters
#[derive(Default)]
pub struct PrintNodeFactory;

impl NodeFactory for PrintNodeFactory {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Print",
            "Print",
            NodeCategory::output(),
            "Print node with configurable output destination, formatting, line endings, and timestamps"
        )
        .with_color(Color32::from_rgb(80, 100, 60))
        .with_icon("📄")
        .with_inputs(vec![
            PortDefinition::required("Value", DataType::Any)
                .with_description("Value to print and output"),
            PortDefinition::optional("Label", DataType::String)
                .with_description("Optional custom label for print output"),
        ])
        .with_outputs(vec![])
        .with_tags(vec!["output", "print", "logging", "console"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Minimal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Pos2;

    #[test]
    fn test_print_node_metadata() {
        let metadata = PrintNodeFactory::metadata();
        assert_eq!(metadata.node_type, "Print");
        assert_eq!(metadata.display_name, "Print");
        assert_eq!(metadata.inputs.len(), 2);
        assert_eq!(metadata.outputs.len(), 0);
        
        // Test input metadata
        assert_eq!(metadata.inputs[0].name, "Value");
        assert_eq!(metadata.inputs[0].data_type, DataType::Any);
        assert!(!metadata.inputs[0].optional);
        
        assert_eq!(metadata.inputs[1].name, "Label");
        assert_eq!(metadata.inputs[1].data_type, DataType::String);
        assert!(metadata.inputs[1].optional);
    }
}