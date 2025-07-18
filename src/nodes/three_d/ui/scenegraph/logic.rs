//! Scenegraph node execution logic

use crate::nodes::{
    Node, NodeId, NodeGraph,
    interface::NodeData,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use log::debug;

/// Cached data with version tracking for tree panel optimization
#[derive(Clone)]
pub struct CachedScenegraphData {
    pub data: NodeData,
    pub version: u64,
    pub last_updated: std::time::Instant,
}

/// Global cache for scenegraph input data to bridge process_node and tree panel
pub static SCENEGRAPH_INPUT_CACHE: Lazy<Arc<Mutex<HashMap<NodeId, CachedScenegraphData>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

/// Version counter for cache invalidation
static CACHE_VERSION_COUNTER: Lazy<Arc<Mutex<u64>>> = Lazy::new(|| {
    Arc::new(Mutex::new(0))
});

/// Get next cache version
fn get_next_version() -> u64 {
    if let Ok(mut counter) = CACHE_VERSION_COUNTER.lock() {
        *counter += 1;
        *counter
    } else {
        1
    }
}

/// Logic for the scenegraph node
pub struct ScenegraphLogic;

impl ScenegraphLogic {
    /// Process the scenegraph node
    pub fn process(
        node: &Node,
        inputs: Vec<Option<NodeData>>,
        _graph: &NodeGraph,
    ) -> HashMap<usize, NodeData> {
        debug!("🌳 Scenegraph node processing inputs: {:?}", inputs.len());
        
        // This node doesn't produce outputs, it just displays the input
        // The actual rendering happens in the tree panel
        
        if let Some(Some(input_data)) = inputs.get(0) {
            // Convert full USD data to lightweight metadata for scenegraph display
            let scenegraph_data = match input_data {
                NodeData::USDSceneData(scene_data) => {
                    debug!("🌳 Scenegraph received USD scene data with {} meshes, {} lights, {} materials",
                        scene_data.meshes.len(),
                        scene_data.lights.len(),
                        scene_data.materials.len()
                    );
                    
                    // Extract lightweight metadata instead of storing full geometry
                    let metadata = crate::workspaces::three_d::usd::usd_engine::USDEngine::extract_scenegraph_metadata(scene_data);
                    debug!("🌳 Scenegraph extracted metadata: {} total vertices, {} total triangles", 
                        metadata.total_vertices, metadata.total_triangles);
                    
                    NodeData::USDScenegraphMetadata(metadata)
                }
                NodeData::String(path) => {
                    debug!("🌳 Scenegraph received USD file path: {}", path);
                    input_data.clone()
                }
                _ => {
                    debug!("🌳 Scenegraph received unsupported data type");
                    input_data.clone()
                }
            };
            
            // Store the lightweight data in cache with version tracking
            if let Ok(mut cache) = SCENEGRAPH_INPUT_CACHE.lock() {
                let version = get_next_version();
                let cached_data = CachedScenegraphData {
                    data: scenegraph_data,
                    version,
                    last_updated: std::time::Instant::now(),
                };
                cache.insert(node.id, cached_data);
                debug!("🌳 Scenegraph stored lightweight metadata in cache for node {} (version {})", node.id, version);
            }
        } else {
            // Clear cache when no input is connected
            if let Ok(mut cache) = SCENEGRAPH_INPUT_CACHE.lock() {
                cache.remove(&node.id);
                debug!("🌳 Scenegraph cleared cache for node {} (no input)", node.id);
            }
            debug!("🌳 Scenegraph has no input connected");
        }
        
        // No outputs from this node
        HashMap::new()
    }
}