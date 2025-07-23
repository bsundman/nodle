//! Unified caching system for node execution
//! 
//! This module provides a centralized caching architecture that supports
//! both single-stage nodes and multi-stage nodes (like USD file readers)
//! with intelligent cache management and ownership optimization.

use std::collections::HashMap;
use crate::nodes::{NodeId, interface::NodeData};
use crate::nodes::ownership::OwnedNodeData;
use serde::{Serialize, Deserialize};

/// Extended cache key that supports multi-stage node caching
#[derive(Hash, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct CacheKey {
    /// The node that produced this cached data
    pub node_id: NodeId,
    /// Optional stage identifier for multi-stage nodes (e.g., "stage1", "stage2")
    /// None for single-stage nodes
    pub stage_id: Option<String>,
    /// Output port index within the stage
    pub port_index: usize,
}

impl CacheKey {
    /// Create a cache key for a single-stage node output
    pub fn new(node_id: NodeId, port_index: usize) -> Self {
        Self {
            node_id,
            stage_id: None,
            port_index,
        }
    }
    
    /// Create a cache key for a multi-stage node output
    pub fn with_stage(node_id: NodeId, stage_id: &str, port_index: usize) -> Self {
        Self {
            node_id,
            stage_id: Some(stage_id.to_string()),
            port_index,
        }
    }
    
    /// Check if this is a stage-specific cache key
    pub fn has_stage(&self) -> bool {
        self.stage_id.is_some()
    }
    
    /// Get the stage ID if this is a multi-stage key
    pub fn get_stage(&self) -> Option<&str> {
        self.stage_id.as_deref()
    }
    
    /// Create a pattern for invalidating all stages of a node
    pub fn node_pattern(node_id: NodeId) -> CacheKeyPattern {
        CacheKeyPattern::Node(node_id)
    }
    
    /// Create a pattern for invalidating a specific stage of a node
    pub fn stage_pattern(node_id: NodeId, stage_id: &str) -> CacheKeyPattern {
        CacheKeyPattern::Stage(node_id, stage_id.to_string())
    }
}

/// Pattern for matching cache keys during invalidation
#[derive(Debug, Clone)]
pub enum CacheKeyPattern {
    /// Match all outputs for a specific node
    Node(NodeId),
    /// Match all outputs for a specific stage of a node  
    Stage(NodeId, String),
    /// Match a specific cache key exactly
    Exact(CacheKey),
}

impl CacheKeyPattern {
    /// Check if this pattern matches a given cache key
    pub fn matches(&self, key: &CacheKey) -> bool {
        match self {
            CacheKeyPattern::Node(node_id) => key.node_id == *node_id,
            CacheKeyPattern::Stage(node_id, stage) => {
                key.node_id == *node_id && key.stage_id.as_ref() == Some(stage)
            },
            CacheKeyPattern::Exact(exact_key) => key == exact_key,
        }
    }
}

/// Statistics about cache performance and usage
#[derive(Debug, Default, Clone)]
pub struct CacheStatistics {
    /// Total number of cache entries
    pub total_entries: usize,
    /// Number of single-stage entries
    pub single_stage_entries: usize,
    /// Number of multi-stage entries
    pub multi_stage_entries: usize,
    /// Cache hits during this session
    pub cache_hits: usize,
    /// Cache misses during this session
    pub cache_misses: usize,
    /// Number of entries evicted due to invalidation
    pub cache_invalidations: usize,
    /// Memory usage estimate (in bytes)
    pub estimated_memory_usage: usize,
}

impl CacheStatistics {
    /// Calculate cache hit ratio
    pub fn hit_ratio(&self) -> f32 {
        let total_accesses = self.cache_hits + self.cache_misses;
        if total_accesses == 0 {
            0.0
        } else {
            self.cache_hits as f32 / total_accesses as f32
        }
    }
    
    /// Get average memory per entry
    pub fn avg_memory_per_entry(&self) -> usize {
        if self.total_entries == 0 {
            0
        } else {
            self.estimated_memory_usage / self.total_entries
        }
    }
}

/// Unified cache for all node execution results
#[derive(Debug)]
pub struct UnifiedNodeCache {
    /// Main cache storage with ownership optimization
    cache: HashMap<CacheKey, OwnedNodeData>,
    /// Performance statistics
    stats: CacheStatistics,
    /// Whether to track detailed statistics (can be disabled for performance)
    track_statistics: bool,
}

