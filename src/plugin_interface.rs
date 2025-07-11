//! Plugin interface traits and types
//! 
//! This module defines the core plugin interface and conversion layer between
//! core and SDK viewport types. This avoids circular dependencies.

use std::any::Any;
use serde::{Serialize, Deserialize};

/// Core plugin trait that SDK will extend
pub trait PluginCore: Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

/// Node data types used by both core and plugins
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeData {
    // Basic types
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]), 
    Vec4([f32; 4]),
    Int(i32),
    Boolean(bool),
    String(String),
    
    // Complex types
    Matrix4([[f32; 4]; 4]),
    Color([f32; 4]),
    FilePath(String),
    
    // Collections
    FloatArray(Vec<f32>),
    Vec3Array(Vec<[f32; 3]>),
    StringArray(Vec<String>),
    
    // Special
    None,
}

/// Parameter change notification
#[derive(Debug, Clone)]
pub struct ParameterChange {
    pub parameter: String,
    pub value: NodeData,
}

/// UI element types for parameter interfaces
#[derive(Debug, Clone, PartialEq)]
pub enum UIElement {
    Label(String),
    Separator,
    Group(String, Vec<UIElement>),
    Row(Vec<UIElement>),
    Column(Vec<UIElement>),
}

/// UI action types
#[derive(Debug, Clone, PartialEq)]
pub enum UIAction {
    None,
    ValueChanged,
    ButtonClicked,
    FileSelected,
}

/// Parameter UI trait for custom parameter interfaces
pub trait ParameterUI: Send + Sync {
    fn build_ui(&mut self, ui: &mut egui::Ui) -> Vec<ParameterChange>;
}

// ===== VIEWPORT TYPE CONVERSIONS =====
// These functions convert between core and SDK viewport types at the plugin boundary

/// Convert SDK ViewportData to Core ViewportData
impl From<nodle_plugin_sdk::viewport::ViewportData> for crate::viewport::ViewportData {
    fn from(sdk_data: nodle_plugin_sdk::viewport::ViewportData) -> Self {
        crate::viewport::ViewportData {
            scene: sdk_data.scene.into(),
            settings: sdk_data.settings.into(),
            dimensions: sdk_data.dimensions,
            scene_dirty: sdk_data.scene_dirty,
            settings_dirty: sdk_data.settings_dirty,
        }
    }
}

/// Convert Core ViewportData to SDK ViewportData
impl From<crate::viewport::ViewportData> for nodle_plugin_sdk::viewport::ViewportData {
    fn from(core_data: crate::viewport::ViewportData) -> Self {
        nodle_plugin_sdk::viewport::ViewportData {
            scene: core_data.scene.into(),
            settings: core_data.settings.into(),
            dimensions: core_data.dimensions,
            scene_dirty: core_data.scene_dirty,
            settings_dirty: core_data.settings_dirty,
        }
    }
}

/// Convert SDK SceneData to Core SceneData
impl From<nodle_plugin_sdk::viewport::SceneData> for crate::viewport::SceneData {
    fn from(sdk_scene: nodle_plugin_sdk::viewport::SceneData) -> Self {
        crate::viewport::SceneData {
            name: sdk_scene.name,
            meshes: sdk_scene.meshes.into_iter().map(|m| m.into()).collect(),
            materials: sdk_scene.materials.into_iter().map(|m| m.into()).collect(),
            lights: sdk_scene.lights.into_iter().map(|l| l.into()).collect(),
            camera: sdk_scene.camera.into(),
            bounding_box: sdk_scene.bounding_box,
        }
    }
}

/// Convert Core SceneData to SDK SceneData  
impl From<crate::viewport::SceneData> for nodle_plugin_sdk::viewport::SceneData {
    fn from(core_scene: crate::viewport::SceneData) -> Self {
        nodle_plugin_sdk::viewport::SceneData {
            name: core_scene.name,
            meshes: core_scene.meshes.into_iter().map(|m| m.into()).collect(),
            materials: core_scene.materials.into_iter().map(|m| m.into()).collect(),
            lights: core_scene.lights.into_iter().map(|l| l.into()).collect(),
            camera: core_scene.camera.into(),
            bounding_box: core_scene.bounding_box,
        }
    }
}

