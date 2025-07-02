//! Output nodes for displaying results

// Modular structure
pub mod debug;           // Modular directory structure
pub mod print;           // Modular directory structure

// Export all modular node factories
pub use debug::DebugNodeFactory;
pub use print::PrintNodeFactory;