impl UnifiedNodeCache {
    /// Create a new unified cache
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            stats: CacheStatistics::default(),
            track_statistics: true,
        }
    }
    
    /// Create a new unified cache with statistics tracking disabled
    pub fn new_without_stats() -> Self {
        Self {
            cache: HashMap::new(),
            stats: CacheStatistics::default(),
            track_statistics: false,
        }
    }
    
    /// Store data in the cache with ownership optimization
    pub fn insert(&mut self, key: CacheKey, data: OwnedNodeData) {
        if self.track_statistics {
            if self.cache.contains_key(&key) {
                // Replacing existing entry
            } else {
                // New entry
                self.stats.total_entries += 1;
                if key.has_stage() {
                    self.stats.multi_stage_entries += 1;
                } else {
                    self.stats.single_stage_entries += 1;
                }
            }
        }
        
        self.cache.insert(key, data);
        self.update_memory_stats();
    }
    
    /// Retrieve data from cache (returns reference)
    pub fn get(&mut self, key: &CacheKey) -> Option<&NodeData> {
        if self.track_statistics {
            if self.cache.contains_key(key) {
                self.stats.cache_hits += 1;
            } else {
                self.stats.cache_misses += 1;
            }
        }
        
        self.cache.get(key).map(|owned| owned.as_ref())
    }
    
    /// Retrieve and remove data from cache (for move semantics)
    pub fn take(&mut self, key: &CacheKey) -> Option<NodeData> {
        if self.track_statistics {
            if self.cache.contains_key(key) {
                self.stats.cache_hits += 1;
                self.stats.total_entries -= 1;
                if key.has_stage() {
                    self.stats.multi_stage_entries -= 1;
                } else {
                    self.stats.single_stage_entries -= 1;
                }
            } else {
                self.stats.cache_misses += 1;
            }
        }
        
        let result = self.cache.remove(key).map(|owned| owned.extract());
        self.update_memory_stats();
        result
    }
    
    /// Check if a key exists in the cache
    pub fn contains(&self, key: &CacheKey) -> bool {
        self.cache.contains_key(key)
    }
    
    /// Invalidate cache entries matching a pattern
    pub fn invalidate(&mut self, pattern: &CacheKeyPattern) -> usize {
        let keys_to_remove: Vec<CacheKey> = self.cache.keys()
            .filter(|key| pattern.matches(key))
            .cloned()
            .collect();
        
        let removed_count = keys_to_remove.len();
        
        for key in keys_to_remove {
            self.cache.remove(&key);
            if self.track_statistics {
                self.stats.total_entries -= 1;
                self.stats.cache_invalidations += 1;
                if key.has_stage() {
                    self.stats.multi_stage_entries -= 1;
                } else {
                    self.stats.single_stage_entries -= 1;
                }
            }
        }
        
        self.update_memory_stats();
        removed_count
    }
    
    /// Clear all cache entries
    pub fn clear(&mut self) {
        let removed_count = self.cache.len();
        self.cache.clear();
        
        if self.track_statistics {
            self.stats.cache_invalidations += removed_count;
            self.stats.total_entries = 0;
            self.stats.single_stage_entries = 0;
            self.stats.multi_stage_entries = 0;
            self.stats.estimated_memory_usage = 0;
        }
    }
    
    /// Get cache statistics
    pub fn get_statistics(&self) -> &CacheStatistics {
        &self.stats
    }
    
    /// Reset statistics counters
    pub fn reset_statistics(&mut self) {
        self.stats.cache_hits = 0;
        self.stats.cache_misses = 0;
        self.stats.cache_invalidations = 0;
        // Keep structural stats (total_entries, etc.)
    }
    
    /// Get all cache keys (for debugging/inspection)
    pub fn get_all_keys(&self) -> Vec<&CacheKey> {
        self.cache.keys().collect()
    }
    
    /// Get cache entries for a specific node
    pub fn get_node_entries(&self, node_id: NodeId) -> Vec<(&CacheKey, &NodeData)> {
        self.cache.iter()
            .filter(|(key, _)| key.node_id == node_id)
            .map(|(key, data)| (key, data.as_ref()))
            .collect()
    }
    
    /// Get cache entries for a specific stage of a node
    pub fn get_stage_entries(&self, node_id: NodeId, stage_id: &str) -> Vec<(&CacheKey, &NodeData)> {
        self.cache.iter()
            .filter(|(key, _)| {
                key.node_id == node_id && key.stage_id.as_deref() == Some(stage_id)
            })
            .map(|(key, data)| (key, data.as_ref()))
            .collect()
    }
    
    /// Estimate memory usage (rough approximation)
    fn update_memory_stats(&mut self) {
        if !self.track_statistics {
            return;
        }
        
        // Rough estimation - in a real implementation you might want more accurate sizing
        self.stats.estimated_memory_usage = self.cache.len() * std::mem::size_of::<(CacheKey, OwnedNodeData)>();
    }
}

impl Default for UnifiedNodeCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_key_creation() {
        let single_stage = CacheKey::new(1, 0);
        assert!(!single_stage.has_stage());
        assert_eq!(single_stage.get_stage(), None);
        
        let multi_stage = CacheKey::with_stage(1, "stage1", 0);
        assert!(multi_stage.has_stage());
        assert_eq!(multi_stage.get_stage(), Some("stage1"));
    }
    
    #[test]
    fn test_cache_key_patterns() {
        let key1 = CacheKey::new(1, 0);
        let key2 = CacheKey::with_stage(1, "stage1", 0);
        let key3 = CacheKey::with_stage(1, "stage2", 0);
        let key4 = CacheKey::new(2, 0);
        
        let node_pattern = CacheKeyPattern::Node(1);
        assert!(node_pattern.matches(&key1));
        assert!(node_pattern.matches(&key2));
        assert!(node_pattern.matches(&key3));
        assert!(!node_pattern.matches(&key4));
        
        let stage_pattern = CacheKeyPattern::Stage(1, "stage1".to_string());
        assert!(!stage_pattern.matches(&key1)); // No stage
        assert!(stage_pattern.matches(&key2));
        assert!(!stage_pattern.matches(&key3)); // Different stage
    }
    
    #[test]
    fn test_unified_cache_basic_operations() {
        let mut cache = UnifiedNodeCache::new();
        let key = CacheKey::new(1, 0);
        let data = OwnedNodeData::shared(NodeData::Float(42.0));
        
        // Insert and retrieve
        cache.insert(key.clone(), data);
        assert!(cache.contains(&key));
        
        let retrieved = cache.get(&key);
        assert!(retrieved.is_some());
        if let Some(NodeData::Float(value)) = retrieved {
            assert_eq!(*value, 42.0);
        } else {
            panic!("Expected Float(42.0)");
        }
        
        // Statistics
        let stats = cache.get_statistics();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 0);
    }
}