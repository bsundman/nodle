//! Local USD installation manager for Nodle
//! Ensures we use our bundled USD version instead of system-wide installations

use std::env;
use std::path::{Path, PathBuf};
use std::sync::Once;

#[cfg(feature = "usd")]
use pyo3::prelude::*;

static USD_INIT: Once = Once::new();

/// Get the path to our local USD installation
pub fn get_usd_root() -> PathBuf {
    // Check environment variable first
    if let Ok(usd_root) = env::var("NODLE_USD_ROOT") {
        return PathBuf::from(usd_root);
    }
    
    // Otherwise use relative path from executable
    let exe_path = env::current_exe().expect("Failed to get executable path");
    let exe_dir = exe_path.parent().expect("Failed to get executable directory");
    
    // Look for vendor/python-runtime/python relative to executable (bundled with app)
    let vendor_path = exe_dir.join("vendor").join("python-runtime").join("python");
    if vendor_path.exists() {
        return vendor_path;
    }
    
    // Development mode: look relative to project root
    let project_root = exe_dir
        .ancestors()
        .find(|p| p.join("Cargo.toml").exists())
        .expect("Failed to find project root");
    
    project_root.join("vendor").join("python-runtime").join("python")
}

/// Get the Python executable from our USD installation
pub fn get_usd_python() -> PathBuf {
    let usd_root = get_usd_root();
    
    #[cfg(target_os = "windows")]
    let python_exe = usd_root.join("bin").join("python.exe");
    
    #[cfg(not(target_os = "windows"))]
    let python_exe = usd_root.join("bin").join("python3");
    
    if !python_exe.exists() {
        panic!(
            "Embedded Python not found at {:?}. Python runtime should be bundled with the application.",
            python_exe
        );
    }
    
    python_exe
}

/// Initialize PyO3 with our embedded Python and USD
#[cfg(feature = "usd")]
pub fn init_local_usd() {
    USD_INIT.call_once(|| {
        use pyo3::prelude::*;
        
        // Get embedded Python root
        let python_root = get_usd_root();
        let python_home = &python_root;
        let python_path = python_root.join("lib").join("python3.9").join("site-packages");
        
        // Configure PyO3 to use our embedded Python
        env::set_var("PYTHONHOME", python_home);
        env::set_var("PYTHONPATH", &python_path);
        
        // Initialize Python with our configuration
        pyo3::prepare_freethreaded_python();
        
        // Verify USD can be imported
        Python::with_gil(|py| {
            match py.import("pxr.Usd") {
                Ok(_) => println!("✓ Embedded USD initialized successfully"),
                Err(e) => panic!("Failed to import USD from embedded Python: {}", e),
            }
        });
    });
}

/// Check if local USD is installed
pub fn is_usd_installed() -> bool {
    let usd_root = get_usd_root();
    let python_exe = get_usd_python();
    
    usd_root.exists() && python_exe.exists()
}

/// Get USD version from local installation
#[cfg(feature = "usd")]
pub fn get_usd_version() -> Result<String, String> {
    init_local_usd();
    
    Python::with_gil(|py| {
        let usd = py
            .import("pxr.Usd")
            .map_err(|e| format!("Failed to import USD: {}", e))?;
        
        let version = usd
            .call_method0("GetVersion")
            .map_err(|e| format!("Failed to get USD version: {}", e))?;
        
        version
            .extract::<String>()
            .map_err(|e| format!("Failed to extract version string: {}", e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_usd_paths() {
        let usd_root = get_usd_root();
        println!("USD root: {:?}", usd_root);
        
        if is_usd_installed() {
            println!("✓ Local USD is installed");
            
            #[cfg(feature = "usd")]
            {
                match get_usd_version() {
                    Ok(version) => println!("USD version: {}", version),
                    Err(e) => println!("Failed to get version: {}", e),
                }
            }
        } else {
            println!("✗ Local USD not installed. Run: python scripts/setup_usd.py");
        }
    }
}