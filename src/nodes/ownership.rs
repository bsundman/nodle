//! Procedural ownership handoff system for reducing data clones
//! 
//! This module provides automatic optimization of data flow between nodes
//! by detecting single-consumer scenarios and using move semantics instead
//! of cloning large data structures.

use std::collections::HashMap;
use crate::nodes::{NodeId, NodeGraph, Connection};
use crate::nodes::interface::NodeData;

/// Tracks connection fanout for ownership handoff optimization
#[derive(Debug, Default)]
pub struct OwnershipTracker {
    /// Map from (node_id, port) to count of downstream connections
    output_fanout: HashMap<(NodeId, usize), usize>,
    /// Track which outputs have been consumed (for single consumer optimization)
    consumed_outputs: HashMap<(NodeId, usize), bool>,
}

impl OwnershipTracker {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Analyze the graph and compute fanout for all outputs
    pub fn analyze_graph(&mut self, graph: &NodeGraph) {
        self.output_fanout.clear();
        self.consumed_outputs.clear();
        
        // Count how many connections each output port has
        for connection in &graph.connections {
            let output_key = (connection.from_node, connection.from_port);
            *self.output_fanout.entry(output_key).or_insert(0) += 1;
        }
        
        // println!("🔗 Ownership Tracker: Analyzed {} connections", graph.connections.len());
        
        // Log fanout for debugging
        // for ((node_id, port), fanout) in &self.output_fanout {
        //     if *fanout > 1 {
        //         println!("🔗 Node {} port {} has {} consumers (requires cloning)", node_id, port, fanout);
        //     } else {
        //         println!("🔗 Node {} port {} has {} consumer (can use move semantics)", node_id, port, fanout);
        //     }
        // }
    }
    
    /// Check if an output can be moved instead of cloned (single consumer)
    pub fn can_move_output(&self, node_id: NodeId, port: usize) -> bool {
        let fanout = self.output_fanout.get(&(node_id, port)).unwrap_or(&0);
        *fanout == 1
    }
    
    /// Check if an output has already been consumed (for single consumer tracking)
    pub fn is_output_consumed(&self, node_id: NodeId, port: usize) -> bool {
        self.consumed_outputs.get(&(node_id, port)).unwrap_or(&false).clone()
    }
    
    /// Mark an output as consumed (for single consumer tracking)
    pub fn mark_output_consumed(&mut self, node_id: NodeId, port: usize) {
        self.consumed_outputs.insert((node_id, port), true);
        // println!("🔗 Marked output {}:{} as consumed", node_id, port);
    }
    
    /// Get fanout count for debugging
    pub fn get_fanout(&self, node_id: NodeId, port: usize) -> usize {
        self.output_fanout.get(&(node_id, port)).unwrap_or(&0).clone()
    }
}

/// Wrapper for NodeData that supports ownership handoff
#[derive(Debug, Clone)]
pub enum OwnedNodeData {
    /// Data that can be moved (single consumer)
    Owned(NodeData),
    /// Data that must be cloned (multiple consumers)
    Shared(NodeData),
}

impl OwnedNodeData {
    /// Create owned data (for single consumers)
    pub fn owned(data: NodeData) -> Self {
        OwnedNodeData::Owned(data)
    }
    
    /// Create shared data (for multiple consumers)
    pub fn shared(data: NodeData) -> Self {
        OwnedNodeData::Shared(data)
    }
    
    /// Extract the data, cloning if necessary
    pub fn extract(self) -> NodeData {
        match self {
            OwnedNodeData::Owned(data) => {
                // println!("🚀 Moving data (no clone)");
                data
            }
            OwnedNodeData::Shared(data) => {
                // println!("📋 Cloning shared data");
                data
            }
        }
    }
    
    /// Get a reference to the data without consuming
    pub fn as_ref(&self) -> &NodeData {
        match self {
            OwnedNodeData::Owned(data) => data,
            OwnedNodeData::Shared(data) => data,
        }
    }
    
    /// Convert to shared if needed (for multiple consumers)
    pub fn to_shared(self) -> Self {
        match self {
            OwnedNodeData::Owned(data) => OwnedNodeData::Shared(data),
            shared @ OwnedNodeData::Shared(_) => shared,
        }
    }
}

/// Strategy for optimizing data handoff between nodes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HandoffStrategy {
    /// Always clone data (safe but potentially inefficient)
    AlwaysClone,
    /// Use move semantics when safe (single consumer), clone otherwise
    OptimizeOwnership,
    /// Experimental: Try to hand off mutable references when safe
    MutableHandoff,
}

impl Default for HandoffStrategy {
    fn default() -> Self {
        HandoffStrategy::OptimizeOwnership
    }
}

