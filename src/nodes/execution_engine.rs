//! Node graph execution engine
//! 
//! This module provides a comprehensive dataflow execution system that handles:
//! - Connection state tracking
//! - Dependency resolution 
//! - Dirty state propagation
//! - Execution ordering via topological sort
//! - Node evaluation triggering

use std::collections::{HashMap, HashSet, VecDeque};
use crate::nodes::{NodeId, NodeGraph, Node, Connection};
use crate::nodes::interface::NodeData;
use crate::nodes::hooks::{NodeExecutionHooks, DefaultHooks};
use crate::nodes::ownership::{OwnershipOptimizer, OwnershipConfig, OwnedNodeData};
use crate::nodes::cache::{UnifiedNodeCache, CacheKey, CacheKeyPattern};

/// Represents the execution state of a node
#[derive(Debug, Clone, PartialEq)]
pub enum NodeState {
    Clean,      // Node is up-to-date
    Dirty,      // Node needs re-evaluation
    Computing,  // Node is currently being processed
    Error,      // Node failed to execute
}

/// Execution mode for the graph engine
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EngineExecutionMode {
    /// Execute immediately when parameters or connections change
    Auto,
    /// Only execute when manually triggered
    Manual,
}

/// Execution engine for node graphs
pub struct NodeGraphEngine {
    /// Current execution state for each node
    node_states: HashMap<NodeId, NodeState>,
    /// Unified cache for all node outputs with stage support
    pub unified_cache: UnifiedNodeCache,
    /// Set of nodes that need re-evaluation
    dirty_nodes: HashSet<NodeId>,
    /// Execution order cache (invalidated when graph changes)
    execution_order_cache: Option<Vec<NodeId>>,
    /// Node-specific execution hooks
    execution_hooks: HashMap<String, Box<dyn NodeExecutionHooks>>,
    /// Execution mode
    execution_mode: EngineExecutionMode,
    /// Ownership optimizer for reducing data clones
    ownership_optimizer: OwnershipOptimizer,
    /// Persistent USD File Reader logic instances for parameter tracking
    usd_reader_instances: HashMap<NodeId, crate::nodes::data::usd_file_reader::logic::UsdFileReaderLogic>,
    /// Global persistent USD file data storage keyed by file hash (survives cache invalidations)
    persistent_usd_file_data: HashMap<String, crate::workspaces::three_d::usd::usd_engine::USDSceneData>,
}

impl NodeGraphEngine {
    /// Create a new execution engine
    pub fn new() -> Self {
        let mut hooks: HashMap<String, Box<dyn NodeExecutionHooks>> = HashMap::new();
        
        // Register hooks for nodes that need special handling
        
        // USD File Reader
        hooks.insert("Data_UsdFileReader".to_string(), 
                    Box::new(crate::nodes::data::usd_file_reader::hooks::UsdFileReaderHooks));
        
        // Viewport
        hooks.insert("Viewport".to_string(),
                    Box::new(crate::nodes::three_d::ui::viewport::hooks::ViewportHooks));
        
        // Attributes
        hooks.insert("Attributes".to_string(),
                    Box::new(crate::nodes::three_d::ui::attributes::hooks::AttributesHooks));
        
        // Scenegraph
        hooks.insert("Scenegraph".to_string(),
                    Box::new(crate::nodes::three_d::ui::scenegraph::hooks::ScenegraphHooks));
        
        // 3D Geometry nodes (they all share the same hooks)
        let geometry_hooks = crate::nodes::three_d::geometry::hooks::GeometryHooks;
        hooks.insert("3D_Cube".to_string(), geometry_hooks.clone_box());
        hooks.insert("3D_Sphere".to_string(), geometry_hooks.clone_box());
        hooks.insert("3D_Cylinder".to_string(), geometry_hooks.clone_box());
        hooks.insert("3D_Cone".to_string(), geometry_hooks.clone_box());
        hooks.insert("3D_Plane".to_string(), geometry_hooks.clone_box());
        hooks.insert("3D_Capsule".to_string(), geometry_hooks.clone_box());
        
        Self {
            node_states: HashMap::new(),
            unified_cache: UnifiedNodeCache::new(),
            dirty_nodes: HashSet::new(),
            execution_order_cache: None,
            execution_hooks: hooks,
            execution_mode: EngineExecutionMode::Auto, // Default to auto
            ownership_optimizer: OwnershipOptimizer::with_default_config(),
            usd_reader_instances: HashMap::new(),
            persistent_usd_file_data: HashMap::new(),
        }
    }

    /// Mark a node as dirty (needs re-evaluation)
    pub fn mark_dirty(&mut self, node_id: NodeId, graph: &NodeGraph) {
        if self.node_states.get(&node_id) == Some(&NodeState::Dirty) {
            return; // Already dirty
        }

        // Marking node as dirty
        self.node_states.insert(node_id, NodeState::Dirty);
        self.dirty_nodes.insert(node_id);
        
        // Invalidate all cache entries for this node (all stages and ports)
        let invalidated_count = self.unified_cache.invalidate(&CacheKeyPattern::Node(node_id));
        if invalidated_count > 0 {
            println!("🗑️ Invalidated {} cache entries for node {}", invalidated_count, node_id);
        }
        
        // Propagate dirty state to downstream nodes
        self.propagate_dirty_downstream(node_id, graph);
        
        // Invalidate execution order cache
        self.execution_order_cache = None;
    }
    
