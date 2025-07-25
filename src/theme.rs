//! Centralized theme and styling constants for the Nōdle editor
//!
//! This module provides a single source of truth for all colors, dimensions,
//! and styling values used throughout the application.

use egui::{Color32, Vec2};

/// Color palette for the Nōdle editor
pub struct Colors {
    // Selection and highlighting
    pub selection_blue: Color32,
    pub hover_highlight: Color32,
    
    // Node colors
    pub node_bg_top: Color32,
    pub node_bg_bottom: Color32,
    pub node_border: Color32,
    pub node_bevel_light: Color32,
    pub node_bevel_dark: Color32,
    
    // Port colors
    pub port_input: Color32,
    pub port_output: Color32,
    pub port_border: Color32,
    
    // Background colors
    pub main_background: Color32,
    pub panel_background: Color32,
    
    // Connection colors
    pub connection_default: Color32,
    pub connection_hover: Color32,
    pub connection_selected: Color32,
}

impl Colors {
    /// Get the default color palette
    pub fn default() -> Self {
        Self {
            // Selection and highlighting
            selection_blue: Color32::from_rgb(100, 150, 255),
            hover_highlight: Color32::from_rgb(120, 170, 255),
            
            // Node colors
            node_bg_top: Color32::from_rgb(127, 127, 127),
            node_bg_bottom: Color32::from_rgb(64, 64, 64),
            node_border: Color32::from_rgb(64, 64, 64),
            node_bevel_light: Color32::from_rgb(166, 166, 166),
            node_bevel_dark: Color32::from_rgb(38, 38, 38),
            
            // Port colors
            port_input: Color32::from_rgb(70, 120, 90),
            port_output: Color32::from_rgb(120, 70, 70),
            port_border: Color32::from_rgb(0, 0, 0),
            
            // Background colors
            main_background: Color32::from_rgb(28, 28, 28),
            panel_background: Color32::from_rgb(22, 27, 34),
            
            // Connection colors
            connection_default: Color32::from_rgb(200, 200, 200),
            connection_hover: Color32::from_rgb(255, 255, 255),
            connection_selected: Color32::from_rgb(100, 150, 255),
        }
    }
}

/// Dimension constants for the Nōdle editor
pub struct Dimensions {
    // Node sizes
    pub default_node_size: Vec2,
    pub workspace_node_size: Vec2,
    
    // UI element sizes
    pub port_radius: f32,
    pub button_radius: f32,
    pub corner_radius: f32,
    pub border_width: f32,
    
    // Layout dimensions
    pub menu_bar_height: f32,
    pub port_spacing: f32,
    
    // Interaction radii
    pub hover_radius: f32,
    pub click_radius_precise: f32,
    pub click_radius_connecting: f32,
}

impl Dimensions {
    /// Get the default dimensions
    pub fn default() -> Self {
        Self {
            // Node sizes
            default_node_size: Vec2::new(150.0, 30.0),
            workspace_node_size: Vec2::new(180.0, 50.0),
            
            // UI element sizes
            port_radius: 5.0,
            button_radius: 10.0,
            corner_radius: 5.0,
            border_width: 1.0,
            
            // Layout dimensions
            menu_bar_height: 34.0,
            port_spacing: 30.0,
            
            // Interaction radii
            hover_radius: 10.0,
            click_radius_precise: 8.0,
            click_radius_connecting: 80.0,
        }
    }
}

/// Animation and timing constants
pub struct Animation {
    // Zoom factors
    pub zoom_normal: f32,
    pub zoom_fast: f32,
    pub zoom_very_fast: f32,
    
    // Pan step values
    pub pan_step_normal: f32,
    pub pan_step_fast: f32,
    
    // Control offsets for bezier curves
    pub bezier_control_offset: f32,
}

impl Animation {
    /// Get the default animation values
    pub fn default() -> Self {
        Self {
            zoom_normal: 1.0,
            zoom_fast: 2.0,
            zoom_very_fast: 4.0,
            
            pan_step_normal: 8.0,
            pan_step_fast: 10.0,
            
            bezier_control_offset: 4.0,
        }
    }
}

/// Complete theme containing all styling constants
pub struct Theme {
    pub colors: Colors,
    pub dimensions: Dimensions,
    pub animation: Animation,
}

impl Theme {
    /// Get the default theme
    pub fn default() -> Self {
        Self {
            colors: Colors::default(),
            dimensions: Dimensions::default(),
            animation: Animation::default(),
        }
    }
}

/// Global theme instance
static GLOBAL_THEME: std::sync::LazyLock<Theme> = std::sync::LazyLock::new(|| Theme::default());

/// Get the global theme
pub fn theme() -> &'static Theme {
    &GLOBAL_THEME
}

/// Convenience functions for commonly used values
pub fn colors() -> &'static Colors {
    &theme().colors
}

pub fn dimensions() -> &'static Dimensions {
    &theme().dimensions
}

pub fn animation() -> &'static Animation {
    &theme().animation
}