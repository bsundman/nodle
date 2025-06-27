//! Centralized menu hierarchy system
//! 
//! This module provides the single source of truth for all node menu structures.
//! All menus (workspace menus, file menus, etc.) should use this system to ensure consistency.

use crate::workspace::WorkspaceMenuItem;

/// Central registry for all menu hierarchies
pub struct GlobalMenuHierarchy;

impl GlobalMenuHierarchy {
    /// Get the root-level menu structure
    /// This is the authoritative definition used by ALL menu systems
    /// Root level only allows workspace creation, not regular nodes
    pub fn get_root_menu() -> Vec<WorkspaceMenuItem> {
        vec![
            // Only workspace creation is allowed at root level
            WorkspaceMenuItem::Category {
                name: "Create Workspace".to_string(),
                items: vec![
                    WorkspaceMenuItem::Node {
                        name: "3D Workspace".to_string(),
                        node_type: "WORKSPACE:3D".to_string(),
                    },
                    WorkspaceMenuItem::Node {
                        name: "Generic Workspace".to_string(),
                        node_type: "WORKSPACE:Generic".to_string(),
                    },
                ],
            },
        ]
    }
    
    /// Get menu structure for 3D workspace
    pub fn get_3d_workspace_menu() -> Vec<WorkspaceMenuItem> {
        vec![
            WorkspaceMenuItem::Category {
                name: "Transform".to_string(),
                items: vec![
                    WorkspaceMenuItem::Node { 
                        name: "Translate".to_string(), 
                        node_type: "3D_Translate".to_string() 
                    },
                    WorkspaceMenuItem::Node { 
                        name: "Rotate".to_string(), 
                        node_type: "3D_Rotate".to_string() 
                    },
                    WorkspaceMenuItem::Node { 
                        name: "Scale".to_string(), 
                        node_type: "3D_Scale".to_string() 
                    },
                ],
            },
            WorkspaceMenuItem::Category {
                name: "Geometry".to_string(),
                items: vec![
                    WorkspaceMenuItem::Node { 
                        name: "Cube".to_string(), 
                        node_type: "3D_Cube".to_string() 
                    },
                    WorkspaceMenuItem::Node { 
                        name: "Sphere".to_string(), 
                        node_type: "3D_Sphere".to_string() 
                    },
                    WorkspaceMenuItem::Node { 
                        name: "Plane".to_string(), 
                        node_type: "3D_Plane".to_string() 
                    },
                ],
            },
            WorkspaceMenuItem::Category {
                name: "Lighting".to_string(),
                items: vec![
                    WorkspaceMenuItem::Node { 
                        name: "Point Light".to_string(), 
                        node_type: "3D_PointLight".to_string() 
                    },
                    WorkspaceMenuItem::Node { 
                        name: "Directional Light".to_string(), 
                        node_type: "3D_DirectionalLight".to_string() 
                    },
                    WorkspaceMenuItem::Node { 
                        name: "Spot Light".to_string(), 
                        node_type: "3D_SpotLight".to_string() 
                    },
                ],
            },
            WorkspaceMenuItem::Category {
                name: "Output".to_string(),
                items: vec![
                    WorkspaceMenuItem::Node { 
                        name: "Viewport".to_string(), 
                        node_type: "3D_Viewport".to_string() 
                    },
                ],
            },
            WorkspaceMenuItem::Category {
                name: "Materials".to_string(),
                items: vec![
                    WorkspaceMenuItem::Workspace { 
                        name: "MaterialX".to_string(), 
                        workspace_id: "materialx".to_string() 
                    },
                ],
            },
        ]
    }
    
    /// Get menu structure for MaterialX workspace
    pub fn get_materialx_workspace_menu() -> Vec<WorkspaceMenuItem> {
        vec![
            WorkspaceMenuItem::Category {
                name: "Shading".to_string(),
                items: vec![
                    WorkspaceMenuItem::Node { 
                        name: "Standard Surface".to_string(), 
                        node_type: "MaterialX_StandardSurface".to_string() 
                    },
                    WorkspaceMenuItem::Node { 
                        name: "Mix".to_string(), 
                        node_type: "MaterialX_Mix".to_string() 
                    },
                ],
            },
            WorkspaceMenuItem::Category {
                name: "Texture".to_string(),
                items: vec![
                    WorkspaceMenuItem::Node { 
                        name: "Image".to_string(), 
                        node_type: "MaterialX_Image".to_string() 
                    },
                    WorkspaceMenuItem::Node { 
                        name: "Noise".to_string(), 
                        node_type: "MaterialX_Noise".to_string() 
                    },
                ],
            },
            WorkspaceMenuItem::Category {
                name: "Output".to_string(),
                items: vec![
                    WorkspaceMenuItem::Node { 
                        name: "3D View".to_string(), 
                        node_type: "MaterialX_3DView".to_string() 
                    },
                    WorkspaceMenuItem::Node { 
                        name: "2D View".to_string(), 
                        node_type: "MaterialX_2DView".to_string() 
                    },
                ],
            },
        ]
    }
    
    /// Get menu structure based on workspace path
    pub fn get_menu_for_workspace(workspace_id: Option<&str>) -> Vec<WorkspaceMenuItem> {
        match workspace_id {
            None => Self::get_root_menu(),
            Some("3d") => Self::get_3d_workspace_menu(),
            Some("materialx") => Self::get_materialx_workspace_menu(),
            _ => Self::get_root_menu(), // Fallback to root
        }
    }
}