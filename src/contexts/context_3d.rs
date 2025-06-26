//! 3D context implementation for 3D graphics workflows

use crate::context::{Context, ContextMenuItem};
use crate::nodes::factory::{NodeRegistry, NodeFactory};
use crate::nodes::three_d::*;

/// 3D context for 3D graphics, rendering, and modeling workflows
pub struct Context3D {
    node_registry: NodeRegistry,
}

impl Context3D {
    pub fn new() -> Self {
        let mut node_registry = NodeRegistry::default();
        
        // Register 3D-specific nodes using the standard NodeFactory pattern
        node_registry.register::<TranslateNode3D>();
        node_registry.register::<RotateNode3D>();
        node_registry.register::<ScaleNode3D>();
        node_registry.register::<CubeNode3D>();
        node_registry.register::<SphereNode3D>();
        node_registry.register::<PlaneNode3D>();
        node_registry.register::<PointLightNode3D>();
        node_registry.register::<DirectionalLightNode3D>();
        node_registry.register::<SpotLightNode3D>();
        node_registry.register::<ViewportNode3D>();
        
        Self {
            node_registry,
        }
    }
}

impl Context for Context3D {
    fn id(&self) -> &'static str {
        "3d"
    }
    
    fn display_name(&self) -> &'static str {
        "3D"
    }
    
    fn get_menu_structure(&self) -> Vec<ContextMenuItem> {
        // Use centralized menu hierarchy
        crate::menu_hierarchy::GlobalMenuHierarchy::get_3d_context_menu()
    }
    
    fn is_generic_node_compatible(&self, node_type: &str) -> bool {
        // Only allow output nodes in 3D context for debugging
        matches!(node_type, 
            "Print" | "Debug"
        )
    }
    
    fn create_context_node(&self, node_type: &str, position: egui::Pos2) -> Option<crate::nodes::Node> {
        // Try to create 3D-specific nodes using the registry first
        if let Some(node) = self.node_registry.create_node(node_type, position) {
            return Some(node);
        }
        
        // Fall back to generic node registry for whitelisted nodes
        if self.is_generic_node_compatible(node_type) {
            self.node_registry.create_node(node_type, position)
        } else {
            None
        }
    }
}

impl Default for Context3D {
    fn default() -> Self {
        Self::new()
    }
}