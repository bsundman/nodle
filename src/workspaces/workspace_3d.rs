//! 3D workspace implementation for 3D graphics workflows

use crate::workspace::{Workspace, WorkspaceMenuItem};
use crate::nodes::factory::NodeRegistry;
use crate::nodes::three_d::*;

/// 3D workspace for 3D graphics, rendering, and modeling workflows
pub struct Workspace3D {
    node_registry: NodeRegistry,
}

impl Workspace3D {
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

impl Workspace for Workspace3D {
    fn id(&self) -> &'static str {
        "3d"
    }
    
    fn display_name(&self) -> &'static str {
        "3D"
    }
    
    fn get_menu_structure(&self) -> Vec<WorkspaceMenuItem> {
        // Use dynamic menu generation from node registry
        self.node_registry.generate_menu_structure(&["3D"])
    }
    
    fn is_generic_node_compatible(&self, node_type: &str) -> bool {
        // Only allow output nodes in 3D workspace for debugging
        matches!(node_type, 
            "Print" | "Debug"
        )
    }
    
    fn create_workspace_node(&self, node_type: &str, position: egui::Pos2) -> Option<crate::nodes::Node> {
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

impl Default for Workspace3D {
    fn default() -> Self {
        Self::new()
    }
}