/// Convert SDK CameraData to Core CameraData
impl From<nodle_plugin_sdk::viewport::CameraData> for crate::viewport::CameraData {
    fn from(sdk_camera: nodle_plugin_sdk::viewport::CameraData) -> Self {
        crate::viewport::CameraData {
            position: sdk_camera.position,
            target: sdk_camera.target,
            up: sdk_camera.up,
            fov: sdk_camera.fov,
            near: sdk_camera.near,
            far: sdk_camera.far,
            aspect: sdk_camera.aspect,
        }
    }
}

/// Convert Core CameraData to SDK CameraData
impl From<crate::viewport::CameraData> for nodle_plugin_sdk::viewport::CameraData {
    fn from(core_camera: crate::viewport::CameraData) -> Self {
        nodle_plugin_sdk::viewport::CameraData {
            position: core_camera.position,
            target: core_camera.target,
            up: core_camera.up,
            fov: core_camera.fov,
            near: core_camera.near,
            far: core_camera.far,
            aspect: core_camera.aspect,
        }
    }
}

/// Convert SDK MeshData to Core MeshData
impl From<nodle_plugin_sdk::viewport::MeshData> for crate::viewport::MeshData {
    fn from(sdk_mesh: nodle_plugin_sdk::viewport::MeshData) -> Self {
        crate::viewport::MeshData {
            id: sdk_mesh.id,
            vertices: sdk_mesh.vertices,
            normals: sdk_mesh.normals,
            uvs: sdk_mesh.uvs,
            indices: sdk_mesh.indices,
            vertex_colors: sdk_mesh.vertex_colors,
            material_id: sdk_mesh.material_id,
            transform: sdk_mesh.transform,
        }
    }
}

/// Convert Core MeshData to SDK MeshData
impl From<crate::viewport::MeshData> for nodle_plugin_sdk::viewport::MeshData {
    fn from(core_mesh: crate::viewport::MeshData) -> Self {
        nodle_plugin_sdk::viewport::MeshData {
            id: core_mesh.id,
            vertices: core_mesh.vertices,
            normals: core_mesh.normals,
            uvs: core_mesh.uvs,
            indices: core_mesh.indices,
            vertex_colors: core_mesh.vertex_colors,
            material_id: core_mesh.material_id,
            transform: core_mesh.transform,
        }
    }
}

/// Convert SDK MaterialData to Core MaterialData
impl From<nodle_plugin_sdk::viewport::MaterialData> for crate::viewport::MaterialData {
    fn from(sdk_material: nodle_plugin_sdk::viewport::MaterialData) -> Self {
        crate::viewport::MaterialData {
            id: sdk_material.id,
            name: sdk_material.name,
            base_color: sdk_material.base_color,
            metallic: sdk_material.metallic,
            roughness: sdk_material.roughness,
            emission: sdk_material.emission,
            diffuse_texture: sdk_material.diffuse_texture,
            normal_texture: sdk_material.normal_texture,
            roughness_texture: sdk_material.roughness_texture,
            metallic_texture: sdk_material.metallic_texture,
        }
    }
}

/// Convert Core MaterialData to SDK MaterialData
impl From<crate::viewport::MaterialData> for nodle_plugin_sdk::viewport::MaterialData {
    fn from(core_material: crate::viewport::MaterialData) -> Self {
        nodle_plugin_sdk::viewport::MaterialData {
            id: core_material.id,
            name: core_material.name,
            base_color: core_material.base_color,
            metallic: core_material.metallic,
            roughness: core_material.roughness,
            emission: core_material.emission,
            diffuse_texture: core_material.diffuse_texture,
            normal_texture: core_material.normal_texture,
            roughness_texture: core_material.roughness_texture,
            metallic_texture: core_material.metallic_texture,
        }
    }
}

