//! Cone node parameter interface with primitive/mesh toggle

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use egui::{Ui, DragValue, ComboBox, Separator};

pub struct ConeParameters;

impl ConeParameters {
    pub fn build_interface(node: &mut Node, ui: &mut Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        // Get current mode
        let mut current_mode = node.parameters.get("mode")
            .and_then(|d| if let NodeData::String(s) = d { Some(s.clone()) } else { None })
            .unwrap_or_else(|| "primitive".to_string());
        
        let is_primitive_mode = current_mode == "primitive";
        
        // Mode selector
        ui.horizontal(|ui| {
            ComboBox::from_label("Mode")
                .selected_text(if is_primitive_mode { "Primitive" } else { "Mesh" })
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut current_mode, "primitive".to_string(), "Primitive").changed() {
                        changes.push(ParameterChange {
                            parameter: "mode".to_string(),
                            value: NodeData::String("primitive".to_string()),
                        });
                    }
                    if ui.selectable_value(&mut current_mode, "mesh".to_string(), "Mesh").changed() {
                        changes.push(ParameterChange {
                            parameter: "mode".to_string(),
                            value: NodeData::String("mesh".to_string()),
                        });
                    }
                });
        });
        
        ui.add(Separator::default());
        
        // Radius parameter
        let mut radius = node.parameters.get("radius")
            .and_then(|d| if let NodeData::Float(f) = d { Some(*f) } else { None })
            .unwrap_or(1.0);
        
        if ui.add(DragValue::new(&mut radius)
            .speed(0.1)
            .range(0.1..=10.0)
            .prefix("Radius: "))
            .changed() {
            changes.push(ParameterChange {
                parameter: "radius".to_string(),
                value: NodeData::Float(radius),
            });
        }
        
        // Height parameter
        let mut height = node.parameters.get("height")
            .and_then(|d| if let NodeData::Float(f) = d { Some(*f) } else { None })
            .unwrap_or(2.0);
        
        if ui.add(DragValue::new(&mut height)
            .speed(0.1)
            .range(0.1..=10.0)
            .prefix("Height: "))
            .changed() {
            changes.push(ParameterChange {
                parameter: "height".to_string(),
                value: NodeData::Float(height),
            });
        }
        
        ui.add(Separator::default());
        
        // Mesh subdivision parameters (disabled in primitive mode)
        ui.label("Mesh Subdivision:");
        
        let mut subdivisions_axis = node.parameters.get("subdivisions_axis")
            .and_then(|d| if let NodeData::Integer(i) = d { Some(*i) } else { None })
            .unwrap_or(20);
        let mut subdivisions_caps = node.parameters.get("subdivisions_caps")
            .and_then(|d| if let NodeData::Integer(i) = d { Some(*i) } else { None })
            .unwrap_or(1);
        let mut subdivisions_height = node.parameters.get("subdivisions_height")
            .and_then(|d| if let NodeData::Integer(i) = d { Some(*i) } else { None })
            .unwrap_or(1);
        
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!is_primitive_mode, |ui| {
                if ui.add(DragValue::new(&mut subdivisions_axis)
                    .speed(1)
                    .range(8..=64)
                    .prefix("Axis: "))
                    .changed() {
                    changes.push(ParameterChange {
                        parameter: "subdivisions_axis".to_string(),
                        value: NodeData::Integer(subdivisions_axis),
                    });
                }
                if ui.add(DragValue::new(&mut subdivisions_caps)
                    .speed(1)
                    .range(1..=20)
                    .prefix("Caps: "))
                    .changed() {
                    changes.push(ParameterChange {
                        parameter: "subdivisions_caps".to_string(),
                        value: NodeData::Integer(subdivisions_caps),
                    });
                }
                if ui.add(DragValue::new(&mut subdivisions_height)
                    .speed(1)
                    .range(1..=20)
                    .prefix("Height: "))
                    .changed() {
                    changes.push(ParameterChange {
                        parameter: "subdivisions_height".to_string(),
                        value: NodeData::Integer(subdivisions_height),
                    });
                }
            });
        });
        
        // Mesh options
        let mut smooth_normals = node.parameters.get("smooth_normals")
            .and_then(|d| if let NodeData::Boolean(b) = d { Some(*b) } else { None })
            .unwrap_or(true);
        let mut generate_uvs = node.parameters.get("generate_uvs")
            .and_then(|d| if let NodeData::Boolean(b) = d { Some(*b) } else { None })
            .unwrap_or(true);
        
        ui.add_enabled_ui(!is_primitive_mode, |ui| {
            if ui.checkbox(&mut smooth_normals, "Smooth Normals").changed() {
                changes.push(ParameterChange {
                    parameter: "smooth_normals".to_string(),
                    value: NodeData::Boolean(smooth_normals),
                });
            }
            if ui.checkbox(&mut generate_uvs, "Generate UVs").changed() {
                changes.push(ParameterChange {
                    parameter: "generate_uvs".to_string(),
                    value: NodeData::Boolean(generate_uvs),
                });
            }
        });
        
        // Show mode info
        ui.add(Separator::default());
        if is_primitive_mode {
            ui.label("🔧 Primitive mode: Uses USD procedural cone primitive");
        } else {
            ui.label("🔧 Mesh mode: Generates tessellated cone mesh");
        }
        
        changes
    }
}