//! Workspace navigation and breadcrumb UI components

use egui::Color32;
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

/// Represents a navigation path through workspace hierarchy
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspacePath {
    /// Path segments like ["3D", "MaterialX"] for /3D/MaterialX/
    pub segments: Vec<String>,
}

impl WorkspacePath {
    /// Create a new path at the root level
    pub fn root() -> Self {
        Self {
            segments: Vec::new(),
        }
    }
    
    /// Create a path from segments
    pub fn from_segments(segments: Vec<String>) -> Self {
        Self { segments }
    }
    
    /// Check if this is the root path
    pub fn is_root(&self) -> bool {
        self.segments.is_empty()
    }
    
    /// Get the current workspace name (last segment)
    pub fn current_workspace(&self) -> Option<&str> {
        self.segments.last().map(|s| s.as_str())
    }
    
    /// Get the parent path
    pub fn parent(&self) -> Self {
        if self.segments.is_empty() {
            return Self::root();
        }
        
        let mut parent_segments = self.segments.clone();
        parent_segments.pop();
        Self::from_segments(parent_segments)
    }
    
    /// Navigate to a child workspace
    pub fn navigate_to(&self, workspace_name: &str) -> Self {
        let mut new_segments = self.segments.clone();
        new_segments.push(workspace_name.to_string());
        Self::from_segments(new_segments)
    }
    
    /// Get the full path string for display
    pub fn display_string(&self) -> String {
        if self.is_root() {
            "/".to_string()
        } else {
            format!("/{}/", self.segments.join("/"))
        }
    }
    
    /// Get path segments for breadcrumb rendering
    pub fn breadcrumb_segments(&self) -> Vec<(String, WorkspacePath)> {
        let mut segments = vec![("root".to_string(), WorkspacePath::root())];
        
        let mut current_path = WorkspacePath::root();
        for segment in &self.segments {
            current_path = current_path.navigate_to(segment);
            segments.push((segment.clone(), current_path.clone()));
        }
        
        segments
    }
}

/// Manages workspace navigation state and UI
pub struct NavigationManager {
    /// Current navigation path
    pub current_path: WorkspacePath,
    /// Stack of workspace nodes we've entered (node IDs)
    pub workspace_stack: Vec<crate::nodes::NodeId>,
    /// Current view state - which graph we're looking at
    current_view: GraphView,
}

impl NavigationManager {
    /// Create a new navigation manager at root
    pub fn new() -> Self {
        Self {
            current_path: WorkspacePath::root(),
            workspace_stack: Vec::new(),
            current_view: GraphView::Root,
        }
    }
    
    /// Navigate to a specific path
    pub fn navigate_to(&mut self, path: WorkspacePath) {
        self.current_path = path;
        // Reset view to root when navigating via path
        self.current_view = GraphView::Root;
    }
    
    /// Navigate to a child workspace
    pub fn enter_workspace(&mut self, workspace_name: &str) {
        self.current_path = self.current_path.navigate_to(workspace_name);
    }
    
    
    /// Navigate to parent workspace
    pub fn go_up(&mut self) {
        self.current_path = self.current_path.parent();
        self.current_view = GraphView::Root;
    }
    
    /// Navigate to root
    pub fn go_to_root(&mut self) {
        self.current_path = WorkspacePath::root();
        self.current_view = GraphView::Root;
        self.workspace_stack.clear();
    }
    
    /// Check if we can go up (not at root)
    pub fn can_go_up(&self) -> bool {
        !self.current_path.is_root() || !self.workspace_stack.is_empty()
    }
    
    /// Enter a workspace node (dive into its internal graph)
    pub fn enter_workspace_node(&mut self, node_id: NodeId, workspace_type: &str) {
        self.workspace_stack.push(node_id);
        self.enter_workspace(workspace_type);
        self.current_view = GraphView::WorkspaceNode(node_id);
    }
    
    /// Exit the current workspace node (go back to parent graph)
    pub fn exit_workspace_node(&mut self) -> Option<NodeId> {
        if let Some(node_id) = self.workspace_stack.pop() {
            if self.workspace_stack.is_empty() {
                self.go_to_root();
            } else {
                self.go_up();
                // If there's still a workspace node on the stack, set view to it
                if let Some(&parent_node_id) = self.workspace_stack.last() {
                    self.current_view = GraphView::WorkspaceNode(parent_node_id);
                }
            }
            Some(node_id)
        } else {
            None
        }
    }
    
    /// Render the navigation breadcrumb bar
    pub fn render_breadcrumb(&mut self, ui: &mut egui::Ui) -> NavigationAction {
        let mut action = NavigationAction::None;
        
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            
            // Always show unified breadcrumb navigation
            let segments = self.current_path.breadcrumb_segments();
            
            for (i, (name, path)) in segments.iter().enumerate() {
                // Add separator between segments (except before first)
                if i > 0 {
                    ui.label("/");
                }
                
                // Render breadcrumb segment - all segments are clickable
                let is_current = path == &self.current_path;
                
                if is_current {
                    // Current segment - highlighted but still clickable
                    let button = ui.button(egui::RichText::new(name).strong().color(Color32::WHITE));
                    if button.clicked() {
                        action = NavigationAction::NavigateTo(path.clone());
                    }
                } else {
                    // Clickable segment
                    let button = ui.button(egui::RichText::new(name).color(Color32::LIGHT_BLUE));
                    if button.clicked() {
                        action = NavigationAction::NavigateTo(path.clone());
                    }
                }
            }
        });
        
        action
    }

    // === VIEW MANAGEMENT METHODS (formerly from ViewManager) ===

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

    /// Exit current workspace and return to root
    pub fn exit_to_root(&mut self) {
        self.current_view = GraphView::Root;
        self.current_path = WorkspacePath::root();
        self.workspace_stack.clear();
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

/// Actions that can result from navigation UI interactions
#[derive(Debug, Clone)]
pub enum NavigationAction {
    None,
    NavigateTo(WorkspacePath),
}

impl Default for NavigationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_workspace_path() {
        let root = WorkspacePath::root();
        assert!(root.is_root());
        assert_eq!(root.display_string(), "/");
        
        let path_3d = root.navigate_to("3D");
        assert_eq!(path_3d.display_string(), "/3D/");
        assert_eq!(path_3d.current_workspace(), Some("3D"));
        
        let path_materialx = path_3d.navigate_to("MaterialX");
        assert_eq!(path_materialx.display_string(), "/3D/MaterialX/");
        assert_eq!(path_materialx.current_workspace(), Some("MaterialX"));
        
        let parent = path_materialx.parent();
        assert_eq!(parent, path_3d);
        
        let grandparent = parent.parent();
        assert_eq!(grandparent, root);
    }
    
    #[test]
    fn test_breadcrumb_segments() {
        let path = WorkspacePath::from_segments(vec!["3D".to_string(), "MaterialX".to_string()]);
        let segments = path.breadcrumb_segments();
        
        assert_eq!(segments.len(), 3); // Root + 3D + MaterialX
        assert_eq!(segments[0].0, "Root");
        assert_eq!(segments[1].0, "3D");
        assert_eq!(segments[2].0, "MaterialX");
    }
}