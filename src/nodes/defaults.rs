//! Default parameter values for nodes
//!
//! This module centralizes default values, ranges, and step sizes for
//! various types of node parameters to ensure consistency across all nodes.

/// Default parameter ranges and values for geometry nodes
pub struct GeometryDefaults;

impl GeometryDefaults {
    // Size parameters
    pub const SIZE_MIN: f32 = 0.001;
    pub const SIZE_MAX: f32 = 100.0;
    pub const SIZE_DEFAULT: f32 = 1.0;
    pub const SIZE_STEP: f32 = 0.1;
    
    // Position parameters
    pub const POSITION_MIN: f32 = -100.0;
    pub const POSITION_MAX: f32 = 100.0;
    pub const POSITION_DEFAULT: f32 = 0.0;
    pub const POSITION_STEP: f32 = 0.1;
    
    // Subdivision parameters (integer)
    pub const SUBDIVISIONS_MIN: i32 = 1;
    pub const SUBDIVISIONS_MAX_CUBE: i32 = 20;
    pub const SUBDIVISIONS_MAX_SPHERE: i32 = 128;
    pub const SUBDIVISIONS_DEFAULT: i32 = 1;
    
    // Color parameters
    pub const COLOR_MIN: f32 = 0.0;
    pub const COLOR_MAX: f32 = 1.0;
    pub const COLOR_DEFAULT: f32 = 0.5;
    pub const COLOR_STEP: f32 = 0.01;
    
    // Rotation parameters (in degrees)
    pub const ROTATION_MIN: f32 = -360.0;
    pub const ROTATION_MAX: f32 = 360.0;
    pub const ROTATION_DEFAULT: f32 = 0.0;
    pub const ROTATION_STEP: f32 = 1.0;
}

/// Default parameter ranges and values for lighting nodes
pub struct LightingDefaults;

impl LightingDefaults {
    // Intensity parameters
    pub const INTENSITY_MIN: f32 = 0.0;
    pub const INTENSITY_MAX: f32 = 100.0;
    pub const INTENSITY_DEFAULT: f32 = 1.0;
    pub const INTENSITY_STEP: f32 = 0.1;
    
    // Color temperature parameters
    pub const TEMP_MIN: f32 = 1000.0;
    pub const TEMP_MAX: f32 = 12000.0;
    pub const TEMP_DEFAULT: f32 = 6500.0;
    pub const TEMP_STEP: f32 = 100.0;
    
    // Attenuation parameters
    pub const ATTENUATION_MIN: f32 = 0.01;
    pub const ATTENUATION_MAX: f32 = 100.0;
    pub const ATTENUATION_DEFAULT: f32 = 1.0;
    pub const ATTENUATION_STEP: f32 = 0.01;
    
    // Angle parameters (for spot lights)
    pub const ANGLE_MIN: f32 = 0.0;
    pub const ANGLE_MAX: f32 = 180.0;
    pub const ANGLE_DEFAULT: f32 = 45.0;
    pub const ANGLE_STEP: f32 = 1.0;
    
    // Position parameters (same as geometry)
    pub const POSITION_MIN: f32 = GeometryDefaults::POSITION_MIN;
    pub const POSITION_MAX: f32 = GeometryDefaults::POSITION_MAX;
    pub const POSITION_DEFAULT: f32 = GeometryDefaults::POSITION_DEFAULT;
    pub const POSITION_STEP: f32 = GeometryDefaults::POSITION_STEP;
}

/// Default parameter ranges for USD-specific parameters
pub struct USDDefaults;

impl USDDefaults {
    // Stage parameters
    pub const STAGE_PATH_DEFAULT: &'static str = "/path/to/stage.usd";
    
    // Prim path parameters
    pub const PRIM_PATH_DEFAULT: &'static str = "/World";
    
    // Attribute parameters
    pub const ATTR_NAME_DEFAULT: &'static str = "points";
    
    // File extensions for USD files
    pub const USD_EXTENSIONS: &'static [&'static str] = &["usd", "usda", "usdc", "usdz"];
}

/// Default font sizes for UI elements
pub struct FontDefaults;

impl FontDefaults {
    pub const NODE_TITLE_SIZE: f32 = 12.0;
    pub const PORT_NAME_SIZE: f32 = 10.0;
    pub const MENU_SIZE: f32 = 14.0;
    pub const TOOLTIP_SIZE: f32 = 11.0;
}

/// Helper functions for creating common parameter ranges
pub mod ranges {
    use super::*;
    
    /// Create a standard size range (min, max, default, step)
    pub fn size_range() -> (f32, f32, f32, f32) {
        (
            GeometryDefaults::SIZE_MIN,
            GeometryDefaults::SIZE_MAX,
            GeometryDefaults::SIZE_DEFAULT,
            GeometryDefaults::SIZE_STEP,
        )
    }
    
    /// Create a standard position range (min, max, default, step)
    pub fn position_range() -> (f32, f32, f32, f32) {
        (
            GeometryDefaults::POSITION_MIN,
            GeometryDefaults::POSITION_MAX,
            GeometryDefaults::POSITION_DEFAULT,
            GeometryDefaults::POSITION_STEP,
        )
    }
    
    /// Create a standard color range (min, max, default, step)
    pub fn color_range() -> (f32, f32, f32, f32) {
        (
            GeometryDefaults::COLOR_MIN,
            GeometryDefaults::COLOR_MAX,
            GeometryDefaults::COLOR_DEFAULT,
            GeometryDefaults::COLOR_STEP,
        )
    }
    
    /// Create a standard intensity range (min, max, default, step)
    pub fn intensity_range() -> (f32, f32, f32, f32) {
        (
            LightingDefaults::INTENSITY_MIN,
            LightingDefaults::INTENSITY_MAX,
            LightingDefaults::INTENSITY_DEFAULT,
            LightingDefaults::INTENSITY_STEP,
        )
    }
    
    /// Create a subdivision range for cubes (min, max, default)
    pub fn cube_subdivision_range() -> (i32, i32, i32) {
        (
            GeometryDefaults::SUBDIVISIONS_MIN,
            GeometryDefaults::SUBDIVISIONS_MAX_CUBE,
            GeometryDefaults::SUBDIVISIONS_DEFAULT,
        )
    }
    
    /// Create a subdivision range for spheres (min, max, default)
    pub fn sphere_subdivision_range() -> (i32, i32, i32) {
        (
            GeometryDefaults::SUBDIVISIONS_MIN,
            GeometryDefaults::SUBDIVISIONS_MAX_SPHERE,
            GeometryDefaults::SUBDIVISIONS_DEFAULT,
        )
    }
}