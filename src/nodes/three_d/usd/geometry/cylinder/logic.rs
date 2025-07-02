//! USD Cylinder node functional operations

use crate::nodes::interface::NodeData;
use crate::nodes::three_d::usd::usd_engine::with_usd_engine;

/// Core logic for USD cylinder creation
pub struct USDCylinderLogic;

impl USDCylinderLogic {
    /// Execute the cylinder creation operation
    pub fn execute(inputs: &std::collections::HashMap<String, NodeData>, parameters: &std::collections::HashMap<String, NodeData>) -> std::collections::HashMap<String, NodeData> {
        let mut outputs = std::collections::HashMap::new();
        
        // Get stage reference
        let stage_id = match inputs.get("Stage") {
            Some(NodeData::String(s)) => s.clone(),
            _ => {
                outputs.insert("Prim Path".to_string(), NodeData::String("".to_string()));
                outputs.insert("Prim".to_string(), NodeData::None);
                return outputs;
            }
        };
        
        // Get parent path
        let parent_path = match inputs.get("Parent Path") {
            Some(NodeData::String(s)) => s.clone(),
            _ => "/World".to_string(),
        };
        
        // Get name or auto-generate
        let name = match inputs.get("Name") {
            Some(NodeData::String(s)) if !s.is_empty() => s.clone(),
            _ => format!("cylinder_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap()),
        };
        
        // Get parameters
        let radius = match inputs.get("Radius").or_else(|| parameters.get("radius")) {
            Some(NodeData::Float(f)) => *f,
            _ => 1.0,
        };
        
        let height = match inputs.get("Height").or_else(|| parameters.get("height")) {
            Some(NodeData::Float(f)) => *f,
            _ => 2.0,
        };
        
        let axis = match parameters.get("axis") {
            Some(NodeData::String(s)) => s.clone(),
            _ => "Y".to_string(),
        };
        
        let top_cap = match parameters.get("top_cap") {
            Some(NodeData::Boolean(b)) => *b,
            _ => true,
        };
        
        let bottom_cap = match parameters.get("bottom_cap") {
            Some(NodeData::Boolean(b)) => *b,
            _ => true,
        };
        
        // Construct prim path
        let prim_path = if parent_path.ends_with('/') {
            format!("{}{}", parent_path, name)
        } else {
            format!("{}/{}", parent_path, name)
        };
        
        // Create the cylinder
        with_usd_engine(|engine| {
            match engine.create_cylinder(&stage_id, &prim_path, radius as f64, height as f64) {
                Ok(prim) => {
                    // Set additional attributes
                    let _ = engine.set_attribute(&stage_id, &prim_path, "axis", &axis);
                    
                    if !top_cap {
                        let _ = engine.set_attribute(&stage_id, &prim_path, "topCap", "false");
                    }
                    if !bottom_cap {
                        let _ = engine.set_attribute(&stage_id, &prim_path, "bottomCap", "false");
                    }
                    
                    outputs.insert("Prim Path".to_string(), NodeData::String(prim.path.clone()));
                    outputs.insert("Prim".to_string(), NodeData::String(prim.path));
                    
                    println!("✓ Created USD cylinder: {} (radius: {}, height: {})", prim_path, radius, height);
                }
                Err(e) => {
                    eprintln!("✗ Failed to create USD cylinder: {}", e);
                    outputs.insert("Prim Path".to_string(), NodeData::String("".to_string()));
                    outputs.insert("Prim".to_string(), NodeData::None);
                }
            }
        });
        
        outputs
    }
}