//! Node creation utilities for the application

use egui::{Color32, Pos2};
use nodle_core::{graph::NodeGraph, node::Node};

/// Creates test nodes for demonstration
pub fn create_test_nodes(graph: &mut NodeGraph) {
    // Math nodes - green colors
    let mut add_node = Node::new(0, "Add", Pos2::new(100.0, 100.0))
        .with_color(Color32::from_rgb(80, 120, 80));
    add_node.add_input("A").add_input("B").add_output("Result");
    graph.add_node(add_node);

    let mut sub_node = Node::new(0, "Subtract", Pos2::new(100.0, 200.0))
        .with_color(Color32::from_rgb(80, 120, 80));
    sub_node.add_input("A").add_input("B").add_output("Result");
    graph.add_node(sub_node);

    let mut mul_node = Node::new(0, "Multiply", Pos2::new(300.0, 100.0))
        .with_color(Color32::from_rgb(80, 120, 80));
    mul_node.add_input("A").add_input("B").add_output("Result");
    graph.add_node(mul_node);

    let mut div_node = Node::new(0, "Divide", Pos2::new(300.0, 200.0))
        .with_color(Color32::from_rgb(80, 120, 80));
    div_node.add_input("A").add_input("B").add_output("Result");
    graph.add_node(div_node);

    // Logic nodes - blue colors
    let mut and_node = Node::new(0, "AND", Pos2::new(500.0, 100.0))
        .with_color(Color32::from_rgb(80, 80, 120));
    and_node.add_input("A").add_input("B").add_output("Result");
    graph.add_node(and_node);

    let mut or_node = Node::new(0, "OR", Pos2::new(500.0, 200.0))
        .with_color(Color32::from_rgb(80, 80, 120));
    or_node.add_input("A").add_input("B").add_output("Result");
    graph.add_node(or_node);

    let mut not_node = Node::new(0, "NOT", Pos2::new(700.0, 150.0))
        .with_color(Color32::from_rgb(80, 80, 120));
    not_node.add_input("Input").add_output("Result");
    graph.add_node(not_node);

    // Data nodes - purple colors
    let mut const_node = Node::new(0, "Constant", Pos2::new(100.0, 350.0))
        .with_color(Color32::from_rgb(120, 80, 120));
    const_node.add_output("Value");
    graph.add_node(const_node);

    let mut var_node = Node::new(0, "Variable", Pos2::new(300.0, 350.0))
        .with_color(Color32::from_rgb(120, 80, 120));
    var_node.add_input("Set").add_output("Get");
    graph.add_node(var_node);

    // Output nodes - red colors
    let mut print_node = Node::new(0, "Print", Pos2::new(500.0, 350.0))
        .with_color(Color32::from_rgb(120, 80, 80));
    print_node.add_input("Value");
    graph.add_node(print_node);

    let mut debug_node = Node::new(0, "Debug", Pos2::new(700.0, 350.0))
        .with_color(Color32::from_rgb(120, 80, 80));
    debug_node.add_input("Value").add_output("Pass");
    graph.add_node(debug_node);
}