//! NOT node parameters using Pattern A: build_interface method

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use super::logic::NotLogic;

/// NOT node with Pattern A interface
#[derive(Debug, Clone)]
pub struct NotNode {
    pub input: bool,
    pub double_invert: bool,
    pub tri_state: bool,
    pub show_truth_table: bool,
    pub output_format: String,
}

impl Default for NotNode {
    fn default() -> Self {
        Self {
            input: false,
            double_invert: false,
            tri_state: false,
            show_truth_table: true,
            output_format: "Boolean".to_string(),
        }
    }
}

impl NotNode {
    /// Pattern A: build_interface method that renders UI and returns parameter changes
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("NOT Logic Parameters");
        ui.separator();
        
        // Input (when not connected)
        ui.horizontal(|ui| {
            ui.label("Input:");
            let mut input = node.parameters.get("input")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false);
            
            if ui.checkbox(&mut input, "").changed() {
                changes.push(ParameterChange {
                    parameter: "input".to_string(),
                    value: NodeData::Boolean(input),
                });
            }
        });
        
        ui.separator();
        
        // Logic operation modes
        ui.horizontal(|ui| {
            ui.label("Double Invert:");
            let mut double_invert = node.parameters.get("double_invert")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false);
            
            if ui.checkbox(&mut double_invert, "Acts as buffer (NOT NOT = original)").changed() {
                changes.push(ParameterChange {
                    parameter: "double_invert".to_string(),
                    value: NodeData::Boolean(double_invert),
                });
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("Tri-State:");
            let mut tri_state = node.parameters.get("tri_state")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false);
            
            if ui.checkbox(&mut tri_state, "Enable tri-state logic for null inputs").changed() {
                changes.push(ParameterChange {
                    parameter: "tri_state".to_string(),
                    value: NodeData::Boolean(tri_state),
                });
            }
        });
        
        ui.separator();
        
        // Operation type display
        let double_invert = node.parameters.get("double_invert")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
        
        let operation_name = if double_invert { "BUFFER" } else { "NOT" };
        ui.label(format!("Current Operation: {}", operation_name));
        ui.separator();
        
        // Output formatting
        ui.horizontal(|ui| {
            ui.label("Output Format:");
            let mut output_format = node.parameters.get("output_format")
                .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or("Boolean".to_string());
            
            egui::ComboBox::from_label("")
                .selected_text(&output_format)
                .show_ui(ui, |ui| {
                    for format in ["Boolean", "Integer", "Text"] {
                        if ui.selectable_value(&mut output_format, format.to_string(), format).changed() {
                            changes.push(ParameterChange {
                                parameter: "output_format".to_string(),
                                value: NodeData::String(output_format.clone()),
                            });
                        }
                    }
                });
        });
        
        // Truth table display options
        ui.horizontal(|ui| {
            ui.label("Show Truth Table:");
            let mut show_truth_table = node.parameters.get("show_truth_table")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(true);
            
            if ui.checkbox(&mut show_truth_table, "").changed() {
                changes.push(ParameterChange {
                    parameter: "show_truth_table".to_string(),
                    value: NodeData::Boolean(show_truth_table),
                });
            }
        });
        
        // Show truth table if enabled
        let show_table = node.parameters.get("show_truth_table")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(true);
        
        if show_table {
            ui.separator();
            ui.label("Truth Table:");
            
            ui.horizontal(|ui| {
                ui.label("Input | Result");
            });
            
            for input_val in [false, true] {
                let result = !input_val;
                let final_result = if double_invert { !result } else { result };
                ui.horizontal(|ui| {
                    ui.label(format!("{} | {}", 
                        if input_val { "T" } else { "F" },
                        if final_result { "T" } else { "F" }
                    ));
                });
            }
            
            ui.separator();
            
            // Quick toggles
            ui.label("Quick Actions:");
            ui.horizontal(|ui| {
                if ui.button("Toggle Input").clicked() {
                    let current_input = node.parameters.get("input")
                        .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                        .unwrap_or(false);
                    changes.push(ParameterChange {
                        parameter: "input".to_string(),
                        value: NodeData::Boolean(!current_input),
                    });
                }
                if ui.button("Reset").clicked() {
                    changes.push(ParameterChange {
                        parameter: "input".to_string(),
                        value: NodeData::Boolean(false),
                    });
                }
            });
            
            ui.separator();
            
            // Display current result
            let current_input = node.parameters.get("input")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false);
            
            let result = !current_input;
            let final_result = if double_invert { !result } else { result };
            
            ui.label(format!("Current Result: {}", 
                if final_result { "TRUE" } else { "FALSE" }
            ));
            
            if double_invert {
                ui.label("Note: Double invert mode acts as a buffer");
            }
        }
        
        changes
    }
    
    /// Apply parameters to create NotLogic instance
    pub fn create_logic(&self) -> NotLogic {
        NotLogic {
            input: self.input,
            double_invert: self.double_invert,
            tri_state: self.tri_state,
        }
    }
    
    /// Format output based on output_format setting
    pub fn format_output(&self, value: bool) -> NodeData {
        match self.output_format.as_str() {
            "Integer" => NodeData::Integer(if value { 1 } else { 0 }),
            "Text" => NodeData::String(if value { "TRUE".to_string() } else { "FALSE".to_string() }),
            _ => NodeData::Boolean(value),
        }
    }
}