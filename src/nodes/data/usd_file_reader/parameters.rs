//! USD File Reader Parameters
//!
//! Provides the parameter interface for the USD File Reader node,
//! including file selection and loading options.

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use egui::Ui;

/// Parameter interface for USD File Reader node
pub struct UsdFileReaderParameters;

impl UsdFileReaderParameters {
    /// Build the parameter interface for the USD File Reader node
    pub fn build_interface(node: &mut Node, ui: &mut Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();

        ui.heading("USD File Reader");
        ui.separator();

        // Get file path for the entire section
        let file_path = node.parameters.get("file_path")
            .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
            .unwrap_or_default();

        // File path selection section
        ui.group(|ui| {
            ui.label("📁 File Selection");
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label("USD File:");
                
                let mut file_path_input = file_path.clone();
                
                // File path text input
                let text_response = ui.add(egui::TextEdit::singleline(&mut file_path_input)
                    .desired_width(200.0)
                    .hint_text("Select USD file..."));
                
                if text_response.changed() {
                    changes.push(ParameterChange {
                        parameter: "file_path".to_string(),
                        value: NodeData::String(file_path_input.clone()),
                    });
                    // Trigger reload by updating a reload flag
                    changes.push(ParameterChange {
                        parameter: "needs_reload".to_string(),
                        value: NodeData::Boolean(true),
                    });
                }
                
                // Browse button with file dialog
                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("USD Files", &["usd", "usda", "usdc", "usdz"])
                        .add_filter("All Files", &["*"])
                        .set_title("Select USD File")
                        .pick_file()
                    {
                        let path_str = path.display().to_string();
                        // Only trigger reload if the file path changed OR the file was modified
                        let should_reload = if path_str != file_path {
                            // File path changed - definitely need to reload
                            true
                        } else {
                            // Same file path - check if file was modified since last load
                            // This matches the cache key logic: file_path + modification_timestamp
                            if let Ok(metadata) = std::fs::metadata(&path_str) {
                                if let Ok(current_modified) = metadata.modified() {
                                    // Get the last known modification time from cache key logic
                                    // For now, we'll be conservative and not reload if same file
                                    false
                                } else {
                                    // Can't read modification time - be safe and reload
                                    true
                                }
                            } else {
                                // Can't read file metadata - be safe and reload
                                true
                            }
                        };
                        
                        if should_reload {
                            changes.push(ParameterChange {
                                parameter: "file_path".to_string(),
                                value: NodeData::String(path_str),
                            });
                            changes.push(ParameterChange {
                                parameter: "needs_reload".to_string(),
                                value: NodeData::Boolean(true),
                            });
                        }
                    }
                }
            });
            
