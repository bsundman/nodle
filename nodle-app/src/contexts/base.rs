//! Base context implementation for generic node editing

use crate::context::{Context, ContextMenuItem};
use crate::nodes::factory::NodeRegistry;

/// Base context for generic node editing (no specialized context)
pub struct BaseContext {
    node_registry: NodeRegistry,
}

impl BaseContext {
    pub fn new() -> Self {
        Self {
            node_registry: NodeRegistry::default(), // Uses all registered enhanced nodes
        }
    }
}

impl Context for BaseContext {
    fn id(&self) -> &'static str {
        "base"
    }
    
    fn display_name(&self) -> &'static str {
        "Generic"
    }
    
    fn get_menu_structure(&self) -> Vec<ContextMenuItem> {
        // Build menu from enhanced node registry categories
        let mut menu_items = Vec::new();
        
        // Math category
        menu_items.push(ContextMenuItem::Category {
            name: "Math".to_string(),
            items: vec![
                ContextMenuItem::Node { name: "Add".to_string(), node_type: "Add".to_string() },
                ContextMenuItem::Node { name: "Subtract".to_string(), node_type: "Subtract".to_string() },
                ContextMenuItem::Node { name: "Multiply".to_string(), node_type: "Multiply".to_string() },
                ContextMenuItem::Node { name: "Divide".to_string(), node_type: "Divide".to_string() },
            ],
        });
        
        // Logic category
        menu_items.push(ContextMenuItem::Category {
            name: "Logic".to_string(),
            items: vec![
                ContextMenuItem::Node { name: "AND".to_string(), node_type: "AND".to_string() },
                ContextMenuItem::Node { name: "OR".to_string(), node_type: "OR".to_string() },
                ContextMenuItem::Node { name: "NOT".to_string(), node_type: "NOT".to_string() },
            ],
        });
        
        // Data category
        menu_items.push(ContextMenuItem::Category {
            name: "Data".to_string(),
            items: vec![
                ContextMenuItem::Node { name: "Constant".to_string(), node_type: "Constant".to_string() },
                ContextMenuItem::Node { name: "Variable".to_string(), node_type: "Variable".to_string() },
            ],
        });
        
        // Output category
        menu_items.push(ContextMenuItem::Category {
            name: "Output".to_string(),
            items: vec![
                ContextMenuItem::Node { name: "Print".to_string(), node_type: "Print".to_string() },
                ContextMenuItem::Node { name: "Debug".to_string(), node_type: "Debug".to_string() },
            ],
        });
        
        menu_items
    }
    
    fn is_generic_node_compatible(&self, _node_type: &str) -> bool {
        true // Base context accepts all generic nodes
    }
    
    fn create_context_node(&self, node_type: &str, position: egui::Pos2) -> Option<crate::nodes::Node> {
        // Delegate to enhanced node registry
        self.node_registry.create_node(node_type, position)
    }
}

impl Default for BaseContext {
    fn default() -> Self {
        Self::new()
    }
}