//! Rotate node parameters using Pattern A: build_interface method

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use super::logic::{RotateLogic, RotationOrder};

/// Rotate node with Pattern A interface
#[derive(Debug, Clone)]
pub struct RotateNode {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rotation_order: RotationOrder,
    pub use_world_space: bool,
    pub use_degrees: bool,
    pub units: String,
    pub snap_to_grid: bool,
    pub grid_size: f32,
}

impl Default for RotateNode {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            rotation_order: RotationOrder::XYZ,
            use_world_space: true,
            use_degrees: true,
            units: "Degrees".to_string(),
            snap_to_grid: false,
            grid_size: 15.0, // 15 degree increments
        }
    }
}

impl RotateNode {
    /// Pattern A: build_interface method that renders UI and returns parameter changes
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("Rotate Parameters");
        ui.separator();
        
        // Quick Rotation Presets
        ui.label("Quick Rotation Presets:");
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
            if ui.button("90° X").clicked() {
                let current_x = node.parameters.get("x")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(0.0);
                changes.push(ParameterChange {
                    parameter: "x".to_string(),
                    value: NodeData::Float(current_x + 90.0),
                });
            }
            if ui.button("90° Y").clicked() {
                let current_y = node.parameters.get("y")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(0.0);
                changes.push(ParameterChange {
                    parameter: "y".to_string(),
                    value: NodeData::Float(current_y + 90.0),
                });
            }
            if ui.button("90° Z").clicked() {
                let current_z = node.parameters.get("z")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(0.0);
                changes.push(ParameterChange {
                    parameter: "z".to_string(),
                    value: NodeData::Float(current_z + 90.0),
                });
            }
        });
        
        ui.separator();
        
        // Rotation Vector Components
        ui.label("Rotation Angles:");
        
        let use_degrees = node.parameters.get("use_degrees")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(true);
        
        let (min_val, max_val, step, unit_label) = if use_degrees {
            (-360.0, 360.0, 1.0, "°")
        } else {
            (-6.28318, 6.28318, 0.01745, " rad") // -2π to 2π, step ~1°
        };
        
        // X Component
        ui.horizontal(|ui| {
            ui.label("X:");
            let mut x = node.parameters.get("x")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(0.0);
            
            if ui.add(egui::Slider::new(&mut x, min_val..=max_val).step_by(step).suffix(unit_label)).changed() {
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
            
            if ui.add(egui::Slider::new(&mut y, min_val..=max_val).step_by(step).suffix(unit_label)).changed() {
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
            
            if ui.add(egui::Slider::new(&mut z, min_val..=max_val).step_by(step).suffix(unit_label)).changed() {
                changes.push(ParameterChange {
                    parameter: "z".to_string(),
                    value: NodeData::Float(z),
                });
            }
        });
        
        ui.separator();
        
        // Rotation Order
        let current_order = node.parameters.get("rotation_order")
            .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
            .unwrap_or(0);
        
        ui.horizontal(|ui| {
            ui.label("Rotation Order:");
            
            let order_names = ["XYZ", "XZY", "YXZ", "YZX", "ZXY", "ZYX"];
            let mut selected = current_order as usize;
            
            egui::ComboBox::from_id_source("rotation_order")
                .selected_text(*order_names.get(selected).unwrap_or(&"XYZ"))
                .show_ui(ui, |ui| {
                    for (i, name) in order_names.iter().enumerate() {
                        if ui.selectable_value(&mut selected, i, *name).changed() {
                            changes.push(ParameterChange {
                                parameter: "rotation_order".to_string(),
                                value: NodeData::Integer(i as i32),
                            });
                        }
                    }
                });
        });
        
        // Units (Degrees/Radians)
        ui.horizontal(|ui| {
            ui.label("Use Degrees:");
            let mut use_degrees = node.parameters.get("use_degrees")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(true);
            
            if ui.checkbox(&mut use_degrees, "").changed() {
                changes.push(ParameterChange {
                    parameter: "use_degrees".to_string(),
                    value: NodeData::Boolean(use_degrees),
                });
                
                // Update units string
                let new_units = if use_degrees { "Degrees" } else { "Radians" };
                changes.push(ParameterChange {
                    parameter: "units".to_string(),
                    value: NodeData::String(new_units.to_string()),
                });
            }
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
                    .unwrap_or(15.0);
                
                let (grid_step, grid_suffix) = if use_degrees {
                    (1.0f64, "°")
                } else {
                    (0.01745f64, " rad")
                };
                
                if ui.add(egui::Slider::new(&mut grid_size, grid_step as f32..=90.0).step_by(grid_step).suffix(grid_suffix)).changed() {
                    changes.push(ParameterChange {
                        parameter: "grid_size".to_string(),
                        value: NodeData::Float(grid_size),
                    });
                }
            });
        }
        
        ui.separator();
        
        // Display current rotation
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
            .unwrap_or("Degrees");
        let order = node.parameters.get("rotation_order")
            .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
            .unwrap_or(0);
        let order_names = ["XYZ", "XZY", "YXZ", "YZX", "ZXY", "ZYX"];
        let order_name = order_names.get(order as usize).unwrap_or(&"XYZ");
        
        ui.label(format!("Rotation: [{:.1}, {:.1}, {:.1}] {} ({})", 
            x, y, z, units, order_name));
        
        changes
    }
    
    /// Convert current parameters to RotateLogic for processing
    pub fn to_rotate_logic(&self) -> RotateLogic {
        RotateLogic {
            rotation: [self.x, self.y, self.z],
            use_world_space: self.use_world_space,
            rotation_order: self.rotation_order.clone(),
            use_degrees: self.use_degrees,
        }
    }
}