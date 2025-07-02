//! Scale node parameters using Pattern A: build_interface method

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use super::logic::{ScaleLogic, ScaleMode};

/// Scale node with Pattern A interface
#[derive(Debug, Clone)]
pub struct ScaleNode {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub uniform_scale: bool,
    pub scale_mode: ScaleMode,
    pub use_world_space: bool,
    pub units: String,
    pub snap_to_grid: bool,
    pub grid_size: f32,
}

impl Default for ScaleNode {
    fn default() -> Self {
        Self {
            x: 1.0,
            y: 1.0,
            z: 1.0,
            uniform_scale: false,
            scale_mode: ScaleMode::Multiply,
            use_world_space: true,
            units: "Scale Factor".to_string(),
            snap_to_grid: false,
            grid_size: 0.1,
        }
    }
}

impl ScaleNode {
    /// Pattern A: build_interface method that renders UI and returns parameter changes
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("Scale Parameters");
        ui.separator();
        
        // Quick Scale Presets
        ui.label("Quick Scale Presets:");
        ui.horizontal(|ui| {
            if ui.button("Reset").clicked() {
                changes.push(ParameterChange {
                    parameter: "x".to_string(),
                    value: NodeData::Float(1.0),
                });
                changes.push(ParameterChange {
                    parameter: "y".to_string(),
                    value: NodeData::Float(1.0),
                });
                changes.push(ParameterChange {
                    parameter: "z".to_string(),
                    value: NodeData::Float(1.0),
                });
            }
            if ui.button("2x").clicked() {
                let uniform = node.parameters.get("uniform_scale")
                    .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                    .unwrap_or(false);
                
                if uniform {
                    for axis in ["x", "y", "z"] {
                        changes.push(ParameterChange {
                            parameter: axis.to_string(),
                            value: NodeData::Float(2.0),
                        });
                    }
                } else {
                    changes.push(ParameterChange {
                        parameter: "x".to_string(),
                        value: NodeData::Float(2.0),
                    });
                }
            }
            if ui.button("0.5x").clicked() {
                let uniform = node.parameters.get("uniform_scale")
                    .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                    .unwrap_or(false);
                
                if uniform {
                    for axis in ["x", "y", "z"] {
                        changes.push(ParameterChange {
                            parameter: axis.to_string(),
                            value: NodeData::Float(0.5),
                        });
                    }
                } else {
                    changes.push(ParameterChange {
                        parameter: "x".to_string(),
                        value: NodeData::Float(0.5),
                    });
                }
            }
            if ui.button("10x").clicked() {
                let uniform = node.parameters.get("uniform_scale")
                    .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                    .unwrap_or(false);
                
                if uniform {
                    for axis in ["x", "y", "z"] {
                        changes.push(ParameterChange {
                            parameter: axis.to_string(),
                            value: NodeData::Float(10.0),
                        });
                    }
                } else {
                    changes.push(ParameterChange {
                        parameter: "x".to_string(),
                        value: NodeData::Float(10.0),
                    });
                }
            }
        });
        
        ui.separator();
        
