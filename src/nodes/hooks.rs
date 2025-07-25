//! Node execution hooks for cache clearing and preparation
//!
//! This module provides a trait-based system for nodes to handle their own
//! cache clearing and resource management during the execution lifecycle.

use crate::nodes::{Node, NodeGraph, NodeId};
use crate::nodes::interface::NodeData;

/// Trait for node-specific execution lifecycle hooks
pub trait NodeExecutionHooks: Send + Sync {
    /// Called before node execution - handle cache clearing and preparation
    fn before_execution(&mut self, node: &Node, graph: &NodeGraph) -> Result<(), String> {
        // Default: no special handling
        Ok(())
    }
    
    /// Called after successful node execution - handle caching and cleanup
    fn after_execution(&mut self, node: &Node, outputs: &[NodeData], graph: &NodeGraph) -> Result<(), String> {
        // Default: no special handling
        Ok(())
    }
    
    /// Called when node is removed from graph - handle cleanup
    fn on_node_removed(&mut self, node_id: NodeId) -> Result<(), String> {
        // Default: no special handling
        Ok(())
    }
    
    /// Called when a connection is added TO this node (this node receives new input)
    fn on_input_connection_added(&mut self, node: &Node, _graph: &NodeGraph) -> Result<(), String> {
        // Default: no special handling
        Ok(())
    }
    
    /// Called when a connection is removed FROM this node (this node loses an input)
    fn on_input_connection_removed(&mut self, node: &Node, _graph: &NodeGraph) -> Result<(), String> {
        // Default: no special handling
        Ok(())
    }
    
    /// Override node execution with custom logic - return None to use default dispatch
    /// This allows nodes to implement custom execution while keeping execution engine generic
    fn custom_execution(
        &mut self, 
        node_id: NodeId,
        node: &Node, 
        inputs: Vec<NodeData>, 
        engine: &mut crate::nodes::NodeGraphEngine
    ) -> Option<Result<Vec<NodeData>, String>> {
        // Default: no custom execution, use standard dispatch
        None
    }
    
    /// Clone the hooks for registration
    fn clone_box(&self) -> Box<dyn NodeExecutionHooks>;
}

/// Default implementation for nodes that don't need special handling
#[derive(Clone)]
pub struct DefaultHooks;

impl NodeExecutionHooks for DefaultHooks {
    fn clone_box(&self) -> Box<dyn NodeExecutionHooks> {
        Box::new(self.clone())
    }
}