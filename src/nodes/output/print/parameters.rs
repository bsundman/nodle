//! Print node parameters using Pattern A: build_interface method

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use super::logic::{PrintFormat, PrintDestination};

/// Print node with Pattern A interface
#[derive(Debug, Clone)]
pub struct PrintNode {
    pub label: String,
    pub output_destination: PrintDestination,
    pub output_format: PrintFormat,
    pub line_endings: LineEnding,
    pub include_timestamp: bool,
    pub include_type_info: bool,
    pub include_line_numbers: bool,
    pub buffer_output: bool,
    pub max_line_length: usize,
    pub date_format: String,
    pub file_path: String,
    pub append_mode: bool,
}

#[derive(Debug, Clone)]
pub enum LineEnding {
    Unix,     // \n
    Windows,  // \r\n
    Mac,      // \r
    Auto,     // System default
}

impl Default for PrintNode {
    fn default() -> Self {
        Self {
            label: String::new(),
            output_destination: PrintDestination::Console,
            output_format: PrintFormat::Simple,
            line_endings: LineEnding::Auto,
            include_timestamp: false,
            include_type_info: false,
            include_line_numbers: false,
            buffer_output: false,
            max_line_length: 1000,
            date_format: "%Y-%m-%d %H:%M:%S".to_string(),
            file_path: "output.txt".to_string(),
            append_mode: true,
        }
    }
}

impl PrintNode {
    /// Pattern A: build_interface method that renders UI and returns parameter changes
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("Print Parameters");
        ui.separator();
        
        // Label
        ui.horizontal(|ui| {
            ui.label("Label:");
            let mut label = node.parameters.get("label")
                .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or_default();
            
            if ui.text_edit_singleline(&mut label).changed() {
                changes.push(ParameterChange {
                    parameter: "label".to_string(),
                    value: NodeData::String(label),
                });
            }
        });
        
        ui.separator();
        
        // Output Destination
        ui.horizontal(|ui| {
            ui.label("Output Destination:");
            let current_dest = node.parameters.get("output_destination")
                .and_then(|v| if let NodeData::String(s) = v { Some(s.as_str()) } else { None })
                .unwrap_or("Console");
            
            for dest_name in ["Console", "Log", "File", "Buffer"] {
                if ui.selectable_label(current_dest == dest_name, dest_name).clicked() {
                    changes.push(ParameterChange {
                        parameter: "output_destination".to_string(),
                        value: NodeData::String(dest_name.to_string()),
                    });
                }
            }
        });
        
        // File path (only show if File destination is selected)
        let is_file_dest = node.parameters.get("output_destination")
            .and_then(|v| if let NodeData::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("Console") == "File";
            
        if is_file_dest {
            ui.horizontal(|ui| {
                ui.label("File Path:");
                let mut file_path = node.parameters.get("file_path")
                    .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_else(|| "output.txt".to_string());
                
                if ui.text_edit_singleline(&mut file_path).changed() {
                    changes.push(ParameterChange {
                        parameter: "file_path".to_string(),
                        value: NodeData::String(file_path),
                    });
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Append Mode:");
                let mut append_mode = node.parameters.get("append_mode")
                    .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                    .unwrap_or(true);
                
                if ui.checkbox(&mut append_mode, "").changed() {
                    changes.push(ParameterChange {
                        parameter: "append_mode".to_string(),
                        value: NodeData::Boolean(append_mode),
                    });
                }
            });
        }
        
        ui.separator();
        
        // Output Format
        ui.horizontal(|ui| {
            ui.label("Output Format:");
            let current_format = node.parameters.get("output_format")
                .and_then(|v| if let NodeData::String(s) = v { Some(s.as_str()) } else { None })
                .unwrap_or("Simple");
            
            for format_name in ["Simple", "Formatted", "JSON", "Debug"] {
                if ui.selectable_label(current_format == format_name, format_name).clicked() {
                    changes.push(ParameterChange {
                        parameter: "output_format".to_string(),
                        value: NodeData::String(format_name.to_string()),
                    });
                }
            }
        });
        
        ui.separator();
        
        // Line Endings
        ui.horizontal(|ui| {
            ui.label("Line Endings:");
            let current_ending = node.parameters.get("line_endings")
                .and_then(|v| if let NodeData::String(s) = v { Some(s.as_str()) } else { None })
                .unwrap_or("Auto");
            
            for ending_name in ["Auto", "Unix", "Windows", "Mac"] {
                if ui.selectable_label(current_ending == ending_name, ending_name).clicked() {
                    changes.push(ParameterChange {
                        parameter: "line_endings".to_string(),
                        value: NodeData::String(ending_name.to_string()),
                    });
                }
            }
        });
        
        ui.separator();
        
        // Boolean options
        ui.label("Options:");
        ui.indent("print_options", |ui| {
            let boolean_options = [
                ("include_timestamp", "Include Timestamp"),
                ("include_type_info", "Include Type Info"),
                ("include_line_numbers", "Include Line Numbers"),
                ("buffer_output", "Buffer Output"),
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
        
        // Max Line Length
        ui.horizontal(|ui| {
            ui.label("Max Line Length:");
            let mut max_length = node.parameters.get("max_line_length")
                .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
                .unwrap_or(1000);
            
            if ui.add(egui::Slider::new(&mut max_length, 50..=10000)).changed() {
                changes.push(ParameterChange {
                    parameter: "max_line_length".to_string(),
                    value: NodeData::Integer(max_length),
                });
            }
        });
        
        // Date Format (only show if timestamp is enabled)
        let timestamp_enabled = node.parameters.get("include_timestamp")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
            
        if timestamp_enabled {
            ui.horizontal(|ui| {
                ui.label("Date Format:");
                let mut date_format = node.parameters.get("date_format")
                    .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_else(|| "%Y-%m-%d %H:%M:%S".to_string());
                
                if ui.text_edit_singleline(&mut date_format).changed() {
                    changes.push(ParameterChange {
                        parameter: "date_format".to_string(),
                        value: NodeData::String(date_format),
                    });
                }
            });
        }
        
        ui.separator();
        
        // Action buttons
        ui.horizontal(|ui| {
            if ui.button("Test Print").clicked() {
                let label = node.parameters.get("label")
                    .and_then(|v| if let NodeData::String(s) = v { Some(s.as_str()) } else { None })
                    .unwrap_or("Print Node");
                println!("Test print from: {}", label);
            }
            if ui.button("Clear History").clicked() {
                // This would be handled by the node logic
                println!("Print history cleared");
            }
        });
        
        changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_print_node() {
        let node = PrintNode::default();
        assert_eq!(node.label, "");
        assert!(matches!(node.output_destination, PrintDestination::Console));
        assert!(matches!(node.output_format, PrintFormat::Simple));
        assert!(matches!(node.line_endings, LineEnding::Auto));
        assert!(!node.include_timestamp);
        assert_eq!(node.max_line_length, 1000);
        assert!(node.append_mode);
    }
}