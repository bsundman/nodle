//! Nodle Core - A flexible node graph library for visual programming
//!
//! This library provides the core functionality for creating node-based
//! visual programming interfaces with customizable node types and contexts.

pub mod graph;
pub mod math;
pub mod node;
pub mod port;

pub use graph::{Connection, NodeGraph};
pub use node::{Node, NodeId};
pub use port::{Port, PortId, PortType};

// Re-export commonly used egui types
pub use egui::{Color32, Pos2, Vec2};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_graph_operations() {
        let mut graph = NodeGraph::new();
        
        // Create a simple node
        let mut node = Node::new(0, "Test Node", Pos2::new(100.0, 100.0));
        node.add_input("Input").add_output("Output");
        
        let node_id = graph.add_node(node);
        assert_eq!(node_id, 0);
        assert!(graph.nodes.contains_key(&node_id));
        
        // Test node removal
        let removed = graph.remove_node(node_id);
        assert!(removed.is_some());
        assert!(!graph.nodes.contains_key(&node_id));
    }
    
    #[test]
    fn test_connection_creation() {
        let mut graph = NodeGraph::new();
        
        let mut node1 = Node::new(0, "Node 1", Pos2::ZERO);
        node1.add_output("Out");
        let id1 = graph.add_node(node1);
        
        let mut node2 = Node::new(0, "Node 2", Pos2::new(200.0, 0.0));
        node2.add_input("In");
        let id2 = graph.add_node(node2);
        
        let connection = Connection::new(id1, 0, id2, 0);
        let result = graph.add_connection(connection);
        assert!(result.is_ok());
        assert_eq!(graph.connections.len(), 1);
    }
}