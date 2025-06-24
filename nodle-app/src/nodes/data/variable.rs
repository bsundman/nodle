//! Variable node implementation

use egui::{Color32, Pos2};
use nodle_core::node::Node;
use crate::{NodeFactory, NodeCategory};

/// Variable node that can store and retrieve values
pub struct VariableNode;

impl NodeFactory for VariableNode {
    fn node_type() -> &'static str {
        "Variable"
    }
    
    fn display_name() -> &'static str {
        "Variable"
    }
    
    fn category() -> NodeCategory {
        NodeCategory::Data
    }
    
    fn color() -> Color32 {
        Color32::from_rgb(55, 45, 65) // Dark purple-grey for data nodes
    }
    
    fn create(position: Pos2) -> Node {
        let mut node = Node::new(0, Self::node_type(), position)
            .with_color(Self::color());
        
        node.add_input("Set")
            .add_output("Get");
            
        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Pos2;

    #[test]
    fn test_variable_node_creation() {
        let node = VariableNode::create(Pos2::new(100.0, 100.0));
        assert_eq!(node.title, "Variable");
        assert_eq!(node.inputs.len(), 1);
        assert_eq!(node.outputs.len(), 1);
        assert_eq!(node.inputs[0].name, "Set");
        assert_eq!(node.outputs[0].name, "Get");
    }
}