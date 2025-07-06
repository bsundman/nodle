//! Debug node parameters using Pattern A: build_interface method

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use super::logic::DebugLevel;

/// Debug node with Pattern A interface
#[derive(Debug, Clone)]
pub struct DebugNode {
    pub custom_label: String,
    pub debug_level: DebugLevel,
    pub include_stack_trace: bool,
    pub include_memory_info: bool,
    pub include_timing: bool,
    pub enable_timestamps: bool,
    pub output_format: DebugOutputFormat,
    pub verbosity_level: i32,
    pub enable_value_debug: bool,
    pub enable_type_debug: bool,
    pub enable_memory_debug: bool,
    pub enable_performance_debug: bool,
    pub max_history_size: usize,
}

#[derive(Debug, Clone)]
pub enum DebugOutputFormat {
    Simple,
    Detailed,
    Json,
    Structured,
}

impl Default for DebugNode {
    fn default() -> Self {
        Self {
            custom_label: String::new(),
            debug_level: DebugLevel::Debug,
            include_stack_trace: false,
            include_memory_info: false,
            include_timing: false,
            enable_timestamps: true,
            output_format: DebugOutputFormat::Simple,
            verbosity_level: 1,
            enable_value_debug: true,
            enable_type_debug: true,
            enable_memory_debug: false,
            enable_performance_debug: false,
            max_history_size: 100,
        }
    }
}

impl DebugLevel {
    /// Get the level name for display
    pub fn get_level_name(&self) -> &'static str {
        match self {
            DebugLevel::Trace => "Trace",
            DebugLevel::Debug => "Debug",
            DebugLevel::Info => "Info",
            DebugLevel::Warn => "Warn",
            DebugLevel::Error => "Error",
        }
    }
}

impl DebugNode {
    /// Pattern A: build_interface method that renders UI and returns parameter changes
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("Debug Parameters");
        ui.separator();
        
        // Custom Label
        ui.horizontal(|ui| {
            ui.label("Custom Label:");
            let mut custom_label = node.parameters.get("custom_label")
                .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or_default();
            
            if ui.text_edit_singleline(&mut custom_label).changed() {
                changes.push(ParameterChange {
                    parameter: "custom_label".to_string(),
                    value: NodeData::String(custom_label),
                });
            }
        });
        
        ui.separator();
        
        // Debug Level
        ui.horizontal(|ui| {
            ui.label("Debug Level:");
            let current_level = node.parameters.get("debug_level")
                .and_then(|v| if let NodeData::String(s) = v { Some(s.as_str()) } else { None })
                .unwrap_or("Debug");
            
            for level_name in ["Trace", "Debug", "Info", "Warn", "Error"] {
                if ui.selectable_label(current_level == level_name, level_name).clicked() {
                    changes.push(ParameterChange {
                        parameter: "debug_level".to_string(),
                        value: NodeData::String(level_name.to_string()),
                    });
                }
            }
        });
        
        ui.separator();
        
        // Output Format
        ui.horizontal(|ui| {
            ui.label("Output Format:");
            let current_format = node.parameters.get("output_format")
                .and_then(|v| if let NodeData::String(s) = v { Some(s.as_str()) } else { None })
                .unwrap_or("Simple");
            
            for format_name in ["Simple", "Detailed", "Json", "Structured"] {
                if ui.selectable_label(current_format == format_name, format_name).clicked() {
                    changes.push(ParameterChange {
                        parameter: "output_format".to_string(),
                        value: NodeData::String(format_name.to_string()),
                    });
                }
            }
        });
        
        ui.separator();
        
        // Verbosity Level
        ui.horizontal(|ui| {
            ui.label("Verbosity Level:");
            let mut verbosity = node.parameters.get("verbosity_level")
                .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
                .unwrap_or(1);
            
            if ui.add(egui::Slider::new(&mut verbosity, 0..=5)).changed() {
                changes.push(ParameterChange {
                    parameter: "verbosity_level".to_string(),
                    value: NodeData::Integer(verbosity),
                });
            }
        });
        
        ui.separator();
        
        // Boolean options
        ui.label("Options:");
        ui.indent("debug_options", |ui| {
            let boolean_options = [
                ("include_stack_trace", "Include Stack Trace"),
                ("include_memory_info", "Include Memory Info"),
                ("include_timing", "Include Timing"),
                ("enable_timestamps", "Enable Timestamps"),
            ];
            
            for (param_name, display_name) in boolean_options {
                let mut value = node.parameters.get(param_name)
                    .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                    .unwrap_or(false);
                
                if ui.checkbox(&mut value, display_name).changed() {
                    changes.push(ParameterChange {
                        parameter: param_name.to_string(),
                        value: NodeData::Boolean(value),
                    });
                }
            }
        });
        
        ui.separator();
        
        // Debug Categories
        ui.label("Debug Categories:");
        ui.indent("debug_categories", |ui| {
            let categories = [
                ("enable_value_debug", "Value Debug"),
                ("enable_type_debug", "Type Debug"),
                ("enable_memory_debug", "Memory Debug"),
                ("enable_performance_debug", "Performance Debug"),
            ];
            
            for (param_name, display_name) in categories {
                let mut enabled = node.parameters.get(param_name)
                    .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                    .unwrap_or(false);
                
                if ui.checkbox(&mut enabled, display_name).changed() {
                    changes.push(ParameterChange {
                        parameter: param_name.to_string(),
                        value: NodeData::Boolean(enabled),
                    });
                }
            }
        });
        
        ui.separator();
        
        // Max History Size
        ui.horizontal(|ui| {
            ui.label("Max History Size:");
            let mut max_history = node.parameters.get("max_history_size")
                .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
                .unwrap_or(100);
            
            if ui.add(egui::Slider::new(&mut max_history, 10..=1000)).changed() {
                changes.push(ParameterChange {
                    parameter: "max_history_size".to_string(),
                    value: NodeData::Integer(max_history),
                });
            }
        });
        
        ui.separator();
        
        // Action buttons
        ui.horizontal(|ui| {
            if ui.button("Test Debug").clicked() {
                let label = node.parameters.get("custom_label")
                    .and_then(|v| if let NodeData::String(s) = v { Some(s.as_str()) } else { None })
                    .unwrap_or("Debug Node");
                println!("[DEBUG] Test debug output from: {}", label);
            }
            if ui.button("Clear History").clicked() {
                // This would be handled by the node logic
                println!("Debug history cleared");
            }
        });
        
        changes
    }
}

#[cfg(test)]  
mod tests {
    use super::*;

    #[test]
    fn test_default_debug_node() {
        let node = DebugNode::default();
        assert_eq!(node.custom_label, "");
        assert!(matches!(node.debug_level, DebugLevel::Debug));
        assert!(!node.include_stack_trace);
        assert!(node.enable_timestamps);
        assert_eq!(node.verbosity_level, 1);
        assert_eq!(node.max_history_size, 100);
    }
}