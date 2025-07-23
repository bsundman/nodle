//! Attributes node execution hooks
//!
//! Handles cache management for attributes spreadsheet operations

use crate::nodes::hooks::NodeExecutionHooks;
use crate::nodes::{Node, NodeGraph, NodeId};
use crate::nodes::interface::NodeData;

/// Execution hooks for Attributes node
#[derive(Clone)]
pub struct AttributesHooks;

impl NodeExecutionHooks for AttributesHooks {
    fn before_execution(&mut self, node: &Node, _graph: &NodeGraph) -> Result<(), String> {
        println!("📊 Attributes: Pre-execution - clearing attributes cache for node {}", node.id);
        
        // Clear the attributes input cache to ensure fresh data
        use super::logic::ATTRIBUTES_INPUT_CACHE;
        
        if let Ok(mut cache) = ATTRIBUTES_INPUT_CACHE.write() {
            if cache.remove(&node.id).is_some() {
                println!("📊 Attributes: Cleared input cache");
            }
        }
        
        Ok(())
    }
    
    fn after_execution(&mut self, _node: &Node, _outputs: &[NodeData], _graph: &NodeGraph) -> Result<(), String> {
        // Attributes caching is handled internally by process_attributes_node
        // which manages the ATTRIBUTES_INPUT_CACHE
        Ok(())
    }
    
    fn on_node_removed(&mut self, node_id: NodeId) -> Result<(), String> {
        // Clear attributes cache when node is removed
        use super::logic::ATTRIBUTES_INPUT_CACHE;
        
        if let Ok(mut cache) = ATTRIBUTES_INPUT_CACHE.write() {
            cache.remove(&node_id);
        }
        
        Ok(())
    }
    
    fn clone_box(&self) -> Box<dyn NodeExecutionHooks> {
        Box::new(self.clone())
    }
}