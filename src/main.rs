//! Nōdle Application - Node-based visual programming editor

use eframe::egui;
use log::{info, error};

mod constants;
mod editor;
mod menu_hierarchy;
// USD menu hierarchy now handled by USD plugin
mod nodes;
mod workspaces;
mod workspace;
mod gpu;
mod startup_checks;
mod theme;
mod plugins;

use editor::NodeEditor;



// Orphaned NodeRegistry wrapper and test code removed - use nodes::factory::NodeRegistry directly

/// Application entry point
fn main() -> Result<(), eframe::Error> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    info!("Starting Nōdle Application");
    
    // Run startup checks
    if let Err(e) = startup_checks::check_dependencies() {
        eprintln!("\nStartup check failed: {}\n", e);
        
        // Check if Python is available for setup
        if !startup_checks::check_python_available() {
            eprintln!("❌ Python not found. Please install Python 3.8+ to continue.");
        }
        
        startup_checks::show_setup_help();
        std::process::exit(1);
    }
    
    // Initialize global plugin system
    println!("🔌 Initializing global plugin system...");
    match workspace::initialize_global_plugin_manager() {
        Ok(()) => {
            if let Some(plugin_manager) = workspace::get_global_plugin_manager() {
                match plugin_manager.lock() {
                    Ok(manager) => {
                        let loaded_plugins = manager.get_loaded_plugins();
                        
                        if loaded_plugins.is_empty() {
                            println!("📦 No plugins found in standard directories");
                            println!("   Looking in: ~/.nodle/plugins/ and ./plugins/");
                        } else {
                            println!("✅ Loaded {} plugin(s):", loaded_plugins.len());
                            for plugin in loaded_plugins {
                                println!("   • {} v{} by {}", plugin.name, plugin.version, plugin.author);
                            }
                            println!("🔗 Plugin system initialized successfully");
                        }
                    }
                    Err(e) => {
                        error!("Failed to lock plugin manager: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("⚠️  Plugin initialization failed: {}", e);
            println!("   Continuing without plugins...");
        }
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_app_id("com.nodle.editor")
            .with_decorations(true)
            .with_title_shown(true)
            .with_resizable(true), // Explicitly allow resizing
        multisampling: 1, // Disable multisampling to avoid surface capability issues
        renderer: eframe::Renderer::Wgpu, // Use wgpu renderer for GPU acceleration
        wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
            supported_backends: wgpu::Backends::all(), // Allow all backends, not just Metal
            power_preference: wgpu::PowerPreference::LowPower, // Use low power to avoid compatibility issues
            device_descriptor: std::sync::Arc::new(|adapter| {
                // Get very conservative limits for maximum compatibility
                let mut limits = wgpu::Limits::downlevel_webgl2_defaults();
                // Limit texture dimensions to ensure compatibility across all surface configurations
                limits.max_texture_dimension_2d = 4096;
                limits.max_buffer_size = 256 * 1024 * 1024; // 256 MB max buffer
                
                wgpu::DeviceDescriptor {
                    label: Some("Nōdle Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: limits,
                    memory_hints: wgpu::MemoryHints::MemoryUsage, // Conservative memory usage
                }
            }),
            present_mode: wgpu::PresentMode::Fifo, // Use VSync to avoid tearing during resize
            ..Default::default()
        },
        ..Default::default()
    };

    eframe::run_native(
        "Nōdle - Node Editor",
        options,
        Box::new(|cc| {
            // Set dark theme
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            cc.egui_ctx.set_theme(egui::Theme::Dark);
            
            Ok(Box::new(NodeEditor::new()))
        }),
    )
}