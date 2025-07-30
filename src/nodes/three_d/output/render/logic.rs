//! Logic implementation for the USD Hydra Render node

use crate::nodes::interface::NodeData;
use crate::nodes::Node;
use std::process::{Command, Stdio};
use std::path::Path;
use std::fs;
use std::env;
use std::io::{BufRead, BufReader};
use std::thread;

#[cfg(feature = "usd")]
use crate::workspaces::three_d::usd::usd_engine::{USDEngine, USDSceneData};


pub struct RenderLogic {
    renderer: String,
    output_path: String,
    temp_folder: String,
    image_width: i32,
    camera_path: String,
    complexity: String,
    color_correction: String,
    trigger_render: bool,
    refresh_renderers: bool,
    open_output: bool,
}

impl RenderLogic {
    pub fn from_node(node: &Node) -> Self {
        let get_string = |key: &str| -> String {
            node.parameters.get(key)
                .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or_default()
        };
        
        let get_int = |key: &str| -> i32 {
            node.parameters.get(key)
                .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
                .unwrap_or(0)
        };
        
        let get_bool = |key: &str| -> bool {
            node.parameters.get(key)
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false)
        };
        
        Self {
            renderer: get_string("renderer"),
            output_path: get_string("output_path"),
            temp_folder: get_string("temp_folder"),
            image_width: get_int("image_width"),
            camera_path: get_string("camera_path"),
            complexity: get_string("complexity"),
            color_correction: get_string("color_correction"),
            trigger_render: get_bool("trigger_render"),
            refresh_renderers: get_bool("refresh_renderers"),
            open_output: get_bool("open_output"),
        }
    }
    
    pub fn process(&mut self, inputs: Vec<NodeData>) -> Vec<NodeData> {
        let mut outputs = vec![NodeData::String("Ready".to_string())];
        
        // Handle renderer refresh
        if self.refresh_renderers {
            if let Ok(renderers) = self.detect_available_renderers() {
                println!("🎬 Available renderers: {:?}", renderers);
                // Reset the refresh flag
                self.refresh_renderers = false;
            }
        }
        
        // Handle render trigger
        if self.trigger_render {
            println!("🎬 Render triggered! Renderer: {}, Output: {}", self.renderer, self.output_path);
            
            // Reset the trigger flag to prevent repeated execution
            self.trigger_render = false;
            
            if let Some(scene_data) = inputs.first() {
                outputs[0] = NodeData::String("Rendering...".to_string());
                
                // TODO: Make this async to avoid blocking the UI
                // For now, just execute synchronously but with better error handling
                println!("🎬 Starting render process...");
                match self.execute_render(scene_data) {
                    Ok(status) => {
                        println!("✅ Render completed: {}", status);
                        outputs[0] = NodeData::String(status);
                        
                        // Handle open output
                        if self.open_output {
                            self.open_output_file();
                            self.open_output = false; // Reset flag
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Render failed: {}", e);
                        outputs[0] = NodeData::String(format!("Error: {}", e));
                    }
                }
            } else {
                outputs[0] = NodeData::String("Error: No scene data input".to_string());
            }
        } else {
            // Always try to refresh available renderers when not rendering
            if let Ok(renderers) = self.detect_available_renderers() {
                if !renderers.is_empty() {
                    outputs[0] = NodeData::String(format!("Ready - Renderers: {}", renderers.join(", ")));
                    // Store renderers for the UI (this is a hack, but works for now)
                    outputs.push(NodeData::String(format!("__available_renderers__{}", renderers.join(","))));
                }
            }
        }
        
        outputs
    }
    
    /// Detect available Hydra render delegates by querying our Hydra script
    fn detect_available_renderers(&self) -> Result<Vec<String>, String> {
        // Get Python executable and Hydra script paths
        let python_path = self.get_python_path()?;
        let render_script_path = self.get_hydra_render_script_path()?;
        
        // Run Hydra script with --list-renderers to see available renderers
        let output = Command::new(&python_path)
            .arg(&render_script_path)
            .arg("--list-renderers")
            .env("PYTHONPATH", self.get_usd_python_path())
            .env("DYLD_LIBRARY_PATH", self.get_usd_lib_path())
            .env("LD_LIBRARY_PATH", self.get_usd_lib_path())
            .env("USD_INSTALL_ROOT", self.get_usd_install_root())
            .env("PYTHONDONTWRITEBYTECODE", "1")
            .env("PYTHONUNBUFFERED", "1")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to run Hydra renderer detection: {}", e))?;
        
        let list_text = String::from_utf8_lossy(&output.stdout);
        let mut renderers = Vec::new();
        
        // Parse the output to find available renderers
        for line in list_text.lines() {
            let line = line.trim();
            if line.starts_with("- ") {
                let renderer = line.trim_start_matches("- ");
                if !renderer.is_empty() {
                    renderers.push(renderer.to_string());
                    println!("🎬 Found Hydra renderer: {}", renderer);
                }
            }
        }
        
        // Default fallback
        if renderers.is_empty() {
            renderers.push("Storm".to_string()); // Storm is always available in USD
        }
        
        Ok(renderers)
    }
    
    /// Execute the render using direct Hydra Python pipeline
    fn execute_render(&self, scene_data: &NodeData) -> Result<String, String> {
        // Create temporary USD file from scene data
        let temp_usd_path = self.create_temp_usd_file(scene_data)?;
        
        // Get Python executable path
        let python_path = self.get_python_path()?;
        
        // Get Hydra render script path
        let render_script_path = self.get_hydra_render_script_path()?;
        
        // Build Python command for Hydra rendering
        let mut cmd = Command::new(&python_path);
        cmd.arg(&render_script_path);
        
        // Basic arguments
        cmd.arg(&temp_usd_path);  // input USD file
        cmd.arg(&self.output_path);  // output image file
        cmd.arg("--renderer").arg(&self.renderer);
        cmd.arg("--width").arg(self.image_width.to_string());
        
        // Calculate height from 16:9 aspect ratio if not specified
        let image_height = (self.image_width as f32 / 16.0 * 9.0) as i32;
        cmd.arg("--height").arg(image_height.to_string());
        
        // Optional arguments based on parameters
        if !self.camera_path.is_empty() {
            cmd.arg("--camera").arg(&self.camera_path);
        }
        
        // Complexity setting
        if !self.complexity.is_empty() {
            cmd.arg("--complexity").arg(&self.complexity);
        }
        
        // Set environment variables for USD/Python
        cmd.env("PYTHONPATH", self.get_usd_python_path());
        cmd.env("DYLD_LIBRARY_PATH", self.get_usd_lib_path());
        cmd.env("LD_LIBRARY_PATH", self.get_usd_lib_path());
        cmd.env("USD_INSTALL_ROOT", self.get_usd_install_root());
        
        // Force Python environment isolation and clean shutdown
        cmd.env("PYTHONDONTWRITEBYTECODE", "1");
        cmd.env("PYTHONUNBUFFERED", "1");
        
        // Add Cycles plugin path if using Cycles, otherwise disable it entirely
        if self.renderer == "Cycles" {
            let cycles_plugin_path = self.get_cycles_plugin_path();
            cmd.env("PXR_PLUGINPATH", cycles_plugin_path);
        } else {
            // Clear PXR_PLUGINPATH and disable Cycles plugin loading
            cmd.env("PXR_PLUGINPATH", "");
            // Tell USD to not load the hdCycles plugin
            cmd.env("PXR_DISABLE_PLUGINS", "hdCycles");
        }
        
        println!("🎬 Executing Hydra render command: {:?}", cmd);
        
        // Execute the command with live output streaming
        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to execute Hydra render script: {}", e))?;
        
        // Stream stdout and stderr in real-time using separate threads
        let stdout_handle = if let Some(stdout) = child.stdout.take() {
            Some(thread::spawn(move || {
                let reader = BufReader::new(stdout);
                let mut lines = Vec::new();
                for line in reader.lines() {
                    if let Ok(line) = line {
                        println!("🎬 [HYDRA] {}", line);
                        lines.push(line);
                    }
                }
                lines
            }))
        } else {
            None
        };
        
        let stderr_handle = if let Some(stderr) = child.stderr.take() {
            Some(thread::spawn(move || {
                let reader = BufReader::new(stderr);
                let mut lines = Vec::new();
                for line in reader.lines() {
                    if let Ok(line) = line {
                        println!("🎬 [HYDRA-ERROR] {}", line);
                        lines.push(line);
                    }
                }
                lines
            }))
        } else {
            None
        };
        
        // Wait for the process to complete
        let status = child.wait()
            .map_err(|e| format!("Failed to wait for Hydra render process: {}", e))?;
        
        println!("🎬 [CLEANUP] Hydra render process completed");
        
        // Collect output from threads
        let _stdout_lines = if let Some(handle) = stdout_handle {
            handle.join().unwrap_or_default()
        } else {
            Vec::new()
        };
        
        let stderr_lines = if let Some(handle) = stderr_handle {
            handle.join().unwrap_or_default()
        } else {
            Vec::new()
        };
        
        // Clean up temp file
        let _ = fs::remove_file(&temp_usd_path);
        
        if status.success() {
            println!("🎬 Hydra render completed successfully!");
            
            // Check if output file was created
            if Path::new(&self.output_path).exists() {
                Ok(format!("Hydra rendered to {}", self.output_path))
            } else {
                Err("Hydra render completed but output file not found".to_string())
            }
        } else {
            let error_msg = if !stderr_lines.is_empty() {
                stderr_lines.join("\n")
            } else {
                format!("Process exited with code: {:?}", status.code())
            };
            Err(format!("Hydra render failed: {}", error_msg))
        }
    }
    
    /// Create a temporary USD file from scene data
    fn create_temp_usd_file(&self, scene_data: &NodeData) -> Result<String, String> {
        // Handle different types of scene data
        match scene_data {
            NodeData::String(usd_path) => {
                // If it's a file path, use it directly
                if Path::new(usd_path).exists() {
                    println!("🎬 Using USD file: {}", usd_path);
                    return Ok(usd_path.clone());
                } else {
                    return Err(format!("USD file not found: {}", usd_path));
                }
            }
            #[cfg(feature = "usd")]
            NodeData::USDSceneData(usd_scene_data) => {
                // For USD scene data, serialize to a temporary USD file in the temp folder
                // Create the temp folder if it doesn't exist
                fs::create_dir_all(&self.temp_folder)
                    .map_err(|e| format!("Failed to create temp folder '{}': {}", self.temp_folder, e))?;
                
                // Create temporary USD file path within the temp folder
                let temp_usd_path = format!("{}/scene_{}.usda", self.temp_folder, std::process::id());
                println!("🎬 Creating temporary USD file: {}", temp_usd_path);
                
                // Use USD engine to properly save the scene data as a USD file
                let mut usd_engine = USDEngine::new();
                usd_engine.save_usd_scene_to_file(usd_scene_data, &temp_usd_path)
                    .map_err(|e| format!("Failed to save USD scene data to file: {}", e))?;
                
                println!("🎬 Successfully saved USD scene data to: {}", temp_usd_path);
                Ok(temp_usd_path)
            }
            #[cfg(not(feature = "usd"))]
            NodeData::USDSceneData(_) => {
                Err("USD support not enabled. Build with --features usd to enable USD rendering.".to_string())
            }
            _ => {
                return Err("Invalid scene data type for rendering. Expected USD file path or scene data.".to_string());
            }
        }
    }
    
    /// Open the output file with the system default application
    fn open_output_file(&self) {
        if Path::new(&self.output_path).exists() {
            #[cfg(target_os = "macos")]
            {
                let _ = Command::new("open").arg(&self.output_path).spawn();
            }
            #[cfg(target_os = "linux")]
            {
                let _ = Command::new("xdg-open").arg(&self.output_path).spawn();
            }
            #[cfg(target_os = "windows")]
            {
                let _ = Command::new("cmd").args(&["/C", "start", &self.output_path]).spawn();
            }
        }
    }
    
    /// Get USD bin path from environment or vendor directory
    fn get_usd_bin_path(&self) -> Result<String, String> {
        // Try environment variable first
        if let Ok(usd_root) = env::var("USD_INSTALL_ROOT") {
            return Ok(format!("{}/bin", usd_root));
        }
        
        // Try relative to current executable (vendor installation)
        let vendor_usd = "/Users/brian/nodle/nodle/vendor/usd/bin";
        if Path::new(vendor_usd).exists() {
            return Ok(vendor_usd.to_string());
        }
        
        Err("USD installation not found. Please set USD_INSTALL_ROOT or ensure vendor/usd exists".to_string())
    }
    
    /// Get USD Python path
    fn get_usd_python_path(&self) -> String {
        let vendor_python = "/Users/brian/nodle/nodle/vendor/usd/lib/python:/Users/brian/nodle/nodle/vendor/python-runtime/python/lib/python3.9/site-packages";
        
        if let Ok(existing) = env::var("PYTHONPATH") {
            format!("{}:{}", vendor_python, existing)
        } else {
            vendor_python.to_string()
        }
    }
    
    /// Get USD library path
    fn get_usd_lib_path(&self) -> String {
        // Base libraries (USD and Python always needed)
        let mut lib_paths = vec![
            "/Users/brian/nodle/nodle/vendor/usd/lib",
            "/Users/brian/nodle/nodle/vendor/python-runtime/python/lib",
        ];
        
        // Add Cycles libraries only when using Cycles renderer
        if self.renderer == "Cycles" {
            lib_paths.push("/Users/brian/nodle/nodle/vendor/cycles/install/lib");
        }
        
        let vendor_lib = lib_paths.join(":");
        
        let env_var = if cfg!(target_os = "macos") {
            "DYLD_LIBRARY_PATH"
        } else {
            "LD_LIBRARY_PATH"
        };
        
        // Don't inherit existing environment variables to avoid global Cycles paths
        // We want to set exactly what we need for this specific renderer
        vendor_lib
    }
    
    /// Get USD install root
    fn get_usd_install_root(&self) -> String {
        env::var("USD_INSTALL_ROOT")
            .unwrap_or_else(|_| "/Users/brian/nodle/nodle/vendor/usd".to_string())
    }
    
    /// Get Cycles plugin path
    fn get_cycles_plugin_path(&self) -> String {
        "/Users/brian/nodle/nodle/vendor/cycles/install/hydra:/Users/brian/nodle/nodle/vendor/cycles/install/usd".to_string()
    }
    
    /// Get Python executable path
    fn get_python_path(&self) -> Result<String, String> {
        // Use embedded Python from vendor directory
        let vendor_python = "/Users/brian/nodle/nodle/vendor/python-runtime/python/bin/python3";
        if Path::new(vendor_python).exists() {
            return Ok(vendor_python.to_string());
        }
        
        // Fallback to system Python if embedded not found
        if let Ok(output) = Command::new("which").arg("python3").output() {
            if output.status.success() {
                let python_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Ok(python_path);
            }
        }
        
        Err("Python executable not found. Ensure embedded Python is installed or Python 3 is available in PATH".to_string())
    }
    
    /// Get Hydra render script path
    fn get_hydra_render_script_path(&self) -> Result<String, String> {
        let script_path = "/Users/brian/nodle/nodle/vendor/usd_hydra_render.py";
        if Path::new(script_path).exists() {
            Ok(script_path.to_string())
        } else {
            Err(format!("Hydra render script not found: {}", script_path))
        }
    }
}