//! MaterialX shading nodes for surface and BSDF operations

use crate::nodes::Node;
use egui::{Color32, Pos2};

/// Create a MaterialX Standard Surface node
pub fn create_standard_surface_node(position: Pos2) -> Node {
    let mut node = Node::new(0, "Standard Surface", position)
        .with_color(Color32::from_rgb(180, 140, 100)); // Brown-ish for materials

    // Standard Surface inputs
    node.add_input("Base Color");
    node.add_input("Metallic");
    node.add_input("Roughness");
    node.add_input("Normal");
    node.add_input("Emission");
    node.add_input("Opacity");
    
    // Output
    node.add_output("Surface");
    
    node
}

/// Create a MaterialX Surface Shader node
pub fn create_surface_shader_node(position: Pos2) -> Node {
    let mut node = Node::new(0, "Surface Shader", position)
        .with_color(Color32::from_rgb(160, 120, 180)); // Purple-ish for shaders

    node.add_input("Surface");
    node.add_input("Opacity");
    node.add_output("Shader");
    
    node
}