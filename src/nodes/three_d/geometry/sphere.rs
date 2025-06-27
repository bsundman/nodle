//! 3D Sphere geometry node

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

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