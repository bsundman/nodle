//! Spot light node parameters using Pattern A: build_interface method

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use super::logic::SpotLightLogic;
use std::f32::consts::PI;

/// Spot light node with Pattern A interface
#[derive(Debug, Clone)]
pub struct SpotLightNode {
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub cone_angle: f32,
    pub inner_cone_angle: f32,
    pub constant_attenuation: f32,
    pub linear_attenuation: f32,
    pub quadratic_attenuation: f32,
    pub cast_shadows: bool,
}

impl Default for SpotLightNode {
    fn default() -> Self {
        let light = SpotLightLogic::default();
        Self {
            position: light.position,
            direction: light.direction,
            color: light.color,
            intensity: light.intensity,
            cone_angle: light.cone_angle,
            inner_cone_angle: light.inner_cone_angle,
            constant_attenuation: light.constant_attenuation,
            linear_attenuation: light.linear_attenuation,
            quadratic_attenuation: light.quadratic_attenuation,
            cast_shadows: light.cast_shadows,
        }
    }
}

impl SpotLightNode {
    /// Pattern A: build_interface method that renders UI and returns parameter changes
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("Spot Light Parameters");
        ui.separator();
        
        // Light Presets
        ui.label("Light Presets:");
        ui.horizontal(|ui| {
            if ui.button("Flashlight").clicked() {
                changes.push(ParameterChange {
                    parameter: "cone_angle".to_string(),
                    value: NodeData::Float(PI * 0.15),
                });
                changes.push(ParameterChange {
                    parameter: "inner_cone_angle".to_string(),
                    value: NodeData::Float(PI * 0.1),
                });
                changes.push(ParameterChange {
                    parameter: "intensity".to_string(),
                    value: NodeData::Float(1.5),
                });
            }
            if ui.button("Stage Light").clicked() {
                changes.push(ParameterChange {
                    parameter: "cone_angle".to_string(),
                    value: NodeData::Float(PI * 0.35),
                });
                changes.push(ParameterChange {
                    parameter: "inner_cone_angle".to_string(),
                    value: NodeData::Float(PI * 0.25),
                });
                changes.push(ParameterChange {
                    parameter: "intensity".to_string(),
                    value: NodeData::Float(2.0),
                });
            }
            if ui.button("Narrow Beam").clicked() {
                changes.push(ParameterChange {
                    parameter: "cone_angle".to_string(),
                    value: NodeData::Float(PI * 0.08),
                });
                changes.push(ParameterChange {
                    parameter: "inner_cone_angle".to_string(),
                    value: NodeData::Float(PI * 0.05),
                });
                changes.push(ParameterChange {
                    parameter: "intensity".to_string(),
                    value: NodeData::Float(3.0),
                });
            }
        });
        
        ui.separator();
        
        // Direction Presets
        ui.label("Direction Presets:");
        ui.horizontal(|ui| {
            if ui.button("Down").clicked() {
                changes.push(ParameterChange {
                    parameter: "direction_x".to_string(),
                    value: NodeData::Float(0.0),
                });
                changes.push(ParameterChange {
                    parameter: "direction_y".to_string(),
                    value: NodeData::Float(-1.0),
                });
                changes.push(ParameterChange {
                    parameter: "direction_z".to_string(),
                    value: NodeData::Float(0.0),
                });
            }
            if ui.button("Forward").clicked() {
                changes.push(ParameterChange {
                    parameter: "direction_x".to_string(),
                    value: NodeData::Float(0.0),
                });
                changes.push(ParameterChange {
                    parameter: "direction_y".to_string(),
                    value: NodeData::Float(0.0),
                });
                changes.push(ParameterChange {
                    parameter: "direction_z".to_string(),
                    value: NodeData::Float(-1.0),
                });
            }
            if ui.button("Diagonal").clicked() {
                let len = (0.5_f32 * 0.5 + 0.7 * 0.7 + 0.5 * 0.5).sqrt();
                changes.push(ParameterChange {
                    parameter: "direction_x".to_string(),
                    value: NodeData::Float(0.5 / len),
                });
                changes.push(ParameterChange {
                    parameter: "direction_y".to_string(),
                    value: NodeData::Float(-0.7 / len),
                });
                changes.push(ParameterChange {
                    parameter: "direction_z".to_string(),
                    value: NodeData::Float(-0.5 / len),
                });
            }
        });
        
