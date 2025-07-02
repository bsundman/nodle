//! Create Stage node functional operations

use crate::nodes::interface::NodeData;
use crate::nodes::three_d::usd::usd_engine::with_usd_engine;

/// Core logic for USD stage creation
pub struct CreateStageLogic;

impl CreateStageLogic {
    /// Execute the stage creation operation
    pub fn execute(inputs: &std::collections::HashMap<String, NodeData>, parameters: &std::collections::HashMap<String, NodeData>) -> std::collections::HashMap<String, NodeData> {
        let mut outputs = std::collections::HashMap::new();
        
        // Get parameters
        let identifier = match inputs.get("Identifier").or_else(|| parameters.get("identifier")) {
            Some(NodeData::String(s)) => s.clone(),
            _ => "default".to_string(),
        };
        
        let in_memory = match parameters.get("in_memory") {
            Some(NodeData::Boolean(b)) => *b,
            _ => true,
        };
        
        let file_path = if !in_memory {
            match parameters.get("file_path") {
                Some(NodeData::String(s)) => Some(s.clone()),
                _ => Some("stage.usda".to_string()),
            }
        } else {
            None
        };
        
        let default_prim = match parameters.get("default_prim") {
            Some(NodeData::String(s)) => s.clone(),
            _ => "/World".to_string(),
        };
        
        // Create the stage
        with_usd_engine(|engine| {
            let result = if let Some(path) = file_path {
                engine.create_stage_to_file(&identifier, &path)
            } else {
                engine.create_stage(&identifier)
            };
            
            match result {
                Ok(stage) => {
                    // Set default prim if specified
                    if !default_prim.is_empty() {
                        let _ = engine.set_default_prim(&identifier, &default_prim);
                    }
                    
                    outputs.insert("Stage".to_string(), NodeData::String(stage.identifier));
                    outputs.insert("Root Path".to_string(), NodeData::String("/".to_string()));
                    
                    println!("✓ Created USD stage: {}", identifier);
                }
                Err(e) => {
                    eprintln!("✗ Failed to create USD stage: {}", e);
                    outputs.insert("Stage".to_string(), NodeData::None);
                    outputs.insert("Root Path".to_string(), NodeData::String("".to_string()));
                }
            }
        });
        
        outputs
    }
}