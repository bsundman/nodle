//! USD Viewport properties and UI controls

use egui::{Ui, Color32};
use crate::nodes::Node;

/// Viewport display properties and settings
#[derive(Debug, Clone)]
pub struct ViewportProperties {
    pub background_color: [f32; 4],
    pub enable_wireframe: bool,
    pub enable_lighting: bool,
    pub enable_grid: bool,
    pub enable_axis_gizmo: bool,
    pub grid_size: f32,
    pub grid_spacing: f32,
    pub samples: i32,
    pub max_samples: i32,
    pub shading_mode: ShadingMode,
    pub camera_mode: CameraMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShadingMode {
    Wireframe,
    Flat,
    Smooth,
    Textured,
    MaterialPreview,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraMode {
    Perspective,
    Orthographic,
}

impl Default for ViewportProperties {
    fn default() -> Self {
        Self {
            background_color: [0.2, 0.2, 0.2, 1.0],
            enable_wireframe: false,
            enable_lighting: true,
            enable_grid: true,
            enable_axis_gizmo: true,
            grid_size: 10.0,
            grid_spacing: 1.0,
            samples: 4,
            max_samples: 16,
            shading_mode: ShadingMode::Smooth,
            camera_mode: CameraMode::Perspective,
        }
    }
}

impl ViewportProperties {
    /// Build the viewport properties UI
    pub fn build_properties_ui(&mut self, ui: &mut Ui, _node: &mut Node) {
        ui.heading("Viewport Properties");
        ui.separator();

        // Display Settings
        ui.collapsing("Display", |ui| {
            ui.horizontal(|ui| {
                ui.label("Background:");
                let mut bg = Color32::from_rgba_premultiplied(
                    (self.background_color[0] * 255.0) as u8,
                    (self.background_color[1] * 255.0) as u8,
                    (self.background_color[2] * 255.0) as u8,
                    (self.background_color[3] * 255.0) as u8,
                );
                if ui.color_edit_button_srgba(&mut bg).changed() {
                    let rgba = bg.to_array();
                    self.background_color = [
                        rgba[0] as f32 / 255.0,
                        rgba[1] as f32 / 255.0,
                        rgba[2] as f32 / 255.0,
                        rgba[3] as f32 / 255.0,
                    ];
                }
            });

            ui.checkbox(&mut self.enable_wireframe, "Wireframe");
            ui.checkbox(&mut self.enable_lighting, "Lighting");
            ui.checkbox(&mut self.enable_grid, "Grid");
            ui.checkbox(&mut self.enable_axis_gizmo, "Axis Gizmo");

            if self.enable_grid {
                ui.add(egui::Slider::new(&mut self.grid_size, 1.0..=100.0).text("Grid Size"));
                ui.add(egui::Slider::new(&mut self.grid_spacing, 0.1..=10.0).text("Grid Spacing"));
            }
        });

        // Shading Settings
        ui.collapsing("Shading", |ui| {
            ui.label("Shading Mode:");
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", self.shading_mode))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.shading_mode, ShadingMode::Wireframe, "Wireframe");
                    ui.selectable_value(&mut self.shading_mode, ShadingMode::Flat, "Flat");
                    ui.selectable_value(&mut self.shading_mode, ShadingMode::Smooth, "Smooth");
                    ui.selectable_value(&mut self.shading_mode, ShadingMode::Textured, "Textured");
                    ui.selectable_value(&mut self.shading_mode, ShadingMode::MaterialPreview, "Material Preview");
                });
        });

        // Camera Settings
        ui.collapsing("Camera", |ui| {
            ui.label("Camera Mode:");
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", self.camera_mode))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.camera_mode, CameraMode::Perspective, "Perspective");
                    ui.selectable_value(&mut self.camera_mode, CameraMode::Orthographic, "Orthographic");
                });
        });

        // Render Settings
        ui.collapsing("Rendering", |ui| {
            ui.add(egui::Slider::new(&mut self.samples, 1..=self.max_samples).text("Anti-aliasing Samples"));
            
            ui.separator();
            ui.label("Quick Presets:");
            ui.horizontal(|ui| {
                if ui.button("Performance").clicked() {
                    self.samples = 1;
                    self.enable_lighting = false;
                }
                if ui.button("Balanced").clicked() {
                    self.samples = 4;
                    self.enable_lighting = true;
                }
                if ui.button("Quality").clicked() {
                    self.samples = 8;
                    self.enable_lighting = true;
                }
            });
        });

        ui.separator();
        
        // Viewport Controls
        ui.label("Navigation:");
        ui.horizontal(|ui| {
            if ui.button("Reset View").clicked() {
                // Reset camera to default - this would trigger a callback
            }
            if ui.button("Fit All").clicked() {
                // Fit all geometry in view - this would trigger a callback
            }
        });
    }
}