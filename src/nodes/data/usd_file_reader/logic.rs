//! USD File Reader Logic
//!
//! Core processing logic for reading USD files and extracting scene data.
//! Uses the embedded USD-core library to parse USD files and convert them
//! to Nodle's internal scene representation.

use crate::nodes::interface::NodeData;
use crate::nodes::Node;
use crate::workspaces::three_d::usd::usd_engine::{USDEngine, USDSceneData};
use std::path::Path;
use glam::Mat4;

/// USD File Reader processing logic
pub struct UsdFileReaderLogic {
    pub file_path: String,
    pub needs_reload: bool,
    pub extract_geometry: bool,
    pub extract_materials: bool,
    pub extract_lights: bool,
    pub extract_cameras: bool,
    pub coordinate_system_mode: String,
    last_file_path: String,
    usd_engine: USDEngine,
    cached_scene_data: Option<USDSceneData>,
}

impl UsdFileReaderLogic {
    /// Create logic instance from node parameters
    pub fn from_node(node: &Node) -> Self {
        let file_path = node.parameters.get("file_path")
            .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
            .unwrap_or_default();
        
        let needs_reload = node.parameters.get("needs_reload")
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
        
        let coordinate_system_mode = node.parameters.get("coordinate_system_mode")
            .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
            .unwrap_or("Auto".to_string());

        Self {
            file_path,
            needs_reload,
            extract_geometry,
            extract_materials,
            extract_lights,
            extract_cameras,
            coordinate_system_mode,
            last_file_path: String::new(),
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

        // Check if file path has changed or reload is explicitly requested
        let file_path_changed = self.file_path != self.last_file_path;
        let should_reload = file_path_changed || self.needs_reload;
        
        // If file path hasn't changed and we don't need to reload, return cached data
        if !should_reload && self.cached_scene_data.is_some() {
            println!("📁 USD File Reader: Using cached scene data");
            return vec![crate::nodes::interface::NodeData::USDSceneData(self.cached_scene_data.clone().unwrap())];
        }
        
        // Update the last file path
        self.last_file_path = self.file_path.clone();

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
                
                // Apply coordinate system conversion based on mode
                let converted_scene_data = if self.coordinate_system_mode != "Y-up" {
                    self.convert_coordinate_system(usd_scene_data)?
                } else {
                    // Y-up mode - no conversion needed
                    println!("📁 USD File Reader: Y-up mode selected - skipping coordinate conversion");
                    usd_scene_data
                };
                
                // Apply user extraction filters to the full scene data
                let filtered_scene_data = self.apply_extraction_filters(converted_scene_data)?;
                
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
    
    /// Convert coordinate system from USD to Nodle viewport (Y-up, right-handed)
    fn convert_coordinate_system(&self, mut usd_scene_data: USDSceneData) -> Result<USDSceneData, String> {
        // Determine which up-axis to convert from
        let source_up_axis = match self.coordinate_system_mode.as_str() {
            "Auto" => {
                // Use the up-axis detected from USD file metadata
                println!("📁 USD File Reader: Auto-detecting coordinate system from USD metadata: {}-up", usd_scene_data.up_axis);
                usd_scene_data.up_axis.as_str()
            }
            "Z-up" => {
                println!("📁 USD File Reader: Manual override - forcing Z-up to Y-up conversion");
                "Z"
            }
            "X-up" => {
                println!("📁 USD File Reader: Manual override - forcing X-up to Y-up conversion");
                "X"
            }
            _ => {
                // Should not reach here since Y-up is handled earlier
                return Err(format!("Invalid coordinate system mode: {}", self.coordinate_system_mode));
            }
        };
        
        println!("📁 USD File Reader: Converting coordinate system from {}-up to Y-up", source_up_axis);
        
        // Determine the transformation matrix and handedness change based on the source up axis
        let (coordinate_transform, flips_handedness) = match source_up_axis {
            "Y" => {
                // Already Y-up, no transformation needed
                println!("📁 USD File Reader: Source is already Y-up, no coordinate conversion needed");
                return Ok(usd_scene_data);
            }
            "Z" => {
                // Z-up to Y-up: rotate -90 degrees around X-axis
                // This converts:
                // - Z-up (0,0,1) -> Y-up (0,1,0)
                // - USD Y-forward (0,1,0) -> Nodle Z-forward (0,0,1)
                // - USD X-right (1,0,0) -> Nodle X-right (1,0,0) [unchanged]
                // HANDEDNESS: Right-handed → Right-handed (NO CHANGE)
                println!("📁 USD File Reader: Z-up → Y-up conversion preserves handedness");
                (Mat4::from_rotation_x(-std::f32::consts::PI / 2.0), false)
            }
            "X" => {
                // X-up to Y-up: rotate 90 degrees around Z-axis
                // This converts:
                // - USD X-up (1,0,0) -> Nodle Y-up (0,1,0)
                // - USD Y-forward (0,1,0) -> Nodle -X-left (-1,0,0)
                // - USD Z-forward (0,0,1) -> Nodle Z-forward (0,0,1) [unchanged]
                // HANDEDNESS: Right-handed → Left-handed (FLIPS!)
                println!("📁 USD File Reader: X-up → Y-up conversion flips handedness - will reverse winding order");
                (Mat4::from_rotation_z(std::f32::consts::PI / 2.0), true)
            }
            _ => {
                println!("⚠️  USD File Reader: Unknown up axis '{}', defaulting to Z-up conversion", source_up_axis);
                // Default to Z-up conversion (preserves handedness)
                (Mat4::from_rotation_x(-std::f32::consts::PI / 2.0), false)
            }
        };
        
        // Convert meshes
        for mesh in &mut usd_scene_data.meshes {
            // Transform vertices
            for vertex in &mut mesh.vertices {
                let transformed = coordinate_transform.transform_point3(*vertex);
                *vertex = transformed;
            }
            
            // Transform normals (use normal matrix to handle non-uniform scaling)
            for normal in &mut mesh.normals {
                let transformed = coordinate_transform.transform_vector3(*normal);
                *normal = transformed.normalize();
            }
            
            // Transform vertex colors if present (no spatial transformation needed)
            // Vertex colors remain unchanged as they are material properties
            
            // Flip winding order only when handedness changes
            if flips_handedness {
                // Handedness flipped (e.g., X-up → Y-up conversion)
                // Reverse triangle winding to maintain correct face normals and culling
                println!("📁 USD File Reader: Reversing triangle winding order due to handedness flip");
                for triangle in mesh.indices.chunks_mut(3) {
                    if triangle.len() == 3 {
                        triangle.swap(1, 2); // Reverse winding: ABC -> ACB
                    }
                }
            } else {
                // Handedness preserved (e.g., Z-up → Y-up conversion)
                // Keep original triangle winding order
                println!("📁 USD File Reader: Preserving triangle winding order (handedness unchanged)");
            }
            
            // Transform mesh transform matrix
            mesh.transform = coordinate_transform * mesh.transform;
        }
        
        // Convert lights
        for light in &mut usd_scene_data.lights {
            // Transform light transform matrix
            light.transform = coordinate_transform * light.transform;
        }
        
        // Convert materials (no spatial transformation needed)
        // Materials don't have spatial components that need transformation
        
        println!("✅ USD File Reader: Coordinate system conversion complete");
        println!("   - Transformed {} meshes from {}-up to Y-up", usd_scene_data.meshes.len(), usd_scene_data.up_axis);
        println!("   - Transformed {} lights", usd_scene_data.lights.len());
        println!("   - Reversed triangle winding order for proper culling");
        
        Ok(usd_scene_data)
    }
}



impl Default for UsdFileReaderLogic {
    fn default() -> Self {
        Self {
            file_path: String::new(),
            needs_reload: false,
            extract_geometry: true,
            extract_materials: true,
            extract_lights: true,
            extract_cameras: false,
            coordinate_system_mode: "Auto".to_string(),
            last_file_path: String::new(),
            usd_engine: USDEngine::new(),
            cached_scene_data: None,
        }
    }
}