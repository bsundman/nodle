//! Print node implementation

use egui::{Color32, Pos2};
use nodle_core::node::Node;
use crate::{NodeFactory, NodeCategory};

/// Print node that outputs values to console
pub struct PrintNode;

impl NodeFactory for PrintNode {
    fn node_type() -> &'static str {
        "Print"
    }
    
    fn display_name() -> &'static str {
        "Print"
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
        
        node.add_input("Value");
            
        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Pos2;

    #[test]
    fn test_print_node_creation() {
        let node = PrintNode::create(Pos2::new(100.0, 100.0));
        assert_eq!(node.title, "Print");
        assert_eq!(node.inputs.len(), 1);
        assert_eq!(node.outputs.len(), 0);
        assert_eq!(node.inputs[0].name, "Value");
    }
}