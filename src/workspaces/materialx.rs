//! MaterialX workspace for material authoring and shading workflows

use crate::workspace::{Workspace, WorkspaceMenuItem};
use crate::nodes::{Node, materialx};
use egui::Pos2;

/// MaterialX workspace for material authoring and shading workflows
/// This workspace has 3D workspace as its parent
pub struct MaterialXWorkspace;

impl MaterialXWorkspace {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MaterialXWorkspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Workspace for MaterialXWorkspace {
    fn id(&self) -> &'static str {
        "materialx"
    }
    
    fn display_name(&self) -> &'static str {
        "MaterialX"
    }
    
    fn get_menu_structure(&self) -> Vec<WorkspaceMenuItem> {
        // Use centralized menu hierarchy
        crate::menu_hierarchy::GlobalMenuHierarchy::get_materialx_workspace_menu()
    }
    
    fn is_generic_node_compatible(&self, node_type: &str) -> bool {
        // Whitelist of generic nodes compatible with MaterialX
        matches!(node_type, 
            "Add" | "Subtract" | "Multiply" | "Divide" |  // Math operations
            "Print" | "Debug"  // Output nodes for debugging
        )
    }
    
    fn create_workspace_node(&self, node_type: &str, position: Pos2) -> Option<Node> {
        // Create MaterialX-specific nodes using the organized node functions
        match node_type {
            // Shading nodes
            "MaterialX_StandardSurface" => Some(materialx::shading::create_standard_surface_node(position)),
            "MaterialX_SurfaceShader" => Some(materialx::shading::create_surface_shader_node(position)),
            "MaterialX_ShaderContext" => Some(materialx::utilities::create_shader_workspace_node(position)),
            
            // Texture nodes
            "MaterialX_Image" => Some(materialx::textures::create_image_node(position)),
            "MaterialX_Noise" => Some(materialx::textures::create_noise_node(position)),
            "MaterialX_Checkerboard" => Some(materialx::textures::create_checkerboard_node(position)),
            
            // Math nodes
            "MaterialX_DotProduct" => Some(materialx::math::create_dot_product_node(position)),
            "MaterialX_Normalize" => Some(materialx::math::create_normalize_node(position)),
            "MaterialX_CrossProduct" => Some(materialx::math::create_cross_product_node(position)),
            
            // Utility nodes
            "MaterialX_Mix" => Some(materialx::utilities::create_mix_node(position)),
            "MaterialX_Switch" => Some(materialx::utilities::create_switch_node(position)),
            "MaterialX_Constant" => Some(materialx::utilities::create_constant_node(position)),
            
            _ => None,
        }
    }
}