//! Sphere node parameters using Pattern A: build_interface method

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use super::logic::{SphereGeometry, SphereType};
use std::f32::consts::PI;

/// Sphere node with Pattern A interface
#[derive(Debug, Clone)]
pub struct SphereNode {
    pub radius: f32,
    pub rings: i32,
    pub segments: i32,
    pub sphere_type: SphereType,
    pub phi_start: f32,
    pub phi_length: f32,
    pub theta_start: f32,
    pub theta_length: f32,
    pub generate_uvs: bool,
    pub generate_normals: bool,
}

impl Default for SphereNode {
    fn default() -> Self {
        Self {
            radius: 1.0,
            rings: 16,
            segments: 32,
            sphere_type: SphereType::Full,
            phi_start: 0.0,
            phi_length: 2.0 * PI,
            theta_start: 0.0,
            theta_length: PI,
            generate_uvs: true,
            generate_normals: true,
        }
    }
}

impl SphereNode {
    /// Pattern A: build_interface method that renders UI and returns parameter changes
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("Sphere Parameters");
        ui.separator();
        
        // Quality Presets
        ui.label("Quality Presets:");
        ui.horizontal(|ui| {
            if ui.button("Low").clicked() {
                changes.push(ParameterChange {
                    parameter: "rings".to_string(),
                    value: NodeData::Integer(8),
                });
                changes.push(ParameterChange {
                    parameter: "segments".to_string(),
                    value: NodeData::Integer(16),
                });
            }
            if ui.button("Medium").clicked() {
                changes.push(ParameterChange {
                    parameter: "rings".to_string(),
                    value: NodeData::Integer(16),
                });
                changes.push(ParameterChange {
                    parameter: "segments".to_string(),
                    value: NodeData::Integer(32),
                });
            }
            if ui.button("High").clicked() {
                changes.push(ParameterChange {
                    parameter: "rings".to_string(),
                    value: NodeData::Integer(32),
                });
                changes.push(ParameterChange {
                    parameter: "segments".to_string(),
                    value: NodeData::Integer(64),
                });
            }
            if ui.button("Ultra").clicked() {
                changes.push(ParameterChange {
                    parameter: "rings".to_string(),
                    value: NodeData::Integer(64),
                });
                changes.push(ParameterChange {
                    parameter: "segments".to_string(),
                    value: NodeData::Integer(128),
                });
            }
        });
        
        ui.separator();
        
        // Size Presets
        ui.label("Size Presets:");
        ui.horizontal(|ui| {
            if ui.button("Marble").clicked() {
                changes.push(ParameterChange {
                    parameter: "radius".to_string(),
                    value: NodeData::Float(0.05),
                });
            }
            if ui.button("Ball").clicked() {
                changes.push(ParameterChange {
                    parameter: "radius".to_string(),
                    value: NodeData::Float(0.5),
                });
            }
            if ui.button("Planet").clicked() {
                changes.push(ParameterChange {
                    parameter: "radius".to_string(),
                    value: NodeData::Float(5.0),
                });
            }
        });
        
        ui.separator();
        
        // Radius
        ui.horizontal(|ui| {
            ui.label("Radius:");
            let mut radius = node.parameters.get("radius")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(1.0);
            
            if ui.add(egui::Slider::new(&mut radius, 0.1..=10.0).step_by(0.1)).changed() {
                changes.push(ParameterChange {
                    parameter: "radius".to_string(),
                    value: NodeData::Float(radius),
                });
            }
        });
        
        // Rings
        ui.horizontal(|ui| {
            ui.label("Rings:");
            let mut rings = node.parameters.get("rings")
                .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
                .unwrap_or(16);
            
            if ui.add(egui::Slider::new(&mut rings, 3..=64)).changed() {
                changes.push(ParameterChange {
                    parameter: "rings".to_string(),
                    value: NodeData::Integer(rings),
                });
            }
        });
        
        // Segments
        ui.horizontal(|ui| {
            ui.label("Segments:");
            let mut segments = node.parameters.get("segments")
                .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
                .unwrap_or(32);
            
            if ui.add(egui::Slider::new(&mut segments, 3..=128)).changed() {
                changes.push(ParameterChange {
                    parameter: "segments".to_string(),
                    value: NodeData::Integer(segments),
                });
            }
        });
        
        ui.separator();
        
        // Type
        let current_type = node.parameters.get("sphere_type")
            .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
            .unwrap_or(0);
            
