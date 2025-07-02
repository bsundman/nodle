//! USD Material node parameter interface

use crate::nodes::interface::{build_parameter_ui, NodeData, ParameterChange};
use crate::nodes::Node;

/// USD Material node with parameter controls
#[derive(Default)]
pub struct USDMaterialNode;

impl USDMaterialNode {
    /// Build the parameter interface
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("USD Material");
        ui.separator();
        
        // Material name
        if let Some(change) = build_parameter_ui(
            ui,
            "name",
            "Name",
            node.parameters.get("name").cloned().unwrap_or(NodeData::String("".to_string())),
            |ui, value| {
                if let NodeData::String(ref s) = value {
                    let mut text = s.clone();
                    let response = ui.text_edit_singleline(&mut text);
                    if response.changed() {
                        return Some(NodeData::String(text));
                    }
                }
                None
            }
        ) {
            node.parameters.insert("name".to_string(), change.clone());
            changes.push(ParameterChange {
                parameter: "name".to_string(),
                value: change,
            });
        }
        
        // Preview surface controls
        ui.separator();
        ui.label("Preview Surface");
        
        // Base color
        if let Some(change) = build_parameter_ui(
            ui,
            "diffuse_color",
            "Diffuse Color",
            node.parameters.get("diffuse_color").cloned().unwrap_or(NodeData::Color([0.8, 0.8, 0.8, 1.0])),
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
            node.parameters.insert("diffuse_color".to_string(), change.clone());
            changes.push(ParameterChange {
                parameter: "diffuse_color".to_string(),
                value: change,
            });
        }
        
        // Metallic
        if let Some(change) = build_parameter_ui(
            ui,
            "metallic",
            "Metallic",
            node.parameters.get("metallic").cloned().unwrap_or(NodeData::Float(0.0)),
            |ui, value| {
                if let NodeData::Float(ref f) = value {
                    let mut val = *f;
                    let response = ui.add(
                        egui::Slider::new(&mut val, 0.0..=1.0)
                            .text("Metallic")
                    );
                    if response.changed() {
                        return Some(NodeData::Float(val));
                    }
                }
                None
            }
        ) {
            node.parameters.insert("metallic".to_string(), change.clone());
            changes.push(ParameterChange {
                parameter: "metallic".to_string(),
                value: change,
            });
        }
        
        // Roughness
        if let Some(change) = build_parameter_ui(
            ui,
            "roughness",
            "Roughness",
            node.parameters.get("roughness").cloned().unwrap_or(NodeData::Float(0.4)),
            |ui, value| {
                if let NodeData::Float(ref f) = value {
                    let mut val = *f;
                    let response = ui.add(
                        egui::Slider::new(&mut val, 0.0..=1.0)
                            .text("Roughness")
                    );
                    if response.changed() {
                        return Some(NodeData::Float(val));
                    }
                }
                None
            }
        ) {
            node.parameters.insert("roughness".to_string(), change.clone());
            changes.push(ParameterChange {
                parameter: "roughness".to_string(),
                value: change,
            });
        }
        
        // Opacity
        if let Some(change) = build_parameter_ui(
            ui,
            "opacity",
            "Opacity",
            node.parameters.get("opacity").cloned().unwrap_or(NodeData::Float(1.0)),
            |ui, value| {
                if let NodeData::Float(ref f) = value {
                    let mut val = *f;
                    let response = ui.add(
                        egui::Slider::new(&mut val, 0.0..=1.0)
                            .text("Opacity")
                    );
                    if response.changed() {
                        return Some(NodeData::Float(val));
                    }
                }
                None
            }
        ) {
            node.parameters.insert("opacity".to_string(), change.clone());
            changes.push(ParameterChange {
                parameter: "opacity".to_string(),
                value: change,
            });
        }
        
        // IOR (Index of Refraction)
        if let Some(change) = build_parameter_ui(
            ui,
            "ior",
            "IOR",
            node.parameters.get("ior").cloned().unwrap_or(NodeData::Float(1.5)),
            |ui, value| {
                if let NodeData::Float(ref f) = value {
                    let mut val = *f;
                    let response = ui.add(
                        egui::DragValue::new(&mut val)
                            .speed(0.01)
                            .clamp_range(1.0..=3.0)
                    );
                    if response.changed() {
                        return Some(NodeData::Float(val));
                    }
                }
                None
            }
        ) {
            node.parameters.insert("ior".to_string(), change.clone());
            changes.push(ParameterChange {
                parameter: "ior".to_string(),
                value: change,
            });
        }
        
        changes
    }
}