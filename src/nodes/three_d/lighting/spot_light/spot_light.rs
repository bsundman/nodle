//! 3D Spot Light node - bare minimum initialization and metadata

use egui::Color32;
use crate::nodes::{NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// 3D Spot Light node - main entry point
#[derive(Default)]
pub struct SpotLightNode3D;

impl NodeFactory for SpotLightNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "3D_SpotLight",
            "Spot Light",
            NodeCategory::new(&["3D", "Lighting"]),
            "Creates a spot light with cone-shaped illumination"
        )
        .with_color(Color32::from_rgb(255, 255, 150)) // Yellow-ish for lights
        .with_icon("🔦")
        .with_inputs(vec![
            PortDefinition::optional("Transform", DataType::Any)
                .with_description("Optional transform matrix to position the light"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Light", DataType::Any)
                .with_description("Light output for scene"),
        ])
        .with_tags(vec!["3d", "lighting", "spot", "cone", "directional"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Low)
        .with_workspace_compatibility(vec!["3D", "USD", "MaterialX"])
    }
}