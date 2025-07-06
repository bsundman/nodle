//! Test node parameters - comprehensive UI testing

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use super::logic::TestLogic;

/// Test node with comprehensive parameter testing
#[derive(Debug, Clone)]
pub struct TestNode {
    // Basic parameters
    pub label: String,
    pub enabled: bool,
    pub description: String,
    
    // Numeric parameters
    pub float_value: f32,
    pub int_value: i32,
    pub slider_value: f32,
    pub drag_value: f32,
    
    // Boolean parameters
    pub checkbox_1: bool,
    pub checkbox_2: bool,
    pub radio_option: i32,
    
    // String parameters
    pub text_input: String,
    pub multiline_text: String,
    
    // Color parameters
    pub color_rgb: [f32; 3],
    pub color_rgba: [f32; 4],
    
    // Vector parameters
    pub vector3: [f32; 3],
    
    // Enum/combo parameters
    pub operation_mode: String,
    pub quality_setting: String,
    
    // Advanced parameters
    pub progress_value: f32,
    pub angle_degrees: f32,
}

impl Default for TestNode {
    fn default() -> Self {
        Self {
            label: "Test Node".to_string(),
            enabled: true,
            description: "Comprehensive parameter testing node".to_string(),
            
            float_value: 1.0,
            int_value: 10,
            slider_value: 0.5,
            drag_value: 2.5,
            
            checkbox_1: false,
            checkbox_2: true,
            radio_option: 0,
            
            text_input: "Hello World".to_string(),
            multiline_text: "Line 1\nLine 2\nLine 3".to_string(),
            
            color_rgb: [1.0, 0.5, 0.0],
            color_rgba: [0.0, 0.5, 1.0, 0.8],
            
            vector3: [1.0, 2.0, 3.0],
            
            operation_mode: "passthrough".to_string(),
            quality_setting: "medium".to_string(),
            
            progress_value: 0.75,
            angle_degrees: 45.0,
        }
    }
}

impl TestNode {
    /// Pattern A: build_interface method with comprehensive parameter testing
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        // HUGE OBVIOUS TEST - Multiple ways to make this visible
        ui.colored_label(egui::Color32::RED, "🚨🚨🚨 TEST NODE CUSTOM INTERFACE IS WORKING! 🚨🚨🚨");
        ui.heading("THIS IS THE TEST NODE BUILD_INTERFACE METHOD");
        ui.colored_label(egui::Color32::GREEN, "If you can see this, the custom interface is being called!");
        if ui.button("🧪 TEST BUTTON - CLICK ME!").clicked() {
            println!("🧪 TEST BUTTON WAS CLICKED!");
        }
        ui.separator();
        
        // NOTE: We don't render the name field here because parameter.rs already does that
        // The name field is handled by the parameter panel itself
        
        // Collapsible Details section (replacing the default info)
        ui.collapsing("Details", |ui| {
            ui.label(format!("Node: {}", node.title));
            ui.label(format!("Type: {:?}", node.node_type));
            ui.label(format!("Position: ({:.1}, {:.1})", node.position.x, node.position.y));
            
            ui.separator();
            
            ui.label("Input Ports:");
            for (i, input) in node.inputs.iter().enumerate() {
                ui.label(format!("  {}: {}", i, input.name));
            }
            
            ui.label("Output Ports:");
            for (i, output) in node.outputs.iter().enumerate() {
                ui.label(format!("  {}: {}", i, output.name));
            }
            
            ui.separator();
            ui.label(format!("Node ID: {}", node.id));
        });
        
        ui.separator();
        
        // === BASIC CONTROLS SECTION ===
        ui.heading("Basic Controls");
        
        // Enabled checkbox
        ui.horizontal(|ui| {
            ui.label("Enabled:");
            let mut enabled = node.parameters.get("enabled")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(true);
            
            if ui.checkbox(&mut enabled, "Process data").changed() {
                changes.push(ParameterChange {
                    parameter: "enabled".to_string(),
                    value: NodeData::Boolean(enabled),
                });
            }
        });
        
        // Description text area
        ui.label("Description:");
        let mut description = node.parameters.get("description")
            .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
            .unwrap_or_else(|| "Comprehensive parameter testing node".to_string());
        
