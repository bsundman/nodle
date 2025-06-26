//! Centralized menu hierarchy system
//! 
//! This module provides the single source of truth for all node menu structures.
//! All menus (context menus, file menus, etc.) should use this system to ensure consistency.

use crate::context::ContextMenuItem;

/// Central registry for all menu hierarchies
pub struct GlobalMenuHierarchy;

impl GlobalMenuHierarchy {
    /// Get the root-level menu structure
    /// This is the authoritative definition used by ALL menu systems
    pub fn get_root_menu() -> Vec<ContextMenuItem> {
        vec![
            // Standard node categories
            ContextMenuItem::Category {
                name: "Math".to_string(),
                items: vec![
                    ContextMenuItem::Node { name: "Add".to_string(), node_type: "Add".to_string() },
                    ContextMenuItem::Node { name: "Subtract".to_string(), node_type: "Subtract".to_string() },
                    ContextMenuItem::Node { name: "Multiply".to_string(), node_type: "Multiply".to_string() },
                    ContextMenuItem::Node { name: "Divide".to_string(), node_type: "Divide".to_string() },
                ],
            },
            ContextMenuItem::Category {
                name: "Logic".to_string(),
                items: vec![
                    ContextMenuItem::Node { name: "AND".to_string(), node_type: "AND".to_string() },
                    ContextMenuItem::Node { name: "OR".to_string(), node_type: "OR".to_string() },
                    ContextMenuItem::Node { name: "NOT".to_string(), node_type: "NOT".to_string() },
                ],
            },
            ContextMenuItem::Category {
                name: "Data".to_string(),
                items: vec![
                    ContextMenuItem::Node { name: "Constant".to_string(), node_type: "Constant".to_string() },
                    ContextMenuItem::Node { name: "Variable".to_string(), node_type: "Variable".to_string() },
                ],
            },
            ContextMenuItem::Category {
                name: "Output".to_string(),
                items: vec![
                    ContextMenuItem::Node { name: "Print".to_string(), node_type: "Print".to_string() },
                    ContextMenuItem::Node { name: "Debug".to_string(), node_type: "Debug".to_string() },
                ],
            },
            // Context entry points
            ContextMenuItem::Category {
                name: "Contexts".to_string(),
                items: vec![
                    ContextMenuItem::Node {
                        name: "3D".to_string(),
                        node_type: "CONTEXT:3D".to_string(),
                    },
                ],
            },
        ]
    }
    
    /// Get menu structure for 3D context
    pub fn get_3d_context_menu() -> Vec<ContextMenuItem> {
        vec![
            ContextMenuItem::Category {
                name: "Transform".to_string(),
                items: vec![
                    ContextMenuItem::Node { 
                        name: "Translate".to_string(), 
                        node_type: "3D_Translate".to_string() 
                    },
                    ContextMenuItem::Node { 
                        name: "Rotate".to_string(), 
                        node_type: "3D_Rotate".to_string() 
                    },
                    ContextMenuItem::Node { 
                        name: "Scale".to_string(), 
                        node_type: "3D_Scale".to_string() 
                    },
                ],
            },
            ContextMenuItem::Category {
                name: "Geometry".to_string(),
                items: vec![
                    ContextMenuItem::Node { 
                        name: "Cube".to_string(), 
                        node_type: "3D_Cube".to_string() 
                    },
                    ContextMenuItem::Node { 
                        name: "Sphere".to_string(), 
                        node_type: "3D_Sphere".to_string() 
                    },
                    ContextMenuItem::Node { 
                        name: "Plane".to_string(), 
                        node_type: "3D_Plane".to_string() 
                    },
                ],
            },
            ContextMenuItem::Category {
                name: "Lighting".to_string(),
                items: vec![
                    ContextMenuItem::Node { 
                        name: "Point Light".to_string(), 
                        node_type: "3D_PointLight".to_string() 
                    },
                    ContextMenuItem::Node { 
                        name: "Directional Light".to_string(), 
                        node_type: "3D_DirectionalLight".to_string() 
                    },
                    ContextMenuItem::Node { 
                        name: "Spot Light".to_string(), 
                        node_type: "3D_SpotLight".to_string() 
                    },
                ],
            },
            ContextMenuItem::Category {
                name: "Output".to_string(),
                items: vec![
                    ContextMenuItem::Node { 
                        name: "Viewport".to_string(), 
                        node_type: "3D_Viewport".to_string() 
                    },
                ],
            },
            ContextMenuItem::Category {
                name: "Materials".to_string(),
                items: vec![
                    ContextMenuItem::Subcontext { 
                        name: "MaterialX".to_string(), 
                        context_id: "materialx".to_string() 
                    },
                ],
            },
        ]
    }
    
    /// Get menu structure for MaterialX context
    pub fn get_materialx_context_menu() -> Vec<ContextMenuItem> {
        vec![
            ContextMenuItem::Category {
                name: "Shading".to_string(),
                items: vec![
                    ContextMenuItem::Node { 
                        name: "Standard Surface".to_string(), 
                        node_type: "MaterialX_StandardSurface".to_string() 
                    },
                    ContextMenuItem::Node { 
                        name: "Mix".to_string(), 
                        node_type: "MaterialX_Mix".to_string() 
                    },
                ],
            },
            ContextMenuItem::Category {
                name: "Texture".to_string(),
                items: vec![
                    ContextMenuItem::Node { 
                        name: "Image".to_string(), 
                        node_type: "MaterialX_Image".to_string() 
                    },
                    ContextMenuItem::Node { 
                        name: "Noise".to_string(), 
                        node_type: "MaterialX_Noise".to_string() 
                    },
                ],
            },
            ContextMenuItem::Category {
                name: "Output".to_string(),
                items: vec![
                    ContextMenuItem::Node { 
                        name: "3D View".to_string(), 
                        node_type: "MaterialX_3DView".to_string() 
                    },
                    ContextMenuItem::Node { 
                        name: "2D View".to_string(), 
                        node_type: "MaterialX_2DView".to_string() 
                    },
                ],
            },
        ]
    }
    
    /// Get menu structure based on context path
    pub fn get_menu_for_context(context_id: Option<&str>) -> Vec<ContextMenuItem> {
        match context_id {
            None => Self::get_root_menu(),
            Some("3d") => Self::get_3d_context_menu(),
            Some("materialx") => Self::get_materialx_context_menu(),
            _ => Self::get_root_menu(), // Fallback to root
        }
    }
}