/// Convert SDK LightData to Core LightData
impl From<nodle_plugin_sdk::viewport::LightData> for crate::viewport::LightData {
    fn from(sdk_light: nodle_plugin_sdk::viewport::LightData) -> Self {
        crate::viewport::LightData {
            id: sdk_light.id,
            light_type: sdk_light.light_type.into(),
            position: sdk_light.position,
            direction: sdk_light.direction,
            color: sdk_light.color,
            intensity: sdk_light.intensity,
            range: sdk_light.range,
            spot_angle: sdk_light.spot_angle,
        }
    }
}

/// Convert Core LightData to SDK LightData
impl From<crate::viewport::LightData> for nodle_plugin_sdk::viewport::LightData {
    fn from(core_light: crate::viewport::LightData) -> Self {
        nodle_plugin_sdk::viewport::LightData {
            id: core_light.id,
            light_type: core_light.light_type.into(),
            position: core_light.position,
            direction: core_light.direction,
            color: core_light.color,
            intensity: core_light.intensity,
            range: core_light.range,
            spot_angle: core_light.spot_angle,
        }
    }
}

/// Convert SDK LightType to Core LightType
impl From<nodle_plugin_sdk::viewport::LightType> for crate::viewport::LightType {
    fn from(sdk_light_type: nodle_plugin_sdk::viewport::LightType) -> Self {
        match sdk_light_type {
            nodle_plugin_sdk::viewport::LightType::Directional => crate::viewport::LightType::Directional,
            nodle_plugin_sdk::viewport::LightType::Point => crate::viewport::LightType::Point,
            nodle_plugin_sdk::viewport::LightType::Spot => crate::viewport::LightType::Spot,
            nodle_plugin_sdk::viewport::LightType::Area => crate::viewport::LightType::Area,
        }
    }
}

/// Convert Core LightType to SDK LightType
impl From<crate::viewport::LightType> for nodle_plugin_sdk::viewport::LightType {
    fn from(core_light_type: crate::viewport::LightType) -> Self {
        match core_light_type {
            crate::viewport::LightType::Directional => nodle_plugin_sdk::viewport::LightType::Directional,
            crate::viewport::LightType::Point => nodle_plugin_sdk::viewport::LightType::Point,
            crate::viewport::LightType::Spot => nodle_plugin_sdk::viewport::LightType::Spot,
            crate::viewport::LightType::Area => nodle_plugin_sdk::viewport::LightType::Area,
        }
    }
}

/// Convert SDK ViewportSettings to Core ViewportSettings
impl From<nodle_plugin_sdk::viewport::ViewportSettings> for crate::viewport::ViewportSettings {
    fn from(sdk_settings: nodle_plugin_sdk::viewport::ViewportSettings) -> Self {
        crate::viewport::ViewportSettings {
            background_color: sdk_settings.background_color,
            wireframe: sdk_settings.wireframe,
            lighting: sdk_settings.lighting,
            show_grid: sdk_settings.show_grid,
            show_ground_plane: sdk_settings.show_ground_plane,
            aa_samples: sdk_settings.aa_samples,
            shading_mode: sdk_settings.shading_mode.into(),
        }
    }
}

/// Convert Core ViewportSettings to SDK ViewportSettings
impl From<crate::viewport::ViewportSettings> for nodle_plugin_sdk::viewport::ViewportSettings {
    fn from(core_settings: crate::viewport::ViewportSettings) -> Self {
        nodle_plugin_sdk::viewport::ViewportSettings {
            background_color: core_settings.background_color,
            wireframe: core_settings.wireframe,
            lighting: core_settings.lighting,
            show_grid: core_settings.show_grid,
            show_ground_plane: core_settings.show_ground_plane,
            aa_samples: core_settings.aa_samples,
            shading_mode: core_settings.shading_mode.into(),
        }
    }
}

