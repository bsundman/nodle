//! 3D geometry node implementations

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// 3D Cube geometry node
#[derive(Default)]
pub struct CubeNode3D;

impl NodeFactory for CubeNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "3D_Cube",
            display_name: "Cube",
            category: NodeCategory::new(&["3D", "Geometry"]),
            description: "Creates a 3D cube primitive",
            color: Color32::from_rgb(160, 120, 200), // Purple-ish for geometry
            inputs: vec![
                PortDefinition::required("Size", DataType::Float)
                    .with_description("Size of the cube"),
            ],
            outputs: vec![
                PortDefinition::required("Geometry", DataType::Any)
                    .with_description("Generated cube geometry"),
            ],
        }
    }
}

/// 3D Sphere geometry node
#[derive(Default)]
pub struct SphereNode3D;

impl NodeFactory for SphereNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "3D_Sphere",
            display_name: "Sphere",
            category: NodeCategory::new(&["3D", "Geometry"]),
            description: "Creates a 3D sphere primitive",
            color: Color32::from_rgb(160, 120, 200), // Purple-ish for geometry
            inputs: vec![
                PortDefinition::required("Radius", DataType::Float)
                    .with_description("Radius of the sphere"),
                PortDefinition::optional("Subdivisions", DataType::Float)
                    .with_description("Number of subdivisions for smoothness"),
            ],
            outputs: vec![
                PortDefinition::required("Geometry", DataType::Any)
                    .with_description("Generated sphere geometry"),
            ],
        }
    }
}

/// 3D Plane geometry node
#[derive(Default)]
pub struct PlaneNode3D;

impl NodeFactory for PlaneNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "3D_Plane",
            display_name: "Plane",
            category: NodeCategory::new(&["3D", "Geometry"]),
            description: "Creates a 3D plane primitive",
            color: Color32::from_rgb(160, 120, 200), // Purple-ish for geometry
            inputs: vec![
                PortDefinition::required("Width", DataType::Float)
                    .with_description("Width of the plane"),
                PortDefinition::required("Height", DataType::Float)
                    .with_description("Height of the plane"),
            ],
            outputs: vec![
                PortDefinition::required("Geometry", DataType::Any)
                    .with_description("Generated plane geometry"),
            ],
        }
    }
}