//! Scenegraph node execution hooks
//!
//! Handles cache management for scenegraph tree operations

use crate::nodes::hooks::NodeExecutionHooks;
use crate::nodes::{Node, NodeGraph, NodeId};
use crate::nodes::interface::NodeData;

/// Execution hooks for Scenegraph node
#[derive(Clone)]
pub struct ScenegraphHooks;

impl NodeExecutionHooks for ScenegraphHooks {
    fn before_execution(&mut self, node: &Node, _graph: &NodeGraph) -> Result<(), String> {
        println!("🌳 Scenegraph: Pre-execution for node {}", node.id);
        
        // Scenegraph doesn't have persistent caches to clear
        // It builds its tree structure fresh each time from input data
        // This is intentional to always show current state
        
        Ok(())
    }
    
    fn after_execution(&mut self, _node: &Node, _outputs: &[NodeData], _graph: &NodeGraph) -> Result<(), String> {
        // Scenegraph doesn't cache output data
        // The tree structure is ephemeral and rebuilt each frame
        Ok(())
    }
    
    fn clone_box(&self) -> Box<dyn NodeExecutionHooks> {
        Box::new(self.clone())
    }
}