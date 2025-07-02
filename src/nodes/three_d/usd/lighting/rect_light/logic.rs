//! USD Rect Light node functional operations

use crate::nodes::interface::NodeData;
use crate::nodes::three_d::usd::usd_engine::with_usd_engine;

/// Core logic for USD rect light creation
pub struct USDRectLightLogic;

impl USDRectLightLogic {
    /// Execute the rect light creation operation
    pub fn execute(inputs: &std::collections::HashMap<String, NodeData>, parameters: &std::collections::HashMap<String, NodeData>) -> std::collections::HashMap<String, NodeData> {
        let mut outputs = std::collections::HashMap::new();
        
        // Get stage reference
        let stage_id = match inputs.get("Stage") {
            Some(NodeData::String(s)) => s.clone(),
            _ => {
                outputs.insert("Light Path".to_string(), NodeData::String("".to_string()));
                outputs.insert("Light".to_string(), NodeData::None);
                return outputs;
            }
        };
        
        // Get parent path
        let parent_path = match inputs.get("Parent Path") {
            Some(NodeData::String(s)) => s.clone(),
            _ => "/World/Lights".to_string(),
        };
        
        // Get name or auto-generate
        let name = match inputs.get("Name") {
            Some(NodeData::String(s)) if !s.is_empty() => s.clone(),
            _ => format!("rectLight_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap()),
        };
        
        // Get light parameters
        let intensity = match parameters.get("intensity") {
            Some(NodeData::Float(f)) => *f,
            _ => 1.0,
        };
        
        let color = match parameters.get("color") {
            Some(NodeData::Color(color)) => [color[0], color[1], color[2]], // Use RGB components
            _ => [1.0, 1.0, 1.0],
        };
        
        let temperature = match parameters.get("temperature") {
            Some(NodeData::Float(f)) => *f,
            _ => 6500.0,
        };
        
        let width = match parameters.get("width") {
            Some(NodeData::Float(f)) => *f,
            _ => 1.0,
        };
        
        let height = match parameters.get("height") {
            Some(NodeData::Float(f)) => *f,
            _ => 1.0,
        };
        
        let enabled = match parameters.get("enabled") {
            Some(NodeData::Boolean(b)) => *b,
            _ => true,
        };
        
        // Construct light path
        let light_path = if parent_path.ends_with('/') {
            format!("{}{}", parent_path, name)
        } else {
            format!("{}/{}", parent_path, name)
        };
        
        // Create the rect light
        with_usd_engine(|engine| {
            match engine.create_rect_light(&stage_id, &light_path, intensity as f64, width as f64, height as f64) {
                Ok(light_prim) => {
                    // Set light attributes
                    let _ = engine.set_attribute(&stage_id, &light_path, "intensity", &intensity.to_string());
                    let _ = engine.set_attribute(&stage_id, &light_path, "color", 
                        &format!("({}, {}, {})", color[0], color[1], color[2]));
                    let _ = engine.set_attribute(&stage_id, &light_path, "colorTemperature", &temperature.to_string());
                    let _ = engine.set_attribute(&stage_id, &light_path, "enableColorTemperature", "true");
                    
                    if !enabled {
                        let _ = engine.set_attribute(&stage_id, &light_path, "visibility", "invisible");
                    }
                    
                    outputs.insert("Light Path".to_string(), NodeData::String(light_prim.path.clone()));
                    outputs.insert("Light".to_string(), NodeData::String(light_prim.path));
                    
                    println!("✓ Created USD rect light: {} ({}x{}, intensity: {})", 
                        light_path, width, height, intensity);
                }
                Err(e) => {
                    eprintln!("✗ Failed to create USD rect light: {}", e);
                    outputs.insert("Light Path".to_string(), NodeData::String("".to_string()));
                    outputs.insert("Light".to_string(), NodeData::None);
                }
            }
        });
        
        outputs
    }
}