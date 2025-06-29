//! USD Material and Shader nodes

use egui::Color32;
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition, ProcessingCost};
use super::usd_engine::with_usd_engine;

/// Creates a USD Material primitive
#[derive(Default)]
pub struct USDMaterial;

/// Creates a USD Shader primitive (Preview Surface, UsdPreviewSurface)
#[derive(Default)]
pub struct USDPreviewSurface;

/// Creates a USD Texture primitive (UsdUVTexture)
#[derive(Default)]
pub struct USDTexture;

impl USDMaterial {
    /// Execute the USD Material creation operation
    pub fn execute(node: &Node) -> Result<String, String> {
        let stage_id = "default_stage";
        let prim_path = format!("/material_{}", node.id);
        
        with_usd_engine(|engine| {
            match engine.create_material(stage_id, &prim_path) {
                Ok(prim) => {
                    println!("✓ Created USD material: {} in stage {}", prim.path, prim.stage_id);
                    Ok(prim.path)
                }
                Err(e) => {
                    eprintln!("✗ Failed to create USD material: {}", e);
                    Err(e)
                }
            }
        })
    }
}

impl USDPreviewSurface {
    /// Execute the USD Preview Surface shader creation operation
    pub fn execute(node: &Node) -> Result<String, String> {
        let stage_id = "default_stage";
        let prim_path = format!("/preview_surface_{}", node.id);
        
        // Default PBR values
        let diffuse_color = [0.8, 0.8, 0.8]; // Light gray
        let metallic = 0.0;
        let roughness = 0.5;
        let specular = 0.5;
        
        with_usd_engine(|engine| {
            match engine.create_preview_surface(stage_id, &prim_path, diffuse_color, metallic, roughness, specular) {
                Ok(prim) => {
                    println!("✓ Created USD preview surface: {} in stage {}", prim.path, prim.stage_id);
                    Ok(prim.path)
                }
                Err(e) => {
                    eprintln!("✗ Failed to create USD preview surface: {}", e);
                    Err(e)
                }
            }
        })
    }
}

impl USDTexture {
    /// Execute the USD Texture creation operation
    pub fn execute(node: &Node) -> Result<String, String> {
        let stage_id = "default_stage";
        let prim_path = format!("/texture_{}", node.id);
        let file_path = "textures/default.jpg"; // Default texture path
        
        with_usd_engine(|engine| {
            match engine.create_texture(stage_id, &prim_path, file_path) {
                Ok(prim) => {
                    println!("✓ Created USD texture: {} in stage {}", prim.path, prim.stage_id);
                    Ok(prim.path)
                }
                Err(e) => {
                    eprintln!("✗ Failed to create USD texture: {}", e);
                    Err(e)
                }
            }
        })
    }
}

impl NodeFactory for USDMaterial {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "USD_Material",
            "USD Material",
            NodeCategory::new(&["3D", "USD", "Materials"]),
            "Creates a USD Material primitive for shading"
        )
        .with_color(Color32::from_rgb(150, 100, 200))
        .with_icon("🎨")
        .with_inputs(vec![
            PortDefinition::required("Stage", DataType::Any)
                .with_description("USD Stage reference"),
            PortDefinition::required("Path", DataType::String)
                .with_description("Material path (e.g., /World/Materials/Metal)"),
            PortDefinition::optional("Surface Shader", DataType::Any)
                .with_description("Surface shader (e.g., PreviewSurface)"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Material", DataType::Any)
                .with_description("USD Material prim"),
            PortDefinition::required("Stage", DataType::Any)
                .with_description("Pass-through stage reference"),
        ])
        .with_workspace_compatibility(vec!["3D", "USD"])
        .with_tags(vec!["usd", "3d", "material", "shading"])
        .with_processing_cost(ProcessingCost::Medium)
    }
}

impl NodeFactory for USDPreviewSurface {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "USD_PreviewSurface",
            "Preview Surface",
            NodeCategory::new(&["3D", "USD", "Shaders"]),
            "Creates a USD Preview Surface shader (PBR)"
        )
        .with_color(Color32::from_rgb(100, 150, 200))
        .with_icon("🌐")
        .with_inputs(vec![
            PortDefinition::required("Stage", DataType::Any)
                .with_description("USD Stage reference"),
            PortDefinition::required("Path", DataType::String)
                .with_description("Shader path (e.g., /World/Shaders/Surface)"),
            PortDefinition::optional("Diffuse Color", DataType::Vector3)
                .with_description("Base color RGB (default: 0.8, 0.8, 0.8)"),
            PortDefinition::optional("Metallic", DataType::Float)
                .with_description("Metallic factor 0-1 (default: 0.0)"),
            PortDefinition::optional("Roughness", DataType::Float)
                .with_description("Roughness factor 0-1 (default: 0.5)"),
            PortDefinition::optional("Specular", DataType::Float)
                .with_description("Specular factor 0-1 (default: 0.5)"),
            PortDefinition::optional("Diffuse Texture", DataType::Any)
                .with_description("Diffuse texture input"),
            PortDefinition::optional("Normal Texture", DataType::Any)
                .with_description("Normal map texture input"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Surface", DataType::Any)
                .with_description("Surface shader output"),
            PortDefinition::required("Stage", DataType::Any)
                .with_description("Pass-through stage reference"),
        ])
        .with_workspace_compatibility(vec!["3D", "USD"])
        .with_tags(vec!["usd", "3d", "shader", "pbr"])
        .with_processing_cost(ProcessingCost::Medium)
    }
}

impl NodeFactory for USDTexture {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "USD_Texture",
            "USD Texture",
            NodeCategory::new(&["3D", "USD", "Textures"]),
            "Creates a USD Texture node (UsdUVTexture)"
        )
        .with_color(Color32::from_rgb(200, 150, 50))
        .with_icon("🖼️")
        .with_inputs(vec![
            PortDefinition::required("Stage", DataType::Any)
                .with_description("USD Stage reference"),
            PortDefinition::required("Path", DataType::String)
                .with_description("Texture path (e.g., /World/Textures/Diffuse)"),
            PortDefinition::required("File", DataType::String)
                .with_description("Texture file path"),
            PortDefinition::optional("UV Coordinates", DataType::Vector3)
                .with_description("UV coordinate input"),
            PortDefinition::optional("Wrap S", DataType::String)
                .with_description("S wrap mode (repeat, clamp, mirror)"),
            PortDefinition::optional("Wrap T", DataType::String)
                .with_description("T wrap mode (repeat, clamp, mirror)"),
        ])
        .with_outputs(vec![
            PortDefinition::required("RGB", DataType::Vector3)
                .with_description("RGB color output"),
            PortDefinition::required("R", DataType::Float)
                .with_description("Red channel output"),
            PortDefinition::required("G", DataType::Float)
                .with_description("Green channel output"),
            PortDefinition::required("B", DataType::Float)
                .with_description("Blue channel output"),
            PortDefinition::required("A", DataType::Float)
                .with_description("Alpha channel output"),
            PortDefinition::required("Stage", DataType::Any)
                .with_description("Pass-through stage reference"),
        ])
        .with_workspace_compatibility(vec!["3D", "USD"])
        .with_tags(vec!["usd", "3d", "texture", "uv"])
        .with_processing_cost(ProcessingCost::Low)
    }
}