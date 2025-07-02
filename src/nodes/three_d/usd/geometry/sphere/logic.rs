//! USD Sphere node functional operations

use crate::nodes::interface::NodeData;
use crate::nodes::three_d::usd::usd_engine::with_usd_engine;

/// Core logic for USD sphere creation
pub struct USDSphereLogic;

impl USDSphereLogic {
    /// Execute the sphere creation operation
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
            _ => format!("sphere_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap()),
        };
        
        // Get parameters
        let radius = match inputs.get("Radius").or_else(|| parameters.get("radius")) {
            Some(NodeData::Float(f)) => *f,
            _ => 1.0,
        };
        
        let subdivisions = match parameters.get("subdivisions") {
            Some(NodeData::Integer(i)) => *i as i32,
            _ => 32,
        };
        
        let purpose = match parameters.get("purpose") {
            Some(NodeData::String(s)) => s.clone(),
            _ => "default".to_string(),
        };
        
        let visibility = match parameters.get("visibility") {
            Some(NodeData::String(s)) => s.clone(),
            _ => "inherited".to_string(),
        };
        
        // Construct prim path
        let prim_path = if parent_path.ends_with('/') {
            format!("{}{}", parent_path, name)
        } else {
            format!("{}/{}", parent_path, name)
        };
        
        // Create the sphere
        with_usd_engine(|engine| {
            match engine.create_sphere(&stage_id, &prim_path, radius as f64) {
                Ok(prim) => {
                    // Set additional attributes
                    let _ = engine.set_prim_purpose(&stage_id, &prim_path, &purpose);
                    let _ = engine.set_prim_visibility(&stage_id, &prim_path, &visibility);
                    
                    // Apply transform if provided
                    if let Some(NodeData::Any(_transform_data)) = inputs.get("Transform") {
                        // TODO: Apply transform matrix
                    }
                    
                    outputs.insert("Prim Path".to_string(), NodeData::String(prim.path.clone()));
                    outputs.insert("Prim".to_string(), NodeData::String(prim.path));
                    
                    println!("✓ Created USD sphere: {} (radius: {})", prim_path, radius);
                }
                Err(e) => {
                    eprintln!("✗ Failed to create USD sphere: {}", e);
                    outputs.insert("Prim Path".to_string(), NodeData::String("".to_string()));
                    outputs.insert("Prim".to_string(), NodeData::None);
                }
            }
        });
        
        outputs
    }
}