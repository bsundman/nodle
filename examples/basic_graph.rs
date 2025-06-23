use nodle_core::{NodeGraph, Node, Connection};
use egui::Pos2;

fn main() {
    let mut graph = NodeGraph::new();

    // Create nodes
    let mut node1 = Node::new(0, "Math", Pos2::new(100.0, 100.0));
    node1.add_input("A").add_input("B").add_output("Result");

    let mut node2 = Node::new(0, "Output", Pos2::new(300.0, 100.0));
    node2.add_input("Value");

    // Add to graph
    let id1 = graph.add_node(node1);
    let id2 = graph.add_node(node2);

    // Connect them
    let connection = Connection::new(id1, 0, id2, 0);
    graph.add_connection(connection).unwrap();

    println!("Created graph with {} nodes and {} connections", 
             graph.nodes.len(), 
             graph.connections.len());

    // Update positions
    graph.update_all_port_positions();
    
    println!("Graph setup complete!");
}