//! Directional light node parameters using Pattern A: build_interface method

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use super::logic::DirectionalLightLogic;
use std::f32::consts::PI;

/// Directional light node with Pattern A interface
#[derive(Debug, Clone)]
pub struct DirectionalLightNode {
    pub direction: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub cast_shadows: bool,
    pub shadow_resolution: i32,
    /// Helper angles for easier UI control
    pub azimuth: f32,
    pub elevation: f32,
}

impl Default for DirectionalLightNode {
    fn default() -> Self {
        let light = DirectionalLightLogic::default();
        Self {
            direction: light.direction,
            color: light.color,
            intensity: light.intensity,
            cast_shadows: light.cast_shadows,
            shadow_resolution: light.shadow_resolution,
            azimuth: 0.0,
            elevation: PI * 0.25, // 45 degrees
        }
    }
}

impl DirectionalLightNode {
    /// Helper method to set direction from angles
    pub fn set_direction_from_angles(&mut self, azimuth: f32, elevation: f32) {
        self.azimuth = azimuth;
        self.elevation = elevation;
        
        // Convert spherical coordinates to direction vector
        let x = elevation.cos() * azimuth.sin();
        let y = -elevation.sin(); // Negative for downward direction
        let z = elevation.cos() * azimuth.cos();
        
        self.direction = [x, y, z];
        
        // Normalize the direction
        let len = (x * x + y * y + z * z).sqrt();
        if len > 0.0 {
            self.direction[0] /= len;
            self.direction[1] /= len;
            self.direction[2] /= len;
        }
    }
    
    /// Pattern A: build_interface method that renders UI and returns parameter changes
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("Directional Light Parameters");
        ui.separator();
        
        // Sun Presets
        ui.label("Sun Presets:");
        ui.horizontal(|ui| {
            if ui.button("Noon").clicked() {
                changes.push(ParameterChange {
                    parameter: "azimuth".to_string(),
                    value: NodeData::Float(0.0),
                });
                changes.push(ParameterChange {
                    parameter: "elevation".to_string(),
                    value: NodeData::Float(PI * 0.5),
                });
            }
            if ui.button("Morning").clicked() {
                changes.push(ParameterChange {
                    parameter: "azimuth".to_string(),
                    value: NodeData::Float(PI * 0.25),
                });
                changes.push(ParameterChange {
                    parameter: "elevation".to_string(),
                    value: NodeData::Float(PI * 0.25),
                });
            }
            if ui.button("Evening").clicked() {
                changes.push(ParameterChange {
                    parameter: "azimuth".to_string(),
                    value: NodeData::Float(PI * 1.75),
                });
                changes.push(ParameterChange {
                    parameter: "elevation".to_string(),
                    value: NodeData::Float(PI * 0.15),
                });
            }
        });
        
        ui.separator();
        
        // Color Presets
        ui.label("Color Presets:");
        ui.horizontal(|ui| {
            if ui.button("Daylight").clicked() {
                changes.push(ParameterChange {
                    parameter: "color_r".to_string(),
                    value: NodeData::Float(1.0),
                });
                changes.push(ParameterChange {
                    parameter: "color_g".to_string(),
                    value: NodeData::Float(1.0),
                });
                changes.push(ParameterChange {
                    parameter: "color_b".to_string(),
                    value: NodeData::Float(1.0),
                });
                changes.push(ParameterChange {
                    parameter: "intensity".to_string(),
                    value: NodeData::Float(1.0),
                });
            }
            if ui.button("Sunset").clicked() {
                changes.push(ParameterChange {
                    parameter: "color_r".to_string(),
                    value: NodeData::Float(1.0),
                });
                changes.push(ParameterChange {
                    parameter: "color_g".to_string(),
                    value: NodeData::Float(0.6),
                });
                changes.push(ParameterChange {
                    parameter: "color_b".to_string(),
                    value: NodeData::Float(0.3),
                });
                changes.push(ParameterChange {
                    parameter: "intensity".to_string(),
                    value: NodeData::Float(0.8),
                });
            }
            if ui.button("Moonlight").clicked() {
                changes.push(ParameterChange {
                    parameter: "color_r".to_string(),
                    value: NodeData::Float(0.7),
                });
                changes.push(ParameterChange {
                    parameter: "color_g".to_string(),
                    value: NodeData::Float(0.8),
                });
                changes.push(ParameterChange {
                    parameter: "color_b".to_string(),
                    value: NodeData::Float(1.0),
                });
                changes.push(ParameterChange {
                    parameter: "intensity".to_string(),
                    value: NodeData::Float(0.2),
                });
            }
        });
        
        ui.separator();
        
        // Azimuth (angle around Y axis)
        ui.horizontal(|ui| {
            ui.label("Azimuth:");
            let mut azimuth = node.parameters.get("azimuth")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(0.0);
            
            if ui.add(egui::Slider::new(&mut azimuth, 0.0..=(2.0 * PI)).step_by(0.01)).changed() {
                changes.push(ParameterChange {
                    parameter: "azimuth".to_string(),
                    value: NodeData::Float(azimuth),
                });
            }
        });
        
        // Elevation (angle from horizontal)
        ui.horizontal(|ui| {
            ui.label("Elevation:");
            let mut elevation = node.parameters.get("elevation")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(PI * 0.25);
            
            if ui.add(egui::Slider::new(&mut elevation, -PI * 0.5..=(PI * 0.5)).step_by(0.01)).changed() {
                changes.push(ParameterChange {
                    parameter: "elevation".to_string(),
                    value: NodeData::Float(elevation),
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
        
        // Shadow Resolution
        ui.horizontal(|ui| {
            ui.label("Shadow Resolution:");
            let mut shadow_resolution = node.parameters.get("shadow_resolution")
                .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
                .unwrap_or(1024);
            
            if ui.add(egui::Slider::new(&mut shadow_resolution, 256..=4096)).changed() {
                changes.push(ParameterChange {
                    parameter: "shadow_resolution".to_string(),
                    value: NodeData::Integer(shadow_resolution),
                });
            }
        });
        
        changes
    }
    
    /// Convert current parameters to DirectionalLightLogic for processing
    pub fn to_directional_light_logic(&self) -> DirectionalLightLogic {
        let mut light = DirectionalLightLogic {
            direction: self.direction,
            color: self.color,
            intensity: self.intensity,
            cast_shadows: self.cast_shadows,
            shadow_resolution: self.shadow_resolution,
        };
        light.normalize_direction();
        light
    }
}