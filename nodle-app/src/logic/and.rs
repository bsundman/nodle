//! AND logic gate node implementation

use egui::{Color32, Pos2};
use nodle_core::node::Node;
use crate::{NodeFactory, NodeCategory};

/// AND logic gate node that outputs true only when both inputs are true
pub struct AndNode;

impl NodeFactory for AndNode {
    fn node_type() -> &'static str {
        "AND"
    }
    
    fn display_name() -> &'static str {
        "AND"
    }
    
    fn category() -> NodeCategory {
        NodeCategory::Logic
    }
    
    fn color() -> Color32 {
        Color32::from_rgb(160, 160, 170) // Light grey with subtle blue tint
    }
    
    fn create(position: Pos2) -> Node {
        let mut node = Node::new(0, Self::node_type(), position)
            .with_color(Self::color());
        
        node.add_input("A")
            .add_input("B")
            .add_output("Result");
            
        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Pos2;

    #[test]
    fn test_and_node_creation() {
        let node = AndNode::create(Pos2::new(100.0, 100.0));
        assert_eq!(node.title, "AND");
        assert_eq!(node.inputs.len(), 2);
        assert_eq!(node.outputs.len(), 1);
        assert_eq!(node.inputs[0].name, "A");
        assert_eq!(node.inputs[1].name, "B");
        assert_eq!(node.outputs[0].name, "Result");
    }
}