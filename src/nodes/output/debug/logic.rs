//! Debug node functional operations - debugging logic

use crate::nodes::interface::NodeData;
use std::collections::HashMap;

/// Core debug data and functionality
#[derive(Debug, Clone)]
pub struct DebugLogic {
    /// Debug level
    pub level: DebugLevel,
    /// Whether to include stack trace information
    pub include_stack_trace: bool,
    /// Whether to include memory usage
    pub include_memory_info: bool,
    /// Whether to include timing information
    pub include_timing: bool,
    /// Custom debug label
    pub custom_label: String,
    /// Debug categories to enable
    pub enabled_categories: Vec<DebugCategory>,
    /// Debug history
    pub debug_history: Vec<DebugRecord>,
    /// Maximum history size
    pub max_history_size: usize,
    /// Performance counters
    pub performance_counters: HashMap<String, u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DebugLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DebugCategory {
    Value,
    Type,
    Memory,
    Performance,
    Flow,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct DebugRecord {
    /// Debug message
    pub message: String,
    /// Debug level
    pub level: DebugLevel,
    /// Value being debugged
    pub value: String,
    /// Value type
    pub value_type: String,
    /// Timestamp
    pub timestamp: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Default for DebugLogic {
    fn default() -> Self {
        Self {
            level: DebugLevel::Debug,
            include_stack_trace: false,
            include_memory_info: false,
            include_timing: false,
            custom_label: String::new(),
            enabled_categories: vec![
                DebugCategory::Value,
                DebugCategory::Type,
            ],
            debug_history: Vec::new(),
            max_history_size: 100,
            performance_counters: HashMap::new(),
        }
    }
}

impl DebugLogic {
    /// Process input data and debug it
    pub fn process(&mut self, inputs: Vec<NodeData>) -> Vec<NodeData> {
        if inputs.is_empty() {
            return vec![];
        }
        
        let value = &inputs[0];
        let label = if inputs.len() >= 2 {
            if let NodeData::String(ref s) = inputs[1] {
                s.clone()
            } else {
                self.custom_label.clone()
            }
        } else {
            self.custom_label.clone()
        };
        
        // Perform debugging
        self.debug_value(value, &label);
        
        // Increment performance counter
        *self.performance_counters.entry("debug_calls".to_string()).or_insert(0) += 1;
        
        // Pass through the original value
        vec![value.clone()]
    }
    
    /// Debug a specific value
    fn debug_value(&mut self, value: &NodeData, label: &str) {
        let mut debug_info = Vec::new();
        
        // Basic value information
        if self.enabled_categories.contains(&DebugCategory::Value) {
            debug_info.push(format!("Value: {}", self.format_value(value)));
        }
        
        // Type information
        if self.enabled_categories.contains(&DebugCategory::Type) {
            debug_info.push(format!("Type: {}", self.get_type_name(value)));
            debug_info.push(format!("Size: {} bytes", self.estimate_size(value)));
        }
        
        // Memory information
        if self.include_memory_info && self.enabled_categories.contains(&DebugCategory::Memory) {
            debug_info.push("Memory: [simulated info]".to_string());
        }
        
        // Performance information
        if self.enabled_categories.contains(&DebugCategory::Performance) {
            if let Some(count) = self.performance_counters.get("debug_calls") {
                debug_info.push(format!("Debug calls: {}", count));
            }
        }
        
        // Timing information
        if self.include_timing {
            debug_info.push("Timestamp: [now]".to_string());
        }
        
        // Create debug message
        let message = if label.is_empty() {
            debug_info.join(", ")
        } else {
            format!("{}: {}", label, debug_info.join(", "))
        };
        
        // Output debug message
        self.output_debug(&message, value);
        
        // Record in history
        self.record_debug(message, value);
    }
    
    /// Output debug message to appropriate destination
    fn output_debug(&self, message: &str, _value: &NodeData) {
        let level_prefix = match self.level {
            DebugLevel::Trace => "[TRACE]",
            DebugLevel::Debug => "[DEBUG]",
            DebugLevel::Info => "[INFO]",
            DebugLevel::Warn => "[WARN]",
            DebugLevel::Error => "[ERROR]",
        };
        
        println!("{} {}", level_prefix, message);
        
        // Stack trace if requested
        if self.include_stack_trace {
            println!("[STACK] Debug node stack trace (simulated)");
        }
    }
    
    /// Record a debug operation in the history
    fn record_debug(&mut self, message: String, value: &NodeData) {
        let mut metadata = HashMap::new();
        metadata.insert("size".to_string(), self.estimate_size(value).to_string());
        
        let record = DebugRecord {
            message,
            level: self.level.clone(),
            value: self.format_value(value),
            value_type: self.get_type_name(value).to_string(),
            timestamp: "now".to_string(),
            metadata,
        };
        
        self.debug_history.push(record);
        
        // Keep history size limited
        if self.debug_history.len() > self.max_history_size {
            self.debug_history.remove(0);
        }
    }
    
    /// Format a value for debugging
    fn format_value(&self, value: &NodeData) -> String {
        match value {
            NodeData::Float(f) => format!("{:.6}", f),
            NodeData::Boolean(b) => b.to_string(),
            NodeData::String(s) => format!("\"{}\"", s),
            NodeData::Vector3(v) => format!("Vector3({:.3}, {:.3}, {:.3})", v[0], v[1], v[2]),
            NodeData::Color(c) => format!("Color(r={:.3}, g={:.3}, b={:.3}, a={:.3})", c[0], c[1], c[2], c[3]),
            _ => "Unknown".to_string(),
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
    
    /// Estimate the size of a value in bytes
    fn estimate_size(&self, value: &NodeData) -> usize {
        match value {
            NodeData::Float(_) => 4,
            NodeData::Boolean(_) => 1,
            NodeData::String(s) => s.len(),
            NodeData::Vector3(_) => 12, // 3 * 4 bytes
            NodeData::Color(_) => 16,   // 4 * 4 bytes
            _ => 0,
        }
    }
    
    /// Get the debug count
    pub fn get_debug_count(&self) -> usize {
        self.debug_history.len()
    }
    
    /// Clear the debug history
    pub fn clear_history(&mut self) {
        self.debug_history.clear();
    }
    
    /// Get the level name for display
    pub fn get_level_name(&self) -> &'static str {
        match self.level {
            DebugLevel::Trace => "Trace",
            DebugLevel::Debug => "Debug",
            DebugLevel::Info => "Info",
            DebugLevel::Warn => "Warn",
            DebugLevel::Error => "Error",
        }
    }
    
    /// Reset performance counters
    pub fn reset_counters(&mut self) {
        self.performance_counters.clear();
    }
}