//! Viewport node interface implementation - Pattern A

use egui::Ui;
use crate::nodes::interface::{ParameterChange, NodeData};
use crate::gpu::{ViewportRenderCallback, ShadingMode};
use glam::Vec3;
use std::sync::{Arc, Mutex};

/// Wrapper for ViewportLogic that implements Pattern A interface
#[derive(Clone)]
pub struct ViewportNode {
    pub viewport: super::logic::ViewportLogic,
    alt_at_drag_start: bool,
    orbit_pivot: Option<Vec3>,
}

impl Default for ViewportNode {
    fn default() -> Self {
        Self {
            viewport: super::logic::ViewportLogic::default(),
            alt_at_drag_start: false,
            orbit_pivot: None,
        }
    }
}

impl ViewportNode {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.viewport.resize_viewport(width, height);
        self
    }
    
    pub fn get_viewport(&self) -> &super::logic::ViewportLogic {
        &self.viewport
    }
    
    pub fn get_viewport_mut(&mut self) -> &mut super::logic::ViewportLogic {
        &mut self.viewport
    }
    
    /// Load USD scene from stage ID (called when receiving USD data)
    pub fn load_usd_scene(&mut self, stage_id: &str) {
        // Extract file path from stage ID
        let file_path = if stage_id.starts_with("file://") {
            stage_id.strip_prefix("file://").unwrap_or(stage_id)
        } else {
            stage_id
        };
        
        println!("Viewport: Loading USD file: {}", file_path);
        
        // Update the current stage in viewport logic
        self.viewport.current_stage = Some(stage_id.to_string());
        
        // Load the stage in the USD renderer
        if let Err(e) = self.viewport.usd_renderer.load_stage(stage_id) {
            eprintln!("Viewport: Failed to load USD stage: {}", e);
        } else {
            println!("Viewport: Successfully loaded USD stage");
        }
    }
    
    /// Load USD file directly from file path (for manual execution)
    pub fn load_usd_file(&mut self, file_path: &str) {
        if file_path.is_empty() {
            return;
        }
        
        if !std::path::Path::new(file_path).exists() {
            eprintln!("Viewport: USD file not found: {}", file_path);
            return;
        }
        
        println!("Viewport: Loading USD file directly: {}", file_path);
        
        // Create stage ID from file path
        let stage_id = format!("file://{}", file_path);
        self.load_usd_scene(&stage_id);
    }
    
    /// Open a file dialog to select USD files
    fn open_usd_file_dialog(&self) -> Result<Option<String>, String> {
        use rfd::FileDialog;
        
        if let Some(path) = FileDialog::new()
            .add_filter("USD Files", &["usd", "usda", "usdc", "usdz"])
            .add_filter("All Files", &["*"])
            .set_title("Select USD File for Viewport")
            .pick_file()
        {
            if let Some(path_str) = path.to_str() {
                Ok(Some(path_str.to_string()))
            } else {
                Err("Invalid file path encoding".to_string())
            }
        } else {
            Ok(None) // User cancelled dialog
        }
    }
}

