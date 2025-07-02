//! 3D workspace implementation for 3D graphics workflows

use crate::workspace::{Workspace, WorkspaceMenuItem};
use crate::nodes::factory::NodeRegistry;
use crate::nodes::three_d::*;
use crate::nodes::three_d::geometry::{CubeNode, SphereNode};
use crate::nodes::three_d::transform::TranslateNode;

/// 3D workspace for 3D graphics, rendering, and modeling workflows
pub struct Workspace3D {
    node_registry: NodeRegistry,
}

impl Workspace3D {
    pub fn new() -> Self {
        let mut node_registry = NodeRegistry::default();
        
        // Register 3D-specific nodes with interface panels
        node_registry.register::<TranslateNode>();
        node_registry.register::<crate::nodes::three_d::transform::RotateNode>();
        node_registry.register::<crate::nodes::three_d::transform::ScaleNode>();
        node_registry.register::<CubeNode>();
        node_registry.register::<SphereNode>();
        node_registry.register::<crate::nodes::three_d::geometry::PlaneNode>();
        node_registry.register::<crate::nodes::three_d::lighting::PointLightNode>();
        node_registry.register::<crate::nodes::three_d::lighting::DirectionalLightNode>();
        node_registry.register::<crate::nodes::three_d::lighting::SpotLightNode>();
        node_registry.register::<crate::nodes::three_d::output::viewport::ViewportNode>();
        
        // Register USD nodes
        node_registry.register::<USDCreateStage>();
        node_registry.register::<crate::nodes::three_d::usd::stage::load_stage::LoadStageNode>();
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
        println!("DEBUG: Workspace3D attempting to create node type: '{}'", node_type);
        // Try to create 3D-specific nodes using the registry first
        if let Some(node) = self.node_registry.create_node(node_type, position) {
            println!("DEBUG: Workspace3D successfully created node type: '{}'", node_type);
            return Some(node);
        }
        
        println!("DEBUG: Workspace3D registry failed to create '{}', trying fallback", node_type);
        // Fall back to generic node registry for whitelisted nodes
        if self.is_generic_node_compatible(node_type) {
            println!("DEBUG: Node '{}' is generic-compatible, trying generic registry", node_type);
            let result = self.node_registry.create_node(node_type, position);
            if result.is_some() {
                println!("DEBUG: Generic registry created node '{}'", node_type);
            } else {
                println!("DEBUG: Generic registry also failed to create '{}'", node_type);
            }
            result
        } else {
            println!("DEBUG: Node '{}' is not generic-compatible", node_type);
            None
        }
    }
}

impl Default for Workspace3D {
    fn default() -> Self {
        Self::new()
    }
}