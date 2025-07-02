//! USD (Universal Scene Description) nodes for 3D workflows

// Local USD installation management
pub mod local_usd;

// USD engine for Python API integration
pub mod usd_engine;

// Categorized USD node modules
pub mod stage;
pub mod geometry;
pub mod transform;
pub mod lighting;
pub mod shading;

// Legacy nodes (keeping for compatibility)
mod usd_xform;
mod usd_mesh;
mod usd_sphere;
mod usd_cube;
mod usd_camera;
mod usd_light;
mod usd_material;
mod usd_viewport;
mod usd_composition;
mod set_attribute;
mod get_attribute;
mod create_stage;
mod load_stage;
mod save_stage;

// Test module
pub mod test_usd;

// Re-export USD engine
pub use usd_engine::{USDEngine, USDStage, USDPrim, with_usd_engine};

// Re-export categorized nodes
pub use stage::*;
pub use geometry::*;
pub use transform::*;
pub use lighting::*;
pub use shading::*;

// Re-export legacy nodes
pub use create_stage::*;
pub use load_stage::*;
pub use save_stage::*;
pub use usd_xform::*;
pub use usd_mesh::*;
pub use usd_sphere::*;
pub use usd_cube::*;
pub use usd_camera::*;
pub use usd_light::*;
pub use usd_material::*;
pub use usd_viewport::*;
pub use usd_composition::*;
pub use set_attribute::*;
pub use get_attribute::*;
pub use test_usd::*;