//! USD Layer Composition nodes - SubLayer, Reference, Payload

use egui::Color32;
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition, ProcessingCost};
use super::usd_engine::with_usd_engine;

/// Adds a SubLayer to a USD stage (layer composition)
#[derive(Default)]
pub struct USDSubLayer;

/// Adds a Reference to a USD prim (asset referencing) 
#[derive(Default)]
pub struct USDReference;

/// Adds a Payload to a USD prim (deferred loading)
#[derive(Default)]
pub struct USDPayload;

impl USDSubLayer {
    /// Execute the USD SubLayer operation
    pub fn execute(node: &Node) -> Result<String, String> {
        let stage_id = "default_stage";
        let layer_path = format!("layers/sublayer_{}.usda", node.id);
        let layer_offset = 0.0; // Time offset for animation layers
        
        with_usd_engine(|engine| {
            match engine.add_sublayer(stage_id, &layer_path, layer_offset) {
                Ok(info) => {
                    println!("✓ Added SubLayer to stage {}: {}", stage_id, info);
                    Ok(info)
                }
                Err(e) => {
                    eprintln!("✗ Failed to add SubLayer: {}", e);
                    Err(e)
                }
            }
        })
    }
}

impl USDReference {
    /// Execute the USD Reference operation
    pub fn execute(node: &Node) -> Result<String, String> {
        let stage_id = "default_stage";
        let prim_path = format!("/references/ref_{}", node.id);
        let asset_path = "assets/character.usda"; // Referenced asset
        let prim_target = "/Character"; // Target prim in asset
        
        with_usd_engine(|engine| {
            match engine.add_reference(stage_id, &prim_path, asset_path, Some(prim_target)) {
                Ok(info) => {
                    println!("✓ Added Reference to {}: {}", prim_path, info);
                    Ok(info)
                }
                Err(e) => {
                    eprintln!("✗ Failed to add Reference: {}", e);
                    Err(e)
                }
            }
        })
    }
}

impl USDPayload {
    /// Execute the USD Payload operation
    pub fn execute(node: &Node) -> Result<String, String> {
        let stage_id = "default_stage";
        let prim_path = format!("/payloads/payload_{}", node.id);
        let asset_path = "assets/environment.usda"; // Payload asset
        let prim_target = "/Environment"; // Target prim in payload
        
        with_usd_engine(|engine| {
            match engine.add_payload(stage_id, &prim_path, asset_path, Some(prim_target)) {
                Ok(info) => {
                    println!("✓ Added Payload to {}: {}", prim_path, info);
                    Ok(info)
                }
                Err(e) => {
                    eprintln!("✗ Failed to add Payload: {}", e);
                    Err(e)
                }
            }
        })
    }
}

impl NodeFactory for USDSubLayer {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "USD_SubLayer",
            "SubLayer",
            NodeCategory::new(&["3D", "USD", "Composition"]),
            "Adds a sublayer to a USD stage for layer composition"
        )
        .with_color(Color32::from_rgb(180, 120, 60))
        .with_icon("📁")
        .with_inputs(vec![
            PortDefinition::required("Stage", DataType::Any)
                .with_description("USD Stage reference"),
            PortDefinition::required("Layer Path", DataType::String)
                .with_description("Path to the layer file (.usda, .usdc, .usd)"),
            PortDefinition::optional("Time Offset", DataType::Float)
                .with_description("Time offset for animation layers (default: 0.0)"),
            PortDefinition::optional("Scale", DataType::Float)
                .with_description("Time scale factor (default: 1.0)"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Stage", DataType::Any)
                .with_description("Stage with sublayer added"),
            PortDefinition::required("Layer Info", DataType::String)
                .with_description("Information about the added layer"),
        ])
        .with_workspace_compatibility(vec!["3D", "USD"])
        .with_tags(vec!["usd", "3d", "composition", "layer"])
        .with_processing_cost(ProcessingCost::Medium)
    }
}

impl NodeFactory for USDReference {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "USD_Reference",
            "Reference",
            NodeCategory::new(&["3D", "USD", "Composition"]),
            "Adds a reference to external USD asset"
        )
        .with_color(Color32::from_rgb(120, 180, 120))
        .with_icon("🔗")
        .with_inputs(vec![
            PortDefinition::required("Stage", DataType::Any)
                .with_description("USD Stage reference"),
            PortDefinition::required("Prim Path", DataType::String)
                .with_description("Path where reference will be created"),
            PortDefinition::required("Asset Path", DataType::String)
                .with_description("Path to the referenced USD asset"),
            PortDefinition::optional("Prim Target", DataType::String)
                .with_description("Specific prim in asset (default: defaultPrim)"),
            PortDefinition::optional("Layer Offset", DataType::Vector3)
                .with_description("Time offset and scale [offset, scale, 0]"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Prim", DataType::Any)
                .with_description("Reference prim created"),
            PortDefinition::required("Stage", DataType::Any)
                .with_description("Pass-through stage reference"),
        ])
        .with_workspace_compatibility(vec!["3D", "USD"])
        .with_tags(vec!["usd", "3d", "composition", "reference"])
        .with_processing_cost(ProcessingCost::Medium)
    }
}

impl NodeFactory for USDPayload {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "USD_Payload",
            "Payload",
            NodeCategory::new(&["3D", "USD", "Composition"]),
            "Adds a payload for deferred loading of heavy assets"
        )
        .with_color(Color32::from_rgb(120, 120, 180))
        .with_icon("💾")
        .with_inputs(vec![
            PortDefinition::required("Stage", DataType::Any)
                .with_description("USD Stage reference"),
            PortDefinition::required("Prim Path", DataType::String)
                .with_description("Path where payload will be created"),
            PortDefinition::required("Asset Path", DataType::String)
                .with_description("Path to the payload USD asset"),
            PortDefinition::optional("Prim Target", DataType::String)
                .with_description("Specific prim in asset (default: defaultPrim)"),
            PortDefinition::optional("Load State", DataType::Boolean)
                .with_description("Whether to load immediately (default: false)"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Prim", DataType::Any)
                .with_description("Payload prim created"),
            PortDefinition::required("Stage", DataType::Any)
                .with_description("Pass-through stage reference"),
        ])
        .with_workspace_compatibility(vec!["3D", "USD"])
        .with_tags(vec!["usd", "3d", "composition", "payload"])
        .with_processing_cost(ProcessingCost::High)
    }
}