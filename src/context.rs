//! Context system for different node editing environments

use egui::Color32;
use crate::nodes::NodeId;
use std::collections::HashSet;

/// Represents a context for node editing (e.g., MaterialX, generic nodes, etc.)
pub trait Context {
    /// Unique identifier for this context
    fn id(&self) -> &'static str;
    
    /// Display name shown in UI
    fn display_name(&self) -> &'static str;
    
    /// Get the context menu structure for this context
    fn get_menu_structure(&self) -> Vec<ContextMenuItem>;
    
    /// Check if a generic node type is compatible with this context
    fn is_generic_node_compatible(&self, node_type: &str) -> bool;
    
    /// Get the color to use for incompatible nodes (typically red)
    fn get_incompatible_color(&self) -> Color32 {
        Color32::from_rgb(200, 50, 50) // Red glow for incompatible nodes
    }
    
    /// Create a context-specific node at the given position
    fn create_context_node(&self, node_type: &str, position: egui::Pos2) -> Option<crate::nodes::Node>;
}

/// Menu item structure for context menus
#[derive(Debug, Clone)]
pub enum ContextMenuItem {
    Category {
        name: String,
        items: Vec<ContextMenuItem>,
    },
    Node {
        name: String,
        node_type: String,
    },
    Subcontext {
        name: String,
        context_id: String,
    },
}

/// Manager for all available contexts with hierarchical navigation support
#[derive(Default)]
pub struct ContextManager {
    contexts: Vec<Box<dyn Context>>,
    active_context: Option<usize>,
    incompatible_nodes: HashSet<NodeId>,
    // Context hierarchy mapping: context_id -> parent_context_id
    context_hierarchy: std::collections::HashMap<String, Option<String>>,
    // Context lookup by ID
    context_lookup: std::collections::HashMap<String, usize>,
}

impl ContextManager {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Register a new context
    pub fn register_context(&mut self, context: Box<dyn Context>) {
        let context_id = context.id().to_string();
        let index = self.contexts.len();
        
        self.contexts.push(context);
        self.context_lookup.insert(context_id.clone(), index);
        
        // Set hierarchy - for now, all contexts are top-level except MaterialX
        if context_id == "materialx" {
            self.context_hierarchy.insert(context_id, Some("3d".to_string()));
        } else {
            self.context_hierarchy.insert(context_id, None);
        }
    }
    
    /// Get all available contexts
    pub fn get_contexts(&self) -> &[Box<dyn Context>] {
        &self.contexts
    }
    
    /// Set the active context
    pub fn set_active_context(&mut self, index: Option<usize>) {
        self.active_context = index;
    }
    
    /// Get the active context
    pub fn get_active_context(&self) -> Option<&dyn Context> {
        self.active_context.and_then(|i| self.contexts.get(i).map(|c| c.as_ref()))
    }
    
    /// Set active context by ID
    pub fn set_active_context_by_id(&mut self, context_id: Option<&str>) {
        self.active_context = context_id
            .and_then(|id| self.context_lookup.get(id))
            .copied();
    }
    
    /// Get context by ID
    pub fn get_context_by_id(&self, context_id: &str) -> Option<&dyn Context> {
        self.context_lookup.get(context_id)
            .and_then(|&index| self.contexts.get(index))
            .map(|context| context.as_ref())
    }
    
    /// Get the context for a given navigation path
    pub fn get_context_for_path(&self, path: &crate::editor::navigation::ContextPath) -> Option<&dyn Context> {
        match path.current_context() {
            Some(context_name) => {
                // Convert display name to context ID (temporary mapping)
                let context_id = match context_name {
                    "3D" => "3d",
                    "MaterialX" => "materialx",
                    _ => context_name,
                };
                self.get_context_by_id(context_id)
            }
            None => None, // Root level - no specific context
        }
    }
    
    /// Get menu structure based on current navigation path
    pub fn get_menu_for_path(&self, path: &crate::editor::navigation::ContextPath) -> Vec<ContextMenuItem> {
        // Use centralized menu hierarchy instead of fragmented logic
        let context_id = if path.is_root() {
            None
        } else {
            path.current_context().map(|name| match name {
                "3D" => "3d",
                "MaterialX" => "materialx",
                _ => name,
            })
        };
        
        crate::menu_hierarchy::GlobalMenuHierarchy::get_menu_for_context(context_id)
    }
    
    
    /// Check if a node is compatible with the current context
    pub fn is_node_compatible(&self, node_type: &str) -> bool {
        if let Some(context) = self.get_active_context() {
            context.is_generic_node_compatible(node_type)
        } else {
            true // No context active, all nodes are compatible
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
        if let Some(context) = self.get_active_context() {
            context.get_incompatible_color()
        } else {
            Color32::from_rgb(200, 50, 50)
        }
    }
}