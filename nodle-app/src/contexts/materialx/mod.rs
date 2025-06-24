//! MaterialX context for shader editing

use crate::context::{Context, ContextMenuItem};
use egui::Pos2;
use nodle_core::Node;

pub mod nodes;

/// MaterialX context for shader node editing
pub struct MaterialXContext;

impl MaterialXContext {
    pub fn new() -> Self {
        Self
    }
}

impl Context for MaterialXContext {
    fn id(&self) -> &'static str {
        "materialx"
    }
    
    fn display_name(&self) -> &'static str {
        "MaterialX"
    }
    
    fn get_menu_structure(&self) -> Vec<ContextMenuItem> {
        vec![
            ContextMenuItem::Category {
                name: "MaterialX".to_string(),
                items: vec![
                    ContextMenuItem::Node {
                        name: "Noise".to_string(),
                        node_type: "MaterialX_Noise".to_string(),
                    },
                    ContextMenuItem::Node {
                        name: "Texture".to_string(),
                        node_type: "MaterialX_Texture".to_string(),
                    },
                    ContextMenuItem::Node {
                        name: "Mix".to_string(),
                        node_type: "MaterialX_Mix".to_string(),
                    },
                    ContextMenuItem::Node {
                        name: "Standard Surface".to_string(),
                        node_type: "MaterialX_StandardSurface".to_string(),
                    },
                    ContextMenuItem::Category {
                        name: "Output".to_string(),
                        items: vec![
                            ContextMenuItem::Node {
                                name: "3D View".to_string(),
                                node_type: "MaterialX_3DView".to_string(),
                            },
                            ContextMenuItem::Node {
                                name: "2D View".to_string(),
                                node_type: "MaterialX_2DView".to_string(),
                            },
                        ],
                    },
                ],
            },
        ]
    }
    
    fn is_generic_node_compatible(&self, node_type: &str) -> bool {
        // Whitelist of generic nodes compatible with MaterialX
        matches!(node_type, 
            "Add" | "Subtract" | "Multiply" | "Divide" |  // Math operations
            "Constant"  // Data nodes that make sense for shaders
        )
    }
    
    fn create_context_node(&self, node_type: &str, position: Pos2) -> Option<Node> {
        match node_type {
            "MaterialX_Noise" => Some(nodes::noise::NoiseNode::create(position)),
            "MaterialX_Texture" => Some(nodes::texture::TextureNode::create(position)),
            "MaterialX_Mix" => Some(nodes::mix::MixNode::create(position)),
            "MaterialX_StandardSurface" => Some(nodes::standard_surface::StandardSurfaceNode::create(position)),
            "MaterialX_3DView" => Some(nodes::output::view_3d::View3DNode::create(position)),
            "MaterialX_2DView" => Some(nodes::output::view_2d::View2DNode::create(position)),
            _ => None,
        }
    }
}