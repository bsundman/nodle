//! 2D workspace for 2D graphics and drawing workflows

use crate::workspace::{Workspace, WorkspaceMenuItem};
use crate::nodes::{Node, factory::NodeRegistry};
use egui::Pos2;

/// 2D workspace for 2D graphics and drawing workflows
pub struct Workspace2D {
    node_registry: NodeRegistry,
}

impl Workspace2D {
    pub fn new() -> Self {
        let mut node_registry = NodeRegistry::new();
        
        // Register 2D-specific nodes here when they are implemented
        // node_registry.register::<Rectangle2DNode>();
        // node_registry.register::<Circle2DNode>();
        // etc.
        
        Self {
            node_registry,
        }
    }
}

impl Default for Workspace2D {
    fn default() -> Self {
        Self::new()
    }
}

impl Workspace for Workspace2D {
    fn id(&self) -> &'static str {
        "2d"
    }
    
    fn display_name(&self) -> &'static str {
        "2D"
    }
    
    fn get_menu_structure(&self) -> Vec<WorkspaceMenuItem> {
        // Use centralized menu hierarchy
        crate::menu_hierarchy::GlobalMenuHierarchy::get_2d_workspace_menu()
    }
    
    fn is_generic_node_compatible(&self, node_type: &str) -> bool {
        // Whitelist of generic nodes compatible with 2D
        matches!(node_type, 
            "Add" | "Subtract" | "Multiply" | "Divide" |  // Math operations
            "Print" | "Debug"  // Output nodes for debugging
        )
    }
    
    fn create_workspace_node(&self, node_type: &str, position: Pos2) -> Option<Node> {
        // For now, try to create using the node registry
        // This will be expanded when 2D-specific nodes are implemented
        self.node_registry.create_node(node_type, position)
    }
}