    /// Handle USD File Reader parameter changes with granular cache invalidation
    fn on_usd_file_reader_parameter_changed(&mut self, node_id: NodeId, graph: &NodeGraph) {
        println!("🔧 ExecutionEngine: USD File Reader parameter changed for node {} in {} mode", 
                 node_id, if self.execution_mode == EngineExecutionMode::Auto { "Auto" } else { "Manual" });
        
        // Mark as dirty without blanket cache invalidation - the USD logic will handle granular invalidation
        self.mark_dirty_without_cache_invalidation(node_id, graph);
        
        // Execute immediately in Auto mode
        if self.execution_mode == EngineExecutionMode::Auto {
            println!("🔧 ExecutionEngine: Executing USD File Reader immediately due to parameter change");
            if let Err(e) = self.execute_dirty_nodes(graph) {
                eprintln!("❌ ExecutionEngine: Failed to execute dirty nodes: {}", e);
            }
        }
    }
    
    /// Mark a node as dirty without invalidating its cache (for nodes that handle their own cache)
    fn mark_dirty_without_cache_invalidation(&mut self, node_id: NodeId, graph: &NodeGraph) {
        if self.node_states.get(&node_id) == Some(&NodeState::Dirty) {
            return; // Already dirty
        }

        // Marking node as dirty without cache invalidation
        self.node_states.insert(node_id, NodeState::Dirty);
        self.dirty_nodes.insert(node_id);
        
        println!("🗑️ Node {} marked dirty - cache invalidation handled by node", node_id);
        
        // Propagate dirty state to downstream nodes
        self.propagate_dirty_downstream(node_id, graph);
        
        // Invalidate execution order cache
        self.execution_order_cache = None;
    }
    
    /// Check if a node handles its own cache invalidation
    fn node_handles_own_cache_invalidation(&self, node_id: NodeId, graph: &NodeGraph) -> bool {
        // Find the node in the graph
        if let Some(node) = graph.nodes.get(&node_id) {
            // USD File Reader nodes handle their own cache invalidation
            node.type_id == "USD File Reader"
        } else {
            false
        }
    }

    /// Propagate dirty state to all downstream nodes
    fn propagate_dirty_downstream(&mut self, node_id: NodeId, graph: &NodeGraph) {
        let downstream_nodes = self.find_downstream_nodes(node_id, graph);
        
        for downstream_id in downstream_nodes {
            if self.node_states.get(&downstream_id) != Some(&NodeState::Dirty) {
                // Propagating dirty to downstream node
                self.node_states.insert(downstream_id, NodeState::Dirty);
                self.dirty_nodes.insert(downstream_id);
                
                // Invalidate all cache entries for downstream node
                self.unified_cache.invalidate(&CacheKeyPattern::Node(downstream_id));
                
                // Recursively propagate
                self.propagate_dirty_downstream(downstream_id, graph);
            }
        }
    }

    /// Find all nodes downstream from the given node
    fn find_downstream_nodes(&self, node_id: NodeId, graph: &NodeGraph) -> Vec<NodeId> {
        let mut downstream = Vec::new();
        
        for connection in &graph.connections {
            if connection.from_node == node_id {
                downstream.push(connection.to_node);
            }
        }
        
        downstream
    }
    
    /// Find all nodes upstream from the given node
    fn find_upstream_nodes(&self, node_id: NodeId, graph: &NodeGraph) -> Vec<NodeId> {
        let mut upstream = Vec::new();
        
        for connection in &graph.connections {
            if connection.to_node == node_id {
                upstream.push(connection.from_node);
            }
        }
        
        upstream
    }
    
    /// Propagate dirty state to all upstream nodes (dependencies)
    fn propagate_dirty_upstream(&mut self, node_id: NodeId, graph: &NodeGraph) {
        let upstream_nodes = self.find_upstream_nodes(node_id, graph);
        
        for upstream_id in upstream_nodes {
            if self.node_states.get(&upstream_id) != Some(&NodeState::Dirty) {
                self.node_states.insert(upstream_id, NodeState::Dirty);
                self.dirty_nodes.insert(upstream_id);
                
                // Invalidate all cache entries for upstream node
                self.unified_cache.invalidate(&CacheKeyPattern::Node(upstream_id));
                
                // Recursively propagate upstream
                self.propagate_dirty_upstream(upstream_id, graph);
            }
        }
    }
    