        ui.separator();
        
        // Position X
        ui.horizontal(|ui| {
            ui.label("Position X:");
            let mut pos_x = node.parameters.get("position_x")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(0.0);
            
            if ui.add(egui::Slider::new(&mut pos_x, -100.0..=100.0).step_by(0.1)).changed() {
                changes.push(ParameterChange {
                    parameter: "position_x".to_string(),
                    value: NodeData::Float(pos_x),
                });
            }
        });
        
        // Position Y
        ui.horizontal(|ui| {
            ui.label("Position Y:");
            let mut pos_y = node.parameters.get("position_y")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(0.0);
            
            if ui.add(egui::Slider::new(&mut pos_y, -100.0..=100.0).step_by(0.1)).changed() {
                changes.push(ParameterChange {
                    parameter: "position_y".to_string(),
                    value: NodeData::Float(pos_y),
                });
            }
        });
        
        // Position Z
        ui.horizontal(|ui| {
            ui.label("Position Z:");
            let mut pos_z = node.parameters.get("position_z")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(0.0);
            
            if ui.add(egui::Slider::new(&mut pos_z, -100.0..=100.0).step_by(0.1)).changed() {
                changes.push(ParameterChange {
                    parameter: "position_z".to_string(),
                    value: NodeData::Float(pos_z),
                });
            }
        });
        
        ui.separator();
        
        // Direction X
        ui.horizontal(|ui| {
            ui.label("Direction X:");
            let mut dir_x = node.parameters.get("direction_x")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(0.0);
            
            if ui.add(egui::Slider::new(&mut dir_x, -1.0..=1.0).step_by(0.01)).changed() {
                changes.push(ParameterChange {
                    parameter: "direction_x".to_string(),
                    value: NodeData::Float(dir_x),
                });
            }
        });
        
        // Direction Y
        ui.horizontal(|ui| {
            ui.label("Direction Y:");
            let mut dir_y = node.parameters.get("direction_y")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(-1.0);
            
            if ui.add(egui::Slider::new(&mut dir_y, -1.0..=1.0).step_by(0.01)).changed() {
                changes.push(ParameterChange {
                    parameter: "direction_y".to_string(),
                    value: NodeData::Float(dir_y),
                });
            }
        });
        
        // Direction Z
        ui.horizontal(|ui| {
            ui.label("Direction Z:");
            let mut dir_z = node.parameters.get("direction_z")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(0.0);
            
            if ui.add(egui::Slider::new(&mut dir_z, -1.0..=1.0).step_by(0.01)).changed() {
                changes.push(ParameterChange {
                    parameter: "direction_z".to_string(),
                    value: NodeData::Float(dir_z),
                });
            }
        });
        
        ui.separator();
        
        // Color R
        ui.horizontal(|ui| {
            ui.label("Color R:");
            let mut color_r = node.parameters.get("color_r")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(1.0);
            
            if ui.add(egui::Slider::new(&mut color_r, 0.0..=1.0).step_by(0.01)).changed() {
                changes.push(ParameterChange {
                    parameter: "color_r".to_string(),
                    value: NodeData::Float(color_r),
                });
            }
        });
        
        // Color G
        ui.horizontal(|ui| {
            ui.label("Color G:");
            let mut color_g = node.parameters.get("color_g")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(1.0);
            
            if ui.add(egui::Slider::new(&mut color_g, 0.0..=1.0).step_by(0.01)).changed() {
                changes.push(ParameterChange {
                    parameter: "color_g".to_string(),
                    value: NodeData::Float(color_g),
                });
            }
        });
        
        // Color B
        ui.horizontal(|ui| {
            ui.label("Color B:");
            let mut color_b = node.parameters.get("color_b")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(1.0);
            
            if ui.add(egui::Slider::new(&mut color_b, 0.0..=1.0).step_by(0.01)).changed() {
                changes.push(ParameterChange {
                    parameter: "color_b".to_string(),
                    value: NodeData::Float(color_b),
                });
            }
        });
        
        ui.separator();
        
