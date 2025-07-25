//! Core viewport module for 3D rendering
//! 
//! This module contains the core's own viewport types and functionality,
//! independent of the plugin SDK.

pub mod types;

// Re-export commonly used types
pub use types::{
    CameraData, MeshData, MaterialData, LightData, LightType,
    SceneData, ViewportSettings, ShadingMode, ViewportData,
    CameraManipulation,
};