    /// Get the execution order using topological sort
    pub fn get_execution_order(&mut self, graph: &NodeGraph) -> Result<Vec<NodeId>, String> {
        // Use cached order if available and graph hasn't changed
        if let Some(ref order) = self.execution_order_cache {
            return Ok(order.clone());
        }

        // Computing execution order
        
        // Build dependency graph
        let mut in_degree = HashMap::new();
        let mut adj_list: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        
        // Initialize all nodes
        for node_id in graph.nodes.keys() {
            in_degree.insert(*node_id, 0);
            adj_list.insert(*node_id, Vec::new());
        }
        
        // Build adjacency list and compute in-degrees
        for connection in &graph.connections {
            adj_list.get_mut(&connection.from_node)
                .unwrap()
                .push(connection.to_node);
            
            *in_degree.get_mut(&connection.to_node).unwrap() += 1;
        }
        
        // Kahn's algorithm for topological sort
        let mut queue = VecDeque::new();
        let mut result = Vec::new();
        
        // Start with nodes that have no dependencies
        for (&node_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node_id);
            }
        }
        
        while let Some(node_id) = queue.pop_front() {
            result.push(node_id);
            
            // Update dependencies of downstream nodes
            if let Some(neighbors) = adj_list.get(&node_id) {
                for &neighbor in neighbors {
                    let degree = in_degree.get_mut(&neighbor).unwrap();
                    *degree -= 1;
                    
                    if *degree == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }
        
        // Check for cycles
        if result.len() != graph.nodes.len() {
            return Err("Cycle detected in node graph".to_string());
        }
        
        // Execution order computed
        
        // Cache the result
        self.execution_order_cache = Some(result.clone());
        Ok(result)
    }

    /// Execute all dirty nodes in dependency order
    /// This method executes regardless of execution mode - caller must check mode
    pub fn execute_dirty_nodes(&mut self, graph: &NodeGraph) -> Result<(), String> {
        // Analyze graph for ownership optimization before execution
        self.ownership_optimizer.analyze_graph(graph);
        
        // Debug: Show all node states
        // Node states checked
        
        if self.dirty_nodes.is_empty() {
            // No dirty nodes to execute
            
            // Check if we have any new nodes that need initial execution
            for &node_id in graph.nodes.keys() {
                if !self.node_states.contains_key(&node_id) {
                    // Found new node - marking as dirty
                    self.mark_dirty(node_id, graph);
                }
            }
            
            // If we found new nodes, try execution again
            if !self.dirty_nodes.is_empty() {
                // Executing newly discovered dirty nodes
            } else {
                return Ok(());
            }
        } else {
            // Executing dirty nodes
        }
        
        let execution_order = self.get_execution_order(graph)?;
        
        // Only execute nodes that are dirty and in our execution order
        for &node_id in &execution_order {
            if self.dirty_nodes.contains(&node_id) {
                self.execute_single_node(node_id, graph)?;
            }
        }
        
        // Clear dirty set after successful execution
        self.dirty_nodes.clear();
        
        // Reset ownership tracking for next execution cycle
        self.ownership_optimizer.reset_consumption_tracking();
        
        // All dirty nodes executed
        
        Ok(())
    }

    /// Execute a single node
    fn execute_single_node(&mut self, node_id: NodeId, graph: &NodeGraph) -> Result<(), String> {
        let node = graph.nodes.get(&node_id)
            .ok_or_else(|| format!("Node {} not found", node_id))?;

        // Executing node
        
        // Mark as computing
        self.node_states.insert(node_id, NodeState::Computing);
        
        // Call pre-execution hook
        if let Some(hooks) = self.execution_hooks.get_mut(&node.type_id) {
            if let Err(e) = hooks.before_execution(node, graph) {
                eprintln!("Pre-execution hook failed for {}: {}", node.type_id, e);
                // Continue execution even if hook fails
            }
        }
        
        // Collect inputs from upstream nodes
        let inputs = self.collect_node_inputs(node_id, graph);
        
        // Special handling for USD File Reader to use unified cache system
        let outputs = if node.type_id == "Data_UsdFileReader" {
            // Get or create persistent logic instance
            if !self.usd_reader_instances.contains_key(&node_id) {
                let logic = crate::nodes::data::usd_file_reader::logic::UsdFileReaderLogic::from_node(node);
                self.usd_reader_instances.insert(node_id, logic);
            }
            
            // Extract the logic instance temporarily to avoid borrow conflicts
            let mut logic = self.usd_reader_instances.remove(&node_id).unwrap();
            logic.update_from_node(node);
            
            // Process with the logic instance
            let result = logic.process_with_unified_cache(node_id, inputs, self);
            
            // Put the logic instance back
            self.usd_reader_instances.insert(node_id, logic);
            
            Ok(result)
        } else {
            // Execute the node using the factory dispatch system
            self.dispatch_node_execution(node, inputs)
        };
        
        let outputs = match outputs {
            Ok(outputs) => outputs,
            Err(e) => {
                // Node execution failed
                self.node_states.insert(node_id, NodeState::Error);
                return Err(e);
            }
        };
        
        // Call post-execution hook with the outputs
        if let Some(hooks) = self.execution_hooks.get_mut(&node.type_id) {
            if let Err(e) = hooks.after_execution(node, &outputs, graph) {
                eprintln!("Post-execution hook failed for {}: {}", node.type_id, e);
                // Continue even if hook fails
            }
        }
        
        // Cache the outputs with ownership optimization in unified cache
        // Caching outputs
        for (port_idx, output) in outputs.into_iter().enumerate() {
            let optimized_output = self.ownership_optimizer.optimize_output(node_id, port_idx, output);
            let cache_key = CacheKey::new(node_id, port_idx);
            self.unified_cache.insert(cache_key, optimized_output);
        }
        
        // Mark as clean
        self.node_states.insert(node_id, NodeState::Clean);
        self.dirty_nodes.remove(&node_id);
        
        // Node executed successfully
        Ok(())
    }

    /// Collect inputs for a node from connected upstream nodes
    fn collect_node_inputs(&mut self, node_id: NodeId, graph: &NodeGraph) -> Vec<NodeData> {
        let node = match graph.nodes.get(&node_id) {
            Some(node) => node,
            None => return vec![],
        };

        let mut inputs = vec![NodeData::None; node.inputs.len()];
        
        // Find all connections feeding into this node
        let mut found_connections = 0;
        for connection in &graph.connections {
            if connection.to_node == node_id {
                found_connections += 1;
                
                // Get the output from the source node via unified cache
                let cache_key = CacheKey::new(connection.from_node, connection.from_port);
                if let Some(cached_data) = self.unified_cache.get(&cache_key) {
                    if connection.to_port < inputs.len() {
                        inputs[connection.to_port] = cached_data.clone();
                    }
                }
            }
        }
        inputs
    }

    /// Dispatch node execution based on node type_id
    fn dispatch_node_execution(&self, node: &Node, inputs: Vec<NodeData>) -> Result<Vec<NodeData>, String> {
        // Use the node type_id to dispatch execution (independent of user-editable title)
        match node.type_id.as_str() {
            // Data nodes
            "Data_UsdFileReader" => {
                // Executing USD File Reader
                Ok(crate::nodes::data::usd_file_reader::UsdFileReaderNode::process_node(node, inputs))
            }
            
            // Viewport/UI nodes
            "Viewport" => {
                // Executing Viewport node
                Ok(crate::nodes::three_d::ui::viewport::ViewportNode::process_node(node, &inputs))
            }
            
            // Math nodes
            "Add" => {
                // Executing Add node
                Ok(crate::nodes::math::add::functions::process_add(inputs))
            }
            "Subtract" => {
                // Executing Subtract node
                Ok(crate::nodes::math::subtract::functions::process_subtract(inputs))
            }
            "Multiply" => {
                // Executing Multiply node
                // Simple multiplication implementation since multiply functions module doesn't exist
                if inputs.len() >= 2 {
                    let a = match &inputs[0] {
                        NodeData::Float(f) => *f,
                        _ => 0.0,
                    };
                    let b = match &inputs[1] {
                        NodeData::Float(f) => *f,
                        _ => 0.0,
                    };
                    Ok(vec![NodeData::Float(a * b)])
                } else {
                    Ok(vec![NodeData::Float(0.0)])
                }
            }
            "Divide" => {
                // Executing Divide node
                Ok(crate::nodes::math::divide::functions::process_divide(inputs))
            }
            
            // Logic nodes (simple implementations since functions modules don't exist)
            "And" => {
                // Executing And node
                if inputs.len() >= 2 {
                    let a = match &inputs[0] {
                        NodeData::Boolean(b) => *b,
                        _ => false,
                    };
                    let b = match &inputs[1] {
                        NodeData::Boolean(b) => *b,
                        _ => false,
                    };
                    Ok(vec![NodeData::Boolean(a && b)])
                } else {
                    Ok(vec![NodeData::Boolean(false)])
                }
            }
            "Or" => {
                // Executing Or node
                if inputs.len() >= 2 {
                    let a = match &inputs[0] {
                        NodeData::Boolean(b) => *b,
                        _ => false,
                    };
                    let b = match &inputs[1] {
                        NodeData::Boolean(b) => *b,
                        _ => false,
                    };
                    Ok(vec![NodeData::Boolean(a || b)])
                } else {
                    Ok(vec![NodeData::Boolean(false)])
                }
            }
            "Not" => {
                // Executing Not node
                if !inputs.is_empty() {
                    let input = match &inputs[0] {
                        NodeData::Boolean(b) => *b,
                        _ => false,
                    };
                    Ok(vec![NodeData::Boolean(!input)])
                } else {
                    Ok(vec![NodeData::Boolean(true)])
                }
            }
            
            // Output nodes (simple implementations)
            "Print" => {
                // Executing Print node
                for (i, input) in inputs.iter().enumerate() {
                    println!("Print Output [{}]: {:?}", i, input);
                }
                // Print nodes typically pass through their inputs
                Ok(inputs)
            }
            "Debug" => {
                // Executing Debug node
                for (i, input) in inputs.iter().enumerate() {
                    println!("Debug Output [{}]: {:?}", i, input);
                }
                // Debug nodes typically pass through their inputs
                Ok(inputs)
            }
            "Scenegraph" => {
                // Executing Scenegraph node
                use crate::nodes::three_d::ui::scenegraph::ScenegraphLogic;
                use crate::nodes::NodeGraph;
                let dummy_graph = NodeGraph::new();
                let outputs = ScenegraphLogic::process(node, inputs.into_iter().map(Some).collect(), &dummy_graph);
                Ok(outputs.into_iter().map(|(_, data)| data).collect())
            }
            "Attributes" => {
                // Executing Attributes node
                use crate::nodes::three_d::ui::attributes::logic::process_attributes_node;
                let mut input_map = HashMap::new();
                if !inputs.is_empty() {
                    input_map.insert("USD Scene".to_string(), inputs[0].clone());
                }
                let mut dummy_cache = HashMap::new();
                let outputs = process_attributes_node(node.id, &input_map, &mut dummy_cache);
                Ok(outputs.into_iter().map(|(_, data)| data).collect())
            }
            
            // 3D Transform nodes
            "3D_Translate" => {
                // Executing Translate node
                let logic = crate::nodes::three_d::transform::translate::logic::TranslateLogic::default();
                Ok(logic.process(inputs))
            }
            "3D_Rotate" => {
                // Executing Rotate node
                // For now, just pass through - implement rotation logic later
                if !inputs.is_empty() {
                    Ok(vec![inputs[0].clone()])
                } else {
                    Ok(vec![NodeData::None])
                }
            }
            "3D_Scale" => {
                // Executing Scale node
                // For now, just pass through - implement scaling logic later
                if !inputs.is_empty() {
                    Ok(vec![inputs[0].clone()])
                } else {
                    Ok(vec![NodeData::None])
                }
            }
            
            // 3D Geometry nodes (USD-based)
            "3D_Cube" => {
                // Executing USD Cube node
                Ok(crate::nodes::three_d::geometry::cube::CubeNode::process_node(node, inputs))
            }
            "3D_Sphere" => {
                // Executing USD Sphere node
                Ok(crate::nodes::three_d::geometry::sphere::SphereNode::process_node(node, inputs))
            }
            "3D_Cylinder" => {
                // Executing USD Cylinder node
                Ok(crate::nodes::three_d::geometry::cylinder::CylinderNode::process_node(node, inputs))
            }
            "3D_Cone" => {
                // Executing USD Cone node
                Ok(crate::nodes::three_d::geometry::cone::ConeNode::process_node(node, inputs))
            }
            "3D_Plane" => {
                // Executing USD Plane node
                Ok(crate::nodes::three_d::geometry::plane::PlaneNode::process_node(node, inputs))
            }
            "3D_Capsule" => {
                // Executing USD Capsule node
                Ok(crate::nodes::three_d::geometry::capsule::CapsuleNode::process_node(node, inputs))
            }
            
            // 3D Lighting nodes
            "Point Light" => {
                // Executing Point Light node
                // For now, just pass through - implement lighting later
                Ok(vec![NodeData::None])
            }
            "Directional Light" => {
                // Executing Directional Light node
                // For now, just pass through - implement lighting later
                Ok(vec![NodeData::None])
            }
            "Spot Light" => {
                // Executing Spot Light node
                // For now, just pass through - implement lighting later
                Ok(vec![NodeData::None])
            }
            
            // 3D Modify nodes
            "3D_Reverse" => {
                // Executing Reverse node
                Ok(crate::nodes::three_d::modify::reverse::parameters::ReverseNode::process_node(node, inputs))
            }
            
            // Data nodes
            "Constant" => {
                // Executing Constant node
                // For now, just pass through - implement constant value logic later
                Ok(vec![NodeData::None])
            }
            "Variable" => {
                // Executing Variable node
                // For now, just pass through - implement variable logic later
                Ok(vec![NodeData::None])
            }
            
            // Unknown node types
            _ => {
                // Unsupported node type
                // Instead of failing, just pass through the inputs (if any) or return None
                if !inputs.is_empty() {
                    Ok(vec![inputs[0].clone()])
                } else {
                    Ok(vec![NodeData::None])
                }
            }
        }
    }

    /// Get the current state of a node
    pub fn get_node_state(&self, node_id: NodeId) -> NodeState {
        self.node_states.get(&node_id).cloned().unwrap_or(NodeState::Clean)
    }

    /// Get cached output for a node's port
    pub fn get_cached_output(&mut self, node_id: NodeId, port_idx: usize) -> Option<&NodeData> {
        let cache_key = CacheKey::new(node_id, port_idx);
        self.unified_cache.get(&cache_key)
    }
    
    /// Get cached output for a specific stage of a node's port
    pub fn get_cached_stage_output(&mut self, node_id: NodeId, stage_id: &str, port_idx: usize) -> Option<&NodeData> {
        let cache_key = CacheKey::with_stage(node_id, stage_id, port_idx);
        self.unified_cache.get(&cache_key)
    }
    
    /// Store data for a specific stage of a node
    pub fn cache_stage_output(&mut self, node_id: NodeId, stage_id: &str, port_idx: usize, data: NodeData) {
        let optimized_output = self.ownership_optimizer.optimize_output(node_id, port_idx, data);
        let cache_key = CacheKey::with_stage(node_id, stage_id, port_idx);
        self.unified_cache.insert(cache_key, optimized_output);
    }
    
    /// Get cached output using stage-qualified cache key (e.g., "0.1" for node 0 stage 1)
    pub fn get_cached_stage_output_by_key(&mut self, stage_qualified_key: &str, stage_id: &str) -> Option<&NodeData> {
        // Parse stage-qualified key like "0.1" -> node_id=0, stage=1
        if let Some((node_part, _stage_part)) = stage_qualified_key.split_once('.') {
            if let Ok(node_id) = node_part.parse::<NodeId>() {
                let cache_key = CacheKey::with_stage(node_id, stage_id, 0);
                return self.unified_cache.get(&cache_key);
            }
        }
        None
    }
    
    /// Store data using stage-qualified cache key (e.g., "0.2" for node 0 stage 2)
    pub fn cache_stage_output_by_key(&mut self, stage_qualified_key: &str, stage_id: &str, data: NodeData) {
        // Parse stage-qualified key like "0.2" -> node_id=0, stage=2
        if let Some((node_part, _stage_part)) = stage_qualified_key.split_once('.') {
            if let Ok(node_id) = node_part.parse::<NodeId>() {
                let optimized_output = self.ownership_optimizer.optimize_output(node_id, 0, data);
                let cache_key = CacheKey::with_stage(node_id, stage_id, 0);
                self.unified_cache.insert(cache_key, optimized_output);
            }
        }
    }
    
    /// Invalidate cache entries for a specific stage
    pub fn invalidate_stage_cache(&mut self, node_id: NodeId, stage_id: &str) -> usize {
        self.unified_cache.invalidate(&CacheKeyPattern::Stage(node_id, stage_id.to_string()))
    }
    
    /// Get unified cache statistics
    pub fn get_cache_statistics(&self) -> &crate::nodes::cache::CacheStatistics {
        self.unified_cache.get_statistics()
    }
    
    /// Get ownership optimization statistics
    pub fn get_ownership_statistics(&self) -> crate::nodes::ownership::OwnershipStatistics {
        self.ownership_optimizer.get_statistics()
    }

    /// Mark all nodes as dirty (force full re-evaluation)
    pub fn mark_all_dirty(&mut self, graph: &NodeGraph) {
        // Marking all nodes as dirty
        
        for &node_id in graph.nodes.keys() {
            self.node_states.insert(node_id, NodeState::Dirty);
            self.dirty_nodes.insert(node_id);
        }
        
        self.unified_cache.clear();
        self.execution_order_cache = None;
    }

    /// Handle a new connection being created
    pub fn on_connection_added(&mut self, connection: &Connection, graph: &NodeGraph) {
        println!("🔗 ExecutionEngine: Connection added {} -> {}", connection.from_node, connection.to_node);
        
        // Call node-specific connection hooks for the target node
        if let Some(target_node) = graph.nodes.get(&connection.to_node) {
            if let Some(hooks) = self.execution_hooks.get_mut(&target_node.type_id) {
                if let Err(e) = hooks.on_input_connection_added(target_node, graph) {
                    eprintln!("❌ Connection added hook failed for node {}: {}", connection.to_node, e);
                }
            }
        }
        
        // Only mark the target node (viewport) as dirty - it needs to re-render with new input
        // Don't mark upstream nodes dirty - they haven't actually changed
        self.node_states.insert(connection.to_node, NodeState::Dirty);
        self.dirty_nodes.insert(connection.to_node);
        
        // Propagate dirty downstream from the target node (in case other nodes depend on it)
        self.propagate_dirty_downstream(connection.to_node, graph);
        
        // Execute immediately if in auto mode
        if self.execution_mode == EngineExecutionMode::Auto {
            if let Err(e) = self.execute_dirty_nodes(graph) {
                eprintln!("Auto execution after connection added failed: {}", e);
            }
        }
    }

    /// Handle a connection being removed
    pub fn on_connection_removed(&mut self, connection: &Connection, graph: &NodeGraph) {
        println!("🔗 ExecutionEngine: Connection removed {} -> {}", connection.from_node, connection.to_node);
        
        // Call node-specific connection hooks for the target node
        if let Some(target_node) = graph.nodes.get(&connection.to_node) {
            if let Some(hooks) = self.execution_hooks.get_mut(&target_node.type_id) {
                if let Err(e) = hooks.on_input_connection_removed(target_node, graph) {
                    eprintln!("❌ Connection removed hook failed for node {}: {}", connection.to_node, e);
                }
            }
        }
        
        // Mark the target node as dirty (this will also propagate downstream)
        self.mark_dirty(connection.to_node, graph);
        
        // Mark upstream dependency chain as dirty to ensure fresh execution
        self.propagate_dirty_upstream(connection.to_node, graph);
        
        // CRITICAL: Clear cached outputs AND mark source node as dirty
        // When we clear a node's cache, it needs to be re-executed when reconnected
        self.unified_cache.invalidate(&CacheKeyPattern::Node(connection.from_node));
        
        // Mark source node dirty with proper cache handling
        if let Some(source_node) = graph.nodes.get(&connection.from_node) {
            if source_node.type_id == "Data_UsdFileReader" {
                // USD File Reader handles its own cache - use special method
                self.mark_dirty_without_cache_invalidation(connection.from_node, graph);
            } else {
                // Standard nodes get standard dirty marking
                self.mark_dirty(connection.from_node, graph);
            }
        }
        
        // Execute immediately if in auto mode
        if self.execution_mode == EngineExecutionMode::Auto {
            if let Err(e) = self.execute_dirty_nodes(graph) {
                eprintln!("Auto execution after connection removed failed: {}", e);
            }
        }
    }
    
    // NOTE: Old cache clearing methods removed - now handled by node hooks
    
    /* REMOVED - Now handled by ViewportHooks
    /// Clear viewport-specific caches for a node
    pub fn clear_viewport_caches(&mut self, node_id: NodeId) {
        use crate::nodes::three_d::ui::viewport::{VIEWPORT_INPUT_CACHE, VIEWPORT_DATA_CACHE, FORCE_VIEWPORT_REFRESH};
        
        // Clear viewport input cache
        if let Ok(mut cache) = VIEWPORT_INPUT_CACHE.lock() {
            cache.remove(&node_id);
        }
        
        // Clear viewport data cache
        if let Ok(mut cache) = VIEWPORT_DATA_CACHE.lock() {
            cache.remove(&node_id);
        }
        
        // Clear force refresh cache
        if let Ok(mut force_set) = FORCE_VIEWPORT_REFRESH.lock() {
            force_set.remove(&node_id);
        }
        
        // Note: USD renderer cache uses file paths as keys, not node IDs
        // We avoid clearing ALL entries since that would affect unrelated viewport nodes
        // Instead, we let the viewport node handle its own USD renderer cache clearing
        // when it detects input changes through the FORCE_VIEWPORT_REFRESH mechanism
        
        // CRITICAL: Clear GPU mesh caches when viewport connections change
        crate::gpu::viewport_3d_callback::clear_all_gpu_mesh_caches();
        
        // Clear USD renderer cache - this is necessary when switching between geometry nodes
        use crate::nodes::three_d::ui::viewport::USD_RENDERER_CACHE;
        if let Ok(mut cache) = USD_RENDERER_CACHE.lock() {
            let renderer_count = cache.renderers.len();
            let bounds_count = cache.scene_bounds.len();
            cache.renderers.clear();
            cache.scene_bounds.clear();
        }
        
        // Clear execution engine output cache for the node
        self.unified_cache.invalidate(&CacheKeyPattern::Node(node_id));
    }
    
    REMOVED - Now handled by AttributesHooks
    /// Clear attributes-specific caches for a node
    pub fn clear_attributes_caches(&mut self, node_id: NodeId) {
        use crate::nodes::three_d::ui::attributes::logic::ATTRIBUTES_INPUT_CACHE;
        
        // Clear attributes input cache
        if let Ok(mut cache) = ATTRIBUTES_INPUT_CACHE.write() {
            cache.remove(&node_id);
        }
        
        // Clear execution engine output cache for the node
        self.unified_cache.invalidate(&CacheKeyPattern::Node(node_id));
    }
    
    REMOVED - Now handled by GeometryHooks and UsdFileReaderHooks 
    /// Clear GPU mesh cache when USD parameters change - only for connected viewport nodes
    fn clear_gpu_mesh_cache_for_usd_changes(&mut self, usd_node_id: NodeId, graph: &NodeGraph) {
        // ... old implementation
    }
    */
    
    /// Handle node removal by clearing all related caches and marking affected nodes as dirty
    pub fn on_node_removed(&mut self, node_id: NodeId, graph: &NodeGraph) {
        // Call node-specific removal hook
        if let Some(node) = graph.nodes.get(&node_id) {
            if let Some(hooks) = self.execution_hooks.get_mut(&node.type_id) {
                if let Err(e) = hooks.on_node_removed(node_id) {
                    eprintln!("Node removal hook failed for {}: {}", node.type_id, e);
                }
            }
        }
        
        // Clear output cache for the removed node
        self.unified_cache.invalidate(&CacheKeyPattern::Node(node_id));
        
        // Find all nodes that were connected to the deleted node
        let mut affected_nodes = Vec::new();
        for connection in &graph.connections {
            if connection.from_node == node_id {
                // The deleted node was providing input to this node
                affected_nodes.push(connection.to_node);
            }
        }
        
        // Mark all affected nodes as dirty
        for affected_node_id in affected_nodes {
            self.mark_dirty(affected_node_id, graph);
        }
    }

    /// Handle a node parameter change
    pub fn on_node_parameter_changed(&mut self, node_id: NodeId, graph: &NodeGraph) {
        // Special handling for USD File Reader nodes - they manage their own cache invalidation
        if let Some(node) = graph.nodes.get(&node_id) {
            if node.type_id == "Data_UsdFileReader" {
                // USD File Reader handles its own granular cache invalidation
                // Use the special method that doesn't invalidate cache
                return self.on_usd_file_reader_parameter_changed(node_id, graph);
            }
        }
        println!("🔧 ExecutionEngine: Parameter changed for node {} in {} mode", node_id, 
                 if self.execution_mode == EngineExecutionMode::Auto { "Auto" } else { "Manual" });
        
        // Check if this node handles its own cache invalidation
        if self.node_handles_own_cache_invalidation(node_id, graph) {
            // Node handles its own cache invalidation - just mark dirty without clearing cache
            println!("🔧 ExecutionEngine: Node {} handles own cache invalidation", node_id);
            self.mark_dirty_without_cache_invalidation(node_id, graph);
        } else {
            // Standard cache invalidation for nodes that don't manage their own caches
            self.mark_dirty(node_id, graph);
        }
        
        // Execute immediately if in auto mode
        if self.execution_mode == EngineExecutionMode::Auto {
            println!("🔧 ExecutionEngine: Executing immediately due to parameter change");
            if let Err(e) = self.execute_dirty_nodes(graph) {
                eprintln!("Auto execution after parameter change failed: {}", e);
            }
        } else {
            println!("🔧 ExecutionEngine: Manual mode - waiting for Cook button");
        }
    }
    
    /// Set the execution mode
    pub fn set_execution_mode(&mut self, mode: EngineExecutionMode) {
        self.execution_mode = mode;
    }
    
    /// Get the current execution mode
    pub fn get_execution_mode(&self) -> EngineExecutionMode {
        self.execution_mode
    }

    /* REMOVED - Now handled by node hooks
    /// Clear GPU mesh cache when USD parameters change - only for connected viewport nodes
    fn clear_gpu_mesh_cache_for_usd_changes(&mut self, usd_node_id: NodeId, graph: &NodeGraph) {
        // CRITICAL: Clear the USD node's output cache so it regenerates data
        self.unified_cache.invalidate(&CacheKeyPattern::Node(usd_node_id));
        
        // Find all downstream viewport nodes that are actually connected
        let downstream_nodes = self.find_downstream_nodes(usd_node_id, graph);
        let mut connected_viewport_nodes = Vec::new();
        
        for downstream_id in &downstream_nodes {
            if let Some(node) = graph.nodes.get(downstream_id) {
                if node.type_id == "Viewport" || node.type_id == "3D_Viewport" {
                    connected_viewport_nodes.push(*downstream_id);
                }
            }
        }
        
        // Only clear caches for viewport nodes that are actually connected to this USD File Reader
        if !connected_viewport_nodes.is_empty() {
            
            use crate::nodes::three_d::ui::viewport::{VIEWPORT_INPUT_CACHE, VIEWPORT_DATA_CACHE, USD_RENDERER_CACHE, EXECUTION_VIEWPORT_CACHE};
            
            // Clear viewport input cache only for connected nodes
            if let Ok(mut cache) = VIEWPORT_INPUT_CACHE.lock() {
                for viewport_node_id in &connected_viewport_nodes {
                    cache.remove(viewport_node_id);
                }
            }
            
            // Clear viewport data cache only for connected nodes
            if let Ok(mut cache) = VIEWPORT_DATA_CACHE.lock() {
                for viewport_node_id in &connected_viewport_nodes {
                    cache.remove(viewport_node_id);
                }
            }
            
            // Clear execution cache only for connected nodes
            if let Ok(mut cache) = EXECUTION_VIEWPORT_CACHE.lock() {
                for viewport_node_id in &connected_viewport_nodes {
                    cache.remove(viewport_node_id);
                }
            }
            
            // CRITICAL: Clear GPU mesh caches to force re-uploading with new data
            // This ensures parameter changes are visible in the viewport
            crate::gpu::viewport_3d_callback::clear_all_gpu_mesh_caches();
            
            // Clear USD renderer cache - this is keyed by file path, so we need to clear all
            // since we don't know which specific file path was affected
            if let Ok(mut cache) = USD_RENDERER_CACHE.lock() {
                let renderer_count = cache.renderers.len();
                let bounds_count = cache.scene_bounds.len();
                cache.renderers.clear();
                cache.scene_bounds.clear();
            }
            
        } else {
        }
        
        // CRITICAL: Force only the connected viewport nodes to refresh immediately
        for viewport_node_id in &connected_viewport_nodes {
            use crate::nodes::three_d::ui::viewport::FORCE_VIEWPORT_REFRESH;
            if let Ok(mut force_set) = FORCE_VIEWPORT_REFRESH.lock() {
                force_set.insert(*viewport_node_id);
            }
        }
        
        // CRITICAL: Mark all downstream nodes as dirty so they re-execute with fresh USD data
        // This ensures viewport nodes get the updated coordinate conversion immediately
        self.propagate_dirty_downstream(usd_node_id, graph);
    }
    */

    /// Get execution statistics
    pub fn get_stats(&self) -> ExecutionStats {
        let mut clean_count = 0;
        let mut dirty_count = 0;
        let mut computing_count = 0;
        let mut error_count = 0;

        for state in self.node_states.values() {
            match state {
                NodeState::Clean => clean_count += 1,
                NodeState::Dirty => dirty_count += 1,
                NodeState::Computing => computing_count += 1,
                NodeState::Error => error_count += 1,
            }
        }

        ExecutionStats {
            total_nodes: self.node_states.len(),
            clean_nodes: clean_count,
            dirty_nodes: dirty_count,
            computing_nodes: computing_count,
            error_nodes: error_count,
            cached_outputs: self.unified_cache.get_statistics().total_entries,
        }
    }
    
    /// GLOBAL FILE CACHE: Store USD file data persistently (survives cache invalidations)
    /// This provides a global catch-all to prevent unnecessary file reloads
    pub fn store_persistent_usd_file_data(&mut self, hash_key: &str, data: crate::workspaces::three_d::usd::usd_engine::USDSceneData) {
        self.persistent_usd_file_data.insert(hash_key.to_string(), data);
        println!("🌍 STORED persistent USD data for hash: {}", hash_key);
    }
    
    /// GLOBAL FILE CACHE: Retrieve USD file data if it exists and file hasn't changed
    /// This provides a global catch-all to prevent unnecessary file reloads
    pub fn get_persistent_usd_file_data(&self, hash_key: &str) -> Option<crate::workspaces::three_d::usd::usd_engine::USDSceneData> {
        if let Some(data) = self.persistent_usd_file_data.get(hash_key) {
            println!("🌍 FOUND persistent USD data for hash: {}", hash_key);
            Some(data.clone())
        } else {
            println!("🌍 NO persistent USD data for hash: {}", hash_key);
            None
        }
    }
    
    /// GLOBAL FILE CACHE: Clear persistent USD file data (for cleanup)
    pub fn clear_persistent_usd_file_data(&mut self) {
        let count = self.persistent_usd_file_data.len();
        self.persistent_usd_file_data.clear();
        println!("🌍 CLEARED {} persistent USD file data entries", count);
    }
}

/// Statistics about the execution engine state
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    pub total_nodes: usize,
    pub clean_nodes: usize,
    pub dirty_nodes: usize,
    pub computing_nodes: usize,
    pub error_nodes: usize,
    pub cached_outputs: usize,
}

impl Default for NodeGraphEngine {
    fn default() -> Self {
        Self::new()
    }
}