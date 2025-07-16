//! Cube node parameter interface with primitive/mesh toggle

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use egui::{Ui, DragValue, ComboBox, Separator};

pub struct CubeParameters;

impl CubeParameters {
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
        
        // Size parameters
        ui.label("Size:");
        
        // Uniform size option
        let mut size = node.parameters.get("size")
            .and_then(|d| if let NodeData::Float(f) = d { Some(*f) } else { None })
            .unwrap_or(2.0);
        
        if ui.add(DragValue::new(&mut size)
            .speed(0.1)
            .range(0.1..=10.0)
            .prefix("Uniform: "))
            .changed() {
            changes.push(ParameterChange {
                parameter: "size".to_string(),
                value: NodeData::Float(size),
            });
            // Update individual size components
            changes.push(ParameterChange {
                parameter: "size_x".to_string(),
                value: NodeData::Float(size),
            });
            changes.push(ParameterChange {
                parameter: "size_y".to_string(),
                value: NodeData::Float(size),
            });
            changes.push(ParameterChange {
                parameter: "size_z".to_string(),
                value: NodeData::Float(size),
            });
            // Trigger reload when size changes
            changes.push(ParameterChange {
                parameter: "needs_reload".to_string(),
                value: NodeData::Boolean(true),
            });
        }
        
        // Individual size controls
        let mut size_x = node.parameters.get("size_x")
            .and_then(|d| if let NodeData::Float(f) = d { Some(*f) } else { None })
            .unwrap_or(2.0);
        let mut size_y = node.parameters.get("size_y")
            .and_then(|d| if let NodeData::Float(f) = d { Some(*f) } else { None })
            .unwrap_or(2.0);
        let mut size_z = node.parameters.get("size_z")
            .and_then(|d| if let NodeData::Float(f) = d { Some(*f) } else { None })
            .unwrap_or(2.0);
        
        ui.horizontal(|ui| {
            if ui.add(DragValue::new(&mut size_x)
                .speed(0.1)
                .range(0.1..=10.0)
                .prefix("X: "))
                .changed() {
                changes.push(ParameterChange {
                    parameter: "size_x".to_string(),
                    value: NodeData::Float(size_x),
                });
                changes.push(ParameterChange {
                    parameter: "needs_reload".to_string(),
                    value: NodeData::Boolean(true),
                });
            }
            if ui.add(DragValue::new(&mut size_y)
                .speed(0.1)
                .range(0.1..=10.0)
                .prefix("Y: "))
                .changed() {
                changes.push(ParameterChange {
                    parameter: "size_y".to_string(),
                    value: NodeData::Float(size_y),
                });
                changes.push(ParameterChange {
                    parameter: "needs_reload".to_string(),
                    value: NodeData::Boolean(true),
                });
            }
            if ui.add(DragValue::new(&mut size_z)
                .speed(0.1)
                .range(0.1..=10.0)
                .prefix("Z: "))
                .changed() {
                changes.push(ParameterChange {
                    parameter: "size_z".to_string(),
                    value: NodeData::Float(size_z),
                });
                changes.push(ParameterChange {
                    parameter: "needs_reload".to_string(),
                    value: NodeData::Boolean(true),
                });
            }
        });
        
        ui.add(Separator::default());
        
        // Mesh subdivision parameters (disabled in primitive mode)
        ui.label("Mesh Subdivision:");
        
        let mut subdivisions_x = node.parameters.get("subdivisions_x")
            .and_then(|d| if let NodeData::Integer(i) = d { Some(*i) } else { None })
            .unwrap_or(1);
        let mut subdivisions_y = node.parameters.get("subdivisions_y")
            .and_then(|d| if let NodeData::Integer(i) = d { Some(*i) } else { None })
            .unwrap_or(1);
        let mut subdivisions_z = node.parameters.get("subdivisions_z")
            .and_then(|d| if let NodeData::Integer(i) = d { Some(*i) } else { None })
            .unwrap_or(1);
        
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!is_primitive_mode, |ui| {
                if ui.add(DragValue::new(&mut subdivisions_x)
                    .speed(1)
                    .range(1..=20)
                    .prefix("X: "))
                    .changed() {
                    changes.push(ParameterChange {
                        parameter: "subdivisions_x".to_string(),
                        value: NodeData::Integer(subdivisions_x),
                    });
                    changes.push(ParameterChange {
                        parameter: "needs_reload".to_string(),
                        value: NodeData::Boolean(true),
                    });
                }
                if ui.add(DragValue::new(&mut subdivisions_y)
                    .speed(1)
                    .range(1..=20)
                    .prefix("Y: "))
                    .changed() {
                    changes.push(ParameterChange {
                        parameter: "subdivisions_y".to_string(),
                        value: NodeData::Integer(subdivisions_y),
                    });
                    changes.push(ParameterChange {
                        parameter: "needs_reload".to_string(),
                        value: NodeData::Boolean(true),
                    });
                }
                if ui.add(DragValue::new(&mut subdivisions_z)
                    .speed(1)
                    .range(1..=20)
                    .prefix("Z: "))
                    .changed() {
                    changes.push(ParameterChange {
                        parameter: "subdivisions_z".to_string(),
                        value: NodeData::Integer(subdivisions_z),
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
            .unwrap_or(false);
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
            ui.label("🔧 Primitive mode: Uses USD procedural cube primitive");
        } else {
            ui.label("🔧 Mesh mode: Generates tessellated mesh geometry");
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