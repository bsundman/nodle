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

        // Marking node as dirty
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
                // Propagating dirty to downstream node
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
                
                // Clear cached outputs
                self.output_cache.retain(|(id, _), _| *id != upstream_id);
                
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
    pub fn execute_dirty_nodes(&mut self, graph: &NodeGraph) -> Result<(), String> {
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
        
        // Collect inputs from upstream nodes
        let inputs = self.collect_node_inputs(node_id, graph);
        
        // Execute the node using the factory dispatch system
        let outputs = match self.dispatch_node_execution(node, inputs) {
            Ok(outputs) => outputs,
            Err(e) => {
                // Node execution failed
                self.node_states.insert(node_id, NodeState::Error);
                return Err(e);
            }
        };
        
        // Cache the outputs
        // Caching outputs
        for (port_idx, output) in outputs.into_iter().enumerate() {
            self.output_cache.insert((node_id, port_idx), output);
        }
        
        // Mark as clean
        self.node_states.insert(node_id, NodeState::Clean);
        self.dirty_nodes.remove(&node_id);
        
        // Node executed successfully
        Ok(())
    }

    /// Collect inputs for a node from connected upstream nodes
    fn collect_node_inputs(&self, node_id: NodeId, graph: &NodeGraph) -> Vec<NodeData> {
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
                
                // Get the output from the source node
                if let Some(output_data) = self.output_cache.get(&(connection.from_node, connection.from_port)) {
                    if connection.to_port < inputs.len() {
                        inputs[connection.to_port] = output_data.clone();
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
    pub fn get_cached_output(&self, node_id: NodeId, port_idx: usize) -> Option<&NodeData> {
        self.output_cache.get(&(node_id, port_idx))
    }

    /// Mark all nodes as dirty (force full re-evaluation)
    pub fn mark_all_dirty(&mut self, graph: &NodeGraph) {
        // Marking all nodes as dirty
        
        for &node_id in graph.nodes.keys() {
            self.node_states.insert(node_id, NodeState::Dirty);
            self.dirty_nodes.insert(node_id);
        }
        
        self.output_cache.clear();
        self.execution_order_cache = None;
    }

    /// Handle a new connection being created
    pub fn on_connection_added(&mut self, connection: &Connection, graph: &NodeGraph) {
        let source_title = graph.nodes.get(&connection.from_node).map(|n| n.title.as_str()).unwrap_or("Unknown");
        let target_title = graph.nodes.get(&connection.to_node).map(|n| n.title.as_str()).unwrap_or("Unknown");
        
        // Mark the target node as dirty (this will also propagate downstream)
        self.mark_dirty(connection.to_node, graph);
        
        // CRITICAL: Mark the entire upstream dependency chain as dirty
        // This ensures USD File Reader → Reverse → Viewport all get executed in correct order
        self.propagate_dirty_upstream(connection.to_node, graph);
        
        // Also mark the source node to ensure it gets executed if it wasn't already
        self.mark_dirty(connection.from_node, graph);
        
        // Clear viewport-specific caches if the target is a viewport node
        if let Some(node) = graph.nodes.get(&connection.to_node) {
            if node.type_id == "Viewport" {
                self.clear_viewport_caches(connection.to_node);
                
                // CRITICAL: Set force refresh flag for viewport
                use crate::nodes::three_d::ui::viewport::FORCE_VIEWPORT_REFRESH;
                if let Ok(mut force_set) = FORCE_VIEWPORT_REFRESH.lock() {
                    force_set.insert(connection.to_node);
                }
            } else if node.type_id == "Attributes" {
                // Clear attributes cache when connections change
                self.clear_attributes_caches(connection.to_node);
            }
        }
        
        // CRITICAL: Execute dirty nodes immediately to prevent lag
        // This ensures connection changes are reflected immediately
        match self.execute_dirty_nodes(graph) {
            Ok(_) => {
                // Immediate execution successful
            }
            Err(e) => {
                eprintln!("Connection addition execution failed: {}", e);
            }
        }
    }

    /// Handle a connection being removed
    pub fn on_connection_removed(&mut self, connection: &Connection, graph: &NodeGraph) {
        let source_title = graph.nodes.get(&connection.from_node).map(|n| n.title.as_str()).unwrap_or("Unknown");
        let target_title = graph.nodes.get(&connection.to_node).map(|n| n.title.as_str()).unwrap_or("Unknown");
        
        // Mark the target node as dirty (this will also propagate downstream)
        self.mark_dirty(connection.to_node, graph);
        
        // Mark upstream dependency chain as dirty to ensure fresh execution
        self.propagate_dirty_upstream(connection.to_node, graph);
        
        // Clear cached outputs from the source node to prevent stale data
        self.output_cache.retain(|(node_id, _), _| *node_id != connection.from_node);
        
        // Clear viewport-specific caches if the target is a viewport node
        if let Some(node) = graph.nodes.get(&connection.to_node) {
            if node.type_id == "Viewport" {
                self.clear_viewport_caches(connection.to_node);
                
                // CRITICAL: Set force refresh flag for viewport
                use crate::nodes::three_d::ui::viewport::FORCE_VIEWPORT_REFRESH;
                if let Ok(mut force_set) = FORCE_VIEWPORT_REFRESH.lock() {
                    force_set.insert(connection.to_node);
                }
            } else if node.type_id == "Attributes" {
                // Clear attributes cache when connections change
                self.clear_attributes_caches(connection.to_node);
            }
        }
        
        // CRITICAL: Execute dirty nodes immediately to prevent lag
        // This ensures connection changes are reflected immediately
        match self.execute_dirty_nodes(graph) {
            Ok(_) => {
                // Immediate execution successful
            }
            Err(e) => {
                eprintln!("Connection removal execution failed: {}", e);
            }
        }
    }
    
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
        self.output_cache.retain(|(id, _), _| *id != node_id);
    }
    
    /// Clear attributes-specific caches for a node
    pub fn clear_attributes_caches(&mut self, node_id: NodeId) {
        use crate::nodes::three_d::ui::attributes::logic::ATTRIBUTES_INPUT_CACHE;
        
        // Clear attributes input cache
        if let Ok(mut cache) = ATTRIBUTES_INPUT_CACHE.write() {
            cache.remove(&node_id);
        }
        
        // Clear execution engine output cache for the node
        self.output_cache.retain(|(id, _), _| *id != node_id);
    }
    
    /// Handle node removal by clearing all related caches and marking affected nodes as dirty
    pub fn on_node_removed(&mut self, node_id: NodeId, graph: &NodeGraph) {
        // Clear all caches for the deleted node
        self.clear_viewport_caches(node_id);
        self.clear_attributes_caches(node_id);
        
        // Clear USD file reader cache if applicable
        if let Some(node) = graph.nodes.get(&node_id) {
            if node.type_id == "Data_UsdFileReader" {
                crate::nodes::data::usd_file_reader::UsdFileReaderNode::clear_cache(node_id);
            }
        }
        
        // Find all nodes that were connected to the deleted node
        let mut affected_nodes = Vec::new();
        for connection in &graph.connections {
            if connection.from_node == node_id {
                // The deleted node was providing input to this node
                affected_nodes.push(connection.to_node);
            }
        }
        
        // Mark all affected nodes as dirty and clear their caches
        for affected_node_id in affected_nodes {
            self.mark_dirty(affected_node_id, graph);
            
            // If the affected node is a viewport node, clear its caches too
            if let Some(node) = graph.nodes.get(&affected_node_id) {
                if node.type_id == "Viewport" {
                    self.clear_viewport_caches(affected_node_id);
                    
                    // Force viewport refresh by adding to force refresh set
                    use crate::nodes::three_d::ui::viewport::FORCE_VIEWPORT_REFRESH;
                    if let Ok(mut force_set) = FORCE_VIEWPORT_REFRESH.lock() {
                        force_set.insert(affected_node_id);
                    }
                } else if node.type_id == "Attributes" {
                    // Clear attributes cache for affected attributes nodes
                    self.clear_attributes_caches(affected_node_id);
                }
            }
        }
        
    }

    /// Handle a node parameter change
    pub fn on_node_parameter_changed(&mut self, node_id: NodeId, graph: &NodeGraph) {
        // Parameter changed
        
        // Show node title for better debugging
        if let Some(node) = graph.nodes.get(&node_id) {
            // Node parameters changed
            
            // If this is a USD source node (File Reader or geometry node), clear GPU mesh cache to force re-upload
            let is_usd_source_node = match node.type_id.as_str() {
                "Data_UsdFileReader" => true,
                "3D_Cube" | "3D_Sphere" | "3D_Cylinder" | "3D_Cone" | "3D_Plane" | "3D_Capsule" => true,
                _ => node.type_id.contains("USD") || node.type_id.contains("3D_"),
            };
            
            if is_usd_source_node {
                self.clear_gpu_mesh_cache_for_usd_changes(node_id, graph);
            }
        }
        
        // Mark the node as dirty
        self.mark_dirty(node_id, graph);
        
        // Node marked as dirty
    }

    /// Clear GPU mesh cache when USD parameters change - only for connected viewport nodes
    fn clear_gpu_mesh_cache_for_usd_changes(&mut self, usd_node_id: NodeId, graph: &NodeGraph) {
        // CRITICAL: Clear the USD node's output cache so it regenerates data
        self.output_cache.retain(|(node_id, _), _| *node_id != usd_node_id);
        
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