//! Variable node parameters using Pattern A: build_interface method

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use super::logic::{VariableLogic, ChangeRecord};
use super::super::constant::logic::ConstantValue;

/// Variable node with Pattern A interface
#[derive(Debug, Clone)]
pub struct VariableNode {
    pub logic: VariableLogic,
}

impl Default for VariableNode {
    fn default() -> Self {
        Self {
            logic: VariableLogic::default(),
        }
    }
}

impl VariableNode {
    /// Pattern A: build_interface method that renders UI and returns parameter changes
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("Variable Parameters");
        ui.separator();
        
        // Variable name
        ui.horizontal(|ui| {
            ui.label("Name:");
            let mut name = node.parameters.get("name")
                .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or_else(|| "variable".to_string());
            
            if ui.text_edit_singleline(&mut name).changed() {
                changes.push(ParameterChange {
                    parameter: "name".to_string(),
                    value: NodeData::String(name),
                });
            }
        });
        
        // Get current value type
        let current_type = node.parameters.get("value_type")
            .and_then(|v| if let NodeData::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("Float");
        
        // Type selection
        ui.label("Value Type:");
        ui.horizontal(|ui| {
            for value_type in ["Float", "Integer", "Boolean", "Text", "Vector3", "Color"] {
                if ui.selectable_label(current_type == value_type, value_type).clicked() {
                    if current_type != value_type {
                        changes.push(ParameterChange {
                            parameter: "value_type".to_string(),
                            value: NodeData::String(value_type.to_string()),
                        });
                        
                        // Set default value for the new type
                        let default_value = match value_type {
                            "Float" => NodeData::Float(0.0),
                            "Integer" => NodeData::Integer(0),
                            "Boolean" => NodeData::Boolean(false),
                            "Text" => NodeData::String("".to_string()),
                            "Vector3" => NodeData::Vector3([0.0, 0.0, 0.0]),
                            "Color" => NodeData::Color([1.0, 1.0, 1.0, 1.0]),
                            _ => NodeData::Float(0.0),
                        };
                        changes.push(ParameterChange {
                            parameter: "value".to_string(),
                            value: default_value,
                        });
                    }
                }
            }
        });
        
        ui.separator();
        
        // Read-only setting
        ui.horizontal(|ui| {
            ui.label("Read Only:");
            let mut read_only = node.parameters.get("read_only")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false);
            
            if ui.checkbox(&mut read_only, "").changed() {
                changes.push(ParameterChange {
                    parameter: "read_only".to_string(),
                    value: NodeData::Boolean(read_only),
                });
            }
        });
        
