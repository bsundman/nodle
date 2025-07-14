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
                        changes.push(ParameterChange {
                            parameter: "file_path".to_string(),
                            value: NodeData::String(path_str),
                        });
                        // Trigger reload by updating a reload flag
                        changes.push(ParameterChange {
                            parameter: "needs_reload".to_string(),
                            value: NodeData::Boolean(true),
                        });
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
            
            // Convert coordinate system toggle
            let mut convert_coords = node.parameters.get("convert_coordinate_system")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(true);
            
            if ui.checkbox(&mut convert_coords, "Convert Z-up to Y-up").changed() {
                changes.push(ParameterChange {
                    parameter: "convert_coordinate_system".to_string(),
                    value: NodeData::Boolean(convert_coords),
                });
            }
            
            ui.label("🔧 Transforms USD's Z-up coordinate system to viewport's Y-up");
            ui.label("   Also reverses triangle winding order for proper rendering");
        });

        ui.add_space(8.0);

        // Reset needs_reload flag after parameter changes have been processed
        if node.parameters.get("needs_reload")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false) 
        {
            changes.push(ParameterChange {
                parameter: "needs_reload".to_string(),
                value: NodeData::Boolean(false),
            });
        }

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