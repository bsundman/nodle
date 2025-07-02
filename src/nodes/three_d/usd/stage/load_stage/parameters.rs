//! Load Stage node parameter interface

use crate::nodes::interface::{build_parameter_ui, NodeData, ParameterChange};
use crate::nodes::Node;

/// USD Load Stage node with parameter controls
#[derive(Default)]
pub struct LoadStageNode;

impl LoadStageNode {
    /// Build the parameter interface
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("Load USD Stage");
        ui.separator();
        
        // File path parameter with file picker
        ui.horizontal(|ui| {
            ui.label("USD File:");
            
            // Get current file path
            let current_path = node.parameters.get("file_path")
                .and_then(|p| if let NodeData::String(s) = p { Some(s.clone()) } else { None })
                .unwrap_or_else(|| "No file selected".to_string());
            
            // Display current path (truncated if too long)
            let display_path = if current_path.len() > 50 {
                format!("...{}", &current_path[current_path.len()-47..])
            } else {
                current_path.clone()
            };
            
            ui.label(format!("📄 {}", display_path));
        });
        
        ui.horizontal(|ui| {
            // Browse button to open file dialog
            if ui.button("📂 Browse...").clicked() {
                if let Ok(Some(path)) = Self::open_usd_file_dialog() {
                    let new_value = NodeData::String(path.clone());
                    node.parameters.insert("file_path".to_string(), new_value.clone());
                    changes.push(ParameterChange {
                        parameter: "file_path".to_string(),
                        value: new_value,
                    });
                }
            }
            
            // Clear button to reset file path
            if ui.button("🗑 Clear").clicked() {
                let new_value = NodeData::String("".to_string());
                node.parameters.insert("file_path".to_string(), new_value.clone());
                changes.push(ParameterChange {
                    parameter: "file_path".to_string(),
                    value: new_value,
                });
            }
        });
        
        // Manual text entry for file path
        if let Some(change) = build_parameter_ui(
            ui,
            "file_path",
            "Manual Path Entry",
            node.parameters.get("file_path").cloned().unwrap_or(NodeData::String("".to_string())),
            |ui, value| {
                if let NodeData::String(ref s) = value {
                    let mut text = s.clone();
                    ui.label("Path:");
                    let response = ui.text_edit_singleline(&mut text);
                    if response.changed() {
                        return Some(NodeData::String(text));
                    }
                }
                None
            }
        ) {
            node.parameters.insert("file_path".to_string(), change.clone());
            changes.push(ParameterChange {
                parameter: "file_path".to_string(),
                value: change,
            });
        }
        
        ui.separator();
        
        // Load options
        ui.collapsing("Load Options", |ui| {
            // Auto-reload on file change
            if let Some(change) = build_parameter_ui(
                ui,
                "auto_reload",
                "Auto Reload",
                node.parameters.get("auto_reload").cloned().unwrap_or(NodeData::Boolean(false)),
                |ui, value| {
                    if let NodeData::Boolean(ref b) = value {
                        let mut checked = *b;
                        let response = ui.checkbox(&mut checked, "Automatically reload when file changes");
                        if response.changed() {
                            return Some(NodeData::Boolean(checked));
                        }
                    }
                    None
                }
            ) {
                node.parameters.insert("auto_reload".to_string(), change.clone());
                changes.push(ParameterChange {
                    parameter: "auto_reload".to_string(),
                    value: change,
                });
            }
            
            // Load all payload references
            if let Some(change) = build_parameter_ui(
                ui,
                "load_payloads",
                "Load Payloads",
                node.parameters.get("load_payloads").cloned().unwrap_or(NodeData::Boolean(true)),
                |ui, value| {
                    if let NodeData::Boolean(ref b) = value {
                        let mut checked = *b;
                        let response = ui.checkbox(&mut checked, "Load all payload references");
                        if response.changed() {
                            return Some(NodeData::Boolean(checked));
                        }
                    }
                    None
                }
            ) {
                node.parameters.insert("load_payloads".to_string(), change.clone());
                changes.push(ParameterChange {
                    parameter: "load_payloads".to_string(),
                    value: change,
                });
            }
            
            // Population mask (for large stages)
            if let Some(change) = build_parameter_ui(
                ui,
                "population_mask",
                "Population Mask",
                node.parameters.get("population_mask").cloned().unwrap_or(NodeData::String("".to_string())),
                |ui, value| {
                    if let NodeData::String(ref s) = value {
                        let mut text = s.clone();
                        ui.label("Population Mask (optional):");
                        ui.small("Specify prim paths to limit loading (e.g., '/World/Geometry/*')");
                        let response = ui.text_edit_singleline(&mut text);
                        if response.changed() {
                            return Some(NodeData::String(text));
                        }
                    }
                    None
                }
            ) {
                node.parameters.insert("population_mask".to_string(), change.clone());
                changes.push(ParameterChange {
                    parameter: "population_mask".to_string(),
                    value: change,
                });
            }
        });
        
        // File info section
        let file_path = node.parameters.get("file_path")
            .and_then(|p| if let NodeData::String(s) = p { Some(s) } else { None });
            
        if let Some(path) = file_path {
            if !path.is_empty() && std::path::Path::new(path).exists() {
                ui.separator();
                ui.collapsing("File Information", |ui| {
                    if let Ok(metadata) = std::fs::metadata(path) {
                        ui.label(format!("📊 Size: {} bytes", metadata.len()));
                        
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                                ui.label(format!("📅 Modified: {}", 
                                    chrono::DateTime::<chrono::Utc>::from_timestamp(duration.as_secs() as i64, 0)
                                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                                        .unwrap_or_else(|| "Unknown".to_string())
                                ));
                            }
                        }
                        
                        // File extension validation
                        let path_obj = std::path::Path::new(path);
                        if let Some(ext) = path_obj.extension().and_then(|e| e.to_str()) {
                            let is_valid = matches!(ext.to_lowercase().as_str(), "usd" | "usda" | "usdc" | "usdz");
                            let color = if is_valid { 
                                egui::Color32::from_rgb(100, 200, 100) 
                            } else { 
                                egui::Color32::from_rgb(200, 100, 100) 
                            };
                            ui.colored_label(color, format!("📋 Format: .{}", ext));
                            
                            if !is_valid {
                                ui.colored_label(egui::Color32::YELLOW, "⚠ Warning: Not a recognized USD format");
                            }
                        }
                    }
                });
            } else if !path.is_empty() {
                ui.separator();
                ui.colored_label(egui::Color32::RED, "❌ File not found or inaccessible");
            }
        }
        
        changes
    }
    
    /// Open a file dialog specifically for USD files
    fn open_usd_file_dialog() -> Result<Option<String>, String> {
        use rfd::FileDialog;
        
        if let Some(path) = FileDialog::new()
            .add_filter("USD Files", &["usd", "usda", "usdc", "usdz"])
            .add_filter("All Files", &["*"])
            .set_title("Select USD File")
            .pick_file()
        {
            if let Some(path_str) = path.to_str() {
                Ok(Some(path_str.to_string()))
            } else {
                Err("Invalid file path encoding".to_string())
            }
        } else {
            Ok(None) // User cancelled dialog
        }
    }
}