        // Log changes setting
        ui.horizontal(|ui| {
            ui.label("Log Changes:");
            let mut log_changes = node.parameters.get("log_changes")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false);
            
            if ui.checkbox(&mut log_changes, "").changed() {
                changes.push(ParameterChange {
                    parameter: "log_changes".to_string(),
                    value: NodeData::Boolean(log_changes),
                });
            }
        });
        
        ui.separator();
        
        // Variable status
        let is_read_only = node.parameters.get("read_only")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
        
        ui.horizontal(|ui| {
            ui.label("Status:");
            if is_read_only {
                ui.colored_label(egui::Color32::YELLOW, "Read-Only");
            } else {
                ui.colored_label(egui::Color32::GREEN, "Writable");
            }
        });
        
        ui.separator();
        
        // Value input based on type (only if not read-only)
        if !is_read_only {
            match current_type {
                "Float" => {
                    ui.horizontal(|ui| {
                        ui.label("Value:");
                        let mut value = node.parameters.get("value")
                            .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                            .unwrap_or(0.0);
                        
                        if ui.add(egui::DragValue::new(&mut value).speed(0.01)).changed() {
                            changes.push(ParameterChange {
                                parameter: "value".to_string(),
                                value: NodeData::Float(value),
                            });
                        }
                    });
                },
                "Integer" => {
                    ui.horizontal(|ui| {
                        ui.label("Value:");
                        let mut value = node.parameters.get("value")
                            .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
                            .unwrap_or(0);
                        
                        if ui.add(egui::DragValue::new(&mut value)).changed() {
                            changes.push(ParameterChange {
                                parameter: "value".to_string(),
                                value: NodeData::Integer(value),
                            });
                        }
                    });
                },
                "Boolean" => {
                    ui.horizontal(|ui| {
                        ui.label("Value:");
                        let mut value = node.parameters.get("value")
                            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                            .unwrap_or(false);
                        
                        if ui.checkbox(&mut value, "").changed() {
                            changes.push(ParameterChange {
                                parameter: "value".to_string(),
                                value: NodeData::Boolean(value),
                            });
                        }
                    });
                },
                "Text" => {
                    ui.horizontal(|ui| {
                        ui.label("Value:");
                        let mut value = node.parameters.get("value")
                            .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
                            .unwrap_or_default();
                        
                        if ui.text_edit_singleline(&mut value).changed() {
                            changes.push(ParameterChange {
                                parameter: "value".to_string(),
                                value: NodeData::String(value),
                            });
                        }
                    });
                },
                "Vector3" => {
                    let current_vec = node.parameters.get("value")
                        .and_then(|v| if let NodeData::Vector3(vec) = v { Some(*vec) } else { None })
                        .unwrap_or([0.0, 0.0, 0.0]);
                    
                    let mut x = current_vec[0];
                    let mut y = current_vec[1];
                    let mut z = current_vec[2];
                    
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        if ui.add(egui::DragValue::new(&mut x).speed(0.01)).changed() {
                            changes.push(ParameterChange {
                                parameter: "value".to_string(),
                                value: NodeData::Vector3([x, y, z]),
                            });
                        }
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Y:");
                        if ui.add(egui::DragValue::new(&mut y).speed(0.01)).changed() {
                            changes.push(ParameterChange {
                                parameter: "value".to_string(),
                                value: NodeData::Vector3([x, y, z]),
                            });
                        }
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Z:");
                        if ui.add(egui::DragValue::new(&mut z).speed(0.01)).changed() {
                            changes.push(ParameterChange {
                                parameter: "value".to_string(),
                                value: NodeData::Vector3([x, y, z]),
                            });
                        }
                    });
                },
                "Color" => {
                    let current_color = node.parameters.get("value")
                        .and_then(|v| if let NodeData::Color(col) = v { Some(*col) } else { None })
                        .unwrap_or([1.0, 1.0, 1.0, 1.0]);
                    
                    let mut r = current_color[0];
                    let mut g = current_color[1];
                    let mut b = current_color[2];
                    let mut a = current_color[3];
                    
                    ui.horizontal(|ui| {
                        ui.label("R:");
                        if ui.add(egui::Slider::new(&mut r, 0.0..=1.0)).changed() {
                            changes.push(ParameterChange {
                                parameter: "value".to_string(),
                                value: NodeData::Color([r, g, b, a]),
                            });
                        }
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("G:");
                        if ui.add(egui::Slider::new(&mut g, 0.0..=1.0)).changed() {
                            changes.push(ParameterChange {
                                parameter: "value".to_string(),
                                value: NodeData::Color([r, g, b, a]),
                            });
                        }
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("B:");
                        if ui.add(egui::Slider::new(&mut b, 0.0..=1.0)).changed() {
                            changes.push(ParameterChange {
                                parameter: "value".to_string(),
                                value: NodeData::Color([r, g, b, a]),
                            });
                        }
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("A:");
                        if ui.add(egui::Slider::new(&mut a, 0.0..=1.0)).changed() {
                            changes.push(ParameterChange {
                                parameter: "value".to_string(),
                                value: NodeData::Color([r, g, b, a]),
                            });
                        }
                    });
                    
                    // Color preview
                    ui.separator();
                    let preview_color = egui::Color32::from_rgba_unmultiplied(
                        (r * 255.0) as u8,
                        (g * 255.0) as u8,
                        (b * 255.0) as u8,
                        (a * 255.0) as u8,
                    );
                    ui.horizontal(|ui| {
                        ui.label("Preview:");
                        ui.colored_label(preview_color, "████");
                    });
                },
                _ => {}
            }
        }
        
        ui.separator();
        
        // Quick actions
        ui.label("Quick Actions:");
        ui.horizontal(|ui| {
            if ui.button("Reset").clicked() && !is_read_only {
                // Reset to default value based on type
                let default_value = match current_type {
                    "Float" => NodeData::Float(0.0),
                    "Integer" => NodeData::Integer(0),
                    "Boolean" => NodeData::Boolean(false),
                    "Text" => NodeData::String("".to_string()),
                    "Vector3" => NodeData::Vector3([0.0, 0.0, 0.0]),
                    "Color" => NodeData::Color([0.0, 0.0, 0.0, 1.0]),
                    _ => NodeData::Float(0.0),
                };
                changes.push(ParameterChange {
                    parameter: "value".to_string(),
                    value: default_value,
                });
            }
            
            if ui.button("Toggle Read-Only").clicked() {
                changes.push(ParameterChange {
                    parameter: "read_only".to_string(),
                    value: NodeData::Boolean(!is_read_only),
                });
            }
        });
        
        // Change history info
        let log_changes = node.parameters.get("log_changes")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
        
        if log_changes {
            ui.separator();
            let change_count = node.parameters.get("change_count")
                .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
                .unwrap_or(0);
            
            ui.label(format!("Changes Recorded: {}", change_count));
            
            if ui.button("Clear History").clicked() {
                changes.push(ParameterChange {
                    parameter: "change_count".to_string(),
                    value: NodeData::Integer(0),
                });
            }
        }
        
        // Display current value
        ui.separator();
        if let Some(formatted_value) = get_formatted_value(node) {
            ui.label(format!("Current Value: {}", formatted_value));
        }
        
        changes
    }
}

/// Helper function to get formatted value string
fn get_formatted_value(node: &Node) -> Option<String> {
    let value_type = node.parameters.get("value_type")
        .and_then(|v| if let NodeData::String(s) = v { Some(s.as_str()) } else { None })
        .unwrap_or("Float");
    
    match value_type {
        "Float" => {
            node.parameters.get("value")
                .and_then(|v| if let NodeData::Float(f) = v { Some(format!("{:.3}", f)) } else { None })
        },
        "Integer" => {
            node.parameters.get("value")
                .and_then(|v| if let NodeData::Integer(i) = v { Some(i.to_string()) } else { None })
        },
        "Boolean" => {
            node.parameters.get("value")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(b.to_string()) } else { None })
        },
        "Text" => {
            node.parameters.get("value")
                .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
        },
        "Vector3" => {
            node.parameters.get("value")
                .and_then(|v| if let NodeData::Vector3(vec) = v { 
                    Some(format!("[{:.2}, {:.2}, {:.2}]", vec[0], vec[1], vec[2]))
                } else { None })
        },
        "Color" => {
            node.parameters.get("value")
                .and_then(|v| if let NodeData::Color(col) = v { 
                    Some(format!("rgba({:.2}, {:.2}, {:.2}, {:.2})", col[0], col[1], col[2], col[3]))
                } else { None })
        },
        _ => None
    }
}