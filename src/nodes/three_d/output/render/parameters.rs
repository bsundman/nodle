//! Parameter interface for the USD Hydra Render node

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use egui::{Ui, Button, ComboBox, DragValue, TextEdit};

pub struct RenderParameters;

impl RenderParameters {
    pub fn build_interface(node: &mut Node, ui: &mut Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.separator();
        ui.strong("Render Settings");
        ui.separator();
        
        // Renderer selection dropdown
        if let Some(NodeData::String(current_renderer)) = node.parameters.get("renderer") {
            let mut selected_renderer = current_renderer.clone();
            
            // For now, provide all possible renderers until we fix dynamic detection
            let available_renderers = vec!["Storm".to_string(), "Cycles".to_string()];
            
            ui.horizontal(|ui| {
                ui.label("Renderer:");
                ComboBox::from_id_salt("renderer_dropdown")
                    .selected_text(&selected_renderer)
                    .show_ui(ui, |ui| {
                        for renderer in &available_renderers {
                            ui.selectable_value(&mut selected_renderer, renderer.clone(), renderer);
                        }
                    });
            });
            
            if selected_renderer != *current_renderer {
                changes.push(ParameterChange {
                    parameter: "renderer".to_string(),
                    value: NodeData::String(selected_renderer),
                });
            }
        }
        
        ui.separator();
        
        // Temporary folder for scene data and other temp files
        if let Some(NodeData::String(temp_folder)) = node.parameters.get("temp_folder") {
            let mut path = temp_folder.clone();
            ui.horizontal(|ui| {
                ui.label("Temp Folder:");
                let response = ui.add(TextEdit::singleline(&mut path).hint_text("/tmp/nodle_render").desired_width(200.0));
                // Update parameter on any change, not just on lost focus
                if response.changed() {
                    changes.push(ParameterChange {
                        parameter: "temp_folder".to_string(),
                        value: NodeData::String(path.clone()),
                    });
                }
                if ui.button("Browse").clicked() {
                    // Open folder picker dialog
                    if let Some(selected_path) = Self::open_folder_dialog() {
                        changes.push(ParameterChange {
                            parameter: "temp_folder".to_string(),
                            value: NodeData::String(selected_path),
                        });
                    }
                }
            });
        }
        
        ui.separator();
        
        // Output path with file picker
        if let Some(NodeData::String(output_path)) = node.parameters.get("output_path") {
            let mut path = output_path.clone();
            ui.horizontal(|ui| {
                ui.label("Output:");
                let response = ui.add(TextEdit::singleline(&mut path).desired_width(200.0));
                // Update parameter on any change, not just on lost focus
                if response.changed() {
                    changes.push(ParameterChange {
                        parameter: "output_path".to_string(),
                        value: NodeData::String(path.clone()),
                    });
                }
                if ui.button("Browse").clicked() {
                    // Open file save dialog for output image
                    if let Some(selected_path) = Self::open_save_file_dialog() {
                        changes.push(ParameterChange {
                            parameter: "output_path".to_string(),
                            value: NodeData::String(selected_path),
                        });
                    }
                }
            });
        }
        
        ui.separator();
        ui.strong("Image Settings");
        ui.separator();
        
        // Image width (height computed automatically by usdrecord)
        ui.horizontal(|ui| {
            ui.label("Image Width:");
            
            if let Some(NodeData::Integer(width)) = node.parameters.get("image_width") {
                let mut w = *width as i32;
                if ui.add(DragValue::new(&mut w).range(64..=7680)).changed() {
                    changes.push(ParameterChange {
                        parameter: "image_width".to_string(),
                        value: NodeData::Integer(w),
                    });
                }
            }
        });
        
        ui.label("📝 Height computed from width and camera aspect ratio");
        
        // Quick resolution presets
        ui.horizontal(|ui| {
            if ui.small_button("HD").clicked() {
                changes.push(ParameterChange {
                    parameter: "image_width".to_string(),
                    value: NodeData::Integer(1280),
                });
            }
            if ui.small_button("FHD").clicked() {
                changes.push(ParameterChange {
                    parameter: "image_width".to_string(),
                    value: NodeData::Integer(1920),
                });
            }
            if ui.small_button("4K").clicked() {
                changes.push(ParameterChange {
                    parameter: "image_width".to_string(),
                    value: NodeData::Integer(3840),
                });
            }
        });
        
        ui.separator();
        ui.strong("Camera & Quality");
        ui.separator();
        
        // Camera path
        if let Some(NodeData::String(camera_path)) = node.parameters.get("camera_path") {
            let mut path = camera_path.clone();
            ui.horizontal(|ui| {
                ui.label("Camera:");
                let response = ui.add(TextEdit::singleline(&mut path).hint_text("/Scene/Camera").desired_width(200.0));
                // Update parameter on any change, not just on lost focus
                if response.changed() {
                    changes.push(ParameterChange {
                        parameter: "camera_path".to_string(),  
                        value: NodeData::String(path),
                    });
                }
            });
        }
        
        // Note about samples
        ui.horizontal(|ui| {
            ui.label("📝 Samples:");
            ui.label("Configure via USD RenderSettings prim");
        });
        
        // Complexity (for rasterizers like Storm)
        if let Some(NodeData::String(complexity)) = node.parameters.get("complexity") {
            let mut selected_complexity = complexity.clone();
            ui.horizontal(|ui| {
                ui.label("Complexity:");
                ComboBox::from_id_salt("complexity_dropdown")
                    .selected_text(&selected_complexity)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut selected_complexity, "low".to_string(), "Low");
                        ui.selectable_value(&mut selected_complexity, "medium".to_string(), "Medium");
                        ui.selectable_value(&mut selected_complexity, "high".to_string(), "High");
                        ui.selectable_value(&mut selected_complexity, "veryhigh".to_string(), "Very High");
                    });
            });
            
            if selected_complexity != *complexity {
                changes.push(ParameterChange {
                    parameter: "complexity".to_string(),
                    value: NodeData::String(selected_complexity),
                });
            }
        }
        
        // Color correction
        if let Some(NodeData::String(color_correction)) = node.parameters.get("color_correction") {
            let mut selected_cc = color_correction.clone();
            ui.horizontal(|ui| {
                ui.label("Color:");
                ComboBox::from_id_salt("color_correction_dropdown")
                    .selected_text(&selected_cc)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut selected_cc, "disabled".to_string(), "Disabled");
                        ui.selectable_value(&mut selected_cc, "sRGB".to_string(), "sRGB");
                        ui.selectable_value(&mut selected_cc, "openColorIO".to_string(), "OpenColorIO");
                    });
            });
            
            if selected_cc != *color_correction {
                changes.push(ParameterChange {
                    parameter: "color_correction".to_string(),
                    value: NodeData::String(selected_cc),
                });
            }
        }
        
        ui.separator();
        ui.strong("Render");
        ui.separator();
        
        // Render status
        if let Some(NodeData::String(status)) = node.parameters.get("last_render_status") {
            ui.horizontal(|ui| {
                ui.label("Status:");
                ui.label(status);
            });
        }
        
        // Render button
        if ui.add(Button::new("🎬 Render").min_size(egui::vec2(100.0, 30.0))).clicked() {
            changes.push(ParameterChange {
                parameter: "trigger_render".to_string(),
                value: NodeData::Boolean(true),
            });
            println!("🎬 Render button clicked - trigger_render set to true");
        }
        
        // Quick actions
        ui.horizontal(|ui| {
            if ui.small_button("Refresh Renderers").clicked() {
                changes.push(ParameterChange {
                    parameter: "refresh_renderers".to_string(),
                    value: NodeData::Boolean(true),
                });
            }
            
            if ui.small_button("Open Output").clicked() {
                changes.push(ParameterChange {
                    parameter: "open_output".to_string(),
                    value: NodeData::Boolean(true),
                });
            }
        });
        
        changes
    }
    
    /// Open a folder picker dialog
    fn open_folder_dialog() -> Option<String> {
        use rfd::FileDialog;
        
        let dialog = FileDialog::new()
            .set_title("Select Temp Folder");
            
        if let Some(folder) = dialog.pick_folder() {
            folder.to_string_lossy().to_string().into()
        } else {
            None
        }
    }
    
    /// Open a save file dialog for output images
    fn open_save_file_dialog() -> Option<String> {
        use rfd::FileDialog;
        
        let dialog = FileDialog::new()
            .set_title("Save Render Output")
            .add_filter("PNG Images", &["png"])
            .add_filter("JPEG Images", &["jpg", "jpeg"])
            .add_filter("EXR Images", &["exr"])
            .add_filter("All Files", &["*"]);
            
        if let Some(path) = dialog.save_file() {
            path.to_string_lossy().to_string().into()
        } else {
            None
        }
    }
}