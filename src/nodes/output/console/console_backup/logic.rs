use crate::nodes::interface::NodeData;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ConsoleLogic {
    pub console_output: String,
    pub max_lines: usize,
    pub auto_scroll: bool,
}

impl Default for ConsoleLogic {
    fn default() -> Self {
        Self {
            console_output: String::new(),
            max_lines: 1000,
            auto_scroll: true,
        }
    }
}

impl ConsoleLogic {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process(&mut self, inputs: &HashMap<String, NodeData>) -> HashMap<String, NodeData> {
        // Get text input
        if let Some(NodeData::String(text)) = inputs.get("Text") {
            if !text.is_empty() {
                self.append_text(text);
            }
        }

        // Console node doesn't produce outputs, just displays text
        HashMap::new()
    }

    pub fn append_text(&mut self, text: &str) {
        // Add timestamp (using simple time counter since chrono may not be available)
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() % 86400; // Get seconds within the day
        let hours = timestamp / 3600;
        let minutes = (timestamp % 3600) / 60;
        let seconds = timestamp % 60;
        let time_str = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);
        let formatted_text = format!("[{}] {}", time_str, text);
        
        // Add to console output
        self.console_output.push_str(&formatted_text);
        self.console_output.push('\n');
        
        // Limit the number of lines to prevent memory issues
        let lines: Vec<&str> = self.console_output.lines().collect();
        if lines.len() > self.max_lines {
            let start = lines.len() - self.max_lines;
            self.console_output = lines[start..].join("\n");
            self.console_output.push('\n');
        }
    }

    pub fn clear_console(&mut self) {
        self.console_output.clear();
    }

    pub fn get_console_text(&self) -> &str {
        &self.console_output
    }
}