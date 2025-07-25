//! OR node parameters using Pattern A: build_interface method

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use super::logic::OrLogic;

/// OR node with Pattern A interface
#[derive(Debug, Clone)]
pub struct OrNode {
    pub input_a: bool,
    pub input_b: bool,
    pub short_circuit: bool,
    pub invert_result: bool,
    pub exclusive: bool,
    pub show_truth_table: bool,
    pub output_format: String,
}

impl Default for OrNode {
    fn default() -> Self {
        Self {
            input_a: false,
            input_b: false,
            short_circuit: true,
            invert_result: false,
            exclusive: false,
            show_truth_table: true,
            output_format: "Boolean".to_string(),
        }
    }
}

impl OrNode {
    /// Pattern A: build_interface method that renders UI and returns parameter changes
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("OR Logic Parameters");
        ui.separator();
        
        // Input A (when not connected)
        ui.horizontal(|ui| {
            ui.label("Input A:");
            let mut input_a = node.parameters.get("input_a")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false);
            
            if ui.checkbox(&mut input_a, "").changed() {
                changes.push(ParameterChange {
                    parameter: "input_a".to_string(),
                    value: NodeData::Boolean(input_a),
                });
            }
        });
        
        // Input B (when not connected)
        ui.horizontal(|ui| {
            ui.label("Input B:");
            let mut input_b = node.parameters.get("input_b")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false);
            
            if ui.checkbox(&mut input_b, "").changed() {
                changes.push(ParameterChange {
                    parameter: "input_b".to_string(),
                    value: NodeData::Boolean(input_b),
                });
            }
        });
        
        ui.separator();
        
        // Logic operation modes
        ui.horizontal(|ui| {
            ui.label("Exclusive (XOR):");
            let mut exclusive = node.parameters.get("exclusive")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false);
            
            if ui.checkbox(&mut exclusive, "Use XOR operation").changed() {
                changes.push(ParameterChange {
                    parameter: "exclusive".to_string(),
                    value: NodeData::Boolean(exclusive),
                });
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("Short Circuit:");
            let mut short_circuit = node.parameters.get("short_circuit")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(true);
            
            if ui.checkbox(&mut short_circuit, "Use short-circuit evaluation").changed() {
                changes.push(ParameterChange {
                    parameter: "short_circuit".to_string(),
                    value: NodeData::Boolean(short_circuit),
                });
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("Invert Result:");
            let mut invert_result = node.parameters.get("invert_result")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false);
            
            if ui.checkbox(&mut invert_result, "Invert output (NOR/XNOR)").changed() {
                changes.push(ParameterChange {
                    parameter: "invert_result".to_string(),
                    value: NodeData::Boolean(invert_result),
                });
            }
        });
        
        ui.separator();
        
        // Operation type display
        let exclusive = node.parameters.get("exclusive")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
        let invert = node.parameters.get("invert_result")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
        
        let operation_name = match (exclusive, invert) {
            (true, false) => "XOR",
            (true, true) => "XNOR",
            (false, false) => "OR",
            (false, true) => "NOR",
        };
        
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
                ui.label("A | B | Result");
            });
            
            for a in [false, true] {
                for b in [false, true] {
                    let result = if exclusive {
                        a ^ b
                    } else {
                        a || b
                    };
                    let final_result = if invert { !result } else { result };
                    ui.horizontal(|ui| {
                        ui.label(format!("{} | {} | {}", 
                            if a { "T" } else { "F" },
                            if b { "T" } else { "F" },
                            if final_result { "T" } else { "F" }
                        ));
                    });
                }
            }
            
            ui.separator();
            
            // Quick toggles
            ui.label("Quick Actions:");
            ui.horizontal(|ui| {
                if ui.button("Toggle A").clicked() {
                    let current_a = node.parameters.get("input_a")
                        .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                        .unwrap_or(false);
                    changes.push(ParameterChange {
                        parameter: "input_a".to_string(),
                        value: NodeData::Boolean(!current_a),
                    });
                }
                if ui.button("Toggle B").clicked() {
                    let current_b = node.parameters.get("input_b")
                        .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                        .unwrap_or(false);
                    changes.push(ParameterChange {
                        parameter: "input_b".to_string(),
                        value: NodeData::Boolean(!current_b),
                    });
                }
                if ui.button("Reset").clicked() {
                    changes.push(ParameterChange {
                        parameter: "input_a".to_string(),
                        value: NodeData::Boolean(false),
                    });
                    changes.push(ParameterChange {
                        parameter: "input_b".to_string(),
                        value: NodeData::Boolean(false),
                    });
                }
            });
            
            ui.separator();
            
            // Display current result
            let current_a = node.parameters.get("input_a")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false);
            let current_b = node.parameters.get("input_b")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false);
            
            let result = if exclusive {
                current_a ^ current_b
            } else {
                current_a || current_b
            };
            let final_result = if invert { !result } else { result };
            
            ui.label(format!("Current Result: {}", 
                if final_result { "TRUE" } else { "FALSE" }
            ));
        }
        
        changes
    }
    
    /// Apply parameters to create OrLogic instance
    pub fn create_logic(&self) -> OrLogic {
        OrLogic {
            a: self.input_a,
            b: self.input_b,
            short_circuit: self.short_circuit,
            invert_result: self.invert_result,
            exclusive: self.exclusive,
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