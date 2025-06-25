//! MaterialX Noise node

use egui::{Color32, Pos2};
use crate::nodes::Node;

pub struct NoiseNode;

impl NoiseNode {
    pub fn create(position: Pos2) -> Node {
        let mut node = Node::new(0, "Noise", position)
            .with_color(Color32::from_rgb(70, 50, 90)); // Dark purple for MaterialX
        
        node.add_input("Amplitude")
            .add_input("Pivot")
            .add_input("Lacunarity")
            .add_input("Diminish")
            .add_input("Octaves")
            .add_output("Output");
        
        node
    }
}