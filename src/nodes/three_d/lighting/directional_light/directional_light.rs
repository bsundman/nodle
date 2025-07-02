//! 3D Directional Light node - bare minimum initialization and metadata

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// 3D Directional Light node - main entry point
#[derive(Default)]
pub struct DirectionalLightNode3D;

impl NodeFactory for DirectionalLightNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "3D_DirectionalLight",
            "Directional Light",
            NodeCategory::new(&["3D", "Lighting"]),
            "Creates a directional light (like sunlight)"
        )
        .with_color(Color32::from_rgb(255, 255, 150)) // Yellow-ish for lights
        .with_icon("☀️")
        .with_inputs(vec![
            PortDefinition::optional("Transform", DataType::Any)
                .with_description("Optional transform matrix to position the light"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Light", DataType::Any)
                .with_description("Light output for scene"),
        ])
        .with_tags(vec!["3d", "lighting", "directional", "sun", "parallel"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Low)
        .with_workspace_compatibility(vec!["3D", "USD", "MaterialX"])
    }
}