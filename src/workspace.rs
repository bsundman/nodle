//! Workspace system for different node editing environments

use egui::Color32;
use crate::nodes::NodeId;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::sync::OnceLock;

/// Represents a workspace for node editing (e.g., MaterialX, 3D, etc.)
pub trait Workspace {
    /// Unique identifier for this workspace
    fn id(&self) -> &'static str;
    
    /// Display name shown in UI
    fn display_name(&self) -> &'static str;
    
    /// Get the workspace menu structure for this workspace
    fn get_menu_structure(&self) -> Vec<WorkspaceMenuItem>;
    
    /// Check if a generic node type is compatible with this workspace
    fn is_generic_node_compatible(&self, node_type: &str) -> bool;
    
    /// Get the color to use for incompatible nodes (typically red)
    fn get_incompatible_color(&self) -> Color32 {
        Color32::from_rgb(200, 50, 50) // Red glow for incompatible nodes
    }
    
    /// Create a workspace-specific node at the given position
    fn create_workspace_node(&self, node_type: &str, position: egui::Pos2) -> Option<crate::nodes::Node>;
}

/// Menu item structure for workspace menus
#[derive(Debug, Clone)]
pub enum WorkspaceMenuItem {
    Category {
        name: String,
        items: Vec<WorkspaceMenuItem>,
    },
    Node {
        name: String,
        node_type: String,
    },
    Workspace {
        name: String,
        workspace_id: String,
    },
}

/// Manager for all available workspaces with hierarchical navigation support
#[derive(Default)]
pub struct WorkspaceManager {
    workspaces: Vec<Box<dyn Workspace>>,
    active_workspace: Option<usize>,
    incompatible_nodes: HashSet<NodeId>,
    // Workspace hierarchy mapping: workspace_id -> parent_workspace_id
    workspace_hierarchy: std::collections::HashMap<String, Option<String>>,
    // Workspace lookup by ID
    workspace_lookup: std::collections::HashMap<String, usize>,
}

impl WorkspaceManager {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Register a new workspace
    pub fn register_workspace(&mut self, workspace: Box<dyn Workspace>) {
        let workspace_id = workspace.id().to_string();
        let index = self.workspaces.len();
        
        self.workspaces.push(workspace);
        self.workspace_lookup.insert(workspace_id.clone(), index);
        
        // Set hierarchy - for now, all workspaces are top-level except MaterialX
        if workspace_id == "materialx" {
            self.workspace_hierarchy.insert(workspace_id, Some("3d".to_string()));
        } else {
            self.workspace_hierarchy.insert(workspace_id, None);
        }
    }
    
    /// Get all available workspaces
    pub fn get_workspaces(&self) -> &[Box<dyn Workspace>] {
        &self.workspaces
    }
    
    /// Set the active workspace
    pub fn set_active_workspace(&mut self, index: Option<usize>) {
        self.active_workspace = index;
    }
    
    /// Get the active workspace
    pub fn get_active_workspace(&self) -> Option<&dyn Workspace> {
        self.active_workspace.and_then(|i| self.workspaces.get(i).map(|w| w.as_ref()))
    }
    
    /// Set active workspace by ID
    pub fn set_active_workspace_by_id(&mut self, workspace_id: Option<&str>) {
        self.active_workspace = workspace_id
            .and_then(|id| self.workspace_lookup.get(id))
            .copied();
    }
    
    /// Get workspace by ID
    pub fn get_workspace_by_id(&self, workspace_id: &str) -> Option<&dyn Workspace> {
        self.workspace_lookup.get(workspace_id)
            .and_then(|&index| self.workspaces.get(index))
            .map(|workspace| workspace.as_ref())
    }
    
    /// Get the workspace for a given navigation path
    pub fn get_workspace_for_path(&self, path: &crate::editor::navigation::WorkspacePath) -> Option<&dyn Workspace> {
        match path.current_workspace() {
            Some(workspace_name) => {
                // Convert display name to workspace ID (temporary mapping)
                let workspace_id = match workspace_name {
                    "3D" => "3d",
                    "MaterialX" => "materialx",
                    _ => workspace_name,
                };
                self.get_workspace_by_id(workspace_id)
            }
            None => None, // Root level - no specific workspace
        }
    }
    
    /// Get menu structure based on current navigation path
    pub fn get_menu_for_path(&self, path: &crate::editor::navigation::WorkspacePath) -> Vec<WorkspaceMenuItem> {
        // Get the workspace for this path
        if let Some(workspace) = self.get_workspace_for_path(path) {
            let mut menu_items = Vec::new();
            
            // Use the workspace's dynamic menu generation
            let workspace_menu = workspace.get_menu_structure();
            menu_items.extend(workspace_menu);
            
            // Plugin nodes are already included in workspace.get_menu_structure() 
            // via the global plugin manager integration in workspace_3d.rs
            
            menu_items
        } else {
            // At root level, use the hardcoded root menu for workspace selection
            crate::menu_hierarchy::GlobalMenuHierarchy::get_menu_for_workspace(None)
        }
    }
    
    /// Get plugin menu items using global plugin manager
    fn get_plugin_menu_items(&self) -> Result<Vec<WorkspaceMenuItem>, String> {
        // Use the global plugin manager instance
        if let Some(plugin_manager) = get_global_plugin_manager() {
            let manager = plugin_manager.lock().map_err(|e| format!("Plugin manager lock error: {}", e))?;
            Ok(manager.get_workspace_menu_items())
        } else {
            Ok(Vec::new()) // No plugins loaded
        }
    }
    
    /// Check if a node is compatible with the current workspace
    pub fn is_node_compatible(&self, node_type: &str) -> bool {
        if let Some(workspace) = self.get_active_workspace() {
            workspace.is_generic_node_compatible(node_type)
        } else {
            true // No workspace active, all nodes are compatible
        }
    }
    
    /// Mark a node as incompatible
    pub fn mark_node_incompatible(&mut self, node_id: NodeId) {
        self.incompatible_nodes.insert(node_id);
    }
    
    /// Check if a node is marked as incompatible
    pub fn is_node_incompatible(&self, node_id: NodeId) -> bool {
        self.incompatible_nodes.contains(&node_id)
    }
    
    /// Clear incompatible node markings
    pub fn clear_incompatible_nodes(&mut self) {
        self.incompatible_nodes.clear();
    }
    
    /// Get the color for incompatible nodes
    pub fn get_incompatible_color(&self) -> Color32 {
        if let Some(workspace) = self.get_active_workspace() {
            workspace.get_incompatible_color()
        } else {
            Color32::from_rgb(200, 50, 50)
        }
    }
}

// Global plugin manager singleton
static GLOBAL_PLUGIN_MANAGER: OnceLock<Arc<Mutex<crate::plugins::PluginManager>>> = OnceLock::new();

/// Initialize the global plugin manager
pub fn initialize_global_plugin_manager() -> Result<(), String> {
    let mut plugin_manager = crate::plugins::PluginManager::new();
    
    // Load plugins once at startup
    if let Err(e) = plugin_manager.discover_and_load_plugins() {
        return Err(format!("Failed to initialize plugins: {}", e));
    }
    
    // Store in global singleton
    GLOBAL_PLUGIN_MANAGER.set(Arc::new(Mutex::new(plugin_manager)))
        .map_err(|_| "Failed to initialize global plugin manager".to_string())?;
    
    Ok(())
}

/// Get the global plugin manager instance
pub fn get_global_plugin_manager() -> Option<Arc<Mutex<crate::plugins::PluginManager>>> {
    GLOBAL_PLUGIN_MANAGER.get().cloned()
}