        if ui.text_edit_multiline(&mut description).changed() {
            changes.push(ParameterChange {
                parameter: "description".to_string(),
                value: NodeData::String(description),
            });
        }
        
        ui.separator();
        
        // === NUMERIC CONTROLS SECTION ===
        ui.collapsing("Numeric Controls", |ui| {
            // Float input
            ui.horizontal(|ui| {
                ui.label("Float Value:");
                let mut float_val = node.parameters.get("float_value")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(1.0);
                
                if ui.add(egui::DragValue::new(&mut float_val).speed(0.1)).changed() {
                    changes.push(ParameterChange {
                        parameter: "float_value".to_string(),
                        value: NodeData::Float(float_val),
                    });
                }
            });
            
            // Integer input
            ui.horizontal(|ui| {
                ui.label("Integer Value:");
                let mut int_val = node.parameters.get("int_value")
                    .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
                    .unwrap_or(10);
                
                if ui.add(egui::DragValue::new(&mut int_val)).changed() {
                    changes.push(ParameterChange {
                        parameter: "int_value".to_string(),
                        value: NodeData::Integer(int_val),
                    });
                }
            });
            
            // Slider
            ui.horizontal(|ui| {
                ui.label("Slider:");
                let mut slider_val = node.parameters.get("slider_value")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(0.5);
                
                if ui.add(egui::Slider::new(&mut slider_val, 0.0..=1.0).text("Value")).changed() {
                    changes.push(ParameterChange {
                        parameter: "slider_value".to_string(),
                        value: NodeData::Float(slider_val),
                    });
                }
            });
            
            // Angle slider
            ui.horizontal(|ui| {
                ui.label("Angle:");
                let mut angle = node.parameters.get("angle_degrees")
                    .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                    .unwrap_or(45.0);
                
                if ui.add(egui::Slider::new(&mut angle, 0.0..=360.0).suffix("°")).changed() {
                    changes.push(ParameterChange {
                        parameter: "angle_degrees".to_string(),
                        value: NodeData::Float(angle),
                    });
                }
            });
        });
        
        // === BOOLEAN CONTROLS SECTION ===
        ui.collapsing("Boolean Controls", |ui| {
            // Multiple checkboxes
            ui.horizontal(|ui| {
                let mut check1 = node.parameters.get("checkbox_1")
                    .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                    .unwrap_or(false);
                
                if ui.checkbox(&mut check1, "Option A").changed() {
                    changes.push(ParameterChange {
                        parameter: "checkbox_1".to_string(),
                        value: NodeData::Boolean(check1),
                    });
                }
                
                let mut check2 = node.parameters.get("checkbox_2")
                    .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                    .unwrap_or(true);
                
                if ui.checkbox(&mut check2, "Option B").changed() {
                    changes.push(ParameterChange {
                        parameter: "checkbox_2".to_string(),
                        value: NodeData::Boolean(check2),
                    });
                }
            });
            
            // Radio buttons
            ui.label("Radio Options:");
            let mut radio_val = node.parameters.get("radio_option")
                .and_then(|v| if let NodeData::Integer(i) = v { Some(*i) } else { None })
                .unwrap_or(0);
            
            ui.horizontal(|ui| {
                if ui.radio_value(&mut radio_val, 0, "Option 1").changed() {
                    changes.push(ParameterChange {
                        parameter: "radio_option".to_string(),
                        value: NodeData::Integer(radio_val),
                    });
                }
                if ui.radio_value(&mut radio_val, 1, "Option 2").changed() {
                    changes.push(ParameterChange {
                        parameter: "radio_option".to_string(),
                        value: NodeData::Integer(radio_val),
                    });
                }
                if ui.radio_value(&mut radio_val, 2, "Option 3").changed() {
                    changes.push(ParameterChange {
                        parameter: "radio_option".to_string(),
                        value: NodeData::Integer(radio_val),
                    });
                }
            });
        });
        
