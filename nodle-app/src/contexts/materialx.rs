//! MaterialX context with integrated node registry

use crate::context::{Context, ContextMenuItem};
use crate::nodes::factory::{NodeRegistry, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};
use crate::nodes::Node;
use egui::{Color32, Pos2};

/// MaterialX context that integrates with the factory system
pub struct MaterialXContext {
    node_registry: NodeRegistry,
}

impl MaterialXContext {
    pub fn new() -> Self {
        let mut registry = NodeRegistry::new();
        
        // Register MaterialX-specific nodes
        registry.register::<MaterialXNoiseNode>();
        registry.register::<MaterialXTextureNode>();
        registry.register::<MaterialXMixNode>();
        registry.register::<MaterialXStandardSurfaceNode>();
        registry.register::<MaterialX3DViewNode>();
        registry.register::<MaterialX2DViewNode>();
        
        Self {
            node_registry: registry,
        }
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
                    ContextMenuItem::Category {
                        name: "Shading".to_string(),
                        items: vec![
                            ContextMenuItem::Node {
                                name: "Standard Surface".to_string(),
                                node_type: "MaterialX_StandardSurface".to_string(),
                            },
                        ],
                    },
                    ContextMenuItem::Category {
                        name: "Texture".to_string(),
                        items: vec![
                            ContextMenuItem::Node {
                                name: "Noise".to_string(),
                                node_type: "MaterialX_Noise".to_string(),
                            },
                            ContextMenuItem::Node {
                                name: "Image".to_string(),
                                node_type: "MaterialX_Texture".to_string(),
                            },
                        ],
                    },
                    ContextMenuItem::Category {
                        name: "Utilities".to_string(),
                        items: vec![
                            ContextMenuItem::Node {
                                name: "Mix".to_string(),
                                node_type: "MaterialX_Mix".to_string(),
                            },
                        ],
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
        // First try MaterialX-specific nodes
        if let Some(node) = self.node_registry.create_node(node_type, position) {
            return Some(node);
        }
        
        // Fall back to generic nodes if compatible
        if self.is_generic_node_compatible(node_type) {
            // Create using default registry for compatible generic nodes
            let default_registry = NodeRegistry::default();
            return default_registry.create_node(node_type, position);
        }
        
        None
    }
}

// MaterialX Node Implementations using enhanced factory system

/// MaterialX Noise node - procedural noise generation
#[derive(Default)]
pub struct MaterialXNoiseNode;

impl NodeFactory for MaterialXNoiseNode {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "MaterialX_Noise",
            display_name: "Noise",
            category: NodeCategory::materialx_texture(),
            description: "Generates procedural noise patterns for MaterialX shading",
            color: Color32::from_rgb(80, 60, 40), // Brown for texture nodes
            inputs: vec![
                PortDefinition::optional("Position", DataType::Vector3)
                    .with_description("3D position for noise sampling"),
                PortDefinition::optional("Scale", DataType::Float)
                    .with_description("Scale factor for noise frequency"),
            ],
            outputs: vec![
                PortDefinition::required("Noise", DataType::Float)
                    .with_description("Generated noise value"),
            ],
        }
    }
}

/// MaterialX Texture node - image sampling
#[derive(Default)]
pub struct MaterialXTextureNode;

impl NodeFactory for MaterialXTextureNode {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "MaterialX_Texture",
            display_name: "Image",
            category: NodeCategory::materialx_texture(),
            description: "Samples color values from image textures",
            color: Color32::from_rgb(80, 60, 40), // Brown for texture nodes
            inputs: vec![
                PortDefinition::required("UV", DataType::Vector3)
                    .with_description("UV coordinates for texture sampling"),
                PortDefinition::optional("File", DataType::String)
                    .with_description("Path to image file"),
            ],
            outputs: vec![
                PortDefinition::required("Color", DataType::Color)
                    .with_description("Sampled color from texture"),
            ],
        }
    }
}

/// MaterialX Mix node - blends between two inputs
#[derive(Default)]
pub struct MaterialXMixNode;

impl NodeFactory for MaterialXMixNode {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "MaterialX_Mix",
            display_name: "Mix",
            category: NodeCategory::new(&["MaterialX", "Utilities"]),
            description: "Linearly interpolates between two input values",
            color: Color32::from_rgb(60, 80, 60), // Green for utility nodes
            inputs: vec![
                PortDefinition::required("Input1", DataType::Color)
                    .with_description("First input value"),
                PortDefinition::required("Input2", DataType::Color)
                    .with_description("Second input value"),
                PortDefinition::required("Mix", DataType::Float)
                    .with_description("Blend factor (0=Input1, 1=Input2)"),
            ],
            outputs: vec![
                PortDefinition::required("Output", DataType::Color)
                    .with_description("Blended result"),
            ],
        }
    }
}

/// MaterialX Standard Surface - main shading node
#[derive(Default)]
pub struct MaterialXStandardSurfaceNode;

impl NodeFactory for MaterialXStandardSurfaceNode {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "MaterialX_StandardSurface",
            display_name: "Standard Surface",
            category: NodeCategory::materialx_shading(),
            description: "Physically-based surface shading model following MaterialX standard",
            color: Color32::from_rgb(60, 60, 80), // Blue for shading nodes
            inputs: vec![
                PortDefinition::optional("base_color", DataType::Color)
                    .with_description("Base color of the surface"),
                PortDefinition::optional("metallic", DataType::Float)
                    .with_description("Metallic factor (0=dielectric, 1=metallic)"),
                PortDefinition::optional("roughness", DataType::Float)
                    .with_description("Surface roughness (0=mirror, 1=rough)"),
                PortDefinition::optional("normal", DataType::Vector3)
                    .with_description("Surface normal direction"),
            ],
            outputs: vec![
                PortDefinition::required("surface", DataType::Any)
                    .with_description("Surface shading result"),
            ],
        }
    }
}

/// MaterialX 3D View - outputs to 3D viewport
#[derive(Default)]
pub struct MaterialX3DViewNode;

impl NodeFactory for MaterialX3DViewNode {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "MaterialX_3DView",
            display_name: "3D View",
            category: NodeCategory::new(&["MaterialX", "Output"]),
            description: "Outputs shading result to 3D viewport for preview",
            color: Color32::from_rgb(80, 40, 40), // Red for output nodes
            inputs: vec![
                PortDefinition::required("surface", DataType::Any)
                    .with_description("Surface shading to display"),
            ],
            outputs: vec![], // Terminal node
        }
    }
}

/// MaterialX 2D View - outputs to 2D texture preview
#[derive(Default)]
pub struct MaterialX2DViewNode;

impl NodeFactory for MaterialX2DViewNode {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "MaterialX_2DView",
            display_name: "2D View",
            category: NodeCategory::new(&["MaterialX", "Output"]),
            description: "Outputs shading result to 2D texture preview",
            color: Color32::from_rgb(80, 40, 40), // Red for output nodes
            inputs: vec![
                PortDefinition::required("surface", DataType::Any)
                    .with_description("Surface shading to display"),
            ],
            outputs: vec![], // Terminal node
        }
    }
}

impl Default for MaterialXContext {
    fn default() -> Self {
        Self::new()
    }
}