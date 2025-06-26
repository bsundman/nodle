//! Addition node implementation

use egui::{Color32, Pos2};
use crate::nodes::Node;
use crate::{NodeFactory, NodeCategory};

/// Addition node that takes two inputs and produces their sum
pub struct AddNode;

impl NodeFactory for AddNode {
    fn node_type() -> &'static str {
        "Add"
    }
    
    fn display_name() -> &'static str {
        "Add"
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
    fn test_add_node_creation() {
        let node = AddNode::create(Pos2::new(100.0, 100.0));
        assert_eq!(node.title, "Add");
        assert_eq!(node.inputs.len(), 2);
        assert_eq!(node.outputs.len(), 1);
        assert_eq!(node.inputs[0].name, "A");
        assert_eq!(node.inputs[1].name, "B");
        assert_eq!(node.outputs[0].name, "Result");
    }
}