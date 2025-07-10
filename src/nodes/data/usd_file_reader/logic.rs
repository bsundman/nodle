//! USD File Reader Logic
//!
//! Core processing logic for reading USD files and extracting scene data.
//! Uses the embedded USD-core library to parse USD files and convert them
//! to Nodle's internal scene representation.

use crate::nodes::interface::NodeData;
use crate::nodes::Node;
use crate::workspaces::three_d::usd::usd_engine::{USDEngine, USDSceneData};
use std::path::Path;

/// USD File Reader processing logic
pub struct UsdFileReaderLogic {
    pub file_path: String,
    pub auto_reload: bool,
    pub extract_geometry: bool,
    pub extract_materials: bool,
    pub extract_lights: bool,
    pub extract_cameras: bool,
    last_modified: Option<std::time::SystemTime>,
    usd_engine: USDEngine,
    cached_scene_data: Option<USDSceneData>,
}

impl UsdFileReaderLogic {
    /// Create logic instance from node parameters
    pub fn from_node(node: &Node) -> Self {
        let file_path = node.parameters.get("file_path")
            .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
            .unwrap_or_default();
        
        let auto_reload = node.parameters.get("auto_reload")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
        
        let extract_geometry = node.parameters.get("extract_geometry")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(true);
        
        let extract_materials = node.parameters.get("extract_materials")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(true);
        
        let extract_lights = node.parameters.get("extract_lights")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(true);
        
        let extract_cameras = node.parameters.get("extract_cameras")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);

        Self {
            file_path,
            auto_reload,
            extract_geometry,
            extract_materials,
            extract_lights,
            extract_cameras,
            last_modified: None,
            usd_engine: USDEngine::new(),
            cached_scene_data: None,
        }
    }

    /// Process the USD file and return scene data
    pub fn process(&mut self, _inputs: Vec<NodeData>) -> Vec<NodeData> {
        // Check if file path is provided
        if self.file_path.is_empty() {
            println!("📁 USD File Reader: No file path specified");
            return vec![NodeData::None];
        }

        // Check if file exists
        let file_path = Path::new(&self.file_path);
        if !file_path.exists() {
            eprintln!("📁 USD File Reader: File not found: {}", self.file_path);
            return vec![NodeData::None];
        }

        // Check if file has been modified (for auto-reload)
        if self.auto_reload {
            if let Ok(metadata) = file_path.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if let Some(last_mod) = self.last_modified {
                        if modified <= last_mod {
                            // File hasn't changed, return cached data if available
                            // For now, we'll reload every time, but this could be optimized
                        }
                    }
                    self.last_modified = Some(modified);
                }
            }
        }

        println!("📁 USD File Reader: Loading USD file: {}", self.file_path);

        // Load USD file using the existing USD infrastructure
        match self.load_usd_file() {
            Ok(scene_data) => {
                println!("✅ USD File Reader: Successfully loaded USD file");
                vec![scene_data]
            }
            Err(e) => {
                eprintln!("❌ USD File Reader: Failed to load USD file: {}", e);
                vec![NodeData::None]
            }
        }
    }

    /// Load USD file and extract data using the real USD engine
    fn load_usd_file(&mut self) -> Result<NodeData, String> {
        println!("📁 USD File Reader: Loading USD file with real USD engine: {}", self.file_path);
        
        // Load USD scene data using the engine
        match self.usd_engine.load_stage(&self.file_path) {
            Ok(usd_scene_data) => {
                println!("✅ USD File Reader: Successfully loaded USD scene with {} meshes, {} lights, {} materials", 
                         usd_scene_data.meshes.len(), 
                         usd_scene_data.lights.len(), 
                         usd_scene_data.materials.len());
                
                // Cache the scene data for reuse
                self.cached_scene_data = Some(usd_scene_data.clone());
                
                // Apply user extraction filters to the full scene data
                let filtered_scene_data = self.apply_extraction_filters(usd_scene_data)?;
                
                // Return the full USDSceneData directly - no more metadata conversion
                Ok(crate::nodes::interface::NodeData::USDSceneData(filtered_scene_data))
            }
            Err(e) => {
                eprintln!("❌ USD File Reader: Failed to load USD file: {}", e);
                Err(format!("USD loading failed: {}", e))
            }
        }
    }


    /// Apply user extraction filters to the full USD scene data
    fn apply_extraction_filters(&self, mut usd_scene_data: USDSceneData) -> Result<USDSceneData, String> {
        println!("📁 USD File Reader: Applying extraction filters - geometry: {}, materials: {}, lights: {}, cameras: {}", 
                 self.extract_geometry, self.extract_materials, self.extract_lights, self.extract_cameras);
        
        // Filter geometry based on user preferences
        if !self.extract_geometry {
            println!("📁 USD File Reader: Filtering out geometry data");
            usd_scene_data.meshes.clear();
        }
        
        // Filter materials based on user preferences  
        if !self.extract_materials {
            println!("📁 USD File Reader: Filtering out material data");
            usd_scene_data.materials.clear();
        }
        
        // Filter lights based on user preferences
        if !self.extract_lights {
            println!("📁 USD File Reader: Filtering out light data");
            usd_scene_data.lights.clear();
        }
        
        // TODO: Filter cameras when USD engine supports them
        if !self.extract_cameras {
            // No camera data to filter yet
        }
        
        println!("✅ USD File Reader: Filtered scene data contains {} meshes, {} materials, {} lights",
                 usd_scene_data.meshes.len(), usd_scene_data.materials.len(), usd_scene_data.lights.len());
        
        Ok(usd_scene_data)
    }
}



impl Default for UsdFileReaderLogic {
    fn default() -> Self {
        Self {
            file_path: String::new(),
            auto_reload: false,
            extract_geometry: true,
            extract_materials: true,
            extract_lights: true,
            extract_cameras: false,
            last_modified: None,
            usd_engine: USDEngine::new(),
            cached_scene_data: None,
        }
    }
}