//! USD Light node - creates light primitives

use egui::Color32;
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition, ProcessingCost};
use super::usd_engine::with_usd_engine;

/// Creates USD Light primitives (Distant, Sphere, Rect, etc.)
#[derive(Default)]
pub struct USDDistantLight;

#[derive(Default)]
pub struct USDSphereLight;

#[derive(Default)]
pub struct USDRectLight;

impl USDDistantLight {
    /// Execute the USD Distant Light creation operation  
    pub fn execute(node: &Node) -> Result<String, String> {
        let stage_id = "default_stage";
        let prim_path = format!("/distant_light_{}", node.id);
        let intensity = 1.0;
        let angle = 0.53; // Angular size in degrees (sun-like)
        
        with_usd_engine(|engine| {
            match engine.create_distant_light(stage_id, &prim_path, intensity, angle) {
                Ok(prim) => {
                    println!("✓ Created USD distant light: {} in stage {}", prim.path, prim.stage_id);
                    Ok(prim.path)
                }
                Err(e) => {
                    eprintln!("✗ Failed to create USD distant light: {}", e);
                    Err(e)
                }
            }
        })
    }
}

impl USDSphereLight {
    /// Execute the USD Sphere Light creation operation
    pub fn execute(node: &Node) -> Result<String, String> {
        let stage_id = "default_stage";
        let prim_path = format!("/sphere_light_{}", node.id);
        let intensity = 1.0;
        let radius = 1.0;
        
        with_usd_engine(|engine| {
            match engine.create_sphere_light(stage_id, &prim_path, intensity, radius) {
                Ok(prim) => {
                    println!("✓ Created USD sphere light: {} in stage {}", prim.path, prim.stage_id);
                    Ok(prim.path)
                }
                Err(e) => {
                    eprintln!("✗ Failed to create USD sphere light: {}", e);
                    Err(e)
                }
            }
        })
    }
}

impl USDRectLight {
    /// Execute the USD Rect Light creation operation
    pub fn execute(node: &Node) -> Result<String, String> {
        let stage_id = "default_stage";
        let prim_path = format!("/rect_light_{}", node.id);
        let intensity = 1.0;
        let width = 2.0;
        let height = 2.0;
        
        with_usd_engine(|engine| {
            match engine.create_rect_light(stage_id, &prim_path, intensity, width, height) {
                Ok(prim) => {
                    println!("✓ Created USD rect light: {} in stage {}", prim.path, prim.stage_id);
                    Ok(prim.path)
                }
                Err(e) => {
                    eprintln!("✗ Failed to create USD rect light: {}", e);
                    Err(e)
                }
            }
        })
    }
}

impl NodeFactory for USDDistantLight {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "USD_DistantLight",
            "Distant Light",
            NodeCategory::new(&["3D", "USD", "Lighting"]),
            "Creates a USD distant light (directional, like sun)"
        )
        .with_color(Color32::from_rgb(255, 220, 120))
        .with_icon("💫")
        .with_inputs(vec![
            PortDefinition::required("Stage", DataType::Any)
                .with_description("USD Stage reference"),
            PortDefinition::required("Path", DataType::String)
                .with_description("Prim path (e.g., /World/SunLight)"),
            PortDefinition::optional("Intensity", DataType::Float)
                .with_description("Light intensity (default: 1.0)"),
            PortDefinition::optional("Angle", DataType::Float)
                .with_description("Angular size in degrees (default: 0.53)"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Prim", DataType::Any)
                .with_description("USD Distant Light prim"),
            PortDefinition::required("Stage", DataType::Any)
                .with_description("Pass-through stage reference"),
        ])
        .with_workspace_compatibility(vec!["3D", "USD"])
        .with_tags(vec!["usd", "3d", "lighting", "directional"])
        .with_processing_cost(ProcessingCost::Low)
    }
}

impl NodeFactory for USDSphereLight {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "USD_SphereLight",
            "Sphere Light",
            NodeCategory::new(&["3D", "USD", "Lighting"]),
            "Creates a USD sphere light (omnidirectional point light)"
        )
        .with_color(Color32::from_rgb(255, 220, 120))
        .with_icon("💡")
        .with_inputs(vec![
            PortDefinition::required("Stage", DataType::Any)
                .with_description("USD Stage reference"),
            PortDefinition::required("Path", DataType::String)
                .with_description("Prim path (e.g., /World/PointLight)"),
            PortDefinition::optional("Intensity", DataType::Float)
                .with_description("Light intensity (default: 1.0)"),
            PortDefinition::optional("Radius", DataType::Float)
                .with_description("Light radius (default: 1.0)"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Prim", DataType::Any)
                .with_description("USD Sphere Light prim"),
            PortDefinition::required("Stage", DataType::Any)
                .with_description("Pass-through stage reference"),
        ])
        .with_workspace_compatibility(vec!["3D", "USD"])
        .with_tags(vec!["usd", "3d", "lighting", "point"])
        .with_processing_cost(ProcessingCost::Low)
    }
}

impl NodeFactory for USDRectLight {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "USD_RectLight",
            "Rect Light",
            NodeCategory::new(&["3D", "USD", "Lighting"]),
            "Creates a USD rectangular area light"
        )
        .with_color(Color32::from_rgb(255, 220, 120))
        .with_icon("🔆")
        .with_inputs(vec![
            PortDefinition::required("Stage", DataType::Any)
                .with_description("USD Stage reference"),
            PortDefinition::required("Path", DataType::String)
                .with_description("Prim path (e.g., /World/AreaLight)"),
            PortDefinition::optional("Intensity", DataType::Float)
                .with_description("Light intensity (default: 1.0)"),
            PortDefinition::optional("Width", DataType::Float)
                .with_description("Light width (default: 2.0)"),
            PortDefinition::optional("Height", DataType::Float)
                .with_description("Light height (default: 2.0)"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Prim", DataType::Any)
                .with_description("USD Rect Light prim"),
            PortDefinition::required("Stage", DataType::Any)
                .with_description("Pass-through stage reference"),
        ])
        .with_workspace_compatibility(vec!["3D", "USD"])
        .with_tags(vec!["usd", "3d", "lighting", "area"])
        .with_processing_cost(ProcessingCost::Low)
    }
}