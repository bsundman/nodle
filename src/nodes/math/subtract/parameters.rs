//! Subtraction node parameters using Pattern A: build_interface method

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;

/// Subtraction node with Pattern A interface
#[derive(Debug, Clone)]
pub struct SubtractNode {
    pub minuend: f32,
    pub subtrahend: f32,
    pub precision: i32,
    pub allow_negative: bool,
    pub clamp_result: bool,
    pub min_clamp: f32,
    pub max_clamp: f32,
}

impl Default for SubtractNode {
    fn default() -> Self {
        Self {
            minuend: 0.0,
            subtrahend: 0.0,
            precision: 2,
            allow_negative: true,
            clamp_result: false,
            min_clamp: 0.0,
            max_clamp: 1000.0,
        }
    }
}

impl SubtractNode {
    /// Pattern A: build_interface method that renders UI and returns parameter changes
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("Subtraction Parameters");
        ui.separator();
        
        // Minuend (A)
        ui.horizontal(|ui| {
            ui.label("Minuend (A):");
            let mut minuend = node.parameters.get("minuend")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(0.0);
            
            if ui.add(egui::Slider::new(&mut minuend, f32::NEG_INFINITY..=f32::INFINITY).step_by(0.1)).changed() {
                changes.push(ParameterChange {
                    parameter: "minuend".to_string(),
                    value: NodeData::Float(minuend),
                });
            }
        });
        
        // Subtrahend (B)
        ui.horizontal(|ui| {
            ui.label("Subtrahend (B):");
            let mut subtrahend = node.parameters.get("subtrahend")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(0.0);
            
            if ui.add(egui::Slider::new(&mut subtrahend, f32::NEG_INFINITY..=f32::INFINITY).step_by(0.1)).changed() {
                changes.push(ParameterChange {
                    parameter: "subtrahend".to_string(),
                    value: NodeData::Float(subtrahend),
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
        
        // Allow Negative
        ui.horizontal(|ui| {
            ui.label("Allow Negative:");
            let mut allow_negative = node.parameters.get("allow_negative")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(true);
            
            if ui.checkbox(&mut allow_negative, "").changed() {
                changes.push(ParameterChange {
                    parameter: "allow_negative".to_string(),
                    value: NodeData::Boolean(allow_negative),
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
                    .unwrap_or(0.0);
                
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
        
        // Apply negative restriction if needed
        if !self.allow_negative && final_result < 0.0 {
            final_result = 0.0;
        }
        
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

    #[test]
    fn test_default_subtract_node() {
        let node = SubtractNode::default();
        assert_eq!(node.minuend, 0.0);
        assert_eq!(node.subtrahend, 0.0);
        assert_eq!(node.precision, 2);
        assert!(node.allow_negative);
        assert!(!node.clamp_result);
    }

    #[test]
    fn test_apply_parameters() {
        let mut node = SubtractNode::default();
        node.precision = 1;
        node.allow_negative = false;
        
        assert_eq!(node.apply_to_result(3.14159), 3.1);
        assert_eq!(node.apply_to_result(-2.5), 0.0); // Negative blocked
        
        node.allow_negative = true;
        node.clamp_result = true;
        node.min_clamp = -5.0;
        node.max_clamp = 5.0;
        
        assert_eq!(node.apply_to_result(10.0), 5.0); // Clamped
        assert_eq!(node.apply_to_result(-10.0), -5.0); // Clamped
    }
}