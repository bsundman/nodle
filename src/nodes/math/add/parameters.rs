//! Addition node parameters using Pattern A: build_interface method

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;

/// Addition node with Pattern A interface
#[derive(Debug, Clone)]
pub struct AddNode {
    pub input_a: f32,
    pub input_b: f32,
    pub precision: i32,
    pub clamp_result: bool,
    pub min_clamp: f32,
    pub max_clamp: f32,
}

impl Default for AddNode {
    fn default() -> Self {
        Self {
            input_a: 0.0,
            input_b: 0.0,
            precision: 2,
            clamp_result: false,
            min_clamp: -1000.0,
            max_clamp: 1000.0,
        }
    }
}

impl AddNode {
    /// Pattern A: build_interface method that renders UI and returns parameter changes
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("Addition Parameters");
        ui.separator();
        
        // Input A
        ui.horizontal(|ui| {
            ui.label("Input A:");
            let mut input_a = node.parameters.get("input_a")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(0.0);
            
            if ui.add(egui::Slider::new(&mut input_a, f32::NEG_INFINITY..=f32::INFINITY).step_by(0.1)).changed() {
                changes.push(ParameterChange {
                    parameter: "input_a".to_string(),
                    value: NodeData::Float(input_a),
                });
            }
        });
        
        // Input B
        ui.horizontal(|ui| {
            ui.label("Input B:");
            let mut input_b = node.parameters.get("input_b")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(0.0);
            
            if ui.add(egui::Slider::new(&mut input_b, f32::NEG_INFINITY..=f32::INFINITY).step_by(0.1)).changed() {
                changes.push(ParameterChange {
                    parameter: "input_b".to_string(),
                    value: NodeData::Float(input_b),
                });
            }
        });
        
        ui.separator();
        
        // Precision
        ui.horizontal(|ui| {
            ui.label("Precision:");
            let mut precision = node.parameters.get("precision")
                .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
                .unwrap_or(2);
            
            if ui.add(egui::Slider::new(&mut precision, 0..=10)).changed() {
                changes.push(ParameterChange {
                    parameter: "precision".to_string(),
                    value: NodeData::Integer(precision),
                });
            }
        });
        
        // Clamp Result
        ui.horizontal(|ui| {
            ui.label("Clamp Result:");
            let mut clamp_result = node.parameters.get("clamp_result")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false);
            
            if ui.checkbox(&mut clamp_result, "").changed() {
                changes.push(ParameterChange {
                    parameter: "clamp_result".to_string(),
                    value: NodeData::Boolean(clamp_result),
                });
            }
        });
        
        // Clamp range (only show if clamping is enabled)
        let clamp_enabled = node.parameters.get("clamp_result")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
            
        if clamp_enabled {
            ui.horizontal(|ui| {
                ui.label("Min Clamp:");
                let mut min_clamp = node.parameters.get("min_clamp")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(-1000.0);
                
                if ui.add(egui::Slider::new(&mut min_clamp, f32::NEG_INFINITY..=f32::INFINITY).step_by(1.0)).changed() {
                    changes.push(ParameterChange {
                        parameter: "min_clamp".to_string(),
                        value: NodeData::Float(min_clamp),
                    });
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Max Clamp:");
                let mut max_clamp = node.parameters.get("max_clamp")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(1000.0);
                
                if ui.add(egui::Slider::new(&mut max_clamp, f32::NEG_INFINITY..=f32::INFINITY).step_by(1.0)).changed() {
                    changes.push(ParameterChange {
                        parameter: "max_clamp".to_string(),
                        value: NodeData::Float(max_clamp),
                    });
                }
            });
        }
        
        changes
    }
    
    /// Apply parameters to a result value
    pub fn apply_to_result(&self, result: f32) -> f32 {
        let mut final_result = result;
        
        // Apply clamping if enabled
        if self.clamp_result {
            final_result = final_result.clamp(self.min_clamp, self.max_clamp);
        }
        
        // Apply precision rounding
        let multiplier = 10_f32.powi(self.precision);
        final_result = (final_result * multiplier).round() / multiplier;
        
        final_result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Pos2;

    #[test]
    fn test_default_add_node() {
        let node = AddNode::default();
        assert_eq!(node.input_a, 0.0);
        assert_eq!(node.input_b, 0.0);
        assert_eq!(node.precision, 2);
        assert!(!node.clamp_result);
    }

    #[test]
    fn test_apply_parameters() {
        let mut node = AddNode::default();
        node.precision = 1;
        node.clamp_result = true;
        node.min_clamp = -5.0;
        node.max_clamp = 5.0;
        
        assert_eq!(node.apply_to_result(3.14159), 3.1);
        assert_eq!(node.apply_to_result(10.0), 5.0); // Clamped
        assert_eq!(node.apply_to_result(-10.0), -5.0); // Clamped
    }
}