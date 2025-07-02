//! Division node parameters using Pattern A: build_interface method

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;

/// Division node with Pattern A interface
#[derive(Debug, Clone)]
pub struct DivideNode {
    pub dividend: f32,
    pub divisor: f32,
    pub precision: i32,
    pub handle_division_by_zero: bool,
    pub zero_result: f32,
    pub clamp_result: bool,
    pub min_clamp: f32,
    pub max_clamp: f32,
}

impl Default for DivideNode {
    fn default() -> Self {
        Self {
            dividend: 0.0,
            divisor: 1.0, // Default to 1 to avoid division by zero
            precision: 2,
            handle_division_by_zero: true,
            zero_result: 0.0,
            clamp_result: false,
            min_clamp: -1000.0,
            max_clamp: 1000.0,
        }
    }
}

impl DivideNode {
    /// Pattern A: build_interface method that renders UI and returns parameter changes
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("Division Parameters");
        ui.separator();
        
        // Dividend
        ui.horizontal(|ui| {
            ui.label("Dividend:");
            let mut dividend = node.parameters.get("dividend")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(0.0);
            
            if ui.add(egui::Slider::new(&mut dividend, f32::NEG_INFINITY..=f32::INFINITY).step_by(0.1)).changed() {
                changes.push(ParameterChange {
                    parameter: "dividend".to_string(),
                    value: NodeData::Float(dividend),
                });
            }
        });
        
        // Divisor
        ui.horizontal(|ui| {
            ui.label("Divisor:");
            let mut divisor = node.parameters.get("divisor")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(1.0);
            
            if ui.add(egui::Slider::new(&mut divisor, f32::NEG_INFINITY..=f32::INFINITY).step_by(0.1)).changed() {
                changes.push(ParameterChange {
                    parameter: "divisor".to_string(),
                    value: NodeData::Float(divisor),
                });
            }
        });
        
        ui.separator();
        
        // Quick Operations
        ui.label("Quick Operations:");
        ui.horizontal(|ui| {
            if ui.button("÷ 2").clicked() {
                changes.push(ParameterChange {
                    parameter: "divisor".to_string(),
                    value: NodeData::Float(2.0),
                });
            }
            if ui.button("÷ 10").clicked() {
                changes.push(ParameterChange {
                    parameter: "divisor".to_string(),
                    value: NodeData::Float(10.0),
                });
            }
            if ui.button("÷ 100").clicked() {
                changes.push(ParameterChange {
                    parameter: "divisor".to_string(),
                    value: NodeData::Float(100.0),
                });
            }
        });
        
        ui.separator();
        
        // Handle Division by Zero
        ui.horizontal(|ui| {
            ui.label("Handle Division by Zero:");
            let mut handle_division_by_zero = node.parameters.get("handle_division_by_zero")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(true);
            
            if ui.checkbox(&mut handle_division_by_zero, "").changed() {
                changes.push(ParameterChange {
                    parameter: "handle_division_by_zero".to_string(),
                    value: NodeData::Boolean(handle_division_by_zero),
                });
            }
        });
        
        // Zero Result (only show if handling division by zero)
        let handle_zero = node.parameters.get("handle_division_by_zero")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(true);
            
        if handle_zero {
            ui.horizontal(|ui| {
                ui.label("Zero Result:");
                let mut zero_result = node.parameters.get("zero_result")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(0.0);
                
                if ui.add(egui::Slider::new(&mut zero_result, f32::NEG_INFINITY..=f32::INFINITY).step_by(0.1)).changed() {
                    changes.push(ParameterChange {
                        parameter: "zero_result".to_string(),
                        value: NodeData::Float(zero_result),
                    });
                }
            });
        }
        
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
        
        // Display current result with warning for division by zero
        let current_divisor = node.parameters.get("divisor")
            .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
            .unwrap_or(1.0);
        let current_dividend = node.parameters.get("dividend")
            .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
            .unwrap_or(0.0);
            
        ui.separator();
        if !handle_zero && current_divisor == 0.0 {
            ui.colored_label(egui::Color32::RED, "⚠ Division by zero!");
        } else {
            let result = if handle_zero && current_divisor == 0.0 {
                node.parameters.get("zero_result")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(0.0)
            } else {
                current_dividend / current_divisor
            };
            ui.label(format!("Result: {:.6}", result));
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
    
    /// Perform division with current parameters
    pub fn divide(&self) -> f32 {
        let result = if self.handle_division_by_zero && self.divisor == 0.0 {
            self.zero_result
        } else {
            self.dividend / self.divisor
        };
        
        self.apply_to_result(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Pos2;

    #[test]
    fn test_default_divide_node() {
        let node = DivideNode::default();
        assert_eq!(node.dividend, 0.0);
        assert_eq!(node.divisor, 1.0);
        assert_eq!(node.precision, 2);
        assert!(node.handle_division_by_zero);
        assert!(!node.clamp_result);
    }

    #[test]
    fn test_apply_parameters() {
        let mut node = DivideNode::default();
        node.precision = 1;
        node.clamp_result = true;
        node.min_clamp = -5.0;
        node.max_clamp = 5.0;
        
        assert_eq!(node.apply_to_result(3.14159), 3.1);
        assert_eq!(node.apply_to_result(10.0), 5.0); // Clamped
        assert_eq!(node.apply_to_result(-10.0), -5.0); // Clamped
    }

    #[test]
    fn test_division_by_zero_handling() {
        let mut node = DivideNode::default();
        node.dividend = 10.0;
        node.divisor = 0.0;
        node.handle_division_by_zero = true;
        node.zero_result = 42.0;
        
        assert_eq!(node.divide(), 42.0);
        
        node.handle_division_by_zero = false;
        assert!(node.divide().is_infinite() || node.divide().is_nan());
    }
}