//! Core viewport data types for 3D scene rendering
//! 
//! These are the core's own viewport types, independent of the plugin SDK.
//! Conversion functions in plugin_interface.rs handle SDK interop.

use serde::{Deserialize, Serialize};

/// 3D camera state data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraData {
    /// Camera position in world space
    pub position: [f32; 3],
    /// Camera look-at target in world space
    pub target: [f32; 3],
    /// Camera up vector
    pub up: [f32; 3],
    /// Field of view in radians
    pub fov: f32,
    /// Near clipping plane distance
    pub near: f32,
    /// Far clipping plane distance
    pub far: f32,
    /// Camera aspect ratio
    pub aspect: f32,
}

impl Default for CameraData {
    fn default() -> Self {
        Self {
            position: [5.0, 5.0, 5.0],
            target: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            fov: 45.0_f32.to_radians(),
            near: 0.1,
            far: 100.0,
            aspect: 1.0,
        }
    }
}

/// 3D mesh geometry data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshData {
    /// Unique identifier for this mesh
    pub id: String,
    /// Vertex positions (x,y,z triplets)
    pub vertices: Vec<f32>,
    /// Vertex normals (x,y,z triplets)
    pub normals: Vec<f32>,
    /// Texture coordinates (u,v pairs)
    pub uvs: Vec<f32>,
    /// Triangle indices
    pub indices: Vec<u32>,
    /// Material ID for this mesh
    pub material_id: Option<String>,
    /// Transformation matrix for this mesh
    pub transform: [[f32; 4]; 4],
}

/// Material data for 3D rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialData {
    /// Unique identifier for this material
    pub id: String,
    /// Material name
    pub name: String,
    /// Base color (RGBA)
    pub base_color: [f32; 4],
    /// Metallic factor (0.0 - 1.0)
    pub metallic: f32,
    /// Roughness factor (0.0 - 1.0)
    pub roughness: f32,
    /// Emission color (RGB)
    pub emission: [f32; 3],
    /// Texture paths
    pub diffuse_texture: Option<String>,
    pub normal_texture: Option<String>,
    pub roughness_texture: Option<String>,
    pub metallic_texture: Option<String>,
}

/// Light data for 3D scene
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightData {
    /// Unique identifier for this light
    pub id: String,
    /// Light type
    pub light_type: LightType,
    /// Light position (for point/spot lights)
    pub position: [f32; 3],
    /// Light direction (for directional/spot lights)
    pub direction: [f32; 3],
    /// Light color (RGB)
    pub color: [f32; 3],
    /// Light intensity
    pub intensity: f32,
    /// Light range (for point/spot lights)
    pub range: f32,
    /// Spot light cone angle in radians
    pub spot_angle: f32,
}

/// Types of lights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LightType {
    Directional,
    Point,
    Spot,
    Area,
}

/// Complete 3D scene data for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneData {
    /// Scene name/identifier
    pub name: String,
    /// All meshes in the scene
    pub meshes: Vec<MeshData>,
    /// All materials in the scene
    pub materials: Vec<MaterialData>,
    /// All lights in the scene
    pub lights: Vec<LightData>,
    /// Current camera state
    pub camera: CameraData,
    /// Scene bounding box (min, max)
    pub bounding_box: Option<([f32; 3], [f32; 3])>,
}

impl Default for SceneData {
    fn default() -> Self {
        Self {
            name: "Empty Scene".to_string(),
            meshes: Vec::new(),
            materials: Vec::new(),
            lights: Vec::new(),
            camera: CameraData::default(),
            bounding_box: None,
        }
    }
}

/// Viewport rendering settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportSettings {
    /// Viewport background color (RGBA)
    pub background_color: [f32; 4],
    /// Enable wireframe rendering
    pub wireframe: bool,
    /// Enable lighting
    pub lighting: bool,
    /// Enable grid display
    pub show_grid: bool,
    /// Enable ground plane
    pub show_ground_plane: bool,
    /// Anti-aliasing samples
    pub aa_samples: u32,
    /// Shading mode
    pub shading_mode: ShadingMode,
}

/// Shading modes for viewport rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShadingMode {
    Wireframe,
    Flat,
    Smooth,
    Textured,
}

impl Default for ViewportSettings {
    fn default() -> Self {
        Self {
            background_color: [0.2, 0.2, 0.2, 1.0],
            wireframe: false,
            lighting: true,
            show_grid: true,
            show_ground_plane: true,
            aa_samples: 4,
            shading_mode: ShadingMode::Smooth,
        }
    }
}

/// Complete viewport data for rendering
#[derive(Debug, Clone)]
pub struct ViewportData {
    /// The 3D scene to render
    pub scene: SceneData,
    /// Viewport rendering settings
    pub settings: ViewportSettings,
    /// Viewport dimensions
    pub dimensions: (u32, u32),
    /// Whether scene data has been updated since last render
    pub scene_dirty: bool,
    /// Whether settings have been updated since last render
    pub settings_dirty: bool,
}

impl Default for ViewportData {
    fn default() -> Self {
        Self {
            scene: SceneData::default(),
            settings: ViewportSettings::default(),
            dimensions: (800, 600),
            scene_dirty: true,
            settings_dirty: true,
        }
    }
}

/// Camera manipulation actions
#[derive(Debug, Clone)]
pub enum CameraManipulation {
    /// Orbit camera around target
    Orbit { delta_x: f32, delta_y: f32 },
    /// Pan camera and target
    Pan { delta_x: f32, delta_y: f32 },
    /// Zoom camera towards/away from target
    Zoom { delta: f32 },
    /// Reset camera to default position
    Reset,
    /// Set camera to specific position and target
    SetPosition { position: [f32; 3], target: [f32; 3] },
}