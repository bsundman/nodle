//! Translate node parameters using Pattern A: build_interface method

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use super::logic::{TranslateLogic, TranslationMode};

/// Translate node with Pattern A interface
#[derive(Debug, Clone)]
pub struct TranslateNode {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub translation_mode: TranslationMode,
    pub use_world_space: bool,
    pub units: String,
    pub snap_to_grid: bool,
    pub grid_size: f32,
}

#[derive(Debug, Clone)]
pub enum TransformUnits {
    Meters,
    Centimeters,
    Millimeters,
    Inches,
    Feet,
}

impl Default for TranslateNode {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            translation_mode: TranslationMode::Absolute,
            use_world_space: true,
            units: "Meters".to_string(),
            snap_to_grid: false,
            grid_size: 1.0,
        }
    }
}

impl TranslateNode {
    /// Pattern A: build_interface method that renders UI and returns parameter changes
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("Translate Parameters");
        ui.separator();
        
        // Quick Translation Presets
        ui.label("Quick Translation Presets:");
        ui.horizontal(|ui| {
            if ui.button("Reset").clicked() {
                changes.push(ParameterChange {
                    parameter: "x".to_string(),
                    value: NodeData::Float(0.0),
                });
                changes.push(ParameterChange {
                    parameter: "y".to_string(),
                    value: NodeData::Float(0.0),
                });
                changes.push(ParameterChange {
                    parameter: "z".to_string(),
                    value: NodeData::Float(0.0),
                });
            }
            if ui.button("+X Unit").clicked() {
                let current_x = node.parameters.get("x")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(0.0);
                changes.push(ParameterChange {
                    parameter: "x".to_string(),
                    value: NodeData::Float(current_x + 1.0),
                });
            }
            if ui.button("+Y Unit").clicked() {
                let current_y = node.parameters.get("y")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(0.0);
                changes.push(ParameterChange {
                    parameter: "y".to_string(),
                    value: NodeData::Float(current_y + 1.0),
                });
            }
            if ui.button("+Z Unit").clicked() {
                let current_z = node.parameters.get("z")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(0.0);
                changes.push(ParameterChange {
                    parameter: "z".to_string(),
                    value: NodeData::Float(current_z + 1.0),
                });
            }
        });
        
        ui.separator();
        
        // Vector Input Components
        ui.label("Translation Vector:");
        
        // X Component
        ui.horizontal(|ui| {
            ui.label("X:");
            let mut x = node.parameters.get("x")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(0.0);
            
            if ui.add(egui::Slider::new(&mut x, -1000.0..=1000.0).step_by(0.1)).changed() {
                changes.push(ParameterChange {
                    parameter: "x".to_string(),
                    value: NodeData::Float(x),
                });
            }
        });
        
        // Y Component
        ui.horizontal(|ui| {
            ui.label("Y:");
            let mut y = node.parameters.get("y")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(0.0);
            
            if ui.add(egui::Slider::new(&mut y, -1000.0..=1000.0).step_by(0.1)).changed() {
                changes.push(ParameterChange {
                    parameter: "y".to_string(),
                    value: NodeData::Float(y),
                });
            }
        });
        
        // Z Component
        ui.horizontal(|ui| {
            ui.label("Z:");
            let mut z = node.parameters.get("z")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(0.0);
            
            if ui.add(egui::Slider::new(&mut z, -1000.0..=1000.0).step_by(0.1)).changed() {
                changes.push(ParameterChange {
                    parameter: "z".to_string(),
                    value: NodeData::Float(z),
                });
            }
        });
        
        ui.separator();
        
        // Transform Mode
        let current_mode = node.parameters.get("translation_mode")
            .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
            .unwrap_or(0);
        
        ui.horizontal(|ui| {
            ui.label("Transform Mode:");
            
            let mode_names = ["Absolute", "Relative"];
            let mut selected = current_mode as usize;
            
            egui::ComboBox::from_id_source("translation_mode")
                .selected_text(*mode_names.get(selected).unwrap_or(&"Absolute"))
                .show_ui(ui, |ui| {
                    for (i, name) in mode_names.iter().enumerate() {
                        if ui.selectable_value(&mut selected, i, *name).changed() {
                            changes.push(ParameterChange {
                                parameter: "translation_mode".to_string(),
                                value: NodeData::Integer(i as i32),
                            });
                        }
                    }
                });
        });
        
        // Units
        let current_units = node.parameters.get("units")
            .and_then(|v| if let NodeData::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("Meters");
        
        ui.horizontal(|ui| {
            ui.label("Units:");
            
            let unit_names = ["Meters", "Centimeters", "Millimeters", "Inches", "Feet"];
            let mut selected_unit = current_units.to_string();
            
            egui::ComboBox::from_id_source("units")
                .selected_text(&selected_unit)
                .show_ui(ui, |ui| {
                    for unit in &unit_names {
                        if ui.selectable_value(&mut selected_unit, unit.to_string(), *unit).changed() {
                            changes.push(ParameterChange {
                                parameter: "units".to_string(),
                                value: NodeData::String(selected_unit.clone()),
                            });
                        }
                    }
                });
        });
        
        ui.separator();
        
        // World Space
        ui.horizontal(|ui| {
            ui.label("Use World Space:");
            let mut use_world_space = node.parameters.get("use_world_space")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(true);
            
            if ui.checkbox(&mut use_world_space, "").changed() {
                changes.push(ParameterChange {
                    parameter: "use_world_space".to_string(),
                    value: NodeData::Boolean(use_world_space),
                });
            }
        });
        
        // Snap to Grid
        ui.horizontal(|ui| {
            ui.label("Snap to Grid:");
            let mut snap_to_grid = node.parameters.get("snap_to_grid")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false);
            
            if ui.checkbox(&mut snap_to_grid, "").changed() {
                changes.push(ParameterChange {
                    parameter: "snap_to_grid".to_string(),
                    value: NodeData::Boolean(snap_to_grid),
                });
            }
        });
        
        // Grid Size (only if snap to grid is enabled)
        if node.parameters.get("snap_to_grid")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false) {
            ui.horizontal(|ui| {
                ui.label("Grid Size:");
                let mut grid_size = node.parameters.get("grid_size")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(1.0);
                
                if ui.add(egui::Slider::new(&mut grid_size, 0.1..=10.0).step_by(0.1)).changed() {
                    changes.push(ParameterChange {
                        parameter: "grid_size".to_string(),
                        value: NodeData::Float(grid_size),
                    });
                }
            });
        }
        
        ui.separator();
        
        // Display current translation
        let x = node.parameters.get("x")
            .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
            .unwrap_or(0.0);
        let y = node.parameters.get("y")
            .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
            .unwrap_or(0.0);
        let z = node.parameters.get("z")
            .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
            .unwrap_or(0.0);
        let units = node.parameters.get("units")
            .and_then(|v| if let NodeData::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("Meters");
        
        ui.label(format!("Translation: [{:.2}, {:.2}, {:.2}] {}", x, y, z, units));
        
        changes
    }
    
    /// Convert current parameters to TranslateLogic for processing
    pub fn to_translate_logic(&self) -> TranslateLogic {
        TranslateLogic {
            translation: [self.x, self.y, self.z],
            use_world_space: self.use_world_space,
            translation_mode: self.translation_mode.clone(),
        }
    }
}