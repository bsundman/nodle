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

/// Represents the execution state of a node
#[derive(Debug, Clone, PartialEq)]
pub enum NodeState {
    Clean,      // Node is up-to-date
    Dirty,      // Node needs re-evaluation
    Computing,  // Node is currently being processed
    Error,      // Node failed to execute
}

/// Execution engine for node graphs
pub struct NodeGraphEngine {
    /// Current execution state for each node
    node_states: HashMap<NodeId, NodeState>,
    /// Cached output values for each node's output ports
    output_cache: HashMap<(NodeId, usize), NodeData>,
    /// Set of nodes that need re-evaluation
    dirty_nodes: HashSet<NodeId>,
    /// Execution order cache (invalidated when graph changes)
    execution_order_cache: Option<Vec<NodeId>>,
}

impl NodeGraphEngine {
    /// Create a new execution engine
    pub fn new() -> Self {
        Self {
            node_states: HashMap::new(),
            output_cache: HashMap::new(),
            dirty_nodes: HashSet::new(),
            execution_order_cache: None,
        }
    }

    /// Mark a node as dirty (needs re-evaluation)
    pub fn mark_dirty(&mut self, node_id: NodeId, graph: &NodeGraph) {
        if self.node_states.get(&node_id) == Some(&NodeState::Dirty) {
            return; // Already dirty
        }

        println!("🟡 Marking node {} as dirty", node_id);
        self.node_states.insert(node_id, NodeState::Dirty);
        self.dirty_nodes.insert(node_id);
        
        // Invalidate cached outputs for this node
        self.output_cache.retain(|(id, _), _| *id != node_id);
        
        // Propagate dirty state to downstream nodes
        self.propagate_dirty_downstream(node_id, graph);
        
        // Invalidate execution order cache
        self.execution_order_cache = None;
    }

