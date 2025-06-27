//! USD Mesh node - creates a mesh primitive

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Creates a USD Mesh primitive
#[derive(Default)]
pub struct USDMesh;

impl NodeFactory for USDMesh {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "USD_Mesh",
            display_name: "USD Mesh",
            category: NodeCategory::new(&["3D", "USD", "Primitives"]),
            description: "Creates a USD mesh primitive from vertex and face data",
            color: Color32::from_rgb(200, 150, 100), // Orange-brown for USD nodes
            inputs: vec![
                PortDefinition::required("Stage", DataType::Any)
                    .with_description("USD Stage reference"),
                PortDefinition::required("Path", DataType::String)
                    .with_description("Prim path (e.g., /World/MyMesh)"),
                PortDefinition::required("Points", DataType::Any)
                    .with_description("Vertex positions array"),
                PortDefinition::required("Face Counts", DataType::Any)
                    .with_description("Vertices per face array"),
                PortDefinition::required("Face Indices", DataType::Any)
                    .with_description("Vertex indices for faces"),
                PortDefinition::optional("Normals", DataType::Any)
                    .with_description("Vertex normals (optional)"),
                PortDefinition::optional("UVs", DataType::Any)
                    .with_description("Texture coordinates (optional)"),
            ],
            outputs: vec![
                PortDefinition::required("Prim", DataType::Any)
                    .with_description("USD Mesh prim"),
                PortDefinition::required("Stage", DataType::Any)
                    .with_description("Pass-through stage reference"),
            ],
        }
    }
}