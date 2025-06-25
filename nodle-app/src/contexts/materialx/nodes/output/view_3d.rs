//! MaterialX 3D View output node

use egui::{Color32, Pos2};
use crate::nodes::Node;

pub struct View3DNode;

impl View3DNode {
    pub fn create(position: Pos2) -> Node {
        let mut node = Node::new(0, "3D View", position)
            .with_color(Color32::from_rgb(80, 45, 90)); // Slightly darker purple for MaterialX outputs
        
        node.add_input("Material");
        
        node
    }
}