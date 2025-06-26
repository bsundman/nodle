//! Division node implementation

use egui::{Color32, Pos2};
use crate::nodes::Node;
use crate::{NodeFactory, NodeCategory};

/// Division node that takes two inputs and produces their quotient
pub struct DivideNode;

impl NodeFactory for DivideNode {
    fn node_type() -> &'static str {
        "Divide"
    }
    
    fn display_name() -> &'static str {
        "Divide"
    }
    
    fn category() -> NodeCategory {
        NodeCategory::Math
    }
    
    fn color() -> Color32 {
        Color32::from_rgb(45, 55, 65) // Dark blue-grey to match terminal
    }
    
    fn create(position: Pos2) -> Node {
        let mut node = Node::new(0, Self::node_type(), position)
            .with_color(Self::color());
        
        node.add_input("A")
            .add_input("B")
            .add_output("Result");
        
        node.update_port_positions();
        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Pos2;

    #[test]
    fn test_divide_node_creation() {
        let node = DivideNode::create(Pos2::new(100.0, 100.0));
        assert_eq!(node.title, "Divide");
        assert_eq!(node.inputs.len(), 2);
        assert_eq!(node.outputs.len(), 1);
        assert_eq!(node.inputs[0].name, "A");
        assert_eq!(node.inputs[1].name, "B");
        assert_eq!(node.outputs[0].name, "Result");
    }
}