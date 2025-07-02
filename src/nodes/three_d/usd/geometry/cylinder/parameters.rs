//! USD Cylinder node parameter interface

use crate::nodes::interface::{build_parameter_ui, NodeData, ParameterChange};
use crate::nodes::Node;

/// USD Cylinder node with parameter controls
#[derive(Default)]
pub struct USDCylinderNode;

impl USDCylinderNode {
    /// Build the parameter interface
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("USD Cylinder");
        ui.separator();
        
        // Radius parameter
        if let Some(change) = build_parameter_ui(
            ui,
            "radius",
            "Radius",
            node.parameters.get("radius").cloned().unwrap_or(NodeData::Float(1.0)),
            |ui, value| {
                if let NodeData::Float(ref f) = value {
                    let mut val = *f;
                    let response = ui.add(
                        egui::DragValue::new(&mut val)
                            .speed(0.01)
                            .clamp_range(0.001..=100.0)
                            .suffix(" units")
                    );
                    if response.changed() {
                        return Some(NodeData::Float(val));
                    }
                }
                None
            }
        ) {
            node.parameters.insert("radius".to_string(), change.clone());
            changes.push(ParameterChange {
                parameter: "radius".to_string(),
                value: change,
            });
        }
        
        // Height parameter
        if let Some(change) = build_parameter_ui(
            ui,
            "height",
            "Height",
            node.parameters.get("height").cloned().unwrap_or(NodeData::Float(2.0)),
            |ui, value| {
                if let NodeData::Float(ref f) = value {
                    let mut val = *f;
                    let response = ui.add(
                        egui::DragValue::new(&mut val)
                            .speed(0.01)
                            .clamp_range(0.001..=100.0)
                            .suffix(" units")
                    );
                    if response.changed() {
                        return Some(NodeData::Float(val));
                    }
                }
                None
            }
        ) {
            node.parameters.insert("height".to_string(), change.clone());
            changes.push(ParameterChange {
                parameter: "height".to_string(),
                value: change,
            });
        }
        
        // Axis parameter
        ui.separator();
        ui.label("Orientation");
        
        if let Some(change) = build_parameter_ui(
            ui,
            "axis",
            "Axis",
            node.parameters.get("axis").cloned().unwrap_or(NodeData::String("Y".to_string())),
            |ui, value| {
                if let NodeData::String(ref s) = value {
                    let mut current = s.clone();
                    let mut changed = false;
                    
                    egui::ComboBox::from_label("")
                        .selected_text(&current)
                        .show_ui(ui, |ui| {
                            for axis in &["X", "Y", "Z"] {
                                if ui.selectable_value(&mut current, axis.to_string(), *axis).clicked() {
                                    changed = true;
                                }
                            }
                        });
                    
                    if changed {
                        return Some(NodeData::String(current));
                    }
                }
                None
            }
        ) {
            node.parameters.insert("axis".to_string(), change.clone());
            changes.push(ParameterChange {
                parameter: "axis".to_string(),
                value: change,
            });
        }
        
        // Caps
        ui.separator();
        ui.label("Caps");
        
        if let Some(change) = build_parameter_ui(
            ui,
            "top_cap",
            "Top Cap",
            node.parameters.get("top_cap").cloned().unwrap_or(NodeData::Boolean(true)),
            |ui, value| {
                if let NodeData::Boolean(ref b) = value {
                    let mut checked = *b;
                    let response = ui.checkbox(&mut checked, "Top cap");
                    if response.changed() {
                        return Some(NodeData::Boolean(checked));
                    }
                }
                None
            }
        ) {
            node.parameters.insert("top_cap".to_string(), change.clone());
            changes.push(ParameterChange {
                parameter: "top_cap".to_string(),
                value: change,
            });
        }
        
        if let Some(change) = build_parameter_ui(
            ui,
            "bottom_cap",
            "Bottom Cap",
            node.parameters.get("bottom_cap").cloned().unwrap_or(NodeData::Boolean(true)),
            |ui, value| {
                if let NodeData::Boolean(ref b) = value {
                    let mut checked = *b;
                    let response = ui.checkbox(&mut checked, "Bottom cap");
                    if response.changed() {
                        return Some(NodeData::Boolean(checked));
                    }
                }
                None
            }
        ) {
            node.parameters.insert("bottom_cap".to_string(), change.clone());
            changes.push(ParameterChange {
                parameter: "bottom_cap".to_string(),
                value: change,
            });
        }
        
        changes
    }
}