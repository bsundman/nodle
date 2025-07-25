//! Multiply node implementation
//!
//! Uses Pattern A: build_interface method
//! - mod.rs: Base node metadata and factory implementation
//! - logic.rs: Core computation logic
//! - parameters.rs: Pattern A interface with build_interface method

pub mod logic;
pub mod parameters;

// Re-exports removed - these were unused

use crate::nodes::NodeFactory;

/// Factory for creating multiply nodes
#[derive(Default)]
pub struct MultiplyNodeFactory;

impl NodeFactory for MultiplyNodeFactory {
    fn metadata() -> crate::nodes::NodeMetadata {
        crate::nodes::NodeMetadata::new(
            "Math_Multiply",
            "Multiply",
            crate::nodes::NodeCategory::new(&["Math", "Arithmetic"]),
            "Multiplies inputs with interface panel controls"
        )
        .with_color(egui::Color32::from_rgb(100, 100, 200))
        .with_icon("×")
        .with_inputs(vec![
            crate::nodes::PortDefinition::required("A", crate::nodes::DataType::Float)
                .with_description("First multiplicand"),
            crate::nodes::PortDefinition::required("B", crate::nodes::DataType::Float)
                .with_description("Second multiplicand"),
        ])
        .with_outputs(vec![
            crate::nodes::PortDefinition::required("Result", crate::nodes::DataType::Float)
                .with_description("Product (A × B)"),
        ])
        .with_tags(vec!["math", "arithmetic", "multiply", "product", "interface"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Low)
        .with_workspace_compatibility(vec!["Math", "General"])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Pos2;

    #[test]
    fn test_multiply_node_metadata() {
        let metadata = MultiplyNodeFactory::metadata();
        assert_eq!(metadata.node_type, "Math_Multiply");
        assert_eq!(metadata.display_name, "Multiply");
        assert_eq!(metadata.inputs.len(), 2);
        assert_eq!(metadata.outputs.len(), 1);
    }

    #[test]
    fn test_multiply_node_creation() {
        let node = MultiplyNodeFactory::create(Pos2::new(100.0, 100.0));
        assert_eq!(node.title, "Multiply");
        assert_eq!(node.inputs.len(), 2);
        assert_eq!(node.outputs.len(), 1);
        assert_eq!(node.inputs[0].name, "A");
        assert_eq!(node.inputs[1].name, "B");
        assert_eq!(node.outputs[0].name, "Result");
    }
}