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

pub mod instance;
pub mod renderer;
pub mod callback;

pub use instance::{NodeInstanceData, PortInstanceData, Uniforms, GpuInstanceManager};
pub use renderer::{GpuNodeRenderer, GLOBAL_GPU_RENDERER};
pub use callback::NodeRenderCallback;