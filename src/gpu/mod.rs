//! GPU rendering module
//! 
//! This module contains all GPU-related functionality for the Nōdle editor,
//! including instance data structures, rendering pipelines, and shader management.
//! 
//! The GPU rendering system provides high-performance rendering of thousands of nodes
//! and ports using wgpu instanced rendering. It maintains visual parity with the CPU
//! rendering path while delivering 60fps performance with 5000+ nodes.
//! 
//! ## Architecture
//! 
//! - [`instance`] - Instance data structures and management
//! - [`renderer`] - Core GPU renderer and pipeline management  
//! - [`callback`] - egui paint callback integration
//! - `shaders/` - WGSL shader files for nodes and ports

pub mod config;
pub mod instance;
pub mod renderer;
pub mod renderer3d;
pub mod usd_renderer;
pub mod callback;
pub mod viewport_callback;

pub use config::{GraphicsConfig, global_sample_count};
pub use instance::{NodeInstanceData, PortInstanceData, ButtonInstanceData, FlagInstanceData, Uniforms, GpuInstanceManager};
pub use renderer::{GpuNodeRenderer, GLOBAL_GPU_RENDERER};
pub use renderer3d::{Renderer3D, Camera3D, Mesh3D, Vertex3D, Uniforms3D};
pub use usd_renderer::{USDRenderer, USDScene, USDGeometry, USDLight, USDMaterial, USDCamera, ShadingMode, CameraMode};
pub use callback::NodeRenderCallback;
pub use viewport_callback::{ViewportRenderCallback, USDRenderPass};