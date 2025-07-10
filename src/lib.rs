//! Nodle core library
//! 
//! This library exposes core functionality with clean separation from plugins.

// Public modules  
pub mod plugin_interface;
pub mod viewport;

// Re-export commonly used types
pub use plugin_interface::{NodeData, ParameterChange, UIElement, UIAction, ParameterUI};