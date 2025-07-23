//! USD File Reader Logic
//!
//! Core processing logic for reading USD files and extracting scene data.
//! Uses the embedded USD-core library to parse USD files and convert them
//! to Nodle's internal scene representation.

use crate::nodes::interface::NodeData;
use crate::nodes::{Node, NodeId};
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
    last_coordinate_system_mode: String,
    last_extract_geometry: bool,
    last_extract_materials: bool,
    last_extract_lights: bool,
    last_extract_cameras: bool,
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
            coordinate_system_mode: coordinate_system_mode.clone(),
            // Initialize last_* as empty so we can detect first run
            last_file_path: String::new(),
            last_coordinate_system_mode: coordinate_system_mode,
            last_extract_geometry: extract_geometry,
            last_extract_materials: extract_materials,
            last_extract_lights: extract_lights,
            last_extract_cameras: extract_cameras,
        }
    }
    
    /// Process using full execution engine integration
    /// Stage 1 and Stage 2 operate as completely separate virtual nodes with independent caches
    pub fn process_with_unified_cache(
        &mut self,
        node_id: NodeId,
        _inputs: Vec<NodeData>,
        engine: &mut crate::nodes::NodeGraphEngine
    ) -> Vec<NodeData> {
        println!("🔥 USD PROCESS_WITH_UNIFIED_CACHE CALLED - Node: {} File: {}", node_id, self.file_path);
        
        // First, handle granular cache invalidation for stages
        self.validate_and_invalidate_caches(node_id, engine);
        
        // Generate stage-qualified cache keys for independent cache management
        let stage1_cache_key = Self::get_stage1_cache_key(node_id);
        let stage2_cache_key = Self::get_stage2_cache_key(node_id);
        
        // Check if file path is provided
        if self.file_path.is_empty() {
            println!("📁 USD File Reader: No file path specified");
            return vec![NodeData::None];
        }

        // =============================================================================
        // STAGE 1: Check execution engine cache with hash key
        // =============================================================================
        let hash_key = match self.generate_stage1_hash_key() {
            Ok(key) => key,
            Err(e) => {
                eprintln!("❌ USD File Reader: Cannot generate hash key: {}", e);
                return vec![NodeData::None];
            }
        };

        // Try to get Stage 1 data from execution engine cache using stage-qualified key
        let raw_usd_data = if let Some(cached_stage1) = engine.get_cached_stage_output_by_key(&stage1_cache_key, &hash_key) {
            if let NodeData::USDSceneData(scene_data) = cached_stage1 {
                println!("✅ USD STAGE 1 CACHE HIT - Stage {} using cached data for hash: {}", stage1_cache_key, hash_key);
                println!("🔍 USD CACHE HIT: File {} already loaded, using cached data", self.file_path);
                scene_data.clone()
            } else {
                eprintln!("❌ USD File Reader Stage 1: Invalid cached data type");
                return vec![NodeData::None];
            }
        } else {
            // Stage 1 cache miss - load from disk and cache using stage-qualified key
            println!("💾 USD STAGE 1 CACHE MISS - Stage {} loading from disk for hash: {}", stage1_cache_key, hash_key);
            match self.load_stage1_from_disk(&hash_key, &stage1_cache_key, engine) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("❌ USD File Reader Stage 1 failed: {}", e);
                    return vec![NodeData::None];
                }
            }
        };

        // =============================================================================
        // STAGE 2: Check execution engine cache with parameter hash
        // =============================================================================
        let params_key = self.generate_stage2_params_key();
        
        // Try to get Stage 2 data from execution engine cache using stage-qualified key
        if let Some(cached_stage2) = engine.get_cached_stage_output_by_key(&stage2_cache_key, &params_key) {
            if let NodeData::USDSceneData(processed_data) = cached_stage2 {
                println!("📁 USD File Reader Stage 2: Stage {} using cached processed data", stage2_cache_key);
                println!("✅ USD File Reader: Using fully cached processed data");
                return vec![NodeData::USDSceneData(processed_data.clone())];
            }
        }

        // Stage 2 cache miss - process data and cache using stage-qualified key
        println!("📁 USD File Reader Stage 2: Stage {} cache miss - processing raw data", stage2_cache_key);
        match self.process_stage2_and_cache(&raw_usd_data, &params_key, &stage2_cache_key, engine) {
            Ok(processed_data) => {
                println!("✅ USD File Reader: Two-stage processing complete");
                vec![NodeData::USDSceneData(processed_data)]
            }
            Err(e) => {
                eprintln!("❌ USD File Reader Stage 2 failed: {}", e);
                vec![NodeData::None]
            }
        }
    }

    /// Generate hash key for Stage 1 caching: file_path + modification_timestamp
    /// Only reads filesystem metadata - does NOT load the file
    /// This is stateless and generates the same key for the same file state
    fn generate_stage1_hash_key(&self) -> Result<String, String> {
        // Check if file path is empty
        if self.file_path.is_empty() {
            return Err("No file path specified".to_string());
        }
        
        // Generate hash key from filesystem metadata
        let path = Path::new(&self.file_path);
        if !path.exists() {
            return Err(format!("File does not exist: {}", self.file_path));
        }
        
        let metadata = path.metadata()
            .map_err(|e| format!("Cannot read file metadata: {}", e))?;
        
        let file_modified = metadata.modified()
            .map_err(|e| format!("Cannot read file modification time: {}", e))?;
        
        // Create deterministic hash key: stage1 + file_path + modification_timestamp
        let hash_key = format!("stage1:{}:{:?}", self.file_path, file_modified);
        
        println!("📁 USD File Reader Stage 1: Generated hash key = {}", hash_key);
        Ok(hash_key)
    }

    /// Generate cache key for Stage 2 based on processing parameters
    fn generate_stage2_params_key(&self) -> String {
        format!("stage2:{}:{}:{}:{}:{}", 
                self.coordinate_system_mode,
                self.extract_geometry,
                self.extract_materials,
                self.extract_lights,
                self.extract_cameras)
    }
    
    /// Generate stage-qualified cache key for Stage 1 (file loading)
    /// Uses dot notation: node 0 Stage 1 = "0.1"
    fn get_stage1_cache_key(base_node_id: NodeId) -> String {
        format!("{}.1", base_node_id)  // e.g., node 0 -> "0.1", node 5 -> "5.1"
    }
    
    /// Generate stage-qualified cache key for Stage 2 (processing)
    /// Uses dot notation: node 0 Stage 2 = "0.2"
    fn get_stage2_cache_key(base_node_id: NodeId) -> String {
        format!("{}.2", base_node_id)  // e.g., node 0 -> "0.2", node 5 -> "5.2"
    }
    
    /// GLOBAL FILE CHANGE DETECTION: Check if file actually changed before reloading
    /// This method implements a global catch-all to prevent unnecessary file reloads
    /// when cache is invalidated but the file itself hasn't changed
    fn check_existing_valid_cache(
        &self, 
        current_hash_key: &str, 
        engine: &mut crate::nodes::NodeGraphEngine
    ) -> Option<USDSceneData> {
        // Check if we have any persistent USD file data for this exact hash
        // The execution engine stores file data keyed by timestamp hash
        if let Some(persistent_data) = engine.get_persistent_usd_file_data(current_hash_key) {
            println!("✅ FILE UNCHANGED: Found persistent data for hash {} - reusing", current_hash_key);
            return Some(persistent_data);
        }
        
        println!("💾 NO PERSISTENT DATA: File needs to be loaded from disk for hash {}", current_hash_key);
        None
    }
    
    /// Handle granular cache invalidation - only invalidate what actually changed
    /// This prevents Stage 1 (file) cache invalidation when only Stage 2 (processing) parameters change
    fn validate_and_invalidate_caches(&mut self, node_id: NodeId, engine: &mut crate::nodes::NodeGraphEngine) {
        let mut stage1_invalid = false;
        let mut stage2_invalid = false;
        
        // Check if Stage 1 parameters (file path) have changed
        if self.file_path != self.last_file_path {
            stage1_invalid = true;
            println!("🗑️ USD File Reader: File path changed - Stage 1 cache invalid");
        }
        
        // Check if Stage 2 parameters (processing settings) have changed
        if self.coordinate_system_mode != self.last_coordinate_system_mode ||
           self.extract_geometry != self.last_extract_geometry ||
           self.extract_materials != self.last_extract_materials ||
           self.extract_lights != self.last_extract_lights ||
           self.extract_cameras != self.last_extract_cameras {
            stage2_invalid = true;
            println!("🗑️ USD File Reader: Processing parameters changed - Stage 2 cache invalid");
        }
        
        // Only invalidate Stage 1 cache if file path changed
        if stage1_invalid {
            if let Ok(hash_key) = self.generate_stage1_hash_key() {
                let stage1_pattern = crate::nodes::cache::CacheKeyPattern::Stage(node_id, hash_key);
                let _ = engine.unified_cache.invalidate(&stage1_pattern);
                println!("🗑️ USD File Reader: Invalidated Stage 1 cache (file changed)");
            }
        } else {
            println!("✅ USD File Reader: Stage 1 cache preserved (file unchanged)");
        }
        
        // Only invalidate Stage 2 cache if processing parameters changed
        if stage2_invalid {
            let stage2_key = self.generate_stage2_params_key();
            let stage2_pattern = crate::nodes::cache::CacheKeyPattern::Stage(node_id, stage2_key);
            let _ = engine.unified_cache.invalidate(&stage2_pattern);
            println!("🗑️ USD File Reader: Invalidated Stage 2 cache (parameters changed)");
            
            // CRITICAL: Clear GPU mesh caches when Stage 2 parameters change
            // This prevents viewport from showing stale transformed geometry
            println!("🗑️ USD File Reader: Clearing GPU mesh caches due to Stage 2 parameter change");
            crate::gpu::viewport_3d_callback::clear_all_gpu_mesh_caches();
        } else {
            println!("✅ USD File Reader: Stage 2 cache preserved (parameters unchanged)");
        }
        
        // Always update tracking parameters after validation to keep them synchronized
        self.last_file_path = self.file_path.clone();
        self.last_coordinate_system_mode = self.coordinate_system_mode.clone();
        self.last_extract_geometry = self.extract_geometry;
        self.last_extract_materials = self.extract_materials;
        self.last_extract_lights = self.extract_lights;
        self.last_extract_cameras = self.extract_cameras;
    }

    /// Load Stage 1 data from disk and cache in execution engine
    /// GLOBAL FILE CHANGE DETECTION: Only loads from disk if file actually changed
    fn load_stage1_from_disk(
        &mut self, 
        hash_key: &str, 
        stage_qualified_key: &str, 
        engine: &mut crate::nodes::NodeGraphEngine
    ) -> Result<USDSceneData, String> {
        // GLOBAL CHECK: Verify if file actually changed before loading from disk
        // This prevents unnecessary file reloads when cache is invalidated but file is unchanged
        if let Some(cached_data) = self.check_existing_valid_cache(hash_key, engine) {
            println!("✅ USD FILE UNCHANGED: Cache invalidated but file hasn't changed - reusing existing data");
            return Ok(cached_data);
        }
        
        // File has actually changed or no valid cache exists - load from disk
        println!("🚨 LOADING USD FROM DISK: {}", self.file_path);
        let mut usd_engine = crate::workspaces::three_d::usd::usd_engine::USDEngine::new();
        
        match usd_engine.load_stage(&self.file_path) {
            Ok(scene_data) => {
                println!("✅ USD DISK LOAD SUCCESS: {} meshes, {} lights, {} materials", 
                         scene_data.meshes.len(), scene_data.lights.len(), scene_data.materials.len());
                
                // Cache in execution engine with stage-qualified key
                let stage1_data = NodeData::USDSceneData(scene_data.clone());
                engine.cache_stage_output_by_key(stage_qualified_key, hash_key, stage1_data);
                println!("💽 CACHED STAGE 1 DATA with stage key: {} hash: {}", stage_qualified_key, hash_key);
                
                // GLOBAL FILE CACHE: Store persistently to survive cache invalidations
                engine.store_persistent_usd_file_data(hash_key, scene_data.clone());
                println!("🌍 STORED PERSISTENT FILE DATA for hash: {}", hash_key);
                
                // Update tracking
                self.last_file_path = self.file_path.clone();
                self.needs_reload = false;
                
                Ok(scene_data)
            }
            Err(e) => Err(format!("Failed to load USD file: {}", e))
        }
    }

    /// Process Stage 2 data and cache in execution engine
    fn process_stage2_and_cache(
        &mut self,
        raw_usd_data: &USDSceneData,
        params_key: &str,
        stage_qualified_key: &str,
        engine: &mut crate::nodes::NodeGraphEngine
    ) -> Result<USDSceneData, String> {
        // Process the raw USD data with current parameters
        match self.process_cached_scene_data(raw_usd_data) {
            Ok(processed_data) => {
                println!("✅ USD File Reader Stage 2: Processing complete");
                
                // Cache in execution engine with stage-qualified key
                let stage2_data = NodeData::USDSceneData(processed_data.clone());
                engine.cache_stage_output_by_key(stage_qualified_key, params_key, stage2_data);
                
                // Update parameter tracking
                self.last_coordinate_system_mode = self.coordinate_system_mode.clone();
                self.last_extract_geometry = self.extract_geometry;
                self.last_extract_materials = self.extract_materials;
                self.last_extract_lights = self.extract_lights;
                self.last_extract_cameras = self.extract_cameras;
                
                Ok(processed_data)
            }
            Err(e) => Err(e)
        }
    }




    
    /// Update logic parameters from current node state
    pub fn update_from_node(&mut self, node: &Node) {
        self.file_path = node.parameters.get("file_path")
            .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
            .unwrap_or_default();
        
        self.needs_reload = node.parameters.get("needs_reload")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
        
        self.extract_geometry = node.parameters.get("extract_geometry")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(true);
        
        self.extract_materials = node.parameters.get("extract_materials")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(true);
        
        self.extract_lights = node.parameters.get("extract_lights")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(true);
        
        self.extract_cameras = node.parameters.get("extract_cameras")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
        
        self.coordinate_system_mode = node.parameters.get("coordinate_system_mode")
            .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
            .unwrap_or("Auto".to_string());
            
        self.needs_reload = node.parameters.get("needs_reload")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
    }


    
    /// Stage 2: Process cached USD data with coordinate conversion and filters
    fn process_cached_scene_data(&self, cached_scene: &USDSceneData) -> Result<USDSceneData, String> {
        println!("📁 USD File Reader Stage 2: Processing cached USD scene data");
        
        // Clone the raw scene data for processing (Stage 1 cache remains intact)
        // TODO: Could optimize this with ownership handoff if this is the only consumer
        let mut scene_data = cached_scene.clone();
        
        // Apply coordinate system conversion based on mode
        if self.coordinate_system_mode != "Y-up" {
            scene_data = self.convert_coordinate_system(scene_data)?;
        } else {
            println!("📁 USD File Reader Stage 2: Y-up mode - skipping coordinate conversion");
        }
        
        // Apply user extraction filters
        scene_data = self.apply_extraction_filters(scene_data)?;
        
        Ok(scene_data)
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
                // println!("📁 USD File Reader: Preserving triangle winding order (handedness unchanged)");
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
            last_coordinate_system_mode: "Auto".to_string(),
            last_extract_geometry: true,
            last_extract_materials: true,
            last_extract_lights: true,
            last_extract_cameras: false,
        }
    }
}