//! Print node functional operations - printing logic

use crate::nodes::interface::NodeData;

/// Core print data and functionality
#[derive(Debug, Clone)]
pub struct PrintLogic {
    /// Label to prepend to printed values
    pub label: String,
    /// Whether to include timestamp
    pub include_timestamp: bool,
    /// Whether to include type information
    pub include_type: bool,
    /// Print format
    pub format: PrintFormat,
    /// Output destination
    pub destination: PrintDestination,
    /// Print history (limited to last N prints)
    pub print_history: Vec<PrintRecord>,
    /// Maximum history size
    pub max_history_size: usize,
}

#[derive(Debug, Clone)]
pub enum PrintFormat {
    Simple,
    Formatted,
    Json,
    Debug,
}

#[derive(Debug, Clone)]
pub enum PrintDestination {
    Console,
    Log,
    File(String),
    Buffer,
}

#[derive(Debug, Clone)]
pub struct PrintRecord {
    /// The printed value
    pub value: String,
    /// Timestamp (simplified as string for now)
    pub timestamp: String,
    /// Value type
    pub value_type: String,
}

impl Default for PrintLogic {
    fn default() -> Self {
        Self {
            label: String::new(),
            include_timestamp: false,
            include_type: false,
            format: PrintFormat::Simple,
            destination: PrintDestination::Console,
            print_history: Vec::new(),
            max_history_size: 50,
        }
    }
}

impl PrintLogic {
    /// Process input data and print it
    pub fn process(&mut self, inputs: Vec<NodeData>) -> Vec<NodeData> {
        if inputs.is_empty() {
            return vec![];
        }
        
        for input in &inputs {
            let output = self.format_value(input);
            self.print_value(&output, input);
            
            // Record in history
            self.record_print(output, input);
        }
        
        // Print nodes typically don't output anything
        vec![]
    }
    
    /// Format a value according to current settings
    fn format_value(&self, value: &NodeData) -> String {
        let value_str = match value {
            NodeData::Float(f) => f.to_string(),
            NodeData::Boolean(b) => b.to_string(),
            NodeData::String(s) => s.clone(),
            NodeData::Vector3(v) => format!("[{}, {}, {}]", v[0], v[1], v[2]),
            NodeData::Color(c) => format!("rgba({}, {}, {}, {})", c[0], c[1], c[2], c[3]),
            _ => "Unknown".to_string(),
        };
        
        let mut output = String::new();
        
        // Add timestamp if requested
        if self.include_timestamp {
            output.push_str("[now] "); // Simplified timestamp
        }
        
        // Add label if provided
        if !self.label.is_empty() {
            output.push_str(&format!("{}: ", self.label));
        }
        
        // Add type if requested
        if self.include_type {
            output.push_str(&format!("({}) ", self.get_type_name(value)));
        }
        
        // Add the actual value based on format
        match self.format {
            PrintFormat::Simple => {
                output.push_str(&value_str);
            },
            PrintFormat::Formatted => {
                match value {
                    NodeData::Float(f) => output.push_str(&format!("{:.6}", f)),
                    NodeData::Vector3(v) => output.push_str(&format!("Vector3({:.3}, {:.3}, {:.3})", v[0], v[1], v[2])),
                    NodeData::Color(c) => output.push_str(&format!("Color(r={:.3}, g={:.3}, b={:.3}, a={:.3})", c[0], c[1], c[2], c[3])),
                    _ => output.push_str(&value_str),
                }
            },
            PrintFormat::Json => {
                // Simplified JSON representation
                match value {
                    NodeData::String(s) => output.push_str(&format!("\"{}\"", s)),
                    NodeData::Vector3(v) => output.push_str(&format!("[{}, {}, {}]", v[0], v[1], v[2])),
                    NodeData::Color(c) => output.push_str(&format!("{{\"r\": {}, \"g\": {}, \"b\": {}, \"a\": {}}}", c[0], c[1], c[2], c[3])),
                    _ => output.push_str(&value_str),
                }
            },
            PrintFormat::Debug => {
                output.push_str(&format!("{:?}", value));
            },
        }
        
        output
    }
    
    /// Print the formatted value to the appropriate destination
    fn print_value(&self, output: &str, _value: &NodeData) {
        match &self.destination {
            PrintDestination::Console => {
                println!("{}", output);
            },
            PrintDestination::Log => {
                // In a real implementation, this would use a logging framework
                println!("[LOG] {}", output);
            },
            PrintDestination::File(_path) => {
                // In a real implementation, this would write to a file
                println!("[FILE] {}", output);
            },
            PrintDestination::Buffer => {
                // Just store in history, don't actually print
            },
        }
    }
    
    /// Record a print operation in the history
    fn record_print(&mut self, output: String, value: &NodeData) {
        let record = PrintRecord {
            value: output,
            timestamp: "now".to_string(),
            value_type: self.get_type_name(value).to_string(),
        };
        
        self.print_history.push(record);
        
        // Keep history size limited
        if self.print_history.len() > self.max_history_size {
            self.print_history.remove(0);
        }
    }
    
    /// Get the type name of a value
    fn get_type_name(&self, value: &NodeData) -> &'static str {
        match value {
            NodeData::Float(_) => "Float",
            NodeData::Boolean(_) => "Boolean",
            NodeData::String(_) => "String",
            NodeData::Vector3(_) => "Vector3",
            NodeData::Color(_) => "Color",
            _ => "Unknown",
        }
    }
    
    /// Get the print count
    pub fn get_print_count(&self) -> usize {
        self.print_history.len()
    }
    
    /// Clear the print history
    pub fn clear_history(&mut self) {
        self.print_history.clear();
    }
    
    /// Get the format name for display
    pub fn get_format_name(&self) -> &'static str {
        match self.format {
            PrintFormat::Simple => "Simple",
            PrintFormat::Formatted => "Formatted",
            PrintFormat::Json => "JSON",
            PrintFormat::Debug => "Debug",
        }
    }
    
    /// Get the destination name for display
    pub fn get_destination_name(&self) -> String {
        match &self.destination {
            PrintDestination::Console => "Console".to_string(),
            PrintDestination::Log => "Log".to_string(),
            PrintDestination::File(path) => format!("File ({})", path),
            PrintDestination::Buffer => "Buffer".to_string(),
        }
    }
}