//! Output nodes for displaying results

// Modular structure
pub mod debug;           // Modular directory structure
pub mod print;           // Modular directory structure
pub mod console;         // Console output node
// scenegraph module moved to nodes::three_d::ui::scenegraph

// Export all modular node factories
pub use debug::DebugNodeFactory;
pub use print::PrintNodeFactory;
pub use console::{ConsoleLogic, ConsoleNodeFactory};
// scenegraph exports now available through nodes::three_d::ui::scenegraph