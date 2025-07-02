//! Cube node parameters using Pattern A: build_interface method

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use super::logic::{CubeGeometry, PivotType};

/// Cube node with Pattern A interface
#[derive(Debug, Clone)]
pub struct CubeNode {
    pub size_x: f32,
    pub size_y: f32,
    pub size_z: f32,
    pub subdiv_x: i32,
    pub subdiv_y: i32,
    pub subdiv_z: i32,
    pub pivot: PivotType,
    pub generate_uvs: bool,
    pub generate_normals: bool,
}

impl Default for CubeNode {
    fn default() -> Self {
        Self {
            size_x: 1.0,
            size_y: 1.0,
            size_z: 1.0,
            subdiv_x: 1,
            subdiv_y: 1,
            subdiv_z: 1,
            pivot: PivotType::Center,
            generate_uvs: true,
            generate_normals: true,
        }
    }
}

impl CubeNode {
    /// Pattern A: build_interface method that renders UI and returns parameter changes
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("Cube Parameters");
        ui.separator();
        
        // Quick Size Presets
        ui.label("Quick Size Presets:");
        ui.horizontal(|ui| {
            if ui.button("Unit Cube").clicked() {
                changes.push(ParameterChange {
                    parameter: "size_x".to_string(),
                    value: NodeData::Float(1.0),
                });
                changes.push(ParameterChange {
                    parameter: "size_y".to_string(),
                    value: NodeData::Float(1.0),
                });
                changes.push(ParameterChange {
                    parameter: "size_z".to_string(),
                    value: NodeData::Float(1.0),
                });
            }
            if ui.button("Thin Panel").clicked() {
                changes.push(ParameterChange {
                    parameter: "size_x".to_string(),
                    value: NodeData::Float(2.0),
                });
                changes.push(ParameterChange {
                    parameter: "size_y".to_string(),
                    value: NodeData::Float(0.1),
                });
                changes.push(ParameterChange {
                    parameter: "size_z".to_string(),
                    value: NodeData::Float(1.0),
                });
            }
            if ui.button("Tall Tower").clicked() {
                changes.push(ParameterChange {
                    parameter: "size_x".to_string(),
                    value: NodeData::Float(0.5),
                });
                changes.push(ParameterChange {
                    parameter: "size_y".to_string(),
                    value: NodeData::Float(3.0),
                });
                changes.push(ParameterChange {
                    parameter: "size_z".to_string(),
                    value: NodeData::Float(0.5),
                });
            }
        });
        
        ui.separator();
        
        // Quick Subdivision Presets
        ui.label("Quick Subdivision Presets:");
        ui.horizontal(|ui| {
            if ui.button("Low Detail").clicked() {
                changes.push(ParameterChange {
                    parameter: "subdiv_x".to_string(),
                    value: NodeData::Integer(1),
                });
                changes.push(ParameterChange {
                    parameter: "subdiv_y".to_string(),
                    value: NodeData::Integer(1),
                });
                changes.push(ParameterChange {
                    parameter: "subdiv_z".to_string(),
                    value: NodeData::Integer(1),
                });
            }
            if ui.button("Medium Detail").clicked() {
                changes.push(ParameterChange {
                    parameter: "subdiv_x".to_string(),
                    value: NodeData::Integer(3),
                });
                changes.push(ParameterChange {
                    parameter: "subdiv_y".to_string(),
                    value: NodeData::Integer(3),
                });
                changes.push(ParameterChange {
                    parameter: "subdiv_z".to_string(),
                    value: NodeData::Integer(3),
                });
            }
            if ui.button("High Detail").clicked() {
                changes.push(ParameterChange {
                    parameter: "subdiv_x".to_string(),
                    value: NodeData::Integer(8),
                });
                changes.push(ParameterChange {
                    parameter: "subdiv_y".to_string(),
                    value: NodeData::Integer(8),
                });
                changes.push(ParameterChange {
                    parameter: "subdiv_z".to_string(),
                    value: NodeData::Integer(8),
                });
            }
        });
        
        ui.separator();
        
        // Size X
        ui.horizontal(|ui| {
            ui.label("Size X:");
            let mut size_x = node.parameters.get("size_x")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(1.0);
            
            if ui.add(egui::Slider::new(&mut size_x, 0.1..=10.0).step_by(0.1)).changed() {
                changes.push(ParameterChange {
                    parameter: "size_x".to_string(),
                    value: NodeData::Float(size_x),
                });
            }
        });
        
