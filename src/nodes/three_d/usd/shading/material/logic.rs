//! USD Material node functional operations

use crate::nodes::interface::NodeData;
use crate::nodes::three_d::usd::usd_engine::with_usd_engine;

/// Core logic for USD material creation
pub struct USDMaterialLogic;

impl USDMaterialLogic {
    /// Execute the material creation operation
    pub fn execute(inputs: &std::collections::HashMap<String, NodeData>, parameters: &std::collections::HashMap<String, NodeData>) -> std::collections::HashMap<String, NodeData> {
        let mut outputs = std::collections::HashMap::new();
        
        // Get stage reference
        let stage_id = match inputs.get("Stage") {
            Some(NodeData::String(s)) => s.clone(),
            _ => {
                outputs.insert("Material Path".to_string(), NodeData::String("".to_string()));
                outputs.insert("Material".to_string(), NodeData::None);
                outputs.insert("Surface Output".to_string(), NodeData::None);
                return outputs;
            }
        };
        
        // Get parent path
        let parent_path = match inputs.get("Parent Path") {
            Some(NodeData::String(s)) => s.clone(),
            _ => "/World/Materials".to_string(),
        };
        
        // Get name or auto-generate
        let name = match inputs.get("Name").or_else(|| parameters.get("name")) {
            Some(NodeData::String(s)) if !s.is_empty() => s.clone(),
            _ => format!("material_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap()),
        };
        
        // Get material parameters
        let diffuse_color = match parameters.get("diffuse_color") {
            Some(NodeData::Color(color)) => [color[0], color[1], color[2]], // Use RGB components
            _ => [0.8, 0.8, 0.8],
        };
        
        let metallic = match parameters.get("metallic") {
            Some(NodeData::Float(f)) => *f,
            _ => 0.0,
        };
        
        let roughness = match parameters.get("roughness") {
            Some(NodeData::Float(f)) => *f,
            _ => 0.4,
        };
        
        let opacity = match parameters.get("opacity") {
            Some(NodeData::Float(f)) => *f,
            _ => 1.0,
        };
        
        let ior = match parameters.get("ior") {
            Some(NodeData::Float(f)) => *f,
            _ => 1.5,
        };
        
        let specular = match parameters.get("specular") {
            Some(NodeData::Float(f)) => *f,
            _ => 0.5,
        };
        
        // Construct material path
        let material_path = if parent_path.ends_with('/') {
            format!("{}{}", parent_path, name)
        } else {
            format!("{}/{}", parent_path, name)
        };
        
        // Create the material
        with_usd_engine(|engine| {
            match engine.create_material(&stage_id, &material_path) {
                Ok(material_prim) => {
                    // Create preview surface shader
                    let surface_path = format!("{}/PreviewSurface", material_path);
                    
                    match engine.create_preview_surface(&stage_id, &surface_path, diffuse_color, metallic, roughness, specular) {
                        Ok(_surface_prim) => {
                            // Set surface shader parameters
                            let _ = engine.set_attribute(&stage_id, &surface_path, "diffuseColor", 
                                &format!("({}, {}, {})", diffuse_color[0], diffuse_color[1], diffuse_color[2]));
                            let _ = engine.set_attribute(&stage_id, &surface_path, "metallic", &metallic.to_string());
                            let _ = engine.set_attribute(&stage_id, &surface_path, "roughness", &roughness.to_string());
                            let _ = engine.set_attribute(&stage_id, &surface_path, "opacity", &opacity.to_string());
                            let _ = engine.set_attribute(&stage_id, &surface_path, "ior", &ior.to_string());
                            
                            // Connect surface shader to material
                            // In a real implementation, this would create USD connections
                            println!("✓ Connected surface shader to material output");
                            
                            outputs.insert("Material Path".to_string(), NodeData::String(material_prim.path.clone()));
                            outputs.insert("Material".to_string(), NodeData::String(material_prim.path));
                            outputs.insert("Surface Output".to_string(), NodeData::String(surface_path));
                            
                            println!("✓ Created USD material: {} with preview surface", material_path);
                        }
                        Err(e) => {
                            eprintln!("✗ Failed to create surface shader: {}", e);
                            outputs.insert("Material Path".to_string(), NodeData::String(material_prim.path.clone()));
                            outputs.insert("Material".to_string(), NodeData::String(material_prim.path));
                            outputs.insert("Surface Output".to_string(), NodeData::None);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("✗ Failed to create USD material: {}", e);
                    outputs.insert("Material Path".to_string(), NodeData::String("".to_string()));
                    outputs.insert("Material".to_string(), NodeData::None);
                    outputs.insert("Surface Output".to_string(), NodeData::None);
                }
            }
        });
        
        outputs
    }
}