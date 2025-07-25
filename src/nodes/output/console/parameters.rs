use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use egui::{ScrollArea, TextStyle};

/// Pattern A: build_interface method that renders console UI
pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
    let mut changes = Vec::new();
    
    ui.heading("Console Output");
    ui.separator();
    
    // Get parameters
    let mut auto_scroll = node.parameters.get("auto_scroll")
        .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
        .unwrap_or(true);
    
    let mut max_lines = node.parameters.get("max_lines")
        .and_then(|v| if let NodeData::Float(f) = v { Some(*f as usize) } else { None })
        .unwrap_or(1000);
    
    let console_text = node.parameters.get("console_output")
        .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_default();
    
    // Console controls
    ui.horizontal(|ui| {
        if ui.button("Clear").clicked() {
            changes.push(ParameterChange {
                parameter: "console_output".to_string(),
                value: NodeData::String(String::new()),
            });
        }
        
        if ui.checkbox(&mut auto_scroll, "Auto-scroll").changed() {
            changes.push(ParameterChange {
                parameter: "auto_scroll".to_string(),
                value: NodeData::Boolean(auto_scroll),
            });
        }
        
        ui.label("Max lines:");
        let mut max_lines_f = max_lines as f32;
        if ui.add(egui::DragValue::new(&mut max_lines_f).range(100.0..=10000.0)).changed() {
            max_lines = max_lines_f as usize;
            changes.push(ParameterChange {
                parameter: "max_lines".to_string(),
                value: NodeData::Float(max_lines_f),
            });
        }
    });
    
    ui.separator();
    
    // Console output area
    let available_height = ui.available_height() - 40.0; // Reserve space for controls
    
    ScrollArea::vertical()
        .max_height(available_height)
        .auto_shrink([false, false])
        .stick_to_bottom(auto_scroll)
        .show(ui, |ui| {
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                if console_text.is_empty() {
                    ui.colored_label(egui::Color32::GRAY, "Console is empty. Connect text input to see output.");
                } else {
                    // Display console text with monospace font
                    ui.style_mut().override_text_style = Some(TextStyle::Monospace);
                    
                    // Split into lines for better formatting
                    for line in console_text.lines() {
                        if line.trim().is_empty() {
                            ui.label(" "); // Empty line
                        } else if line.contains("[ERROR]") || line.contains("ERROR:") {
                            ui.colored_label(egui::Color32::from_rgb(255, 100, 100), line);
                        } else if line.contains("[WARN]") || line.contains("WARN:") {
                            ui.colored_label(egui::Color32::from_rgb(255, 200, 100), line);
                        } else if line.contains("[INFO]") || line.contains("INFO:") {
                            ui.colored_label(egui::Color32::from_rgb(100, 200, 255), line);
                        } else {
                            ui.label(line);
                        }
                    }
                }
            });
        });
    
    changes
}