//! 3D node implementations using modular NodeFactory pattern

pub mod transform;
pub mod geometry;
pub mod lighting;
pub mod output;

// Re-export all 3D nodes for easy access
pub use transform::*;
pub use geometry::*;
pub use lighting::*;
pub use output::*;