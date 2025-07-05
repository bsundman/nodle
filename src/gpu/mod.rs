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
//! - [`canvas_instance`] - Canvas instance data structures and management
//! - [`canvas_rendering`] - Core GPU canvas renderer and pipeline management  
//! - [`canvas_callback`] - egui paint callback integration for canvas
//! - [`viewport_3d_rendering`] - 3D viewport renderer and pipeline management
//! - [`viewport_3d_callback`] - egui paint callback integration for 3D viewport
//! - `shaders/` - WGSL shader files for nodes and ports

pub mod config;
pub mod canvas_instance;
pub mod canvas_rendering;
pub mod viewport_3d_rendering;
pub mod canvas_callback;
pub mod viewport_3d_callback;

pub use config::{GraphicsConfig, global_sample_count};
pub use canvas_instance::{NodeInstanceData, PortInstanceData, ButtonInstanceData, FlagInstanceData, Uniforms, GpuInstanceManager};
pub use canvas_rendering::{GpuNodeRenderer, GLOBAL_GPU_RENDERER};
pub use viewport_3d_rendering::{Renderer3D, Camera3D};
// USD rendering now handled by USD plugin
pub use canvas_callback::NodeRenderCallback;
pub use viewport_3d_callback::{ViewportRenderCallback};