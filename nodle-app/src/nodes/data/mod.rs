//! Data nodes for constants and variables

mod constant;
mod constant_enhanced;
mod variable;
mod variable_enhanced;

pub use constant::ConstantNode;
pub use constant_enhanced::ConstantNodeEnhanced;
pub use variable::VariableNode;
pub use variable_enhanced::VariableNodeEnhanced;