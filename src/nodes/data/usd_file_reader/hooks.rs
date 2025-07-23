//! USD File Reader execution hooks
//!
//! Handles cache management for USD file reading operations

use crate::nodes::hooks::NodeExecutionHooks;
use crate::nodes::{Node, NodeGraph, NodeId};
use crate::nodes::interface::NodeData;
use std::collections::HashSet;

/// Execution hooks for USD File Reader node
#[derive(Clone)]
pub struct UsdFileReaderHooks;

impl UsdFileReaderHooks {
    // Removed: Cache clearing methods moved to viewport nodes
    // The two-stage USD processing means cache management is handled by consumers
}

impl NodeExecutionHooks for UsdFileReaderHooks {
    fn before_execution(&mut self, _node: &Node, _graph: &NodeGraph) -> Result<(), String> {
        // USD File Reader now uses two-stage processing
        // Stage 1: File loading (handled internally)
        // Stage 2: Data processing (no pre-execution clearing needed)
        // Cache clearing is now handled by the viewport nodes themselves
        Ok(())
    }
    
    fn after_execution(&mut self, _node: &Node, _outputs: &[NodeData], _graph: &NodeGraph) -> Result<(), String> {
        // USD File Reader caching is handled internally by the logic instance
        // No additional post-execution caching needed here
        Ok(())
    }
    
    fn on_node_removed(&mut self, _node_id: NodeId) -> Result<(), String> {
        // No caching cleanup needed - Stage 1 cache is global, Stage 2 has no persistent cache
        Ok(())
    }
    
    fn clone_box(&self) -> Box<dyn NodeExecutionHooks> {
        Box::new(self.clone())
    }
}