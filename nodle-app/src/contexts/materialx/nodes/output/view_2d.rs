//! MaterialX 2D View output node

use egui::{Color32, Pos2};
use crate::nodes::Node;

pub struct View2DNode;

impl View2DNode {
    pub fn create(position: Pos2) -> Node {
        let mut node = Node::new(0, "2D View", position)
            .with_color(Color32::from_rgb(80, 45, 90)); // Slightly darker purple for MaterialX outputs
        
        node.add_input("Texture");
        
        node
    }
}