//! Console viewer implementation
//!
//! Terminal-like console display for text output

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use egui::{ScrollArea, TextStyle, Color32, Frame};
use std::collections::VecDeque;

/// Console viewer state
#[derive(Debug, Clone)]
pub struct ConsoleViewer {
    /// Console text buffer with circular buffer
    pub lines: VecDeque<String>,
    /// Maximum number of lines to keep
    pub max_lines: usize,
    /// Auto-scroll to bottom
    pub auto_scroll: bool,
    /// Font size multiplier
    pub font_scale: f32,
    /// Background color
    pub bg_color: Color32,
    /// Text color
    pub text_color: Color32,
}

impl Default for ConsoleViewer {
    fn default() -> Self {
        Self {
            lines: VecDeque::new(),
            max_lines: 1000,
            auto_scroll: true,
            font_scale: 1.0,
            bg_color: Color32::from_rgb(0, 0, 0), // Black terminal background
            text_color: Color32::from_rgb(0, 255, 0), // Green terminal text
        }
    }
}

impl ConsoleViewer {
    /// Add a line to the console
    pub fn add_line(&mut self, line: String) {
        // Add timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() % 86400;
        let hours = timestamp / 3600;
        let minutes = (timestamp % 3600) / 60;
        let seconds = timestamp % 60;
        let time_str = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);
        let formatted_line = format!("[{}] {}", time_str, line);
        
        self.lines.push_back(formatted_line);
        
        // Maintain max lines limit
        while self.lines.len() > self.max_lines {
            self.lines.pop_front();
        }
    }
    
    /// Clear all console lines
    pub fn clear(&mut self) {
        self.lines.clear();
    }
    
    /// Render the console viewer UI
    pub fn render_viewer(&mut self, ui: &mut egui::Ui, node: &Node) -> Vec<ParameterChange> {
        let changes = Vec::new();
        
        // Process input text if connected
        if let Some(NodeData::String(input_text)) = node.parameters.get("Text") {
            if !input_text.is_empty() {
                // Split input by lines and add each
                for line in input_text.lines() {
                    if !line.trim().is_empty() {
                        self.add_line(line.to_string());
                    }
                }
            }
        }
        
        // Terminal-like frame with black background
        Frame::none()
            .fill(self.bg_color)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // Console title bar
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::from_rgb(255, 255, 255), "📟 Console");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("Clear").clicked() {
                                self.clear();
                            }
                            ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
                        });
                    });
                    
                    ui.separator();
                    
                    // Console output area with terminal styling
                    let available_height = ui.available_height() - 10.0;
                    
                    ScrollArea::vertical()
                        .max_height(available_height)
                        .auto_shrink([false, false])
                        .stick_to_bottom(self.auto_scroll)
                        .show(ui, |ui| {
                            // Use monospace font for terminal feel
                            ui.style_mut().override_text_style = Some(TextStyle::Monospace);
                            
                            if self.lines.is_empty() {
                                ui.colored_label(
                                    Color32::from_rgb(128, 128, 128), 
                                    "Console ready. Connect text input to display output."
                                );
                            } else {
                                for line in &self.lines {
                                    // Color code different types of messages
                                    let color = if line.contains("[ERROR]") || line.contains("ERROR:") {
                                        Color32::from_rgb(255, 100, 100) // Red
                                    } else if line.contains("[WARN]") || line.contains("WARN:") {
                                        Color32::from_rgb(255, 200, 100) // Yellow
                                    } else if line.contains("[INFO]") || line.contains("INFO:") {
                                        Color32::from_rgb(100, 200, 255) // Blue
                                    } else if line.contains("[DEBUG]") || line.contains("DEBUG:") {
                                        Color32::from_rgb(200, 100, 255) // Purple
                                    } else {
                                        self.text_color // Default green
                                    };
                                    
                                    ui.colored_label(color, line);
                                }
                            }
                        });
                });
            });
        
        changes
    }
}