        ui.horizontal(|ui| {
            ui.label("Type:");
            
            let type_names = ["Full", "Hemisphere", "Quarter", "Custom"];
            let mut selected = current_type as usize;
            
            egui::ComboBox::from_id_salt("sphere_type")
                .selected_text(*type_names.get(selected).unwrap_or(&"Full"))
                .show_ui(ui, |ui| {
                    for (i, name) in type_names.iter().enumerate() {
                        if ui.selectable_value(&mut selected, i, *name).changed() {
                            changes.push(ParameterChange {
                                parameter: "sphere_type".to_string(),
                                value: NodeData::Integer(i as i32),
                            });
                            
                            // Auto-update angular parameters for non-custom types
                            if i != 3 { // Not Custom
                                match i {
                                    0 => { // Full
                                        changes.push(ParameterChange {
                                            parameter: "phi_start".to_string(),
                                            value: NodeData::Float(0.0),
                                        });
                                        changes.push(ParameterChange {
                                            parameter: "phi_length".to_string(),
                                            value: NodeData::Float(2.0 * PI),
                                        });
                                        changes.push(ParameterChange {
                                            parameter: "theta_start".to_string(),
                                            value: NodeData::Float(0.0),
                                        });
                                        changes.push(ParameterChange {
                                            parameter: "theta_length".to_string(),
                                            value: NodeData::Float(PI),
                                        });
                                    },
                                    1 => { // Hemisphere
                                        changes.push(ParameterChange {
                                            parameter: "phi_start".to_string(),
                                            value: NodeData::Float(0.0),
                                        });
                                        changes.push(ParameterChange {
                                            parameter: "phi_length".to_string(),
                                            value: NodeData::Float(2.0 * PI),
                                        });
                                        changes.push(ParameterChange {
                                            parameter: "theta_start".to_string(),
                                            value: NodeData::Float(0.0),
                                        });
                                        changes.push(ParameterChange {
                                            parameter: "theta_length".to_string(),
                                            value: NodeData::Float(PI * 0.5),
                                        });
                                    },
                                    2 => { // Quarter
                                        changes.push(ParameterChange {
                                            parameter: "phi_start".to_string(),
                                            value: NodeData::Float(0.0),
                                        });
                                        changes.push(ParameterChange {
                                            parameter: "phi_length".to_string(),
                                            value: NodeData::Float(PI * 0.5),
                                        });
                                        changes.push(ParameterChange {
                                            parameter: "theta_start".to_string(),
                                            value: NodeData::Float(0.0),
                                        });
                                        changes.push(ParameterChange {
                                            parameter: "theta_length".to_string(),
                                            value: NodeData::Float(PI * 0.5),
                                        });
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
                });
        });
        
        // Angular parameters (show for Custom type or always visible)
        let show_angles = current_type == 3; // Custom type
        
        if show_angles {
            ui.separator();
            ui.label("Angular Parameters:");
            
            // Phi Start
            ui.horizontal(|ui| {
                ui.label("Phi Start:");
                let mut phi_start = node.parameters.get("phi_start")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(0.0);
                
                if ui.add(egui::Slider::new(&mut phi_start, 0.0..=2.0 * PI).step_by(0.1)).changed() {
                    changes.push(ParameterChange {
                        parameter: "phi_start".to_string(),
                        value: NodeData::Float(phi_start),
                    });
                    // Auto-set to custom type when manually adjusting
                    if current_type != 3 {
                        changes.push(ParameterChange {
                            parameter: "sphere_type".to_string(),
                            value: NodeData::Integer(3),
                        });
                    }
                }
            });
            
            // Phi Length
            ui.horizontal(|ui| {
                ui.label("Phi Length:");
                let mut phi_length = node.parameters.get("phi_length")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(2.0 * PI);
                
                if ui.add(egui::Slider::new(&mut phi_length, 0.0..=2.0 * PI).step_by(0.1)).changed() {
                    changes.push(ParameterChange {
                        parameter: "phi_length".to_string(),
                        value: NodeData::Float(phi_length),
                    });
                    // Auto-set to custom type when manually adjusting
                    if current_type != 3 {
                        changes.push(ParameterChange {
                            parameter: "sphere_type".to_string(),
                            value: NodeData::Integer(3),
                        });
                    }
                }
            });
            
            // Theta Start
            ui.horizontal(|ui| {
                ui.label("Theta Start:");
                let mut theta_start = node.parameters.get("theta_start")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(0.0);
                
                if ui.add(egui::Slider::new(&mut theta_start, 0.0..=PI).step_by(0.1)).changed() {
                    changes.push(ParameterChange {
                        parameter: "theta_start".to_string(),
                        value: NodeData::Float(theta_start),
                    });
                    // Auto-set to custom type when manually adjusting
                    if current_type != 3 {
                        changes.push(ParameterChange {
                            parameter: "sphere_type".to_string(),
                            value: NodeData::Integer(3),
                        });
                    }
                }
            });
            
            // Theta Length
            ui.horizontal(|ui| {
                ui.label("Theta Length:");
                let mut theta_length = node.parameters.get("theta_length")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(PI);
                
                if ui.add(egui::Slider::new(&mut theta_length, 0.0..=PI).step_by(0.1)).changed() {
                    changes.push(ParameterChange {
                        parameter: "theta_length".to_string(),
                        value: NodeData::Float(theta_length),
                    });
                    // Auto-set to custom type when manually adjusting
                    if current_type != 3 {
                        changes.push(ParameterChange {
                            parameter: "sphere_type".to_string(),
                            value: NodeData::Integer(3),
                        });
                    }
                }
            });
        }
        
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
    
    /// Convert current parameters to SphereGeometry for processing
    pub fn to_sphere_geometry(&self) -> SphereGeometry {
        SphereGeometry {
            radius: self.radius,
            rings: self.rings,
            segments: self.segments,
            phi_start: self.phi_start,
            phi_length: self.phi_length,
            theta_start: self.theta_start,
            theta_length: self.theta_length,
            generate_uvs: self.generate_uvs,
            generate_normals: self.generate_normals,
            sphere_type: self.sphere_type.clone(),
        }
    }
}