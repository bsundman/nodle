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
}

/// Manager for all available contexts
#[derive(Default)]
pub struct ContextManager {
    contexts: Vec<Box<dyn Context>>,
    active_context: Option<usize>,
    incompatible_nodes: HashSet<NodeId>,
}

impl ContextManager {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Register a new context
    pub fn register_context(&mut self, context: Box<dyn Context>) {
        self.contexts.push(context);
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