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
                
                // Create filtered scene data based on user preferences
                let scene_data = self.create_filtered_scene_data(&usd_scene_data)?;
                
                Ok(scene_data)
            }
            Err(e) => {
                eprintln!("❌ USD File Reader: Failed to load USD file: {}", e);
                Err(format!("USD loading failed: {}", e))
            }
        }
    }


    /// Create filtered scene data based on user extraction preferences
    fn create_filtered_scene_data(&self, usd_scene_data: &USDSceneData) -> Result<NodeData, String> {
        let scene_info = UsdSceneExtractedData {
            name: format!("USD Scene: {}", 
                Path::new(&self.file_path).file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")),
            source_file: self.file_path.clone(),
            
            // Filter data based on user preferences
            geometries: if self.extract_geometry { 
                usd_scene_data.meshes.iter().map(|mesh| {
                    GeometryInfo {
                        prim_path: mesh.prim_path.clone(),
                        vertex_count: mesh.vertices.len(),
                        triangle_count: mesh.indices.len() / 3,
                        has_normals: !mesh.normals.is_empty(),
                        has_uvs: !mesh.uvs.is_empty(),
                    }
                }).collect()
            } else { 
                vec![] 
            },
            
            materials: if self.extract_materials { 
                usd_scene_data.materials.iter().map(|mat| {
                    MaterialInfo {
                        prim_path: mat.prim_path.clone(),
                        diffuse_color: [mat.diffuse_color.x, mat.diffuse_color.y, mat.diffuse_color.z],
                        metallic: mat.metallic,
                        roughness: mat.roughness,
                    }
                }).collect()
            } else { 
                vec![] 
            },
            
            lights: if self.extract_lights { 
                usd_scene_data.lights.iter().map(|light| {
                    LightInfo {
                        prim_path: light.prim_path.clone(),
                        light_type: light.light_type.clone(),
                        intensity: light.intensity,
                        color: [light.color.x, light.color.y, light.color.z],
                    }
                }).collect()
            } else { 
                vec![] 
            },
            
            // Cameras would be extracted here when USD engine supports them
            cameras: if self.extract_cameras { 
                vec![] // TODO: Extract cameras when USD engine supports them
            } else { 
                vec![] 
            },
        };
        
        Ok(NodeData::String(serde_json::to_string(&scene_info).map_err(|e| e.to_string())?))
    }
}


/// USD Scene extracted data - filtered content based on user preferences
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]  
pub struct UsdSceneExtractedData {
    pub name: String,
    pub source_file: String,
    pub geometries: Vec<GeometryInfo>,
    pub materials: Vec<MaterialInfo>,
    pub lights: Vec<LightInfo>,
    pub cameras: Vec<CameraInfo>,
}

/// Geometry information extracted from USD mesh
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GeometryInfo {
    pub prim_path: String,
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub has_normals: bool,
    pub has_uvs: bool,
}

/// Material information extracted from USD material
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MaterialInfo {
    pub prim_path: String,
    pub diffuse_color: [f32; 3],
    pub metallic: f32,
    pub roughness: f32,
}

/// Light information extracted from USD light
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LightInfo {
    pub prim_path: String,
    pub light_type: String,
    pub intensity: f32,
    pub color: [f32; 3],
}

/// Camera information extracted from USD camera  
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CameraInfo {
    pub prim_path: String,
    // TODO: Add camera-specific fields when USD engine supports cameras
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