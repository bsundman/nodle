//! MaterialX utility nodes for mixing, switching, and converting

use crate::nodes::Node;
use egui::{Color32, Pos2};

/// Create a MaterialX Mix node
pub fn create_mix_node(position: Pos2) -> Node {
    let mut node = Node::new(0, "Mix", position)
        .with_color(Color32::from_rgb(200, 150, 100)); // Orange-ish for utilities

    node.add_input("Input A");
    node.add_input("Input B");
    node.add_input("Mix Factor");
    node.add_output("Output");
    
    node
}

/// Create a MaterialX Switch node
pub fn create_switch_node(position: Pos2) -> Node {
    let mut node = Node::new(0, "Switch", position)
        .with_color(Color32::from_rgb(200, 150, 100)); // Orange-ish for utilities

    node.add_input("Input A");
    node.add_input("Input B");
    node.add_input("Selector");
    node.add_output("Output");
    
    node
}

/// Create a MaterialX Constant node
pub fn create_constant_node(position: Pos2) -> Node {
    let mut node = Node::new(0, "Constant", position)
        .with_color(Color32::from_rgb(200, 150, 100)); // Orange-ish for utilities

    node.add_input("Value");
    node.add_output("Output");
    
    node
}

/// Create a demonstration MaterialX Shader workspace node with port mapping
pub fn create_shader_workspace_node(position: Pos2) -> Node {
    use crate::nodes::{Node, NodeType, NodeGraph, Connection};
    
    let mut workspace_node = Node::new_workspace(0, "MaterialX Shader", position)
        .with_color(Color32::from_rgb(120, 80, 140)); // Purple for shader workspace
    
    // Create internal graph with some demo nodes
    let mut internal_graph = NodeGraph::new();
    
    // Add a constant node for base color input (internal node id 1)
    let mut base_color_node = create_constant_node(Pos2::new(100.0, 100.0));
    base_color_node.id = 1;
    internal_graph.add_node(base_color_node);
    
    // Add a standard surface node as the output (internal node id 2)
    let mut surface_node = crate::nodes::materialx::shading::create_standard_surface_node(Pos2::new(300.0, 100.0));
    surface_node.id = 2;
    internal_graph.add_node(surface_node);
    
    // Connect base color to surface
    let _ = internal_graph.add_connection(Connection {
        from_node: 1,
        from_port: 0,
        to_node: 2,
        to_port: 0,
    });
    
    // Set up the workspace node with the internal graph
    workspace_node.node_type = NodeType::Workspace {
        graph: internal_graph,
        workspace_type: "MaterialX Shader".to_string(),
        port_mappings: Vec::new(),
    };
    
    // Add external ports with mappings to internal nodes
    let _ = workspace_node.add_external_input("Base Color", 1, "Value");
    let _ = workspace_node.add_external_output("Surface", 2, "Surface");
    
    workspace_node
}