//! Global USD File Cache
//! 
//! Stage 1 of USD processing - handles file loading and caching raw USD data
//! This cache is independent of the execution engine and only reloads when files change

use std::collections::HashMap;
use std::sync::{Mutex, Arc};
use std::time::SystemTime;
use std::path::Path;
use lazy_static::lazy_static;
use crate::workspaces::three_d::usd::usd_engine::{USDEngine, USDSceneData};

/// Cached USD file data with metadata
#[derive(Clone)]
pub struct CachedUsdFile {
    /// Raw USD scene data as loaded from file
    pub scene_data: USDSceneData,
    /// Path to the USD file
    pub file_path: String,
    /// Last modified time of the file
    pub last_modified: SystemTime,
    /// File size in bytes
    pub file_size: u64,
}

/// Global USD file cache
lazy_static! {
    static ref USD_FILE_CACHE: Mutex<HashMap<String, Arc<CachedUsdFile>>> = Mutex::new(HashMap::new());
}

/// Stage 1: Load USD file if needed and cache the raw data
/// This function is called when:
/// - File path changes
/// - File is modified on disk
/// - Explicit reload is requested
pub fn load_usd_file_stage1(file_path: &str, force_reload: bool) -> Result<Arc<CachedUsdFile>, String> {
    // Check if file exists
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }
    
    // Get file metadata
    let metadata = path.metadata()
        .map_err(|e| format!("Failed to get file metadata: {}", e))?;
    let current_modified = metadata.modified()
        .map_err(|e| format!("Failed to get modification time: {}", e))?;
    let file_size = metadata.len();
    
    // Check cache
    let mut cache = USD_FILE_CACHE.lock()
        .map_err(|e| format!("Failed to acquire cache lock: {}", e))?;
    
    // Check if we have a cached version
    if let Some(cached) = cache.get(file_path) {
        // If not forcing reload, check if file hasn't changed
        if !force_reload && cached.last_modified == current_modified && cached.file_size == file_size {
            println!("📁 Stage 1: Using cached USD file (unchanged): {}", file_path);
            return Ok(Arc::clone(cached));
        }
    }
    
    // Load the file
    println!("📁 Stage 1: Loading USD file from disk: {}", file_path);
    let mut usd_engine = USDEngine::new();
    
    match usd_engine.load_stage(file_path) {
        Ok(scene_data) => {
            println!("✅ Stage 1: Successfully loaded USD file with {} meshes, {} lights, {} materials", 
                     scene_data.meshes.len(), 
                     scene_data.lights.len(), 
                     scene_data.materials.len());
            
            // Create cached entry
            let cached_file = Arc::new(CachedUsdFile {
                scene_data,
                file_path: file_path.to_string(),
                last_modified: current_modified,
                file_size,
            });
            
            // Store in cache
            cache.insert(file_path.to_string(), Arc::clone(&cached_file));
            
            Ok(cached_file)
        }
        Err(e) => {
            Err(format!("Failed to load USD file: {}", e))
        }
    }
}

/// Clear a specific file from the cache
pub fn clear_usd_file_from_cache(file_path: &str) {
    if let Ok(mut cache) = USD_FILE_CACHE.lock() {
        cache.remove(file_path);
        println!("📁 Stage 1: Cleared USD file from cache: {}", file_path);
    }
}

/// Clear all cached USD files
pub fn clear_all_usd_files_from_cache() {
    if let Ok(mut cache) = USD_FILE_CACHE.lock() {
        let count = cache.len();
        cache.clear();
        println!("📁 Stage 1: Cleared {} USD files from cache", count);
    }
}

/// Get cache statistics
pub fn get_usd_cache_stats() -> (usize, usize) {
    if let Ok(cache) = USD_FILE_CACHE.lock() {
        let count = cache.len();
        let total_size: usize = cache.values()
            .map(|c| c.scene_data.meshes.len() + c.scene_data.lights.len() + c.scene_data.materials.len())
            .sum();
        (count, total_size)
    } else {
        (0, 0)
    }
}