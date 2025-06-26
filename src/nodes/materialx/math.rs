//! MaterialX math operations for shader calculations

use crate::nodes::Node;
use egui::{Color32, Pos2};

/// Create a MaterialX Dot Product node
pub fn create_dot_product_node(position: Pos2) -> Node {
    let mut node = Node::new(0, "Dot Product", position)
        .with_color(Color32::from_rgb(100, 150, 200)); // Blue-ish for math

    node.add_input("Vector A");
    node.add_input("Vector B");
    node.add_output("Result");
    
    node
}

/// Create a MaterialX Normalize node
pub fn create_normalize_node(position: Pos2) -> Node {
    let mut node = Node::new(0, "Normalize", position)
        .with_color(Color32::from_rgb(100, 150, 200)); // Blue-ish for math

    node.add_input("Vector");
    node.add_output("Normalized");
    
    node
}

/// Create a MaterialX Cross Product node
pub fn create_cross_product_node(position: Pos2) -> Node {
    let mut node = Node::new(0, "Cross Product", position)
        .with_color(Color32::from_rgb(100, 150, 200)); // Blue-ish for math

    node.add_input("Vector A");
    node.add_input("Vector B");
    node.add_output("Result");
    
    node
}