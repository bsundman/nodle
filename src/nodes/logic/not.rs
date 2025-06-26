//! NOT logic gate node implementation

use egui::{Color32, Pos2};
use crate::nodes::Node;
use crate::{NodeFactory, NodeCategory};

/// NOT logic gate node that inverts the input boolean value
pub struct NotNode;

impl NodeFactory for NotNode {
    fn node_type() -> &'static str {
        "NOT"
    }
    
    fn display_name() -> &'static str {
        "NOT"
    }
    
    fn category() -> NodeCategory {
        NodeCategory::Logic
    }
    
    fn color() -> Color32 {
        Color32::from_rgb(40, 50, 70) // Dark blue-grey for logic nodes
    }
    
    fn create(position: Pos2) -> Node {
        let mut node = Node::new(0, Self::node_type(), position)
            .with_color(Self::color());
        
        node.add_input("Input")
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
    fn test_not_node_creation() {
        let node = NotNode::create(Pos2::new(100.0, 100.0));
        assert_eq!(node.title, "NOT");
        assert_eq!(node.inputs.len(), 1);
        assert_eq!(node.outputs.len(), 1);
        assert_eq!(node.inputs[0].name, "Input");
        assert_eq!(node.outputs[0].name, "Result");
    }
}