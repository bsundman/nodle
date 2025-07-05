//! Null node parameters using Pattern A: build_interface method

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use super::logic::NullLogic;

/// Null node with Pattern A interface
#[derive(Debug, Clone)]
pub struct NullNode {
    pub label: String,
    pub enabled: bool,
    pub description: String,
    pub color: [f32; 3], // RGB color for visual organization
    pub visible_in_hierarchy: bool,
}

impl Default for NullNode {
    fn default() -> Self {
        Self {
            label: "Null".to_string(),
            enabled: true,
            description: "Passthrough node for organization".to_string(),
            color: [0.5, 0.5, 0.5], // Default gray
            visible_in_hierarchy: true,
        }
    }
}

impl NullNode {
    /// Pattern A: build_interface method that renders UI and returns parameter changes
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("Null Node Parameters");
        ui.separator();
        
        // Label/Name
        ui.horizontal(|ui| {
            ui.label("Label:");
            let mut label = node.parameters.get("label")
                .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or_else(|| "Null".to_string());
            
            if ui.text_edit_singleline(&mut label).changed() {
                changes.push(ParameterChange {
                    parameter: "label".to_string(),
                    value: NodeData::String(label),
                });
            }
        });
        
        ui.add_space(5.0);
        
        // Enabled toggle
        ui.horizontal(|ui| {
            ui.label("Enabled:");
            let mut enabled = node.parameters.get("enabled")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(true);
            
            if ui.checkbox(&mut enabled, "Pass data through").changed() {
                changes.push(ParameterChange {
                    parameter: "enabled".to_string(),
                    value: NodeData::Boolean(enabled),
                });
            }
        });
        
        if !node.parameters.get("enabled")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(true) {
            ui.label("⚠️ Disabled - outputs None regardless of input");
        }
        
        ui.separator();
        
        // Description
        ui.label("Description:");
        let mut description = node.parameters.get("description")
            .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
            .unwrap_or_else(|| "Passthrough node for organization".to_string());
        
        if ui.text_edit_multiline(&mut description).changed() {
            changes.push(ParameterChange {
                parameter: "description".to_string(),
                value: NodeData::String(description),
            });
        }
        
        ui.separator();
        
        // Visual organization
        ui.collapsing("Visual Options", |ui| {
            // Color picker for organization
            ui.horizontal(|ui| {
                ui.label("Color:");
                let current_color = node.parameters.get("color")
                    .and_then(|v| if let NodeData::Color(c) = v { Some(*c) } else { None })
                    .unwrap_or([0.5, 0.5, 0.5, 1.0]);
                
                let mut color = egui::Color32::from_rgba_premultiplied(
                    (current_color[0] * 255.0) as u8,
                    (current_color[1] * 255.0) as u8,
                    (current_color[2] * 255.0) as u8,
                    (current_color[3] * 255.0) as u8,
                );
                
                if ui.color_edit_button_srgba(&mut color).changed() {
                    let [r, g, b, a] = color.to_array();
                    changes.push(ParameterChange {
                        parameter: "color".to_string(),
                        value: NodeData::Color([
                            r as f32 / 255.0,
                            g as f32 / 255.0,
                            b as f32 / 255.0,
                            a as f32 / 255.0,
                        ]),
                    });
                }
            });
            
            // Hierarchy visibility
            ui.horizontal(|ui| {
                ui.label("Visible in Hierarchy:");
                let mut visible = node.parameters.get("visible_in_hierarchy")
                    .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                    .unwrap_or(true);
                
                if ui.checkbox(&mut visible, "Show in node hierarchy").changed() {
                    changes.push(ParameterChange {
                        parameter: "visible_in_hierarchy".to_string(),
                        value: NodeData::Boolean(visible),
                    });
                }
            });
        });
        
        ui.separator();
        
        // Quick actions
        ui.horizontal(|ui| {
            if ui.button("Reset to Default").clicked() {
                changes.push(ParameterChange {
                    parameter: "label".to_string(),
                    value: NodeData::String("Null".to_string()),
                });
                changes.push(ParameterChange {
                    parameter: "enabled".to_string(),
                    value: NodeData::Boolean(true),
                });
                changes.push(ParameterChange {
                    parameter: "description".to_string(),
                    value: NodeData::String("Passthrough node for organization".to_string()),
                });
                changes.push(ParameterChange {
                    parameter: "color".to_string(),
                    value: NodeData::Color([0.5, 0.5, 0.5, 1.0]),
                });
                changes.push(ParameterChange {
                    parameter: "visible_in_hierarchy".to_string(),
                    value: NodeData::Boolean(true),
                });
            }
            
            if ui.button("Disable Node").clicked() {
                changes.push(ParameterChange {
                    parameter: "enabled".to_string(),
                    value: NodeData::Boolean(false),
                });
            }
        });
        
        ui.separator();
        
        // Status display
        let enabled = node.parameters.get("enabled")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(true);
        let label = node.parameters.get("label")
            .and_then(|v| if let NodeData::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("Null");
        
        ui.label(format!("Status: {} ({})", 
            if enabled { "Enabled" } else { "Disabled" },
            label
        ));
        
        changes
    }
    
    /// Convert current parameters to NullLogic for processing
    pub fn to_null_logic(&self) -> NullLogic {
        NullLogic {
            label: self.label.clone(),
            enabled: self.enabled,
            description: self.description.clone(),
        }
    }
}