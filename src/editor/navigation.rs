//! Workspace navigation and breadcrumb UI components

use egui::{Color32, Pos2, Rect, Stroke, Vec2};

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
        let mut segments = vec![("Root".to_string(), WorkspacePath::root())];
        
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
}

impl NavigationManager {
    /// Create a new navigation manager at root
    pub fn new() -> Self {
        Self {
            current_path: WorkspacePath::root(),
            workspace_stack: Vec::new(),
        }
    }
    
    /// Navigate to a specific path
    pub fn navigate_to(&mut self, path: WorkspacePath) {
        self.current_path = path;
    }
    
    /// Navigate to a child workspace
    pub fn enter_workspace(&mut self, workspace_name: &str) {
        self.current_path = self.current_path.navigate_to(workspace_name);
    }
    
    /// Navigate to a specific workspace (alias for enter_workspace)
    pub fn navigate_to_workspace(&mut self, workspace_name: &str) {
        self.enter_workspace(workspace_name);
    }
    
    /// Navigate to parent workspace
    pub fn go_up(&mut self) {
        self.current_path = self.current_path.parent();
    }
    
    /// Navigate to root
    pub fn go_to_root(&mut self) {
        self.current_path = WorkspacePath::root();
    }
    
    /// Check if we can go up (not at root)
    pub fn can_go_up(&self) -> bool {
        !self.current_path.is_root() || !self.workspace_stack.is_empty()
    }
    
    /// Enter a workspace node (dive into its internal graph)
    pub fn enter_workspace_node(&mut self, node_id: crate::nodes::NodeId, workspace_type: &str) {
        self.workspace_stack.push(node_id);
        self.enter_workspace(workspace_type);
    }
    
    /// Exit the current workspace node (go back to parent graph)
    pub fn exit_workspace_node(&mut self) -> Option<crate::nodes::NodeId> {
        if let Some(node_id) = self.workspace_stack.pop() {
            if self.workspace_stack.is_empty() {
                self.go_to_root();
            } else {
                self.go_up();
            }
            Some(node_id)
        } else {
            None
        }
    }
    
    /// Check if we're currently inside a workspace node
    pub fn is_inside_workspace_node(&self) -> bool {
        !self.workspace_stack.is_empty()
    }
    
    /// Get the current workspace node ID if we're inside one
    pub fn current_workspace_node(&self) -> Option<crate::nodes::NodeId> {
        self.workspace_stack.last().copied()
    }
    
    /// Render the navigation breadcrumb bar
    pub fn render_breadcrumb(&mut self, ui: &mut egui::Ui) -> NavigationAction {
        let mut action = NavigationAction::None;
        
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            
            // If we're inside a workspace node, show special navigation
            if !self.workspace_stack.is_empty() {
                // Show up button to exit workspace
                if ui.button("⬆ Up").clicked() {
                    if let Some(_node_id) = self.exit_workspace_node() {
                        action = NavigationAction::GoUp;
                    }
                }
                ui.separator();
                
                // Show workspace node path
                let node_count = self.workspace_stack.len();
                ui.label(egui::RichText::new(format!("Inside {} workspace node{}", 
                    node_count, 
                    if node_count > 1 { "s" } else { "" }
                )).color(Color32::LIGHT_BLUE));
                
                // Show current workspace path
                if !self.current_path.is_root() {
                    ui.label("/");
                    ui.label(egui::RichText::new(&self.current_path.display_string())
                        .strong()
                        .color(Color32::WHITE));
                }
            } else {
                // Normal breadcrumb navigation when not inside a workspace node
                let segments = self.current_path.breadcrumb_segments();
                
                for (i, (name, path)) in segments.iter().enumerate() {
                    // Add separator between segments (except before first)
                    if i > 0 {
                        ui.label("/");
                    }
                    
                    // Render breadcrumb segment
                    let is_current = path == &self.current_path;
                    
                    if is_current {
                        // Current segment - not clickable
                        ui.label(egui::RichText::new(name).strong().color(Color32::WHITE));
                    } else {
                        // Clickable segment
                        let button = ui.button(egui::RichText::new(name).color(Color32::LIGHT_BLUE));
                        if button.clicked() {
                            action = NavigationAction::NavigateTo(path.clone());
                        }
                    }
                }
            }
            
            // Add some spacing
            ui.add_space(20.0);
            
            // Show debug info
            ui.label(egui::RichText::new(&format!("Path: {} | Stack: {}", 
                self.current_path.display_string(),
                self.workspace_stack.len()
            ))
                .color(Color32::GRAY)
                .size(12.0));
        });
        
        action
    }
}

/// Actions that can result from navigation UI interactions
#[derive(Debug, Clone)]
pub enum NavigationAction {
    None,
    NavigateTo(WorkspacePath),
    EnterWorkspace(String),
    GoUp,
    GoToRoot,
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