        // Intensity
        ui.horizontal(|ui| {
            ui.label("Intensity:");
            let mut intensity = node.parameters.get("intensity")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(1.0);
            
            if ui.add(egui::Slider::new(&mut intensity, 0.0..=10.0).step_by(0.1)).changed() {
                changes.push(ParameterChange {
                    parameter: "intensity".to_string(),
                    value: NodeData::Float(intensity),
                });
            }
        });
        
        // Cone Angle
        ui.horizontal(|ui| {
            ui.label("Cone Angle:");
            let mut cone_angle = node.parameters.get("cone_angle")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(PI * 0.25);
            
            if ui.add(egui::Slider::new(&mut cone_angle, 0.0..=(PI * 0.5)).step_by(0.01)).changed() {
                changes.push(ParameterChange {
                    parameter: "cone_angle".to_string(),
                    value: NodeData::Float(cone_angle),
                });
            }
        });
        
        // Inner Cone Angle
        ui.horizontal(|ui| {
            ui.label("Inner Cone Angle:");
            let mut inner_cone_angle = node.parameters.get("inner_cone_angle")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(PI * 0.2);
            
            if ui.add(egui::Slider::new(&mut inner_cone_angle, 0.0..=(PI * 0.5)).step_by(0.01)).changed() {
                changes.push(ParameterChange {
                    parameter: "inner_cone_angle".to_string(),
                    value: NodeData::Float(inner_cone_angle),
                });
            }
        });
        
        ui.separator();
        
        // Constant Attenuation
        ui.horizontal(|ui| {
            ui.label("Constant Attenuation:");
            let mut constant_atten = node.parameters.get("constant_attenuation")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(1.0);
            
            if ui.add(egui::Slider::new(&mut constant_atten, 0.0..=10.0).step_by(0.01)).changed() {
                changes.push(ParameterChange {
                    parameter: "constant_attenuation".to_string(),
                    value: NodeData::Float(constant_atten),
                });
            }
        });
        
        // Linear Attenuation
        ui.horizontal(|ui| {
            ui.label("Linear Attenuation:");
            let mut linear_atten = node.parameters.get("linear_attenuation")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(0.09);
            
            if ui.add(egui::Slider::new(&mut linear_atten, 0.0..=1.0).step_by(0.001)).changed() {
                changes.push(ParameterChange {
                    parameter: "linear_attenuation".to_string(),
                    value: NodeData::Float(linear_atten),
                });
            }
        });
        
        // Quadratic Attenuation
        ui.horizontal(|ui| {
            ui.label("Quadratic Attenuation:");
            let mut quadratic_atten = node.parameters.get("quadratic_attenuation")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(0.032);
            
            if ui.add(egui::Slider::new(&mut quadratic_atten, 0.0..=1.0).step_by(0.001)).changed() {
                changes.push(ParameterChange {
                    parameter: "quadratic_attenuation".to_string(),
                    value: NodeData::Float(quadratic_atten),
                });
            }
        });
        
        ui.separator();
        
        // Cast Shadows
        ui.horizontal(|ui| {
            ui.label("Cast Shadows:");
            let mut cast_shadows = node.parameters.get("cast_shadows")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(true);
            
            if ui.checkbox(&mut cast_shadows, "").changed() {
                changes.push(ParameterChange {
                    parameter: "cast_shadows".to_string(),
                    value: NodeData::Boolean(cast_shadows),
                });
            }
        });
        
        // Display calculated values
        let cone_angle = node.parameters.get("cone_angle")
            .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
            .unwrap_or(PI * 0.25);
        
        ui.separator();
        ui.label(format!("Cone Angle: {:.1}°", cone_angle.to_degrees()));
        
        changes
    }
    
    /// Convert current parameters to SpotLightLogic for processing
    pub fn to_spot_light_logic(&self) -> SpotLightLogic {
        let mut light = SpotLightLogic {
            position: self.position,
            direction: self.direction,
            color: self.color,
            intensity: self.intensity,
            cone_angle: self.cone_angle,
            inner_cone_angle: self.inner_cone_angle,
            constant_attenuation: self.constant_attenuation,
            linear_attenuation: self.linear_attenuation,
            quadratic_attenuation: self.quadratic_attenuation,
            cast_shadows: self.cast_shadows,
        };
        light.normalize_direction();
        light
    }
}