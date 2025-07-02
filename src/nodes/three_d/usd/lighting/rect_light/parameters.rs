//! USD Rect Light node parameter interface

use crate::nodes::interface::{build_parameter_ui, NodeData, ParameterChange};
use crate::nodes::Node;

/// USD Rect Light node with parameter controls
#[derive(Default)]
pub struct USDRectLightNode;

impl USDRectLightNode {
    /// Build the parameter interface
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("USD Rect Light");
        ui.separator();
        
        // Intensity
        if let Some(change) = build_parameter_ui(
            ui,
            "intensity",
            "Intensity",
            node.parameters.get("intensity").cloned().unwrap_or(NodeData::Float(1.0)),
            |ui, value| {
                if let NodeData::Float(ref f) = value {
                    let mut val = *f;
                    let response = ui.add(
                        egui::DragValue::new(&mut val)
                            .speed(0.01)
                            .clamp_range(0.0..=100.0)
                    );
                    if response.changed() {
                        return Some(NodeData::Float(val));
                    }
                }
                None
            }
        ) {
            node.parameters.insert("intensity".to_string(), change.clone());
            changes.push(ParameterChange {
                parameter: "intensity".to_string(),
                value: change,
            });
        }
        
        // Color
        if let Some(change) = build_parameter_ui(
            ui,
            "color",
            "Color",
            node.parameters.get("color").cloned().unwrap_or(NodeData::Color([1.0, 1.0, 1.0, 1.0])),
            |ui, value| {
                if let NodeData::Color(ref color) = value {
                    let mut col = [color[0], color[1], color[2]];
                    let response = ui.color_edit_button_rgb(&mut col);
                    if response.changed() {
                        return Some(NodeData::Color([col[0], col[1], col[2], color[3]]));
                    }
                }
                None
            }
        ) {
            node.parameters.insert("color".to_string(), change.clone());
            changes.push(ParameterChange {
                parameter: "color".to_string(),
                value: change,
            });
        }
        
        // Temperature
        if let Some(change) = build_parameter_ui(
            ui,
            "temperature",
            "Temperature",
            node.parameters.get("temperature").cloned().unwrap_or(NodeData::Float(6500.0)),
            |ui, value| {
                if let NodeData::Float(ref f) = value {
                    let mut val = *f;
                    let response = ui.add(
                        egui::DragValue::new(&mut val)
                            .speed(10.0)
                            .clamp_range(1000.0..=12000.0)
                            .suffix(" K")
                    );
                    if response.changed() {
                        return Some(NodeData::Float(val));
                    }
                }
                None
            }
        ) {
            node.parameters.insert("temperature".to_string(), change.clone());
            changes.push(ParameterChange {
                parameter: "temperature".to_string(),
                value: change,
            });
        }
        
        // Dimensions
        ui.separator();
        ui.label("Dimensions");
        
        if let Some(change) = build_parameter_ui(
            ui,
            "width",
            "Width",
            node.parameters.get("width").cloned().unwrap_or(NodeData::Float(1.0)),
            |ui, value| {
                if let NodeData::Float(ref f) = value {
                    let mut val = *f;
                    let response = ui.add(
                        egui::DragValue::new(&mut val)
                            .speed(0.01)
                            .clamp_range(0.01..=100.0)
                            .suffix(" units")
                    );
                    if response.changed() {
                        return Some(NodeData::Float(val));
                    }
                }
                None
            }
        ) {
            node.parameters.insert("width".to_string(), change.clone());
            changes.push(ParameterChange {
                parameter: "width".to_string(),
                value: change,
            });
        }
        
        if let Some(change) = build_parameter_ui(
            ui,
            "height",
            "Height",
            node.parameters.get("height").cloned().unwrap_or(NodeData::Float(1.0)),
            |ui, value| {
                if let NodeData::Float(ref f) = value {
                    let mut val = *f;
                    let response = ui.add(
                        egui::DragValue::new(&mut val)
                            .speed(0.01)
                            .clamp_range(0.01..=100.0)
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
        
        // Enable/disable
        ui.separator();
        if let Some(change) = build_parameter_ui(
            ui,
            "enabled",
            "Enabled",
            node.parameters.get("enabled").cloned().unwrap_or(NodeData::Boolean(true)),
            |ui, value| {
                if let NodeData::Boolean(ref b) = value {
                    let mut checked = *b;
                    let response = ui.checkbox(&mut checked, "Light enabled");
                    if response.changed() {
                        return Some(NodeData::Boolean(checked));
                    }
                }
                None
            }
        ) {
            node.parameters.insert("enabled".to_string(), change.clone());
            changes.push(ParameterChange {
                parameter: "enabled".to_string(),
                value: change,
            });
        }
        
        changes
    }
}