        // Uniform Scale Toggle
        ui.horizontal(|ui| {
            ui.label("Uniform Scale:");
            let mut uniform_scale = node.parameters.get("uniform_scale")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false);
            
            if ui.checkbox(&mut uniform_scale, "Lock proportions").changed() {
                changes.push(ParameterChange {
                    parameter: "uniform_scale".to_string(),
                    value: NodeData::Boolean(uniform_scale),
                });
                
                // If uniform scaling is enabled, set Y and Z to match X
                if uniform_scale {
                    let x_value = node.parameters.get("x")
                        .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                        .unwrap_or(1.0);
                    
                    changes.push(ParameterChange {
                        parameter: "y".to_string(),
                        value: NodeData::Float(x_value),
                    });
                    changes.push(ParameterChange {
                        parameter: "z".to_string(),
                        value: NodeData::Float(x_value),
                    });
                }
            }
        });
        
        ui.separator();
        
        // Scale Vector Components
        ui.label("Scale Factors:");
        
        let uniform_scale = node.parameters.get("uniform_scale")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
        
        // X Component (or uniform if enabled)
        ui.horizontal(|ui| {
            let label = if uniform_scale { "Uniform:" } else { "X:" };
            ui.label(label);
            let mut x = node.parameters.get("x")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(1.0);
            
            if ui.add(egui::Slider::new(&mut x, 0.001..=100.0).step_by(0.01).logarithmic(true)).changed() {
                changes.push(ParameterChange {
                    parameter: "x".to_string(),
                    value: NodeData::Float(x),
                });
                
                // If uniform scaling, also update Y and Z
                if uniform_scale {
                    changes.push(ParameterChange {
                        parameter: "y".to_string(),
                        value: NodeData::Float(x),
                    });
                    changes.push(ParameterChange {
                        parameter: "z".to_string(),
                        value: NodeData::Float(x),
                    });
                }
            }
        });
        
        // Y Component (only if not uniform)
        if !uniform_scale {
            ui.horizontal(|ui| {
                ui.label("Y:");
                let mut y = node.parameters.get("y")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(1.0);
                
                if ui.add(egui::Slider::new(&mut y, 0.001..=100.0).step_by(0.01).logarithmic(true)).changed() {
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
                    .unwrap_or(1.0);
                
                if ui.add(egui::Slider::new(&mut z, 0.001..=100.0).step_by(0.01).logarithmic(true)).changed() {
                    changes.push(ParameterChange {
                        parameter: "z".to_string(),
                        value: NodeData::Float(z),
                    });
                }
            });
        }
        
        ui.separator();
        
        // Scale Mode
        let current_mode = node.parameters.get("scale_mode")
            .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
            .unwrap_or(0);
        
        ui.horizontal(|ui| {
            ui.label("Scale Mode:");
            
            let mode_names = ["Multiply", "Absolute"];
            let mut selected = current_mode as usize;
            
            egui::ComboBox::from_id_source("scale_mode")
                .selected_text(*mode_names.get(selected).unwrap_or(&"Multiply"))
                .show_ui(ui, |ui| {
                    for (i, name) in mode_names.iter().enumerate() {
                        if ui.selectable_value(&mut selected, i, *name).changed() {
                            changes.push(ParameterChange {
                                parameter: "scale_mode".to_string(),
                                value: NodeData::Integer(i as i32),
                            });
                        }
                    }
                });
        });
        
        // Units
        let current_units = node.parameters.get("units")
            .and_then(|v| if let NodeData::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("Scale Factor");
        
        ui.horizontal(|ui| {
            ui.label("Units:");
            
            let unit_names = ["Scale Factor", "Percentage", "Ratio"];
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
                    .unwrap_or(0.1);
                
                if ui.add(egui::Slider::new(&mut grid_size, 0.01..=1.0).step_by(0.01)).changed() {
                    changes.push(ParameterChange {
                        parameter: "grid_size".to_string(),
                        value: NodeData::Float(grid_size),
                    });
                }
            });
        }
        
        ui.separator();
        
        // Display current scale
        let x = node.parameters.get("x")
            .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
            .unwrap_or(1.0);
        let y = node.parameters.get("y")
            .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
            .unwrap_or(1.0);
        let z = node.parameters.get("z")
            .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
            .unwrap_or(1.0);
        let units = node.parameters.get("units")
            .and_then(|v| if let NodeData::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("Scale Factor");
        
        if uniform_scale {
            ui.label(format!("Uniform Scale: {:.3} {}", x, units));
        } else {
            ui.label(format!("Scale: [{:.3}, {:.3}, {:.3}] {}", x, y, z, units));
        }
        
        changes
    }
    
    /// Convert current parameters to ScaleLogic for processing
    pub fn to_scale_logic(&self) -> ScaleLogic {
        ScaleLogic {
            scale: [self.x, self.y, self.z],
            uniform_scale: self.uniform_scale,
            use_world_space: self.use_world_space,
            scale_mode: self.scale_mode.clone(),
        }
    }
}