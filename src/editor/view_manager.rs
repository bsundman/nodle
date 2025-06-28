//! View management system for the node editor
//!
//! Handles switching between different graph views (root vs workspace nodes)
//! and managing the current viewing context.

use std::collections::HashMap;
use crate::nodes::{NodeGraph, Node, NodeId, Connection};

/// Tracks which graph we're currently viewing
#[derive(Debug, Clone)]
pub enum GraphView {
    /// Viewing the root graph
    Root,
    /// Viewing a workspace node's internal graph
    WorkspaceNode(NodeId),
}

/// Manages view state and switching between different graph contexts
pub struct ViewManager {
    /// Current view state - which graph we're looking at
    current_view: GraphView,
}

impl ViewManager {
    /// Create a new view manager starting with root view
    pub fn new() -> Self {
        Self {
            current_view: GraphView::Root,
        }
    }

    /// Get the current view
    pub fn current_view(&self) -> &GraphView {
        &self.current_view
    }

    /// Set the current view to root
    pub fn set_root_view(&mut self) {
        self.current_view = GraphView::Root;
    }

    /// Set the current view to a workspace node
    pub fn set_workspace_view(&mut self, node_id: NodeId) {
        self.current_view = GraphView::WorkspaceNode(node_id);
    }

    /// Check if currently viewing root graph
    pub fn is_root_view(&self) -> bool {
        matches!(self.current_view, GraphView::Root)
    }

    /// Check if currently viewing a workspace node
    pub fn is_workspace_view(&self) -> bool {
        matches!(self.current_view, GraphView::WorkspaceNode(_))
    }

    /// Get the workspace node ID if currently viewing a workspace
    pub fn get_workspace_node_id(&self) -> Option<NodeId> {
        match self.current_view {
            GraphView::WorkspaceNode(node_id) => Some(node_id),
            GraphView::Root => None,
        }
    }

    /// Get the nodes that should be visible in the current view
    pub fn get_viewed_nodes(&self, graph: &NodeGraph) -> HashMap<NodeId, Node> {
        match &self.current_view {
            GraphView::Root => graph.nodes.clone(),
            GraphView::WorkspaceNode(node_id) => {
                if let Some(workspace_node) = graph.nodes.get(node_id) {
                    if let Some(internal_graph) = workspace_node.get_internal_graph() {
                        internal_graph.nodes.clone()
                    } else {
                        HashMap::new()
                    }
                } else {
                    HashMap::new()
                }
            }
        }
    }

    /// Get the connections that should be visible in the current view
    pub fn get_viewed_connections(&self, graph: &NodeGraph) -> Vec<Connection> {
        match &self.current_view {
            GraphView::Root => graph.connections.clone(),
            GraphView::WorkspaceNode(node_id) => {
                if let Some(workspace_node) = graph.nodes.get(node_id) {
                    if let Some(internal_graph) = workspace_node.get_internal_graph() {
                        internal_graph.connections.clone()
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            }
        }
    }

    /// Get the active graph for the current view
    pub fn get_active_graph<'a>(&self, graph: &'a NodeGraph) -> &'a NodeGraph {
        match self.current_view {
            GraphView::Root => graph,
            GraphView::WorkspaceNode(workspace_node_id) => {
                if let Some(workspace_node) = graph.nodes.get(&workspace_node_id) {
                    if let Some(internal_graph) = workspace_node.get_internal_graph() {
                        internal_graph
                    } else {
                        graph // Fallback to root if no internal graph
                    }
                } else {
                    graph // Fallback to root if workspace node doesn't exist
                }
            }
        }
    }

    /// Build a temporary graph for the current view
    pub fn build_temp_graph(&self, viewed_nodes: &HashMap<NodeId, Node>, graph: &NodeGraph) -> NodeGraph {
        let mut temp_graph = NodeGraph::new();
        temp_graph.nodes = viewed_nodes.clone();
        temp_graph.connections = self.get_viewed_connections(graph);
        temp_graph
    }

    /// Check if a workspace node can be entered (has internal graph)
    pub fn can_enter_workspace(&self, graph: &NodeGraph, node_id: NodeId) -> bool {
        if let Some(node) = graph.nodes.get(&node_id) {
            node.get_internal_graph().is_some()
        } else {
            false
        }
    }

    /// Enter a workspace node if possible
    pub fn try_enter_workspace(&mut self, graph: &NodeGraph, node_id: NodeId) -> bool {
        if self.can_enter_workspace(graph, node_id) {
            self.current_view = GraphView::WorkspaceNode(node_id);
            true
        } else {
            false
        }
    }

    /// Exit current workspace and return to root
    pub fn exit_to_root(&mut self) {
        self.current_view = GraphView::Root;
    }

    /// Get workspace type string if currently in a workspace
    pub fn get_workspace_type(&self, graph: &NodeGraph) -> Option<String> {
        match self.current_view {
            GraphView::Root => None,
            GraphView::WorkspaceNode(workspace_node_id) => {
                if let Some(workspace_node) = graph.nodes.get(&workspace_node_id) {
                    workspace_node.get_workspace_type().map(|s| s.to_string())
                } else {
                    None
                }
            }
        }
    }

    /// Check if currently in a specific workspace type
    pub fn is_in_workspace_type(&self, graph: &NodeGraph, workspace_type: &str) -> bool {
        match self.get_workspace_type(graph) {
            Some(current_type) => current_type == workspace_type,
            None => false,
        }
    }
}

impl Default for ViewManager {
    fn default() -> Self {
        Self::new()
    }
}