//! MaterialX-specific node implementations
//! 
//! This module contains all MaterialX context-specific nodes for material authoring,
//! shading, texturing, and rendering workflows.

// MaterialX node categories
pub mod shading;
pub mod textures; 
pub mod math;
pub mod utilities;

// Re-export commonly used types
pub use shading::*;
pub use textures::*;
pub use math::*;
pub use utilities::*;