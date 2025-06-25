//! MaterialX Texture node

use egui::{Color32, Pos2};
use crate::nodes::Node;

pub struct TextureNode;

impl TextureNode {
    pub fn create(position: Pos2) -> Node {
        let mut node = Node::new(0, "Texture", position)
            .with_color(Color32::from_rgb(70, 50, 90)); // Dark purple for MaterialX
        
        node.add_input("File")
            .add_input("UV")
            .add_input("Filtertype")
            .add_output("Output");
        
        node
    }
}