//! Viewport node execution hooks
//!
//! Handles cache management for viewport rendering operations

use crate::nodes::hooks::NodeExecutionHooks;
use crate::nodes::{Node, NodeGraph, NodeId};
use crate::nodes::interface::NodeData;

/// Execution hooks for Viewport node
#[derive(Clone)]
pub struct ViewportHooks;

impl ViewportHooks {
    /// Clear GPU viewport cache for this node
    fn clear_viewport_caches(&self, node_id: NodeId) {
        use super::viewport_node::{GPU_VIEWPORT_CACHE, USD_RENDERER_CACHE};
        
        println!("🎬 Viewport: Clearing GPU viewport cache for node {}", node_id);
        
        // Clear GPU viewport cache - unified cache handles everything else
        if let Ok(mut gpu_cache) = GPU_VIEWPORT_CACHE.lock() {
            if gpu_cache.remove(&node_id).is_some() {
                println!("🗑️ Viewport: Cleared GPU viewport cache for node {}", node_id);
            }
        }
        
        // Note: USD_RENDERER_CACHE stays as-is since it's keyed by file path
        // and shared across viewports for GPU resource management
    }
}

impl NodeExecutionHooks for ViewportHooks {
    fn before_execution(&mut self, node: &Node, _graph: &NodeGraph) -> Result<(), String> {
        // AGGRESSIVE CACHE CLEARING: Viewport nodes don't cache anything
        // Clear all GPU caches and buffers to ensure completely fresh rendering
        println!("🎬 Viewport: Before execution - clearing ALL caches for node {}", node.id);
        self.clear_viewport_caches(node.id);
        
        // Also clear global GPU mesh caches to ensure fresh geometry upload
        crate::gpu::viewport_3d_callback::clear_all_gpu_mesh_caches();
        
        Ok(())
    }
    
    fn after_execution(&mut self, node: &Node, outputs: &[NodeData], _graph: &NodeGraph) -> Result<(), String> {
        // SIMPLIFIED: ViewportNode::process_node handles all GPU cache storage
        // No additional post-execution work needed
        Ok(())
    }
    
    fn on_node_removed(&mut self, node_id: NodeId) -> Result<(), String> {
        // Clear all caches when viewport is removed
        self.clear_viewport_caches(node_id);
        
        // Also clear GPU mesh cache as this viewport's meshes are no longer needed
        crate::gpu::viewport_3d_callback::clear_all_gpu_mesh_caches();
        
        Ok(())
    }
    
    fn on_input_connection_added(&mut self, node: &Node, _graph: &NodeGraph) -> Result<(), String> {
        println!("🔗 Viewport: Input connection added to node {} - clearing ALL caches", node.id);
        
        // CRITICAL: Clear all caches when viewport input changes
        // This prevents showing stale data when switching between different inputs
        self.clear_viewport_caches(node.id);
        crate::gpu::viewport_3d_callback::clear_all_gpu_mesh_caches();
        
        Ok(())
    }
    
    fn on_input_connection_removed(&mut self, node: &Node, _graph: &NodeGraph) -> Result<(), String> {
        println!("🔗 Viewport: Input connection removed from node {} - clearing ALL caches", node.id);
        
        // CRITICAL: Clear all caches when viewport input is disconnected
        // This ensures clean state when input is removed
        self.clear_viewport_caches(node.id);
        crate::gpu::viewport_3d_callback::clear_all_gpu_mesh_caches();
        
        Ok(())
    }
    
    fn clone_box(&self) -> Box<dyn NodeExecutionHooks> {
        Box::new(self.clone())
    }
}