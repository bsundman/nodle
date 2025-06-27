//! 3D workspace implementation for 3D graphics workflows

use crate::workspace::{Workspace, WorkspaceMenuItem};
use crate::nodes::factory::NodeRegistry;
use crate::nodes::three_d::*;
use crate::nodes::three_d::geometry::{CubeNodeWithInterface, SphereNodeWithInterface};
use crate::nodes::three_d::transform::TranslateNodeWithInterface;

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
        
        // Register interface panel versions
        node_registry.register::<TranslateNodeWithInterface>();
        node_registry.register::<CubeNodeWithInterface>();
        node_registry.register::<SphereNodeWithInterface>();
        
        // Register USD nodes
        node_registry.register::<USDCreateStage>();
        node_registry.register::<USDLoadStage>();
        node_registry.register::<USDSaveStage>();
        node_registry.register::<USDXform>();
        node_registry.register::<USDMesh>();
        node_registry.register::<USDSphere>();
        node_registry.register::<USDCube>();
        node_registry.register::<USDCamera>();
        node_registry.register::<USDDistantLight>();
        node_registry.register::<USDSphereLight>();
        node_registry.register::<USDRectLight>();
        node_registry.register::<USDMaterial>();
        node_registry.register::<USDPreviewSurface>();
        node_registry.register::<USDTexture>();
        node_registry.register::<USDViewport>();
        node_registry.register::<USDSubLayer>();
        node_registry.register::<USDReference>();
        node_registry.register::<USDPayload>();
        node_registry.register::<USDSetAttribute>();
        node_registry.register::<USDGetAttribute>();
        
        // Debug: Print registered nodes
        println!("🔍 3D Workspace registered nodes:");
        for node_type in node_registry.node_types() {
            if let Some(metadata) = node_registry.get_metadata(node_type) {
                println!("  {} -> {:?}", node_type, metadata.category.path());
            }
        }
        
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