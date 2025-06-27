//! Base workspace implementation for generic node editing

use crate::workspace::{Workspace, WorkspaceMenuItem};
use crate::nodes::factory::NodeRegistry;

/// Base workspace for generic node editing (no specialized workspace)
pub struct BaseWorkspace {
    node_registry: NodeRegistry,
}

impl BaseWorkspace {
    pub fn new() -> Self {
        Self {
            node_registry: NodeRegistry::default(), // Uses all registered enhanced nodes
        }
    }
}

impl Workspace for BaseWorkspace {
    fn id(&self) -> &'static str {
        "base"
    }
    
    fn display_name(&self) -> &'static str {
        "Generic"
    }
    
    fn get_menu_structure(&self) -> Vec<WorkspaceMenuItem> {
        // Build menu from enhanced node registry categories
        let mut menu_items = Vec::new();
        
        // Math category
        menu_items.push(WorkspaceMenuItem::Category {
            name: "Math".to_string(),
            items: vec![
                WorkspaceMenuItem::Node { name: "Add".to_string(), node_type: "Add".to_string() },
                WorkspaceMenuItem::Node { name: "Subtract".to_string(), node_type: "Subtract".to_string() },
                WorkspaceMenuItem::Node { name: "Multiply".to_string(), node_type: "Multiply".to_string() },
                WorkspaceMenuItem::Node { name: "Divide".to_string(), node_type: "Divide".to_string() },
            ],
        });
        
        // Logic category
        menu_items.push(WorkspaceMenuItem::Category {
            name: "Logic".to_string(),
            items: vec![
                WorkspaceMenuItem::Node { name: "AND".to_string(), node_type: "AND".to_string() },
                WorkspaceMenuItem::Node { name: "OR".to_string(), node_type: "OR".to_string() },
                WorkspaceMenuItem::Node { name: "NOT".to_string(), node_type: "NOT".to_string() },
            ],
        });
        
        // Data category
        menu_items.push(WorkspaceMenuItem::Category {
            name: "Data".to_string(),
            items: vec![
                WorkspaceMenuItem::Node { name: "Constant".to_string(), node_type: "Constant".to_string() },
                WorkspaceMenuItem::Node { name: "Variable".to_string(), node_type: "Variable".to_string() },
            ],
        });
        
        // Output category
        menu_items.push(WorkspaceMenuItem::Category {
            name: "Output".to_string(),
            items: vec![
                WorkspaceMenuItem::Node { name: "Print".to_string(), node_type: "Print".to_string() },
                WorkspaceMenuItem::Node { name: "Debug".to_string(), node_type: "Debug".to_string() },
            ],
        });
        
        menu_items
    }
    
    fn is_generic_node_compatible(&self, _node_type: &str) -> bool {
        true // Base workspace accepts all generic nodes
    }
    
    fn create_workspace_node(&self, node_type: &str, position: egui::Pos2) -> Option<crate::nodes::Node> {
        // Delegate to enhanced node registry
        self.node_registry.create_node(node_type, position)
    }
}

impl Default for BaseWorkspace {
    fn default() -> Self {
        Self::new()
    }
}