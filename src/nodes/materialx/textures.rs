//! MaterialX texture and image nodes

use crate::nodes::Node;
use egui::{Color32, Pos2};

/// Create a MaterialX Image node
pub fn create_image_node(position: Pos2) -> Node {
    let mut node = Node::new(0, "Image", position)
        .with_color(Color32::from_rgb(140, 180, 140)); // Green-ish for textures

    node.add_input("File");
    node.add_input("UV");
    node.add_output("Color");
    node.add_output("Alpha");
    
    node
}

/// Create a MaterialX Noise node
pub fn create_noise_node(position: Pos2) -> Node {
    let mut node = Node::new(0, "Noise", position)
        .with_color(Color32::from_rgb(140, 180, 140)); // Green-ish for textures

    node.add_input("UV");
    node.add_input("Scale");
    node.add_input("Octaves");
    node.add_output("Color");
    
    node
}

/// Create a MaterialX Checkerboard node
pub fn create_checkerboard_node(position: Pos2) -> Node {
    let mut node = Node::new(0, "Checkerboard", position)
        .with_color(Color32::from_rgb(140, 180, 140)); // Green-ish for textures

    node.add_input("UV");
    node.add_input("Color1");
    node.add_input("Color2");
    node.add_output("Color");
    
    node
}