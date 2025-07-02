//! Data nodes for constants and variables - modular structure

pub mod constant;   // Modular directory structure
pub mod variable;   // Modular directory structure

// Export all factories for the new Pattern A nodes
pub use constant::ConstantNodeFactory;
pub use variable::VariableNodeFactory;

// Keep legacy exports for backwards compatibility
pub use constant::ConstantNode as LegacyConstantNode;
pub use variable::VariableNode as LegacyVariableNode;