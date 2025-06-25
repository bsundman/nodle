//! MaterialX Mix node

use egui::{Color32, Pos2};
use crate::nodes::Node;

pub struct MixNode;

impl MixNode {
    pub fn create(position: Pos2) -> Node {
        let mut node = Node::new(0, "Mix", position)
            .with_color(Color32::from_rgb(70, 50, 90)); // Dark purple for MaterialX
        
        node.add_input("Fg")
            .add_input("Bg")
            .add_input("Mix")
            .add_output("Output");
        
        node
    }
}