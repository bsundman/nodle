//! Constant value node implementation

use egui::{Color32, Pos2};
use crate::nodes::Node;
use crate::{NodeFactory, NodeCategory};

/// Constant node that outputs a fixed value
pub struct ConstantNode;

impl NodeFactory for ConstantNode {
    fn node_type() -> &'static str {
        "Constant"
    }
    
    fn display_name() -> &'static str {
        "Constant"
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
        
        node.add_output("Value");
            
        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Pos2;

    #[test]
    fn test_constant_node_creation() {
        let node = ConstantNode::create(Pos2::new(100.0, 100.0));
        assert_eq!(node.title, "Constant");
        assert_eq!(node.inputs.len(), 0);
        assert_eq!(node.outputs.len(), 1);
        assert_eq!(node.outputs[0].name, "Value");
    }
}