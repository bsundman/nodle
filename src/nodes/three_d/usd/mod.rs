//! USD (Universal Scene Description) nodes for 3D workflows

// USD engine for Python API integration
pub mod usd_engine;

// Stage management nodes
mod create_stage;
mod load_stage;
mod save_stage;

// Primitive nodes
mod usd_xform;
mod usd_mesh;
mod usd_sphere;
mod usd_cube;
mod usd_camera;
mod usd_light;

// Material and shader nodes
mod usd_material;

// Viewport and output nodes
mod usd_viewport;

// Composition nodes (layer management)
mod usd_composition;

// Attribute nodes
mod set_attribute;
mod get_attribute;

// Test module
pub mod test_usd;

// Re-export all USD components
pub use usd_engine::{USDEngine, USDStage, USDPrim, with_usd_engine};
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