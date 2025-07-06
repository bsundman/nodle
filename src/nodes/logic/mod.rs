//! Logic operation nodes - modular structure with Pattern A interfaces

pub mod and;    // AND logic gate with Pattern A interface
pub mod or;     // OR/XOR logic gate with Pattern A interface
pub mod not;    // NOT/BUFFER logic gate with Pattern A interface

// Export all node factories
pub use and::AndNodeFactory;
pub use or::OrNodeFactory;
pub use not::NotNodeFactory;

// Parameter struct exports removed - unused