            // File info display
            if !file_path.is_empty() {
                ui.horizontal(|ui| {
                    ui.label("📄");
                    if std::path::Path::new(&file_path).exists() {
                        ui.colored_label(egui::Color32::LIGHT_GREEN, "File found");
                    } else {
                        ui.colored_label(egui::Color32::LIGHT_RED, "File not found");
                    }
                });
            }
        });

        ui.add_space(8.0);

        // Loading options section  
        ui.group(|ui| {
            ui.label("⚙️ Loading Options");
            ui.separator();

            // Auto-reload is now automatic when file path changes
            ui.separator();
            ui.label("Extract Content:");

            // Geometry extraction
            let mut extract_geometry = node.parameters.get("extract_geometry")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(true);
            
            if ui.checkbox(&mut extract_geometry, "🎲 Geometry & Meshes").changed() {
                changes.push(ParameterChange {
                    parameter: "extract_geometry".to_string(),
                    value: NodeData::Boolean(extract_geometry),
                });
            }

            // Materials extraction  
            let mut extract_materials = node.parameters.get("extract_materials")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(true);
            
            if ui.checkbox(&mut extract_materials, "🎨 Materials & Shaders").changed() {
                changes.push(ParameterChange {
                    parameter: "extract_materials".to_string(),
                    value: NodeData::Boolean(extract_materials),
                });
            }

            // Lights extraction
            let mut extract_lights = node.parameters.get("extract_lights")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(true);
            
            if ui.checkbox(&mut extract_lights, "💡 Lights & Lighting").changed() {
                changes.push(ParameterChange {
                    parameter: "extract_lights".to_string(),
                    value: NodeData::Boolean(extract_lights),
                });
            }

            // Cameras extraction
            let mut extract_cameras = node.parameters.get("extract_cameras")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false);
            
            if ui.checkbox(&mut extract_cameras, "📷 Cameras & Views").changed() {
                changes.push(ParameterChange {
                    parameter: "extract_cameras".to_string(),
                    value: NodeData::Boolean(extract_cameras),
                });
            }
        });

        ui.add_space(8.0);

        // Coordinate system conversion section
        ui.group(|ui| {
            ui.label("🔄 Coordinate System");
            ui.separator();
            
            // Get current coordinate system mode
            let current_mode = node.parameters.get("coordinate_system_mode")
                .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or("Auto".to_string());
            
            ui.label("Convert to Y-up (viewport standard):");
            ui.separator();
            
            // Auto-detect mode
            let mut is_auto = current_mode == "Auto";
            if ui.radio_value(&mut is_auto, true, "Auto-detect from USD file").changed() && is_auto {
                changes.push(ParameterChange {
                    parameter: "coordinate_system_mode".to_string(),
                    value: NodeData::String("Auto".to_string()),
                });
            }
            
            // Manual override modes
            ui.label("Manual override:");
            ui.indent("manual_modes", |ui| {
                let mut is_manual_z = current_mode == "Z-up";
                if ui.radio_value(&mut is_manual_z, true, "Z-up").changed() && is_manual_z {
                    changes.push(ParameterChange {
                        parameter: "coordinate_system_mode".to_string(),
                        value: NodeData::String("Z-up".to_string()),
                    });
                }
                
                let mut is_manual_y = current_mode == "Y-up";
                if ui.radio_value(&mut is_manual_y, true, "Y-up (no conversion)").changed() && is_manual_y {
                    changes.push(ParameterChange {
                        parameter: "coordinate_system_mode".to_string(),
                        value: NodeData::String("Y-up".to_string()),
                    });
                }
                
                let mut is_manual_x = current_mode == "X-up";
                if ui.radio_value(&mut is_manual_x, true, "X-up").changed() && is_manual_x {
                    changes.push(ParameterChange {
                        parameter: "coordinate_system_mode".to_string(),
                        value: NodeData::String("X-up".to_string()),
                    });
                }
            });
            
            // Description based on current mode
            ui.separator();
            match current_mode.as_str() {
                "Auto" => {
                    ui.label("Reads up-axis from USD metadata and converts automatically");
                    ui.label("Supports X-up, Y-up, and Z-up USD files");
                }
                "Z-up" => {
                    ui.label("Always converts from Z-up to Y-up coordinate system");
                    ui.label("Use when USD metadata is incorrect or missing");
                }
                "Y-up" => {
                    ui.label("No coordinate conversion applied");
                    ui.label("Use when USD file is already in Y-up coordinates");
                }
                "X-up" => {
                    ui.label("Always converts from X-up to Y-up coordinate system");
                    ui.label("Use when USD metadata is incorrect or missing");
                }
                _ => {}
            }
            ui.label("   Also reverses triangle winding order when conversion is applied");
        });

        ui.add_space(8.0);

        // NOTE: Don't auto-reset needs_reload as it causes unnecessary cache invalidation
        // The execution engine will handle the reload logic internally

        // Status section
        ui.group(|ui| {
            ui.label("📊 Status");
            ui.separator();
            
            if file_path.is_empty() {
                ui.colored_label(egui::Color32::GRAY, "No file selected");
            } else if !std::path::Path::new(&file_path).exists() {
                ui.colored_label(egui::Color32::LIGHT_RED, "File not found - check path");
            } else {
                ui.colored_label(egui::Color32::LIGHT_GREEN, "Ready to load USD file");
                
                // Show file size if available
                if let Ok(metadata) = std::fs::metadata(&file_path) {
                    let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
                    ui.label(format!("File size: {:.2} MB", size_mb));
                }
            }
        });

        changes
    }
}