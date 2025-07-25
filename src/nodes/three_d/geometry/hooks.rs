//! Shared execution hooks for 3D geometry nodes
//!
//! Handles cache management for geometry generation operations

use crate::nodes::hooks::NodeExecutionHooks;
use crate::nodes::{Node, NodeGraph, NodeId};
use crate::nodes::interface::NodeData;
use std::collections::HashSet;

/// Execution hooks for 3D geometry nodes (Cube, Sphere, Cylinder, etc.)
#[derive(Clone)]
pub struct GeometryHooks;

impl GeometryHooks {
    /// Find all downstream viewport nodes connected to this geometry node
    fn find_connected_viewport_nodes(&self, geometry_node_id: NodeId, graph: &NodeGraph) -> Vec<NodeId> {
        let mut connected_viewports = Vec::new();
        let mut visited = HashSet::new();
        let mut to_visit = vec![geometry_node_id];
        
        while let Some(current_id) = to_visit.pop() {
            if !visited.insert(current_id) {
                continue;
            }
            
            // Find all nodes this one connects to
            for connection in &graph.connections {
                if connection.from_node == current_id {
                    let target_id = connection.to_node;
                    
                    if let Some(target_node) = graph.nodes.get(&target_id) {
                        if target_node.type_id == "Viewport" || target_node.type_id == "3D_Viewport" {
                            connected_viewports.push(target_id);
                        }
                        // Continue searching downstream
                        to_visit.push(target_id);
                    }
                }
            }
        }
        
        connected_viewports
    }
}

impl NodeExecutionHooks for GeometryHooks {
    fn before_execution(&mut self, node: &Node, graph: &NodeGraph) -> Result<(), String> {
        println!("🎲 Geometry {}: Pre-execution - clearing GPU caches", node.type_id);
        
        // Find connected viewports and clear their GPU mesh caches
        let connected_viewports = self.find_connected_viewport_nodes(node.id, graph);
        
        if !connected_viewports.is_empty() {
            // Clear GPU mesh caches to ensure geometry updates are visible
            crate::gpu::viewport_3d_callback::clear_all_gpu_mesh_caches();
            
            // SIMPLIFIED: Clear GPU viewport cache for connected viewports
            // Unified cache system handles data invalidation automatically
            use crate::nodes::three_d::ui::viewport::GPU_VIEWPORT_CACHE;
            if let Ok(mut gpu_cache) = GPU_VIEWPORT_CACHE.lock() {
                for viewport_id in &connected_viewports {
                    gpu_cache.remove(viewport_id);
                }
            }
        }
        
        Ok(())
    }
    
    fn after_execution(&mut self, _node: &Node, _outputs: &[NodeData], _graph: &NodeGraph) -> Result<(), String> {
        // Geometry nodes generate fresh USD data each time
        // No caching needed at this level
        Ok(())
    }
    
    fn clone_box(&self) -> Box<dyn NodeExecutionHooks> {
        Box::new(self.clone())
    }
}