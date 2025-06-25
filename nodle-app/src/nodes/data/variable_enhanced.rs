//! Enhanced variable data node implementation using new factory system

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Variable node that can store and output a variable value
#[derive(Default)]
pub struct VariableNodeEnhanced;

impl NodeFactory for VariableNodeEnhanced {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "Variable",
            display_name: "Variable",
            category: NodeCategory::data(),
            description: "Stores and outputs a variable value",
            color: Color32::from_rgb(65, 45, 65), // Purple tint
            inputs: vec![
                PortDefinition::optional("Set", DataType::Float)
                    .with_description("Optional input to set variable value"),
            ],
            outputs: vec![
                PortDefinition::required("Value", DataType::Float)
                    .with_description("Current variable value"),
            ],
        }
    }
}

// Also implement the old trait for backward compatibility during transition
impl crate::NodeFactory for VariableNodeEnhanced {
    fn node_type() -> &'static str {
        "Variable"
    }
    
    fn display_name() -> &'static str {
        "Variable"
    }
    
    fn category() -> crate::NodeCategory {
        crate::NodeCategory::Data
    }
    
    fn color() -> Color32 {
        Color32::from_rgb(65, 45, 65)
    }
    
    fn create(position: Pos2) -> Node {
        <Self as NodeFactory>::create(position)
    }
}