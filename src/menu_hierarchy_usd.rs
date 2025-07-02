//! USD Menu Hierarchy - Categorized USD node organization

use crate::menu_hierarchy::WorkspaceMenuItem;

/// Get USD menu hierarchy with proper categorization
pub fn get_usd_menu_hierarchy() -> Vec<WorkspaceMenuItem> {
    vec![
        WorkspaceMenuItem::Category {
            name: "USD".to_string(),
            items: vec![
                // Stage Management
                WorkspaceMenuItem::Category {
                    name: "Stage".to_string(),
                    items: vec![
                        WorkspaceMenuItem::Node {
                            name: "Create Stage".to_string(),
                            node_type: "USD_Stage_Create".to_string(),
                        },
                        WorkspaceMenuItem::Node {
                            name: "Load Stage".to_string(),
                            node_type: "USD_Stage_Load".to_string(),
                        },
                        WorkspaceMenuItem::Node {
                            name: "Save Stage".to_string(),
                            node_type: "USD_Stage_Save".to_string(),
                        },
                        WorkspaceMenuItem::Node {
                            name: "Export Stage".to_string(),
                            node_type: "USD_Stage_Export".to_string(),
                        },
                        WorkspaceMenuItem::Node {
                            name: "Clear Stage".to_string(),
                            node_type: "USD_Stage_Clear".to_string(),
                        },
                    ],
                },
                // Geometry Primitives
                WorkspaceMenuItem::Category {
                    name: "Geometry".to_string(),
                    items: vec![
                        WorkspaceMenuItem::Category {
                            name: "Primitives".to_string(),
                            items: vec![
                                WorkspaceMenuItem::Node {
                                    name: "Sphere".to_string(),
                                    node_type: "USD_Geometry_Sphere".to_string(),
                                },
                                WorkspaceMenuItem::Node {
                                    name: "Cube".to_string(),
                                    node_type: "USD_Geometry_Cube".to_string(),
                                },
                                WorkspaceMenuItem::Node {
                                    name: "Cylinder".to_string(),
                                    node_type: "USD_Geometry_Cylinder".to_string(),
                                },
                                WorkspaceMenuItem::Node {
                                    name: "Cone".to_string(),
                                    node_type: "USD_Geometry_Cone".to_string(),
                                },
                                WorkspaceMenuItem::Node {
                                    name: "Plane".to_string(),
                                    node_type: "USD_Geometry_Plane".to_string(),
                                },
                                WorkspaceMenuItem::Node {
                                    name: "Capsule".to_string(),
                                    node_type: "USD_Geometry_Capsule".to_string(),
                                },
                                WorkspaceMenuItem::Node {
                                    name: "Torus".to_string(),
                                    node_type: "USD_Geometry_Torus".to_string(),
                                },
                            ],
                        },
                        WorkspaceMenuItem::Category {
                            name: "Advanced".to_string(),
                            items: vec![
                                WorkspaceMenuItem::Node {
                                    name: "Mesh".to_string(),
                                    node_type: "USD_Geometry_Mesh".to_string(),
                                },
                                WorkspaceMenuItem::Node {
                                    name: "Points".to_string(),
                                    node_type: "USD_Geometry_Points".to_string(),
                                },
                                WorkspaceMenuItem::Node {
                                    name: "Curves".to_string(),
                                    node_type: "USD_Geometry_Curves".to_string(),
                                },
                            ],
                        },
                    ],
                },
                // Transform Operations
                WorkspaceMenuItem::Category {
                    name: "Transform".to_string(),
                    items: vec![
                        WorkspaceMenuItem::Node {
                            name: "Xform".to_string(),
                            node_type: "USD_Transform_Xform".to_string(),
                        },
                        WorkspaceMenuItem::Node {
                            name: "Translate".to_string(),
                            node_type: "USD_Transform_Translate".to_string(),
                        },
                        WorkspaceMenuItem::Node {
                            name: "Rotate".to_string(),
                            node_type: "USD_Transform_Rotate".to_string(),
                        },
                        WorkspaceMenuItem::Node {
                            name: "Scale".to_string(),
                            node_type: "USD_Transform_Scale".to_string(),
                        },
                        WorkspaceMenuItem::Node {
                            name: "Matrix Transform".to_string(),
                            node_type: "USD_Transform_Matrix".to_string(),
                        },
                    ],
                },
                // Lighting
                WorkspaceMenuItem::Category {
                    name: "Lighting".to_string(),
                    items: vec![
                        WorkspaceMenuItem::Node {
                            name: "Distant Light".to_string(),
                            node_type: "USD_Lighting_DistantLight".to_string(),
                        },
                        WorkspaceMenuItem::Node {
                            name: "Rect Light".to_string(),
                            node_type: "USD_Lighting_RectLight".to_string(),
                        },
                        WorkspaceMenuItem::Node {
                            name: "Sphere Light".to_string(),
                            node_type: "USD_Lighting_SphereLight".to_string(),
                        },
                        WorkspaceMenuItem::Node {
                            name: "Cylinder Light".to_string(),
                            node_type: "USD_Lighting_CylinderLight".to_string(),
                        },
                        WorkspaceMenuItem::Node {
                            name: "Dome Light".to_string(),
                            node_type: "USD_Lighting_DomeLight".to_string(),
                        },
                        WorkspaceMenuItem::Node {
                            name: "Disk Light".to_string(),
                            node_type: "USD_Lighting_DiskLight".to_string(),
                        },
                    ],
                },
                // Shading and Materials
                WorkspaceMenuItem::Category {
                    name: "Shading".to_string(),
                    items: vec![
                        WorkspaceMenuItem::Node {
                            name: "Material".to_string(),
                            node_type: "USD_Shading_Material".to_string(),
                        },
                        WorkspaceMenuItem::Node {
                            name: "Shader".to_string(),
                            node_type: "USD_Shading_Shader".to_string(),
                        },
                        WorkspaceMenuItem::Node {
                            name: "Preview Surface".to_string(),
                            node_type: "USD_Shading_PreviewSurface".to_string(),
                        },
                        WorkspaceMenuItem::Node {
                            name: "Texture Reader".to_string(),
                            node_type: "USD_Shading_TextureReader".to_string(),
                        },
                        WorkspaceMenuItem::Node {
                            name: "Primvar Reader".to_string(),
                            node_type: "USD_Shading_PrimvarReader".to_string(),
                        },
                        WorkspaceMenuItem::Node {
                            name: "Node Graph".to_string(),
                            node_type: "USD_Shading_NodeGraph".to_string(),
                        },
                    ],
                },
                // Camera and Output
                WorkspaceMenuItem::Category {
                    name: "Camera & Output".to_string(),
                    items: vec![
                        WorkspaceMenuItem::Node {
                            name: "Camera".to_string(),
                            node_type: "USD_Camera".to_string(),
                        },
                        WorkspaceMenuItem::Node {
                            name: "Viewport".to_string(),
                            node_type: "USD_Viewport".to_string(),
                        },
                    ],
                },
                // Utilities
                WorkspaceMenuItem::Category {
                    name: "Utilities".to_string(),
                    items: vec![
                        WorkspaceMenuItem::Node {
                            name: "Set Attribute".to_string(),
                            node_type: "USD_SetAttribute".to_string(),
                        },
                        WorkspaceMenuItem::Node {
                            name: "Get Attribute".to_string(),
                            node_type: "USD_GetAttribute".to_string(),
                        },
                        WorkspaceMenuItem::Node {
                            name: "Composition".to_string(),
                            node_type: "USD_Composition".to_string(),
                        },
                    ],
                },
            ],
        },
    ]
}