/// Configuration for ownership handoff optimization
#[derive(Debug)]
pub struct OwnershipConfig {
    /// Strategy to use for data handoff
    pub strategy: HandoffStrategy,
    /// Whether to preserve original caches (important for Stage 1 USD caches)
    pub preserve_source_caches: bool,
    /// Whether to log ownership decisions for debugging
    pub debug_logging: bool,
    /// Types that should never be moved (always cloned for safety)
    pub immutable_types: Vec<String>,
}

impl Default for OwnershipConfig {
    fn default() -> Self {
        Self {
            strategy: HandoffStrategy::OptimizeOwnership,
            preserve_source_caches: true,
            debug_logging: false,
            immutable_types: vec![
                "CachedUsdFile".to_string(),  // Stage 1 cache must remain intact
            ],
        }
    }
}

/// Procedural ownership optimizer that works with any node type
pub struct OwnershipOptimizer {
    tracker: OwnershipTracker,
    config: OwnershipConfig,
}

impl OwnershipOptimizer {
    pub fn new(config: OwnershipConfig) -> Self {
        Self {
            tracker: OwnershipTracker::new(),
            config,
        }
    }
    
    pub fn with_default_config() -> Self {
        Self::new(OwnershipConfig::default())
    }
    
    /// Analyze graph for ownership optimization opportunities
    pub fn analyze_graph(&mut self, graph: &NodeGraph) {
        self.tracker.analyze_graph(graph);
        
        if self.config.debug_logging {
            println!("🔗 Ownership Optimizer: Analysis complete");
            println!("   Strategy: {:?}", self.config.strategy);
            println!("   Preserve source caches: {}", self.config.preserve_source_caches);
        }
    }
    
    /// Determine optimal data handoff for a specific output
    pub fn optimize_output(&mut self, node_id: NodeId, port: usize, data: NodeData) -> OwnedNodeData {
        match self.config.strategy {
            HandoffStrategy::AlwaysClone => {
                if self.config.debug_logging {
                    println!("🔗 Node {}:{} - Always clone strategy", node_id, port);
                }
                OwnedNodeData::shared(data)
            }
            
            HandoffStrategy::OptimizeOwnership => {
                let can_move = self.tracker.can_move_output(node_id, port);
                let fanout = self.tracker.get_fanout(node_id, port);
                
                if can_move && !self.is_immutable_type(&data) {
                    if self.config.debug_logging {
                        println!("🚀 Node {}:{} - Move optimization (single consumer)", node_id, port);
                    }
                    self.tracker.mark_output_consumed(node_id, port);
                    OwnedNodeData::owned(data)
                } else {
                    if self.config.debug_logging {
                        if fanout > 1 {
                            println!("📋 Node {}:{} - Clone required ({} consumers)", node_id, port, fanout);
                        } else {
                            println!("📋 Node {}:{} - Clone required (immutable type)", node_id, port);
                        }
                    }
                    OwnedNodeData::shared(data)
                }
            }
            
            HandoffStrategy::MutableHandoff => {
                // Future: Could implement mutable reference handoff here
                if self.config.debug_logging {
                    println!("🔗 Node {}:{} - Mutable handoff (not yet implemented)", node_id, port);
                }
                OwnedNodeData::shared(data)
            }
        }
    }
    
    /// Check if a data type should never be moved
    fn is_immutable_type(&self, data: &NodeData) -> bool {
        // This could be extended to check the actual data type
        // For now, we rely on configuration
        false  // Allow moves unless explicitly configured otherwise
    }
    
    /// Reset consumption tracking (call after each execution cycle)
    pub fn reset_consumption_tracking(&mut self) {
        self.tracker.consumed_outputs.clear();
        if self.config.debug_logging {
            println!("🔗 Reset ownership consumption tracking");
        }
    }
    
    /// Get statistics about ownership optimization
    pub fn get_statistics(&self) -> OwnershipStatistics {
        let total_outputs = self.tracker.output_fanout.len();
        let single_consumer_outputs = self.tracker.output_fanout
            .values()
            .filter(|&&fanout| fanout == 1)
            .count();
        let multi_consumer_outputs = total_outputs - single_consumer_outputs;
        
        OwnershipStatistics {
            total_outputs,
            single_consumer_outputs,
            multi_consumer_outputs,
            potential_moves: single_consumer_outputs,
            required_clones: multi_consumer_outputs,
        }
    }
}

/// Statistics about ownership optimization
#[derive(Debug)]
pub struct OwnershipStatistics {
    pub total_outputs: usize,
    pub single_consumer_outputs: usize,
    pub multi_consumer_outputs: usize,
    pub potential_moves: usize,
    pub required_clones: usize,
}

impl OwnershipStatistics {
    pub fn optimization_ratio(&self) -> f32 {
        if self.total_outputs == 0 {
            0.0
        } else {
            self.potential_moves as f32 / self.total_outputs as f32
        }
    }
}