    /// Propagate dirty state to all downstream nodes
    fn propagate_dirty_downstream(&mut self, node_id: NodeId, graph: &NodeGraph) {
        let downstream_nodes = self.find_downstream_nodes(node_id, graph);
        
        for downstream_id in downstream_nodes {
            if self.node_states.get(&downstream_id) != Some(&NodeState::Dirty) {
                println!("🟡 Propagating dirty to downstream node {}", downstream_id);
                self.node_states.insert(downstream_id, NodeState::Dirty);
                self.dirty_nodes.insert(downstream_id);
                
                // Clear cached outputs
                self.output_cache.retain(|(id, _), _| *id != downstream_id);
                
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

    /// Get the execution order using topological sort
    pub fn get_execution_order(&mut self, graph: &NodeGraph) -> Result<Vec<NodeId>, String> {
        // Use cached order if available and graph hasn't changed
        if let Some(ref order) = self.execution_order_cache {
            return Ok(order.clone());
        }

        println!("🔍 Computing execution order via topological sort");
        
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
        
        println!("🔍 Execution order: {:?}", result);
        
        // Cache the result
        self.execution_order_cache = Some(result.clone());
        Ok(result)
    }

    /// Execute all dirty nodes in dependency order
    pub fn execute_dirty_nodes(&mut self, graph: &NodeGraph) -> Result<(), String> {
        // Debug: Show all node states
        println!("🔍 ENGINE: Node states before execution:");
        for (&node_id, state) in &self.node_states {
            println!("  Node {}: {:?}", node_id, state);
        }
        println!("🔍 ENGINE: Dirty nodes set: {:?}", self.dirty_nodes);
        
        if self.dirty_nodes.is_empty() {
            println!("✅ No dirty nodes to execute");
            
            // Check if we have any new nodes that need initial execution
            for &node_id in graph.nodes.keys() {
                if !self.node_states.contains_key(&node_id) {
                    println!("🟡 Found new node {} - marking as dirty for initial execution", node_id);
                    self.mark_dirty(node_id, graph);
                }
            }
            
            // If we found new nodes, try execution again
            if !self.dirty_nodes.is_empty() {
                println!("🔄 Executing {} newly discovered dirty nodes", self.dirty_nodes.len());
            } else {
                return Ok(());
            }
        } else {
            println!("🔄 Executing {} dirty nodes", self.dirty_nodes.len());
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
        println!("✅ All dirty nodes executed successfully");
        
        Ok(())
    }

    /// Execute a single node
    fn execute_single_node(&mut self, node_id: NodeId, graph: &NodeGraph) -> Result<(), String> {
        let node = graph.nodes.get(&node_id)
            .ok_or_else(|| format!("Node {} not found", node_id))?;

        println!("🔄 Executing node {} ({})", node_id, node.title);
        
        // Mark as computing
        self.node_states.insert(node_id, NodeState::Computing);
        
        // Collect inputs from upstream nodes
        let inputs = self.collect_node_inputs(node_id, graph);
        
        // Execute the node using the factory dispatch system
        let outputs = match self.dispatch_node_execution(node, inputs) {
            Ok(outputs) => outputs,
            Err(e) => {
                println!("❌ Node {} execution failed: {}", node_id, e);
                self.node_states.insert(node_id, NodeState::Error);
                return Err(e);
            }
        };
        
        // Cache the outputs
        println!("🔄 Caching {} outputs for node {}", outputs.len(), node_id);
        for (port_idx, output) in outputs.into_iter().enumerate() {
            println!("  Output port {}: {:?}", port_idx, output);
            self.output_cache.insert((node_id, port_idx), output);
        }
        
        // Mark as clean
        self.node_states.insert(node_id, NodeState::Clean);
        self.dirty_nodes.remove(&node_id);
        
        println!("✅ Node {} executed successfully", node_id);
        Ok(())
    }

    /// Collect inputs for a node from connected upstream nodes
    fn collect_node_inputs(&self, node_id: NodeId, graph: &NodeGraph) -> Vec<NodeData> {
        let node = match graph.nodes.get(&node_id) {
            Some(node) => node,
            None => return vec![],
        };

        let mut inputs = vec![NodeData::None; node.inputs.len()];
        
        println!("🔍 Collecting inputs for node {} ({}):", node_id, node.title);
        println!("  Node has {} input ports", node.inputs.len());
        
        // Find all connections feeding into this node
        let mut found_connections = 0;
        for connection in &graph.connections {
            if connection.to_node == node_id {
                found_connections += 1;
                println!("  Found connection from node {} port {} to port {}", 
                    connection.from_node, connection.from_port, connection.to_port);
                
                // Get the output from the source node
                if let Some(output_data) = self.output_cache.get(&(connection.from_node, connection.from_port)) {
                    if connection.to_port < inputs.len() {
                        inputs[connection.to_port] = output_data.clone();
                        println!("    ✅ Found cached data: {:?}", output_data);
                    } else {
                        println!("    ❌ Target port {} out of range (max {})", connection.to_port, inputs.len());
                    }
                } else {
                    println!("    ⚠️ No cached output found for source node {} port {}", 
                        connection.from_node, connection.from_port);
                }
            }
        }
        
        println!("  Total connections found: {}", found_connections);
        println!("  Final inputs: {:?}", inputs);
        inputs
    }

    /// Dispatch node execution based on node title
    fn dispatch_node_execution(&self, node: &Node, inputs: Vec<NodeData>) -> Result<Vec<NodeData>, String> {
        // Use the node title to dispatch execution (this matches the current system)
        match node.title.as_str() {
            // Data nodes
            "USD File Reader" => {
                println!("📁 Executing USD File Reader node");
                Ok(crate::nodes::data::usd_file_reader::UsdFileReaderNode::process_node(node))
            }
            
            // Viewport/UI nodes
            "Viewport" => {
                println!("🖼️ Executing Viewport node with {} inputs", inputs.len());
                Ok(crate::nodes::three_d::ui::viewport::ViewportNode::process_node(node, &inputs))
            }
            
            // Math nodes
            "Add" => {
                println!("➕ Executing Add node with {} inputs", inputs.len());
                Ok(crate::nodes::math::add::functions::process_add(inputs))
            }
            "Subtract" => {
                println!("➖ Executing Subtract node with {} inputs", inputs.len());
                Ok(crate::nodes::math::subtract::functions::process_subtract(inputs))
            }
            "Multiply" => {
                println!("✖️ Executing Multiply node with {} inputs", inputs.len());
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
                println!("➗ Executing Divide node with {} inputs", inputs.len());
                Ok(crate::nodes::math::divide::functions::process_divide(inputs))
            }
            
            // Logic nodes (simple implementations since functions modules don't exist)
            "And" => {
                println!("🔗 Executing And node with {} inputs", inputs.len());
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
                println!("🔀 Executing Or node with {} inputs", inputs.len());
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
                println!("🚫 Executing Not node with {} inputs", inputs.len());
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
                println!("🖨️ Executing Print node with {} inputs", inputs.len());
                for (i, input) in inputs.iter().enumerate() {
                    println!("Print Output [{}]: {:?}", i, input);
                }
                // Print nodes typically pass through their inputs
                Ok(inputs)
            }
            "Debug" => {
                println!("🐛 Executing Debug node with {} inputs", inputs.len());
                for (i, input) in inputs.iter().enumerate() {
                    println!("Debug Output [{}]: {:?}", i, input);
                }
                // Debug nodes typically pass through their inputs
                Ok(inputs)
            }
            
            // 3D Transform nodes
            "Translate" | "3D_Translate" => {
                println!("↔️ Executing Translate node with {} inputs", inputs.len());
                let logic = crate::nodes::three_d::transform::translate::logic::TranslateLogic::default();
                Ok(logic.process(inputs))
            }
            "Rotate" | "3D_Rotate" => {
                println!("🔄 Executing Rotate node with {} inputs", inputs.len());
                // For now, just pass through - implement rotation logic later
                if !inputs.is_empty() {
                    Ok(vec![inputs[0].clone()])
                } else {
                    Ok(vec![NodeData::None])
                }
            }
            "Scale" | "3D_Scale" => {
                println!("📐 Executing Scale node with {} inputs", inputs.len());
                // For now, just pass through - implement scaling logic later
                if !inputs.is_empty() {
                    Ok(vec![inputs[0].clone()])
                } else {
                    Ok(vec![NodeData::None])
                }
            }
            
            // 3D Geometry nodes
            "Cube" => {
                println!("🧊 Executing Cube node with {} inputs", inputs.len());
                // For now, just pass through - implement cube generation later
                Ok(vec![NodeData::None])
            }
            "Sphere" => {
                println!("🔮 Executing Sphere node with {} inputs", inputs.len());
                // For now, just pass through - implement sphere generation later
                Ok(vec![NodeData::None])
            }
            "Plane" => {
                println!("🟫 Executing Plane node with {} inputs", inputs.len());
                // For now, just pass through - implement plane generation later
                Ok(vec![NodeData::None])
            }
            
            // 3D Lighting nodes
            "Point Light" => {
                println!("💡 Executing Point Light node with {} inputs", inputs.len());
                // For now, just pass through - implement lighting later
                Ok(vec![NodeData::None])
            }
            "Directional Light" => {
                println!("🔦 Executing Directional Light node with {} inputs", inputs.len());
                // For now, just pass through - implement lighting later
                Ok(vec![NodeData::None])
            }
            "Spot Light" => {
                println!("🎯 Executing Spot Light node with {} inputs", inputs.len());
                // For now, just pass through - implement lighting later
                Ok(vec![NodeData::None])
            }
            
            // Data nodes
            "Constant" => {
                println!("🔢 Executing Constant node with {} inputs", inputs.len());
                // For now, just pass through - implement constant value logic later
                Ok(vec![NodeData::None])
            }
            "Variable" => {
                println!("📊 Executing Variable node with {} inputs", inputs.len());
                // For now, just pass through - implement variable logic later
                Ok(vec![NodeData::None])
            }
            
            // Unknown node types
            _ => {
                println!("⚠️ Unsupported node type for execution: {}", node.title);
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
    pub fn get_cached_output(&self, node_id: NodeId, port_idx: usize) -> Option<&NodeData> {
        self.output_cache.get(&(node_id, port_idx))
    }

    /// Mark all nodes as dirty (force full re-evaluation)
    pub fn mark_all_dirty(&mut self, graph: &NodeGraph) {
        println!("🟡 Marking all nodes as dirty");
        
        for &node_id in graph.nodes.keys() {
            self.node_states.insert(node_id, NodeState::Dirty);
            self.dirty_nodes.insert(node_id);
        }
        
        self.output_cache.clear();
        self.execution_order_cache = None;
    }

    /// Handle a new connection being created
    pub fn on_connection_added(&mut self, connection: &Connection, graph: &NodeGraph) {
        println!("🔗 ENGINE: Connection added: Node {} port {} → Node {} port {}", 
            connection.from_node, connection.from_port, connection.to_node, connection.to_port);
        println!("🔗 ENGINE: Graph now has {} total connections", graph.connections.len());
        
        // Mark BOTH source and target nodes as dirty to ensure data flow
        println!("🟡 Marking source node {} as dirty (connection created)", connection.from_node);
        self.mark_dirty(connection.from_node, graph);
        
        println!("🟡 Marking target node {} as dirty (connection created)", connection.to_node);
        self.mark_dirty(connection.to_node, graph);
        
        // Debug: List all connections in the graph
        for (i, conn) in graph.connections.iter().enumerate() {
            println!("🔗 ENGINE: Connection {}: {} port {} → {} port {}", 
                i, conn.from_node, conn.from_port, conn.to_node, conn.to_port);
        }
    }

    /// Handle a connection being removed
    pub fn on_connection_removed(&mut self, connection: &Connection, graph: &NodeGraph) {
        println!("🔗 Connection removed: {} -> {}", connection.from_node, connection.to_node);
        
        // Mark the target node as dirty
        self.mark_dirty(connection.to_node, graph);
    }

    /// Handle a node parameter change
    pub fn on_node_parameter_changed(&mut self, node_id: NodeId, graph: &NodeGraph) {
        println!("🔧 Parameter changed on node {}", node_id);
        
        // Mark the node as dirty
        self.mark_dirty(node_id, graph);
    }

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
            cached_outputs: self.output_cache.len(),
        }
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