        // === TEXT CONTROLS SECTION ===
        ui.collapsing("Text Controls", |ui| {
            // Single line text
            ui.horizontal(|ui| {
                ui.label("Text Input:");
                let mut text = node.parameters.get("text_input")
                    .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_else(|| "Hello World".to_string());
                
                if ui.text_edit_singleline(&mut text).changed() {
                    changes.push(ParameterChange {
                        parameter: "text_input".to_string(),
                        value: NodeData::String(text),
                    });
                }
            });
            
            // Multi-line text
            ui.label("Multi-line Text:");
            let mut multitext = node.parameters.get("multiline_text")
                .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or_else(|| "Line 1\nLine 2\nLine 3".to_string());
            
            if ui.text_edit_multiline(&mut multitext).changed() {
                changes.push(ParameterChange {
                    parameter: "multiline_text".to_string(),
                    value: NodeData::String(multitext),
                });
            }
        });
        
        // === COMBO BOX SECTION ===
        ui.collapsing("Combo Boxes", |ui| {
            // Operation mode combo
            ui.horizontal(|ui| {
                ui.label("Operation:");
                let mut op_mode = node.parameters.get("operation_mode")
                    .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_else(|| "passthrough".to_string());
                
                egui::ComboBox::from_label("")
                    .selected_text(&op_mode)
                    .show_ui(ui, |ui| {
                        if ui.selectable_value(&mut op_mode, "passthrough".to_string(), "Passthrough").changed() {
                            changes.push(ParameterChange {
                                parameter: "operation_mode".to_string(),
                                value: NodeData::String(op_mode.clone()),
                            });
                        }
                        if ui.selectable_value(&mut op_mode, "multiply".to_string(), "Multiply").changed() {
                            changes.push(ParameterChange {
                                parameter: "operation_mode".to_string(),
                                value: NodeData::String(op_mode.clone()),
                            });
                        }
                        if ui.selectable_value(&mut op_mode, "debug".to_string(), "Debug").changed() {
                            changes.push(ParameterChange {
                                parameter: "operation_mode".to_string(),
                                value: NodeData::String(op_mode.clone()),
                            });
                        }
                    });
            });
            
            // Quality combo
            ui.horizontal(|ui| {
                ui.label("Quality:");
                let mut quality = node.parameters.get("quality_setting")
                    .and_then(|v| if let NodeData::String(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_else(|| "medium".to_string());
                
                egui::ComboBox::from_label("")
                    .selected_text(&quality)
                    .show_ui(ui, |ui| {
                        if ui.selectable_value(&mut quality, "low".to_string(), "Low").changed() {
                            changes.push(ParameterChange {
                                parameter: "quality_setting".to_string(),
                                value: NodeData::String(quality.clone()),
                            });
                        }
                        if ui.selectable_value(&mut quality, "medium".to_string(), "Medium").changed() {
                            changes.push(ParameterChange {
                                parameter: "quality_setting".to_string(),
                                value: NodeData::String(quality.clone()),
                            });
                        }
                        if ui.selectable_value(&mut quality, "high".to_string(), "High").changed() {
                            changes.push(ParameterChange {
                                parameter: "quality_setting".to_string(),
                                value: NodeData::String(quality.clone()),
                            });
                        }
                    });
            });
        });
        
        // === COLOR CONTROLS SECTION ===
        ui.collapsing("Color Controls", |ui| {
            // RGB color
            ui.horizontal(|ui| {
                ui.label("RGB Color:");
                let color_data = node.parameters.get("color_rgb")
                    .and_then(|v| if let NodeData::Color(c) = v { Some(*c) } else { None })
                    .unwrap_or([1.0, 0.5, 0.0, 1.0]);
                
                let mut color = [color_data[0], color_data[1], color_data[2]];
                
                if ui.color_edit_button_rgb(&mut color).changed() {
                    changes.push(ParameterChange {
                        parameter: "color_rgb".to_string(),
                        value: NodeData::Color([color[0], color[1], color[2], 1.0]),
                    });
                }
            });
            
            // RGBA color
            ui.horizontal(|ui| {
                ui.label("RGBA Color:");
                let color_data = node.parameters.get("color_rgba")
                    .and_then(|v| if let NodeData::Color(c) = v { Some(*c) } else { None })
                    .unwrap_or([0.0, 0.5, 1.0, 0.8]);
                
                let mut color = egui::Color32::from_rgba_premultiplied(
                    (color_data[0] * 255.0) as u8,
                    (color_data[1] * 255.0) as u8,
                    (color_data[2] * 255.0) as u8,
                    (color_data[3] * 255.0) as u8,
                );
                
                if ui.color_edit_button_srgba(&mut color).changed() {
                    let color_array = color.to_srgba_unmultiplied();
                    changes.push(ParameterChange {
                        parameter: "color_rgba".to_string(),
                        value: NodeData::Color([
                            color_array[0] as f32 / 255.0,
                            color_array[1] as f32 / 255.0,
                            color_array[2] as f32 / 255.0,
                            color_array[3] as f32 / 255.0,
                        ]),
                    });
                }
            });
        });
        
        // === VECTOR CONTROLS SECTION ===
        ui.collapsing("Vector Controls", |ui| {
            // Vector3
            ui.horizontal(|ui| {
                ui.label("Vector3:");
                let mut vec3 = node.parameters.get("vector3")
                    .and_then(|v| if let NodeData::Vector3(v) = v { Some(*v) } else { None })
                    .unwrap_or([1.0, 2.0, 3.0]);
                
                ui.label("X:");
                if ui.add(egui::DragValue::new(&mut vec3[0]).speed(0.1)).changed() {
                    changes.push(ParameterChange {
                        parameter: "vector3".to_string(),
                        value: NodeData::Vector3(vec3),
                    });
                }
                ui.label("Y:");
                if ui.add(egui::DragValue::new(&mut vec3[1]).speed(0.1)).changed() {
                    changes.push(ParameterChange {
                        parameter: "vector3".to_string(),
                        value: NodeData::Vector3(vec3),
                    });
                }
                ui.label("Z:");
                if ui.add(egui::DragValue::new(&mut vec3[2]).speed(0.1)).changed() {
                    changes.push(ParameterChange {
                        parameter: "vector3".to_string(),
                        value: NodeData::Vector3(vec3),
                    });
                }
            });
        });
        
        // === PROGRESS AND STATUS SECTION ===
        ui.collapsing("Progress & Status", |ui| {
            // Progress bar
            let progress = node.parameters.get("progress_value")
                .and_then(|v| if let NodeData::Float(f) = v { Some(*f) } else { None })
                .unwrap_or(0.75);
            
            ui.add(egui::ProgressBar::new(progress).text(format!("{:.0}%", progress * 100.0)));
            
            ui.horizontal(|ui| {
                if ui.button("Reset Progress").clicked() {
                    changes.push(ParameterChange {
                        parameter: "progress_value".to_string(),
                        value: NodeData::Float(0.0),
                    });
                }
                if ui.button("Complete Progress").clicked() {
                    changes.push(ParameterChange {
                        parameter: "progress_value".to_string(),
                        value: NodeData::Float(1.0),
                    });
                }
            });
        });
        
        // === ACTION BUTTONS SECTION ===
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("🔄 Reset All").clicked() {
                // Add reset changes for all parameters
                changes.push(ParameterChange {
                    parameter: "enabled".to_string(),
                    value: NodeData::Boolean(true),
                });
                changes.push(ParameterChange {
                    parameter: "float_value".to_string(),
                    value: NodeData::Float(1.0),
                });
                changes.push(ParameterChange {
                    parameter: "slider_value".to_string(),
                    value: NodeData::Float(0.5),
                });
                // Add more reset values as needed...
            }
            
            if ui.button("🎯 Test Action").clicked() {
                println!("Test button clicked!");
                changes.push(ParameterChange {
                    parameter: "test_clicked".to_string(),
                    value: NodeData::Boolean(true),
                });
            }
            
            if ui.button("⚡ Random Values").clicked() {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                
                changes.push(ParameterChange {
                    parameter: "float_value".to_string(),
                    value: NodeData::Float(rng.gen_range(0.0..10.0)),
                });
                changes.push(ParameterChange {
                    parameter: "slider_value".to_string(),
                    value: NodeData::Float(rng.gen()),
                });
            }
        });
        
        changes
    }
    
    /// Convert current parameters to TestLogic for processing
    pub fn to_test_logic(&self) -> TestLogic {
        TestLogic {
            enabled: self.enabled,
            multiplier: self.float_value,
            operation_mode: self.operation_mode.clone(),
        }
    }
}