#!/usr/bin/env rust-script
//! Test the plugin system independently

use std::path::PathBuf;

fn main() {
    println!("Testing USD Plugin Separation...");
    
    // Check if plugin file exists
    let plugin_path = PathBuf::from("/Users/brian/nodle-claude/nodle/plugins/libnodle_usd_plugin_comprehensive.dylib");
    
    if plugin_path.exists() {
        println!("✅ USD Plugin file found: {}", plugin_path.display());
        
        // Check file size
        if let Ok(metadata) = std::fs::metadata(&plugin_path) {
            println!("✅ Plugin size: {} KB", metadata.len() / 1024);
        }
        
        // Try to load the library
        match unsafe { libloading::Library::new(&plugin_path) } {
            Ok(lib) => {
                println!("✅ USD Plugin library loaded successfully");
                
                // Try to find the create_plugin function
                match unsafe { lib.get::<fn() -> *mut dyn nodle_plugin_sdk::NodePlugin>(b"create_plugin") } {
                    Ok(create_fn) => {
                        println!("✅ Plugin create function found");
                        
                        // Create the plugin instance
                        let plugin_ptr = create_fn();
                        if !plugin_ptr.is_null() {
                            println!("✅ Plugin instance created successfully");
                            
                            // Get plugin info
                            let plugin = unsafe { &*plugin_ptr };
                            let info = plugin.plugin_info();
                            
                            println!("✅ Plugin Info:");
                            println!("   Name: {}", info.name);
                            println!("   Version: {}", info.version);
                            println!("   Description: {}", info.description);
                            
                            // Cleanup
                            unsafe { let _ = Box::from_raw(plugin_ptr); }
                            println!("✅ Plugin cleaned up successfully");
                        } else {
                            println!("❌ Plugin instance creation failed");
                        }
                    }
                    Err(e) => println!("❌ Plugin create function not found: {}", e),
                }
            }
            Err(e) => println!("❌ Failed to load USD Plugin library: {}", e),
        }
    } else {
        println!("❌ USD Plugin file not found at: {}", plugin_path.display());
    }
    
    // Test core separation
    println!("\nTesting Core Separation...");
    
    // Check that core doesn't contain USD files
    let usd_path = PathBuf::from("/Users/brian/nodle-claude/nodle/src/nodes/three_d/usd");
    if !usd_path.exists() {
        println!("✅ USD directory removed from core");
    } else {
        println!("❌ USD directory still exists in core");
    }
    
    // Check Cargo.toml doesn't have USD dependencies
    let cargo_toml = std::fs::read_to_string("/Users/brian/nodle-claude/nodle/Cargo.toml").unwrap_or_default();
    if !cargo_toml.contains("pyo3") {
        println!("✅ PyO3 dependency removed from core");
    } else {
        println!("❌ PyO3 dependency still in core");
    }
    
    println!("\n🎉 USD Plugin Separation Test Complete!");
}