//! Multiply node parameters using Pattern A: build_interface method

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;

/// Multiply node with Pattern A interface
#[derive(Debug, Clone)]
pub struct MultiplyNode {
    pub a: f32,
    pub b: f32,
    pub clamp_result: bool,
    pub min_result: f32,
    pub max_result: f32,
}

impl Default for MultiplyNode {
    fn default() -> Self {
        Self {
            a: 1.0,
            b: 1.0,
            clamp_result: false,
            min_result: -1000.0,
            max_result: 1000.0,
        }
    }
}

impl MultiplyNode {
    /// Pattern A: build_interface method that renders UI and returns parameter changes
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("Multiply Parameters");
        ui.separator();
        
        // Input A
        ui.horizontal(|ui| {
            ui.label("Input A:");
            let mut a = node.parameters.get("a")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(1.0);
            
            if ui.add(egui::Slider::new(&mut a, -1000.0..=1000.0).step_by(0.1)).changed() {
                changes.push(ParameterChange {
                    parameter: "a".to_string(),
                    value: NodeData::Float(a),
                });
            }
        });
        
        // Input B
        ui.horizontal(|ui| {
            ui.label("Input B:");
            let mut b = node.parameters.get("b")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(1.0);
            
            if ui.add(egui::Slider::new(&mut b, -1000.0..=1000.0).step_by(0.1)).changed() {
                changes.push(ParameterChange {
                    parameter: "b".to_string(),
                    value: NodeData::Float(b),
                });
            }
        });
        
        ui.separator();
        
        // Quick Multipliers
        ui.label("Quick Multipliers:");
        ui.horizontal(|ui| {
            if ui.button("× 2").clicked() {
                changes.push(ParameterChange {
                    parameter: "b".to_string(),
                    value: NodeData::Float(2.0),
                });
            }
            if ui.button("× 10").clicked() {
                changes.push(ParameterChange {
                    parameter: "b".to_string(),
                    value: NodeData::Float(10.0),
                });
            }
            if ui.button("× 0.5").clicked() {
                changes.push(ParameterChange {
                    parameter: "b".to_string(),
                    value: NodeData::Float(0.5),
                });
            }
            if ui.button("× -1").clicked() {
                changes.push(ParameterChange {
                    parameter: "b".to_string(),
                    value: NodeData::Float(-1.0),
                });
            }
        });
        
        ui.separator();
        
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
                ui.label("Min Result:");
                let mut min_result = node.parameters.get("min_result")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(-1000.0);
                
                if ui.add(egui::Slider::new(&mut min_result, -10000.0..=10000.0).step_by(1.0)).changed() {
                    changes.push(ParameterChange {
                        parameter: "min_result".to_string(),
                        value: NodeData::Float(min_result),
                    });
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Max Result:");
                let mut max_result = node.parameters.get("max_result")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(1000.0);
                
                if ui.add(egui::Slider::new(&mut max_result, -10000.0..=10000.0).step_by(1.0)).changed() {
                    changes.push(ParameterChange {
                        parameter: "max_result".to_string(),
                        value: NodeData::Float(max_result),
                    });
                }
            });
        }
        
        // Display current result
        let a = node.parameters.get("a")
            .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
            .unwrap_or(1.0);
        let b = node.parameters.get("b")
            .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
            .unwrap_or(1.0);
        let result = a * b;
        ui.label(format!("Result: {:.6}", result));
        
        changes
    }
    
    /// Perform multiplication with current values
    pub fn multiply(&self) -> f32 {
        let result = self.a * self.b;
        if self.clamp_result {
            result.clamp(self.min_result, self.max_result)
        } else {
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_multiply_node() {
        let node = MultiplyNode::default();
        assert_eq!(node.a, 1.0);
        assert_eq!(node.b, 1.0);
        assert!(!node.clamp_result);
        assert_eq!(node.min_result, -1000.0);
        assert_eq!(node.max_result, 1000.0);
    }

    #[test]
    fn test_multiply() {
        let node = MultiplyNode::default();
        assert_eq!(node.multiply(), 1.0);
        
        let mut node = MultiplyNode {
            a: 3.0,
            b: 4.0,
            clamp_result: false,
            ..Default::default()
        };
        assert_eq!(node.multiply(), 12.0);
        
        node.clamp_result = true;
        node.min_result = -5.0;
        node.max_result = 5.0;
        assert_eq!(node.multiply(), 5.0); // Clamped
    }
}