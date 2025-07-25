//! Sphere node parameter interface with primitive/mesh toggle

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use egui::{Ui, DragValue, ComboBox, Separator};

pub struct SphereParameters;

impl SphereParameters {
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
                        // Trigger reload when mode changes
                        changes.push(ParameterChange {
                            parameter: "needs_reload".to_string(),
                            value: NodeData::Boolean(true),
                        });
                    }
                    if ui.selectable_value(&mut current_mode, "mesh".to_string(), "Mesh").changed() {
                        changes.push(ParameterChange {
                            parameter: "mode".to_string(),
                            value: NodeData::String("mesh".to_string()),
                        });
                        // Trigger reload when mode changes
                        changes.push(ParameterChange {
                            parameter: "needs_reload".to_string(),
                            value: NodeData::Boolean(true),
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
            // Trigger reload when radius changes
            changes.push(ParameterChange {
                parameter: "needs_reload".to_string(),
                value: NodeData::Boolean(true),
            });
        }
        
        ui.add(Separator::default());
        
        // Mesh subdivision parameters (disabled in primitive mode)
        ui.label("Mesh Subdivision:");
        
        let mut rings = node.parameters.get("rings")
            .and_then(|d| if let NodeData::Integer(i) = d { Some(*i) } else { None })
            .unwrap_or(16);
        let mut segments = node.parameters.get("segments")
            .and_then(|d| if let NodeData::Integer(i) = d { Some(*i) } else { None })
            .unwrap_or(20);
        
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!is_primitive_mode, |ui| {
                if ui.add(DragValue::new(&mut rings)
                    .speed(1)
                    .range(4..=64)
                    .prefix("Rings: "))
                    .changed() {
                    changes.push(ParameterChange {
                        parameter: "rings".to_string(),
                        value: NodeData::Integer(rings),
                    });
                    changes.push(ParameterChange {
                        parameter: "needs_reload".to_string(),
                        value: NodeData::Boolean(true),
                    });
                }
                if ui.add(DragValue::new(&mut segments)
                    .speed(1)
                    .range(8..=128)
                    .prefix("Segments: "))
                    .changed() {
                    changes.push(ParameterChange {
                        parameter: "segments".to_string(),
                        value: NodeData::Integer(segments),
                    });
                    changes.push(ParameterChange {
                        parameter: "needs_reload".to_string(),
                        value: NodeData::Boolean(true),
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
                changes.push(ParameterChange {
                    parameter: "needs_reload".to_string(),
                    value: NodeData::Boolean(true),
                });
            }
            if ui.checkbox(&mut generate_uvs, "Generate UVs").changed() {
                changes.push(ParameterChange {
                    parameter: "generate_uvs".to_string(),
                    value: NodeData::Boolean(generate_uvs),
                });
                changes.push(ParameterChange {
                    parameter: "needs_reload".to_string(),
                    value: NodeData::Boolean(true),
                });
            }
        });
        
        // Show mode info
        ui.add(Separator::default());
        if is_primitive_mode {
            ui.label("🔧 Primitive mode: Uses USD procedural sphere primitive");
        } else {
            ui.label("🔧 Mesh mode: Generates tessellated sphere mesh");
        }
        
        // Reset needs_reload flag after parameter changes have been processed
        if node.parameters.get("needs_reload")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false) 
        {
            changes.push(ParameterChange {
                parameter: "needs_reload".to_string(),
                value: NodeData::Boolean(false),
            });
        }
        
        changes
    }
}