/// Pattern A interface implementation for ViewportNode
pub fn build_interface(node: &mut ViewportNode, ui: &mut Ui) -> Vec<ParameterChange> {
    let mut changes = Vec::new();
    
    // Main 3D Viewport Area
    ui.heading("3D Viewport");
    
    // Calculate how much space we want for the viewport
    let available_rect = ui.available_rect_before_wrap();
    let available_width = available_rect.width();
    
    // Estimate space for collapsed controls
    let collapsed_controls_height = 100.0;
    
    // Give most of the space to viewport, but leave room for controls
    let viewport_height = (available_rect.height() - collapsed_controls_height).max(100.0);
    
    let viewport_size = egui::vec2(
        available_width.max(1.0),
        viewport_height
    );
    
    let (viewport_rect, viewport_response) = ui.allocate_exact_size(
        viewport_size, 
        egui::Sense::click_and_drag()
    );
    
    // Update viewport dimensions
    let new_width = viewport_size.x as i32;
    let new_height = viewport_size.y as i32;
    
    let width_diff = (node.viewport.viewport_width - new_width).abs();
    let height_diff = (node.viewport.viewport_height - new_height).abs();
    
    node.viewport.viewport_width = new_width;
    node.viewport.viewport_height = new_height;
    
    if width_diff > 10 || height_diff > 10 {
        node.viewport.resize_viewport(new_width as u32, new_height as u32);
        changes.push(ParameterChange {
            parameter: "viewport_size".to_string(),
            value: NodeData::String(format!("{}x{}", new_width, new_height)),
        });
    }
    
    // Get mouse position relative to viewport
    let mouse_pos = ui.input(|i| i.pointer.hover_pos());
    let mouse_in_viewport = if let Some(pos) = mouse_pos {
        let rel_x = (pos.x - viewport_rect.min.x) / viewport_rect.width();
        let rel_y = (pos.y - viewport_rect.min.y) / viewport_rect.height();
        Some((rel_x, rel_y))
    } else {
        None
    };
    
    // Handle Maya-style viewport navigation
    if viewport_response.hovered() {
        let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
        if scroll_delta != 0.0 {
            if let Some((mouse_x, mouse_y)) = mouse_in_viewport {
                node.viewport.zoom_camera_to_mouse(scroll_delta * 0.002, mouse_x, mouse_y);
            } else {
                node.viewport.zoom_camera(scroll_delta * 0.002);
            }
            changes.push(ParameterChange {
                parameter: "camera_zoom".to_string(),
                value: NodeData::String("zoom_changed".to_string()),
            });
        }
    }
    
    // Maya-style navigation with Alt/Cmd key
    if viewport_response.dragged() {
        let drag_delta = viewport_response.drag_delta();
        let ctx = ui.ctx();
        
        // Capture Alt state at drag start
        if viewport_response.drag_started() {
            let alt_held = ctx.input(|i| i.modifiers.alt) || ui.input(|i| i.modifiers.alt);
            let cmd_held = ctx.input(|i| i.modifiers.command) || ui.input(|i| i.modifiers.command);
            
            node.alt_at_drag_start = alt_held || cmd_held;
            
            if node.alt_at_drag_start && viewport_response.dragged_by(egui::PointerButton::Primary) {
                if let Some((mouse_x, mouse_y)) = mouse_in_viewport {
                    node.orbit_pivot = Some(node.viewport.camera.find_orbit_pivot(mouse_x, mouse_y, &node.viewport.usd_renderer.current_scene.geometries));
                } else {
                    node.orbit_pivot = Some(node.viewport.camera.target);
                }
            }
        }
        
        let alt_held = node.alt_at_drag_start;
        let scale_factor = 2.0 / viewport_size.x.min(viewport_size.y);
        
        if alt_held && viewport_response.dragged_by(egui::PointerButton::Primary) {
            // Alt + LMB = Orbit
            if let Some(pivot_point) = node.orbit_pivot {
                node.viewport.camera.orbit_around_point(
                    pivot_point,
                    drag_delta.x * scale_factor, 
                    -drag_delta.y * scale_factor
                );
            } else {
                node.viewport.orbit_camera(
                    drag_delta.x * scale_factor, 
                    drag_delta.y * scale_factor
                );
            }
            changes.push(ParameterChange {
                parameter: "camera_orbit".to_string(),
                value: NodeData::String(format!("{}:{}", drag_delta.x, drag_delta.y)),
            });
        } else if alt_held && viewport_response.dragged_by(egui::PointerButton::Middle) {
            // Alt + MMB = Pan
            node.viewport.pan_camera(
                -drag_delta.x, 
                drag_delta.y,
                viewport_size.y
            );
            changes.push(ParameterChange {
                parameter: "camera_pan".to_string(),
                value: NodeData::String(format!("{}:{}", drag_delta.x, drag_delta.y)),
            });
        } else if alt_held && viewport_response.dragged_by(egui::PointerButton::Secondary) {
            // Alt + RMB = Zoom
            if let Some((mouse_x, mouse_y)) = mouse_in_viewport {
                node.viewport.zoom_camera_to_mouse(-drag_delta.y * scale_factor * 0.5, mouse_x, mouse_y);
            } else {
                node.viewport.zoom_camera(-drag_delta.y * scale_factor);
            }
            changes.push(ParameterChange {
                parameter: "camera_zoom".to_string(),
                value: NodeData::String("zoom_dragged".to_string()),
            });
        }
        
        // Reset on drag stop
        if viewport_response.drag_stopped() {
            node.alt_at_drag_start = false;
            node.orbit_pivot = None;
        }
    }

    // Draw the 3D viewport
    if ui.is_rect_visible(viewport_rect) {
        let painter = ui.painter();
        
        // Background
        painter.rect_filled(
            viewport_rect,
            egui::Rounding::same(4.0),
            egui::Color32::from_rgba_premultiplied(
                (node.viewport.background_color[0] * 255.0) as u8,
                (node.viewport.background_color[1] * 255.0) as u8,
                (node.viewport.background_color[2] * 255.0) as u8,
                (node.viewport.background_color[3] * 255.0) as u8,
            )
        );
        
        // Render 3D content
        let mut render_usd_renderer = node.viewport.usd_renderer.clone();
        render_usd_renderer.base_renderer.camera = node.viewport.camera.clone();
        
        let callback = ViewportRenderCallback::new(
            render_usd_renderer,
            viewport_rect,
            node.viewport.background_color,
            node.viewport.camera.clone(),
        );
        
        let callback = egui_wgpu::Callback::new_paint_callback(viewport_rect, callback);
        painter.add(callback);
        
        // Border
        painter.rect_stroke(
            viewport_rect,
            egui::Rounding::same(4.0),
            egui::Stroke::new(2.0, egui::Color32::from_gray(100))
        );
        
        // Overlay info
        let info_pos = viewport_rect.min + egui::vec2(10.0, 10.0);
        
        let stage_info = if let Some(stage_id) = &node.viewport.current_stage {
            format!("USD: {} | {}x{} | FOV: {:.1}°", 
                stage_id,
                node.viewport.viewport_width, 
                node.viewport.viewport_height, 
                node.viewport.camera.fov.to_degrees())
        } else {
            format!("No USD Stage | {}x{} | FOV: {:.1}°", 
                node.viewport.viewport_width, 
                node.viewport.viewport_height, 
                node.viewport.camera.fov.to_degrees())
        };
        
        painter.text(
            info_pos,
            egui::Align2::LEFT_TOP,
            stage_info,
            egui::FontId::monospace(12.0),
            egui::Color32::WHITE
        );
        
        // Maya navigation hint
        painter.text(
            viewport_rect.min + egui::vec2(10.0, viewport_rect.height() - 20.0),
            egui::Align2::LEFT_BOTTOM,
            "Alt+LMB: Orbit | Alt+MMB: Pan | Alt+RMB: Zoom | Scroll: Zoom",
            egui::FontId::monospace(9.0),
            egui::Color32::from_gray(160)
        );
    }
    
    ui.separator();
    
    // Controls in a scroll area
    egui::ScrollArea::vertical()
        .auto_shrink([false, true])
        .show(ui, |ui| {
            // Viewport Controls
            ui.collapsing("Viewport Controls", |ui| {
                // Shading mode
                ui.horizontal(|ui| {
                    ui.label("Shading:");
                    let current_mode = node.viewport.usd_renderer.render_settings.shading_mode.clone();
                    
                    if ui.selectable_label(matches!(current_mode, ShadingMode::Wireframe), "Wire").clicked() {
                        node.viewport.set_shading_mode(ShadingMode::Wireframe);
                        changes.push(ParameterChange {
                            parameter: "shading_mode".to_string(),
                            value: NodeData::String("wireframe".to_string()),
                        });
                    }
                    if ui.selectable_label(matches!(current_mode, ShadingMode::FlatShaded), "Flat").clicked() {
                        node.viewport.set_shading_mode(ShadingMode::FlatShaded);
                        changes.push(ParameterChange {
                            parameter: "shading_mode".to_string(),
                            value: NodeData::String("flat".to_string()),
                        });
                    }
                    if ui.selectable_label(matches!(current_mode, ShadingMode::SmoothShaded), "Smooth").clicked() {
                        node.viewport.set_shading_mode(ShadingMode::SmoothShaded);
                        changes.push(ParameterChange {
                            parameter: "shading_mode".to_string(),
                            value: NodeData::String("smooth".to_string()),
                        });
                    }
                    if ui.selectable_label(matches!(current_mode, ShadingMode::MaterialPreview), "Material").clicked() {
                        node.viewport.set_shading_mode(ShadingMode::MaterialPreview);
                        changes.push(ParameterChange {
                            parameter: "shading_mode".to_string(),
                            value: NodeData::String("material".to_string()),
                        });
                    }
                });
                
                // Camera controls
                ui.horizontal(|ui| {
                    if ui.button("Reset").clicked() {
                        node.viewport.reset_camera();
                        changes.push(ParameterChange {
                            parameter: "camera_reset".to_string(),
                            value: NodeData::Boolean(true),
                        });
                    }
                    if ui.button("Top").clicked() {
                        node.viewport.set_top_view();
                        changes.push(ParameterChange {
                            parameter: "camera_view".to_string(),
                            value: NodeData::String("top".to_string()),
                        });
                    }
                    if ui.button("Front").clicked() {
                        node.viewport.set_front_view();
                        changes.push(ParameterChange {
                            parameter: "camera_view".to_string(),
                            value: NodeData::String("front".to_string()),
                        });
                    }
                });
                
                // Background color
                ui.horizontal(|ui| {
                    ui.label("Background:");
                    let mut color = egui::Color32::from_rgba_premultiplied(
                        (node.viewport.background_color[0] * 255.0) as u8,
                        (node.viewport.background_color[1] * 255.0) as u8,
                        (node.viewport.background_color[2] * 255.0) as u8,
                        (node.viewport.background_color[3] * 255.0) as u8,
                    );
                    if ui.color_edit_button_srgba(&mut color).changed() {
                        let [r, g, b, a] = color.to_array();
                        node.viewport.background_color[0] = r as f32 / 255.0;
                        node.viewport.background_color[1] = g as f32 / 255.0;
                        node.viewport.background_color[2] = b as f32 / 255.0;
                        node.viewport.background_color[3] = a as f32 / 255.0;
                        changes.push(ParameterChange {
                            parameter: "background_color".to_string(),
                            value: NodeData::String(format!("{},{},{},{}", r, g, b, a)),
                        });
                    }
                });
            });
            
            // USD Scene
            ui.collapsing("USD Scene", |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Load Test Stage").clicked() {
                        node.viewport.load_test_stage();
                        changes.push(ParameterChange {
                            parameter: "load_test_stage".to_string(),
                            value: NodeData::Boolean(true),
                        });
                    }
                    
                    if ui.button("Clear Scene").clicked() {
                        node.viewport.current_stage = None;
                        node.viewport.usd_renderer = crate::gpu::USDRenderer::new();
                        changes.push(ParameterChange {
                            parameter: "clear_scene".to_string(),
                            value: NodeData::Boolean(true),
                        });
                    }
                });
                
                let scene = node.viewport.get_scene().clone();
                if !scene.geometries.is_empty() {
                    ui.label(format!("Geometry ({}):", scene.geometries.len()));
                    for geometry in &scene.geometries {
                        ui.label(&geometry.prim_path);
                    }
                }
                
                // Display current stage info
                if let Some(stage_id) = &node.viewport.current_stage {
                    ui.separator();
                    ui.label("Current Stage:");
                    ui.small(stage_id);
                }
            });
            
            // Render Stats
            ui.collapsing("Render Stats", |ui| {
                let scene = node.viewport.get_scene().clone();
                let total_triangles: usize = scene.geometries.iter().map(|g| g.indices.len() / 3).sum();
                
                ui.label(format!("Triangles: {}", total_triangles));
                ui.label(format!("Draw calls: {}", scene.geometries.len()));
                ui.label(format!("Shading: {:?}", node.viewport.usd_renderer.render_settings.shading_mode));
                
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut node.viewport.enable_lighting, "Lighting").changed() {
                        changes.push(ParameterChange {
                            parameter: "lighting".to_string(),
                            value: NodeData::Boolean(node.viewport.enable_lighting),
                        });
                    }
                    if ui.checkbox(&mut node.viewport.enable_grid, "Grid").changed() {
                        changes.push(ParameterChange {
                            parameter: "grid".to_string(),
                            value: NodeData::Boolean(node.viewport.enable_grid),
                        });
                    }
                });
            });
        });
    
    changes
}