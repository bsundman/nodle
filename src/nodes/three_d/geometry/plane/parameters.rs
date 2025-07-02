//! Plane node parameters using Pattern A: build_interface method

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use super::logic::PlaneGeometry;

/// Plane node with Pattern A interface
#[derive(Debug, Clone)]
pub struct PlaneNode {
    pub width: f32,
    pub height: f32,
    pub width_segments: i32,
    pub height_segments: i32,
    pub generate_uvs: bool,
    pub generate_normals: bool,
}

impl Default for PlaneNode {
    fn default() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
            width_segments: 1,
            height_segments: 1,
            generate_uvs: true,
            generate_normals: true,
        }
    }
}

impl PlaneNode {
    /// Pattern A: build_interface method that renders UI and returns parameter changes
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("Plane Parameters");
        ui.separator();
        
        // Size Presets
        ui.label("Size Presets:");
        ui.horizontal(|ui| {
            if ui.button("Square").clicked() {
                changes.push(ParameterChange {
                    parameter: "width".to_string(),
                    value: NodeData::Float(1.0),
                });
                changes.push(ParameterChange {
                    parameter: "height".to_string(),
                    value: NodeData::Float(1.0),
                });
            }
            if ui.button("Rectangle").clicked() {
                changes.push(ParameterChange {
                    parameter: "width".to_string(),
                    value: NodeData::Float(2.0),
                });
                changes.push(ParameterChange {
                    parameter: "height".to_string(),
                    value: NodeData::Float(1.0),
                });
            }
            if ui.button("Wall").clicked() {
                changes.push(ParameterChange {
                    parameter: "width".to_string(),
                    value: NodeData::Float(1.0),
                });
                changes.push(ParameterChange {
                    parameter: "height".to_string(),
                    value: NodeData::Float(3.0),
                });
            }
            if ui.button("Floor").clicked() {
                changes.push(ParameterChange {
                    parameter: "width".to_string(),
                    value: NodeData::Float(5.0),
                });
                changes.push(ParameterChange {
                    parameter: "height".to_string(),
                    value: NodeData::Float(5.0),
                });
            }
        });
        
        ui.separator();
        
        // Subdivision Presets
        ui.label("Subdivision Presets:");
        ui.horizontal(|ui| {
            if ui.button("Low").clicked() {
                changes.push(ParameterChange {
                    parameter: "width_segments".to_string(),
                    value: NodeData::Integer(1),
                });
                changes.push(ParameterChange {
                    parameter: "height_segments".to_string(),
                    value: NodeData::Integer(1),
                });
            }
            if ui.button("Medium").clicked() {
                changes.push(ParameterChange {
                    parameter: "width_segments".to_string(),
                    value: NodeData::Integer(5),
                });
                changes.push(ParameterChange {
                    parameter: "height_segments".to_string(),
                    value: NodeData::Integer(5),
                });
            }
            if ui.button("High").clicked() {
                changes.push(ParameterChange {
                    parameter: "width_segments".to_string(),
                    value: NodeData::Integer(10),
                });
                changes.push(ParameterChange {
                    parameter: "height_segments".to_string(),
                    value: NodeData::Integer(10),
                });
            }
        });
        
        ui.separator();
        
        // Width
        ui.horizontal(|ui| {
            ui.label("Width:");
            let mut width = node.parameters.get("width")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(1.0);
            
            if ui.add(egui::Slider::new(&mut width, 0.1..=20.0).step_by(0.1)).changed() {
                changes.push(ParameterChange {
                    parameter: "width".to_string(),
                    value: NodeData::Float(width),
                });
            }
        });
        
        // Height
        ui.horizontal(|ui| {
            ui.label("Height:");
            let mut height = node.parameters.get("height")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(1.0);
            
            if ui.add(egui::Slider::new(&mut height, 0.1..=20.0).step_by(0.1)).changed() {
                changes.push(ParameterChange {
                    parameter: "height".to_string(),
                    value: NodeData::Float(height),
                });
            }
        });
        
        ui.separator();
        
        // Width Segments
        ui.horizontal(|ui| {
            ui.label("Width Segments:");
            let mut width_segments = node.parameters.get("width_segments")
                .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
                .unwrap_or(1);
            
            if ui.add(egui::Slider::new(&mut width_segments, 1..=50)).changed() {
                changes.push(ParameterChange {
                    parameter: "width_segments".to_string(),
                    value: NodeData::Integer(width_segments),
                });
            }
        });
        
        // Height Segments
        ui.horizontal(|ui| {
            ui.label("Height Segments:");
            let mut height_segments = node.parameters.get("height_segments")
                .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
                .unwrap_or(1);
            
            if ui.add(egui::Slider::new(&mut height_segments, 1..=50)).changed() {
                changes.push(ParameterChange {
                    parameter: "height_segments".to_string(),
                    value: NodeData::Integer(height_segments),
                });
            }
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
    
    /// Convert current parameters to PlaneGeometry for processing
    pub fn to_plane_geometry(&self) -> PlaneGeometry {
        PlaneGeometry {
            width: self.width,
            height: self.height,
            width_segments: self.width_segments,
            height_segments: self.height_segments,
            generate_uvs: self.generate_uvs,
            generate_normals: self.generate_normals,
        }
    }
}