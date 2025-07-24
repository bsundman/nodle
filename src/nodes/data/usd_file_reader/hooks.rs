//! USD File Reader execution hooks
//!
//! Handles USD-specific execution logic and cache management

use crate::nodes::hooks::NodeExecutionHooks;
use crate::nodes::{Node, NodeGraph, NodeId};
use crate::nodes::interface::NodeData;
use std::collections::HashMap;
use super::logic::UsdFileReaderLogic;

/// Execution hooks for USD File Reader node
pub struct UsdFileReaderHooks {
    /// Logic instances for USD File Reader nodes (for stateful processing)
    logic_instances: HashMap<NodeId, UsdFileReaderLogic>,
}

impl UsdFileReaderHooks {
    /// Create new USD File Reader hooks
    pub fn new() -> Self {
        Self {
            logic_instances: HashMap::new(),
        }
    }
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
    
    fn on_node_removed(&mut self, node_id: NodeId) -> Result<(), String> {
        // Clean up the logic instance for this node
        self.logic_instances.remove(&node_id);
        Ok(())
    }
    
    /// Custom execution for USD File Reader - implements the process_with_unified_cache logic
    fn custom_execution(
        &mut self, 
        node_id: NodeId,
        node: &Node, 
        inputs: Vec<NodeData>, 
        engine: &mut crate::nodes::NodeGraphEngine
    ) -> Option<Result<Vec<NodeData>, String>> {
        // USD File Reader needs custom execution via process_with_unified_cache
        
        // Get or create persistent logic instance
        if !self.logic_instances.contains_key(&node_id) {
            let logic = UsdFileReaderLogic::from_node(node);
            self.logic_instances.insert(node_id, logic);
        }
        
        // Extract the logic instance temporarily to avoid borrow conflicts
        let mut logic = self.logic_instances.remove(&node_id).unwrap();
        logic.update_from_node(node);
        
        // Process with the logic instance using the USD-specific two-stage cache system
        let result = logic.process_with_unified_cache(node_id, inputs, engine);
        
        // Put the logic instance back
        self.logic_instances.insert(node_id, logic);
        
        // Return the custom execution result
        Some(Ok(result))
    }
    
    fn clone_box(&self) -> Box<dyn NodeExecutionHooks> {
        Box::new(UsdFileReaderHooks::new())
    }
}