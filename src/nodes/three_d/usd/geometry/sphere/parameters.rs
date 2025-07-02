//! USD Sphere node parameter interface

use crate::nodes::interface::{build_parameter_ui, NodeData, ParameterChange};
use crate::nodes::Node;

/// USD Sphere node with parameter controls
#[derive(Default)]
pub struct USDSphereNode;

impl USDSphereNode {
    /// Build the parameter interface
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("USD Sphere");
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
        
        // Subdivisions (for tessellation)
        ui.separator();
        ui.label("Tessellation");
        
        if let Some(change) = build_parameter_ui(
            ui,
            "subdivisions",
            "Subdivisions",
            node.parameters.get("subdivisions").cloned().unwrap_or(NodeData::Integer(32)),
            |ui, value| {
                if let NodeData::Integer(ref i) = value {
                    let mut val = *i;
                    let response = ui.add(
                        egui::DragValue::new(&mut val)
                            .speed(1.0)
                            .clamp_range(4..=128)
                    );
                    if response.changed() {
                        return Some(NodeData::Integer(val));
                    }
                }
                None
            }
        ) {
            node.parameters.insert("subdivisions".to_string(), change.clone());
            changes.push(ParameterChange {
                parameter: "subdivisions".to_string(),
                value: change,
            });
        }
        
        // Display options
        ui.separator();
        ui.label("Display");
        
        if let Some(change) = build_parameter_ui(
            ui,
            "purpose",
            "Purpose",
            node.parameters.get("purpose").cloned().unwrap_or(NodeData::String("default".to_string())),
            |ui, value| {
                if let NodeData::String(ref s) = value {
                    let mut current = s.clone();
                    let mut changed = false;
                    
                    egui::ComboBox::from_label("")
                        .selected_text(&current)
                        .show_ui(ui, |ui| {
                            for purpose in &["default", "render", "proxy", "guide"] {
                                if ui.selectable_value(&mut current, purpose.to_string(), *purpose).clicked() {
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
            node.parameters.insert("purpose".to_string(), change.clone());
            changes.push(ParameterChange {
                parameter: "purpose".to_string(),
                value: change,
            });
        }
        
        if let Some(change) = build_parameter_ui(
            ui,
            "visibility",
            "Visibility",
            node.parameters.get("visibility").cloned().unwrap_or(NodeData::String("inherited".to_string())),
            |ui, value| {
                if let NodeData::String(ref s) = value {
                    let mut current = s.clone();
                    let mut changed = false;
                    
                    egui::ComboBox::from_label("")
                        .selected_text(&current)
                        .show_ui(ui, |ui| {
                            for vis in &["inherited", "visible", "invisible"] {
                                if ui.selectable_value(&mut current, vis.to_string(), *vis).clicked() {
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
            node.parameters.insert("visibility".to_string(), change.clone());
            changes.push(ParameterChange {
                parameter: "visibility".to_string(),
                value: change,
            });
        }
        
        changes
    }
}