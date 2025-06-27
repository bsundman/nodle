//! 3D Viewport output node

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// 3D Viewport output node
#[derive(Default)]
pub struct ViewportNode3D;

impl NodeFactory for ViewportNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "3D_Viewport",
            display_name: "Viewport",
            category: NodeCategory::new(&["3D", "Output"]),
            description: "Renders 3D scene to viewport for display",
            color: Color32::from_rgb(100, 200, 100), // Green-ish for output nodes
            inputs: vec![
                PortDefinition::required("Scene", DataType::Any)
                    .with_description("Scene input for rendering"),
                PortDefinition::optional("Camera", DataType::Any)
                    .with_description("Camera input for view settings"),
            ],
            outputs: vec![
                // Viewport nodes are typically endpoints with no outputs
            ],
        }
    }
}