/// Convert SDK ShadingMode to Core ShadingMode
impl From<nodle_plugin_sdk::viewport::ShadingMode> for crate::viewport::ShadingMode {
    fn from(sdk_mode: nodle_plugin_sdk::viewport::ShadingMode) -> Self {
        match sdk_mode {
            nodle_plugin_sdk::viewport::ShadingMode::Wireframe => crate::viewport::ShadingMode::Wireframe,
            nodle_plugin_sdk::viewport::ShadingMode::Flat => crate::viewport::ShadingMode::Flat,
            nodle_plugin_sdk::viewport::ShadingMode::Smooth => crate::viewport::ShadingMode::Smooth,
            nodle_plugin_sdk::viewport::ShadingMode::Textured => crate::viewport::ShadingMode::Textured,
        }
    }
}

/// Convert Core ShadingMode to SDK ShadingMode
impl From<crate::viewport::ShadingMode> for nodle_plugin_sdk::viewport::ShadingMode {
    fn from(core_mode: crate::viewport::ShadingMode) -> Self {
        match core_mode {
            crate::viewport::ShadingMode::Wireframe => nodle_plugin_sdk::viewport::ShadingMode::Wireframe,
            crate::viewport::ShadingMode::Flat => nodle_plugin_sdk::viewport::ShadingMode::Flat,
            crate::viewport::ShadingMode::Smooth => nodle_plugin_sdk::viewport::ShadingMode::Smooth,
            crate::viewport::ShadingMode::Textured => nodle_plugin_sdk::viewport::ShadingMode::Textured,
        }
    }
}

/// Convert SDK CameraManipulation to Core CameraManipulation
impl From<nodle_plugin_sdk::viewport::CameraManipulation> for crate::viewport::CameraManipulation {
    fn from(sdk_manip: nodle_plugin_sdk::viewport::CameraManipulation) -> Self {
        match sdk_manip {
            nodle_plugin_sdk::viewport::CameraManipulation::Orbit { delta_x, delta_y } => {
                crate::viewport::CameraManipulation::Orbit { delta_x, delta_y }
            }
            nodle_plugin_sdk::viewport::CameraManipulation::Pan { delta_x, delta_y } => {
                crate::viewport::CameraManipulation::Pan { delta_x, delta_y }
            }
            nodle_plugin_sdk::viewport::CameraManipulation::Zoom { delta } => {
                crate::viewport::CameraManipulation::Zoom { delta }
            }
            nodle_plugin_sdk::viewport::CameraManipulation::Reset => {
                crate::viewport::CameraManipulation::Reset
            }
            nodle_plugin_sdk::viewport::CameraManipulation::SetPosition { position, target } => {
                crate::viewport::CameraManipulation::SetPosition { position, target }
            }
        }
    }
}

/// Convert Core CameraManipulation to SDK CameraManipulation
impl From<crate::viewport::CameraManipulation> for nodle_plugin_sdk::viewport::CameraManipulation {
    fn from(core_manip: crate::viewport::CameraManipulation) -> Self {
        match core_manip {
            crate::viewport::CameraManipulation::Orbit { delta_x, delta_y } => {
                nodle_plugin_sdk::viewport::CameraManipulation::Orbit { delta_x, delta_y }
            }
            crate::viewport::CameraManipulation::Pan { delta_x, delta_y } => {
                nodle_plugin_sdk::viewport::CameraManipulation::Pan { delta_x, delta_y }
            }
            crate::viewport::CameraManipulation::Zoom { delta } => {
                nodle_plugin_sdk::viewport::CameraManipulation::Zoom { delta }
            }
            crate::viewport::CameraManipulation::Reset => {
                nodle_plugin_sdk::viewport::CameraManipulation::Reset
            }
            crate::viewport::CameraManipulation::SetPosition { position, target } => {
                nodle_plugin_sdk::viewport::CameraManipulation::SetPosition { position, target }
            }
        }
    }
}