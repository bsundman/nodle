//! Node graph data structures and operations

use super::node::{Node, NodeId};
use super::port::PortId;
use std::collections::HashMap;

/// Represents a connection between two ports on different nodes
#[derive(Debug, Clone, PartialEq)]
pub struct Connection {
    pub from_node: NodeId,
    pub from_port: PortId,
    pub to_node: NodeId,
    pub to_port: PortId,
}

impl Connection {
    /// Creates a new connection
    pub fn new(from_node: NodeId, from_port: PortId, to_node: NodeId, to_port: PortId) -> Self {
        Self {
            from_node,
            from_port,
            to_node,
            to_port,
        }
    }
}

/// A graph containing nodes and their connections
#[derive(Debug, Clone)]
pub struct NodeGraph {
    pub nodes: HashMap<NodeId, Node>,
    pub connections: Vec<Connection>,
    next_node_id: NodeId,
}

impl NodeGraph {
    /// Creates a new empty node graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
            next_node_id: 0,
        }
    }

    /// Adds a node to the graph and returns its ID
    pub fn add_node(&mut self, mut node: Node) -> NodeId {
        let id = self.next_node_id;
        node.id = id;
        self.nodes.insert(id, node);
        self.next_node_id += 1;
        id
    }

    /// Removes a node and all its connections
    pub fn remove_node(&mut self, node_id: NodeId) -> Option<Node> {
        // Remove all connections to/from this node
        self.connections
            .retain(|conn| conn.from_node != node_id && conn.to_node != node_id);
        
        // Remove the node
        self.nodes.remove(&node_id)
    }

    /// Adds a connection between two ports
    pub fn add_connection(&mut self, connection: Connection) -> Result<(), &'static str> {
        // Validate the connection
        if connection.from_node == connection.to_node {
            return Err("Cannot connect a node to itself");
        }

        // Check if nodes exist
        if !self.nodes.contains_key(&connection.from_node) {
            return Err("Source node does not exist");
        }
        if !self.nodes.contains_key(&connection.to_node) {
            return Err("Target node does not exist");
        }

        // TODO: Validate port indices and types

        self.connections.push(connection);
        Ok(())
    }

    /// Removes a connection by index
    pub fn remove_connection(&mut self, index: usize) -> Option<Connection> {
        if index < self.connections.len() {
            Some(self.connections.remove(index))
        } else {
            None
        }
    }

    /// Updates port positions for all nodes
    pub fn update_all_port_positions(&mut self) {
        for node in self.nodes.values_mut() {
            node.update_port_positions();
        }
    }
}

impl Default for NodeGraph {
    fn default() -> Self {
        Self::new()
    }
}