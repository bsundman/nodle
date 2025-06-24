//! Debug node implementation

use egui::{Color32, Pos2};
use nodle_core::node::Node;
use crate::{NodeFactory, NodeCategory};

/// Debug node that outputs values for debugging and passes them through
pub struct DebugNode;

impl NodeFactory for DebugNode {
    fn node_type() -> &'static str {
        "Debug"
    }
    
    fn display_name() -> &'static str {
        "Debug"
    }
    
    fn category() -> NodeCategory {
        NodeCategory::Output
    }
    
    fn color() -> Color32 {
        Color32::from_rgb(65, 45, 45) // Dark red-grey for output nodes
    }
    
    fn create(position: Pos2) -> Node {
        let mut node = Node::new(0, Self::node_type(), position)
            .with_color(Self::color());
        
        node.add_input("Value")
            .add_output("Pass");
            
        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Pos2;

    #[test]
    fn test_debug_node_creation() {
        let node = DebugNode::create(Pos2::new(100.0, 100.0));
        assert_eq!(node.title, "Debug");
        assert_eq!(node.inputs.len(), 1);
        assert_eq!(node.outputs.len(), 1);
        assert_eq!(node.inputs[0].name, "Value");
        assert_eq!(node.outputs[0].name, "Pass");
    }
}