        // Size Y
        ui.horizontal(|ui| {
            ui.label("Size Y:");
            let mut size_y = node.parameters.get("size_y")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(1.0);
            
            if ui.add(egui::Slider::new(&mut size_y, 0.1..=10.0).step_by(0.1)).changed() {
                changes.push(ParameterChange {
                    parameter: "size_y".to_string(),
                    value: NodeData::Float(size_y),
                });
            }
        });
        
        // Size Z
        ui.horizontal(|ui| {
            ui.label("Size Z:");
            let mut size_z = node.parameters.get("size_z")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(1.0);
            
            if ui.add(egui::Slider::new(&mut size_z, 0.1..=10.0).step_by(0.1)).changed() {
                changes.push(ParameterChange {
                    parameter: "size_z".to_string(),
                    value: NodeData::Float(size_z),
                });
            }
        });
        
        ui.separator();
        
        // Subdiv X
        ui.horizontal(|ui| {
            ui.label("Subdiv X:");
            let mut subdiv_x = node.parameters.get("subdiv_x")
                .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
                .unwrap_or(1);
            
            if ui.add(egui::Slider::new(&mut subdiv_x, 1..=20)).changed() {
                changes.push(ParameterChange {
                    parameter: "subdiv_x".to_string(),
                    value: NodeData::Integer(subdiv_x),
                });
            }
        });
        
        // Subdiv Y
        ui.horizontal(|ui| {
            ui.label("Subdiv Y:");
            let mut subdiv_y = node.parameters.get("subdiv_y")
                .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
                .unwrap_or(1);
            
            if ui.add(egui::Slider::new(&mut subdiv_y, 1..=20)).changed() {
                changes.push(ParameterChange {
                    parameter: "subdiv_y".to_string(),
                    value: NodeData::Integer(subdiv_y),
                });
            }
        });
        
        // Subdiv Z
        ui.horizontal(|ui| {
            ui.label("Subdiv Z:");
            let mut subdiv_z = node.parameters.get("subdiv_z")
                .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
                .unwrap_or(1);
            
            if ui.add(egui::Slider::new(&mut subdiv_z, 1..=20)).changed() {
                changes.push(ParameterChange {
                    parameter: "subdiv_z".to_string(),
                    value: NodeData::Integer(subdiv_z),
                });
            }
        });
        
        ui.separator();
        
        // Pivot
        let current_pivot = node.parameters.get("pivot")
            .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
            .unwrap_or(0);
            
        ui.horizontal(|ui| {
            ui.label("Pivot:");
            
            let pivot_names = ["Center", "Corner", "Bottom"];
            let mut selected = current_pivot as usize;
            
            egui::ComboBox::from_id_source("pivot")
                .selected_text(*pivot_names.get(selected).unwrap_or(&"Center"))
                .show_ui(ui, |ui| {
                    for (i, name) in pivot_names.iter().enumerate() {
                        if ui.selectable_value(&mut selected, i, *name).changed() {
                            changes.push(ParameterChange {
                                parameter: "pivot".to_string(),
                                value: NodeData::Integer(i as i32),
                            });
                        }
                    }
                });
        });
        
        ui.separator();
        
        // Generate UVs
        ui.horizontal(|ui| {
            ui.label("Generate UVs:");
            let mut generate_uvs = node.parameters.get("generate_uvs")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(true);
            
            if ui.checkbox(&mut generate_uvs, "").changed() {
                changes.push(ParameterChange {
                    parameter: "generate_uvs".to_string(),
                    value: NodeData::Boolean(generate_uvs),
                });
            }
        });
        
        // Generate Normals
        ui.horizontal(|ui| {
            ui.label("Generate Normals:");
            let mut generate_normals = node.parameters.get("generate_normals")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(true);
            
            if ui.checkbox(&mut generate_normals, "").changed() {
                changes.push(ParameterChange {
                    parameter: "generate_normals".to_string(),
                    value: NodeData::Boolean(generate_normals),
                });
            }
        });
        
        changes
    }
    
    /// Convert current parameters to CubeGeometry for processing
    pub fn to_cube_geometry(&self) -> CubeGeometry {
        CubeGeometry {
            size: [self.size_x, self.size_y, self.size_z],
            subdivisions: [self.subdiv_x, self.subdiv_y, self.subdiv_z],
            pivot: self.pivot.clone(),
            generate_uvs: self.generate_uvs,
            generate_normals: self.generate_normals,
        }
    }
}