//! 3D Geometry nodes

pub mod cube;  // Modular directory structure
pub mod sphere;  // Modular directory structure
pub mod plane;

pub use cube::{CubeNode3D, CubeNode};
pub use sphere::{SphereNode3D, SphereNode};
pub use plane::{PlaneNode3D, PlaneNode};