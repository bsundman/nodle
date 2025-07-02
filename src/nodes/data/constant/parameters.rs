//! Constant node parameters using Pattern A: build_interface method

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use super::logic::{ConstantLogic, ConstantValue};

/// Constant node with Pattern A interface
#[derive(Debug, Clone)]
pub struct ConstantNode {
    pub logic: ConstantLogic,
}

impl Default for ConstantNode {
    fn default() -> Self {
        Self {
            logic: ConstantLogic::default(),
        }
    }
}

impl ConstantNode {
    /// Pattern A: build_interface method that renders UI and returns parameter changes
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("Constant Parameters");
        ui.separator();
        
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
        
        // Value input based on type
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
                
                // Float-specific options
                ui.horizontal(|ui| {
                    ui.label("Format Output:");
                    let mut format_output = node.parameters.get("format_output")
                        .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                        .unwrap_or(false);
                    
                    if ui.checkbox(&mut format_output, "").changed() {
                        changes.push(ParameterChange {
                            parameter: "format_output".to_string(),
                            value: NodeData::Boolean(format_output),
                        });
                    }
                });
                
                let format_enabled = node.parameters.get("format_output")
                    .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                    .unwrap_or(false);
                
                if format_enabled {
                    ui.horizontal(|ui| {
                        ui.label("Decimal Places:");
                        let mut decimal_places = node.parameters.get("decimal_places")
                            .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
                            .unwrap_or(2);
                        
                        if ui.add(egui::Slider::new(&mut decimal_places, 0..=10)).changed() {
                            changes.push(ParameterChange {
                                parameter: "decimal_places".to_string(),
                                value: NodeData::Integer(decimal_places),
                            });
                        }
                    });
                }
                
                // Float presets
                ui.separator();
                ui.label("Presets:");
                ui.horizontal(|ui| {
                    for preset in [0.0, 1.0, -1.0, 3.14159, 2.71828] {
                        if ui.button(format!("{:.3}", preset)).clicked() {
                            changes.push(ParameterChange {
                                parameter: "value".to_string(),
                                value: NodeData::Float(preset),
                            });
                        }
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
                
                // Integer presets
                ui.separator();
                ui.label("Presets:");
                ui.horizontal(|ui| {
                    for preset in [0, 1, -1, 10, 100] {
                        if ui.button(preset.to_string()).clicked() {
                            changes.push(ParameterChange {
                                parameter: "value".to_string(),
                                value: NodeData::Integer(preset),
                            });
                        }
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
                
                // Boolean presets
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("True").clicked() {
                        changes.push(ParameterChange {
                            parameter: "value".to_string(),
                            value: NodeData::Boolean(true),
                        });
                    }
                    if ui.button("False").clicked() {
                        changes.push(ParameterChange {
                            parameter: "value".to_string(),
                            value: NodeData::Boolean(false),
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
            if let Some(NodeData::Float(f)) = node.parameters.get("value") {
                let format_output = node.parameters.get("format_output")
                    .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                    .unwrap_or(false);
                
                if format_output {
                    let decimal_places = node.parameters.get("decimal_places")
                        .and_then(|v| if let NodeData::Integer(i) = v { Some(*i as usize) } else { None })
                        .unwrap_or(2);
                    Some(format!("{:.1$}", f, decimal_places))
                } else {
                    Some(f.to_string())
                }
            } else {
                None
            }
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