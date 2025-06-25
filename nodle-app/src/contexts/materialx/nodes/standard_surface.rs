//! MaterialX Standard Surface node

use egui::{Color32, Pos2};
use crate::nodes::Node;

pub struct StandardSurfaceNode;

impl StandardSurfaceNode {
    pub fn create(position: Pos2) -> Node {
        let mut node = Node::new(0, "Standard Surface", position)
            .with_color(Color32::from_rgb(70, 50, 90)); // Dark purple for MaterialX
        
        node.add_input("Base Color")
            .add_input("Metallic")
            .add_input("Roughness")
            .add_input("Normal")
            .add_input("Emission")
            .add_output("Output");
        
        node
    }
}