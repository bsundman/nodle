use nodle::nodes::{NodeGraph, Connection};
use nodle::nodes::data::usd_file_reader::UsdFileReaderNodeFactory;
use nodle::nodes::three_d::ui::viewport::ViewportNodeFactory;
use nodle::nodes::NodeFactory;
use egui::Pos2;

fn main() {
    println!("Testing connection system...");
    
    // Create a graph
    let mut graph = NodeGraph::new();
    
    // Create USD File Reader node
    let usd_reader = UsdFileReaderNodeFactory::create(Pos2::new(100.0, 100.0));
    let usd_reader_id = graph.add_node(usd_reader);
    println!("Created USD File Reader with ID: {}", usd_reader_id);
    
    // Create Viewport node
    let viewport = ViewportNodeFactory::create(Pos2::new(400.0, 100.0));
    let viewport_id = graph.add_node(viewport);
    println!("Created Viewport with ID: {}", viewport_id);
    
    // Create connection from USD File Reader output (port 0) to Viewport input (port 0)
    let connection = Connection::new(usd_reader_id, 0, viewport_id, 0);
    match graph.add_connection(connection) {
        Ok(_) => println!("✅ Connection created successfully!"),
        Err(e) => println!("❌ Failed to create connection: {}", e),
    }
    
    // Verify the connection exists
    println!("Total connections in graph: {}", graph.connections.len());
    for (i, conn) in graph.connections.iter().enumerate() {
        println!("Connection {}: Node {} port {} -> Node {} port {}", 
                 i, conn.from_node, conn.from_port, conn.to_node, conn.to_port);
    }
}