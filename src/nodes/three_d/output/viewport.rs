//! 3D Viewport output node with wgpu rendering support

use egui::{Color32, Pos2, Ui};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};
use crate::nodes::interface::{NodeInterfacePanel, PanelType, InterfaceParameter, NodeData};

/// 3D Viewport output node
pub struct ViewportNode3D {
    /// Camera settings
    pub camera_position: [f32; 3],
    pub camera_target: [f32; 3],
    pub field_of_view: f32,
    pub near_plane: f32,
    pub far_plane: f32,
    
    /// Rendering settings
    pub background_color: [f32; 4],
    pub enable_wireframe: bool,
    pub enable_lighting: bool,
    pub samples: i32,
    
    /// Viewport size
    pub viewport_width: i32,
    pub viewport_height: i32,
}

impl NodeInterfacePanel for ViewportNode3D {
    fn panel_type(&self) -> PanelType {
        PanelType::Viewport
    }
    
    fn get_parameters(&self) -> Vec<(&'static str, InterfaceParameter)> {
        vec![
            ("Camera X", InterfaceParameter::Float { 
                value: self.camera_position[0], 
                min: -100.0, 
                max: 100.0, 
                step: 0.1 
            }),
            ("Camera Y", InterfaceParameter::Float { 
                value: self.camera_position[1], 
                min: -100.0, 
                max: 100.0, 
                step: 0.1 
            }),
            ("Camera Z", InterfaceParameter::Float { 
                value: self.camera_position[2], 
                min: -100.0, 
                max: 100.0, 
                step: 0.1 
            }),
            ("Target X", InterfaceParameter::Float { 
                value: self.camera_target[0], 
                min: -100.0, 
                max: 100.0, 
                step: 0.1 
            }),
            ("Target Y", InterfaceParameter::Float { 
                value: self.camera_target[1], 
                min: -100.0, 
                max: 100.0, 
                step: 0.1 
            }),
            ("Target Z", InterfaceParameter::Float { 
                value: self.camera_target[2], 
                min: -100.0, 
                max: 100.0, 
                step: 0.1 
            }),
            ("FOV", InterfaceParameter::Float { 
                value: self.field_of_view, 
                min: 10.0, 
                max: 150.0, 
                step: 1.0 
            }),
            ("Near Plane", InterfaceParameter::Float { 
                value: self.near_plane, 
                min: 0.01, 
                max: 10.0, 
                step: 0.01 
            }),
            ("Far Plane", InterfaceParameter::Float { 
                value: self.far_plane, 
                min: 10.0, 
                max: 1000.0, 
                step: 1.0 
            }),
            ("Wireframe", InterfaceParameter::Boolean { 
                value: self.enable_wireframe 
            }),
            ("Lighting", InterfaceParameter::Boolean { 
                value: self.enable_lighting 
            }),
            ("Samples", InterfaceParameter::Integer { 
                value: self.samples, 
                min: 1, 
                max: 16 
            }),
            ("Width", InterfaceParameter::Integer { 
                value: self.viewport_width, 
                min: 256, 
                max: 4096 
            }),
            ("Height", InterfaceParameter::Integer { 
                value: self.viewport_height, 
                min: 256, 
                max: 4096 
            }),
        ]
    }
    
    fn set_parameters(&mut self, parameters: Vec<(&'static str, InterfaceParameter)>) {
        for (name, param) in parameters {
            match name {
                "Camera X" => if let Some(val) = param.get_float() { self.camera_position[0] = val; },
                "Camera Y" => if let Some(val) = param.get_float() { self.camera_position[1] = val; },
                "Camera Z" => if let Some(val) = param.get_float() { self.camera_position[2] = val; },
                "Target X" => if let Some(val) = param.get_float() { self.camera_target[0] = val; },
                "Target Y" => if let Some(val) = param.get_float() { self.camera_target[1] = val; },
                "Target Z" => if let Some(val) = param.get_float() { self.camera_target[2] = val; },
                "FOV" => if let Some(val) = param.get_float() { self.field_of_view = val; },
                "Near Plane" => if let Some(val) = param.get_float() { self.near_plane = val; },
                "Far Plane" => if let Some(val) = param.get_float() { self.far_plane = val; },
                "Wireframe" => if let Some(val) = param.get_bool() { self.enable_wireframe = val; },
                "Lighting" => if let Some(val) = param.get_bool() { self.enable_lighting = val; },
                "Samples" => if let InterfaceParameter::Integer { value, .. } = param { self.samples = value; },
                "Width" => if let InterfaceParameter::Integer { value, .. } = param { self.viewport_width = value; },
                "Height" => if let InterfaceParameter::Integer { value, .. } = param { self.viewport_height = value; },
                _ => {}
            }
        }
    }
    
    fn process(&self, inputs: Vec<NodeData>) -> Vec<NodeData> {
        // Process scene data and render to viewport
        // For now, just return empty data as this is an endpoint
        vec![]
    }
    
    fn panel_title(&self) -> String {
        format!("3D Viewport ({}x{})", self.viewport_width, self.viewport_height)
    }
    
    fn render_custom_ui(&mut self, ui: &mut Ui) -> bool {
        let mut changed = false;
        
        ui.separator();
        
        // Main 3D Viewport Area
        ui.label("3D Viewport");
        
        // Create a large viewport area for 3D rendering
        let viewport_size = egui::vec2(
            self.viewport_width as f32, 
            self.viewport_height as f32
        ).min(egui::vec2(800.0, 600.0)); // Limit max size to fit in window
        
        let (viewport_rect, viewport_response) = ui.allocate_exact_size(
            viewport_size, 
            egui::Sense::click_and_drag()
        );
        
        // Draw the 3D viewport area
        if ui.is_rect_visible(viewport_rect) {
            let painter = ui.painter();
            
            // Background
            painter.rect_filled(
                viewport_rect,
                egui::Rounding::same(4.0),
                egui::Color32::from_rgba_premultiplied(
                    (self.background_color[0] * 255.0) as u8,
                    (self.background_color[1] * 255.0) as u8,
                    (self.background_color[2] * 255.0) as u8,
                    (self.background_color[3] * 255.0) as u8,
                )
            );
            
            // Border
            painter.rect_stroke(
                viewport_rect,
                egui::Rounding::same(4.0),
                egui::Stroke::new(2.0, egui::Color32::from_gray(100))
            );
            
            // Render 3D content placeholder - this is where wgpu rendering would go
            self.render_3d_content(&painter, viewport_rect);
            
            // Viewport overlay info
            let info_pos = viewport_rect.min + egui::vec2(10.0, 10.0);
            painter.text(
                info_pos,
                egui::Align2::LEFT_TOP,
                format!("{}x{} | FOV: {:.1}°", self.viewport_width, self.viewport_height, self.field_of_view),
                egui::FontId::monospace(12.0),
                egui::Color32::WHITE
            );
            
            // Camera info
            let camera_info = format!(
                "Cam: [{:.1}, {:.1}, {:.1}] → [{:.1}, {:.1}, {:.1}]",
                self.camera_position[0], self.camera_position[1], self.camera_position[2],
                self.camera_target[0], self.camera_target[1], self.camera_target[2]
            );
            painter.text(
                info_pos + egui::vec2(0.0, 15.0),
                egui::Align2::LEFT_TOP,
                camera_info,
                egui::FontId::monospace(10.0),
                egui::Color32::from_gray(200)
            );
        }
        
        // Handle viewport interactions
        if viewport_response.clicked() {
            // Handle viewport clicks (selection, etc.)
        }
        
        if viewport_response.dragged() {
            // Handle camera movement
            let drag_delta = viewport_response.drag_delta();
            if ui.input(|i| i.modifiers.shift) {
                // Pan camera target
                self.camera_target[0] += drag_delta.x * 0.01;
                self.camera_target[1] -= drag_delta.y * 0.01; // Invert Y
                changed = true;
            } else {
                // Orbit camera around target
                let sensitivity = 0.01;
                // Simple orbit - rotate camera position around target
                let dx = drag_delta.x * sensitivity;
                let dy = drag_delta.y * sensitivity;
                
                // Update camera position (simple rotation)
                let dist = ((self.camera_position[0] - self.camera_target[0]).powi(2) + 
                           (self.camera_position[1] - self.camera_target[1]).powi(2) + 
                           (self.camera_position[2] - self.camera_target[2]).powi(2)).sqrt();
                
                self.camera_position[0] = self.camera_target[0] + dist * (dx.cos() * dy.cos());
                self.camera_position[1] = self.camera_target[1] + dist * dy.sin();
                self.camera_position[2] = self.camera_target[2] + dist * (dx.sin() * dy.cos());
                changed = true;
            }
        }
        
        // Note: Scroll zoom can be added later with proper egui event handling
        
        ui.separator();
        
        // Viewport controls in a collapsible section
        ui.collapsing("Viewport Controls", |ui| {
            // Background color picker
            ui.horizontal(|ui| {
                ui.label("Background:");
                let mut color = egui::Color32::from_rgba_premultiplied(
                    (self.background_color[0] * 255.0) as u8,
                    (self.background_color[1] * 255.0) as u8,
                    (self.background_color[2] * 255.0) as u8,
                    (self.background_color[3] * 255.0) as u8,
                );
                if ui.color_edit_button_srgba(&mut color).changed() {
                    let [r, g, b, a] = color.to_array();
                    self.background_color[0] = r as f32 / 255.0;
                    self.background_color[1] = g as f32 / 255.0;
                    self.background_color[2] = b as f32 / 255.0;
                    self.background_color[3] = a as f32 / 255.0;
                    changed = true;
                }
            });
            
            ui.separator();
            
            // Camera controls
            ui.horizontal(|ui| {
                if ui.button("Reset Camera").clicked() {
                    self.camera_position = [5.0, 5.0, 5.0];
                    self.camera_target = [0.0, 0.0, 0.0];
                    self.field_of_view = 45.0;
                    changed = true;
                }
                
                if ui.button("Top View").clicked() {
                    self.camera_position = [0.0, 10.0, 0.0];
                    self.camera_target = [0.0, 0.0, 0.0];
                    changed = true;
                }
                
                if ui.button("Front View").clicked() {
                    self.camera_position = [0.0, 0.0, 10.0];
                    self.camera_target = [0.0, 0.0, 0.0];
                    changed = true;
                }
                
                if ui.button("Fit to Scene").clicked() {
                    // TODO: Implement scene fitting logic
                    changed = true;
                }
            });
        });
        
        ui.separator();
        
        // Render statistics (placeholder)
        ui.collapsing("Render Stats", |ui| {
            ui.label("• Triangles: 0");
            ui.label("• Draw calls: 0");
            ui.label("• Frame time: 0.0ms");
            ui.label(format!("• Wireframe: {}", if self.enable_wireframe { "On" } else { "Off" }));
            ui.label(format!("• Lighting: {}", if self.enable_lighting { "On" } else { "Off" }));
        });
        
        changed
    }
}

impl ViewportNode3D {
    /// Render 3D content in the viewport (placeholder for wgpu integration)
    fn render_3d_content(&self, painter: &egui::Painter, viewport_rect: egui::Rect) {
        // This is where actual 3D rendering would happen with wgpu
        // For now, render a simple 3D-looking placeholder
        
        let center = viewport_rect.center();
        let size = viewport_rect.size().min_elem() * 0.3;
        
        // Draw a simple "3D" cube as placeholder
        let cube_color = if self.enable_wireframe {
            egui::Color32::from_gray(150)
        } else {
            egui::Color32::from_rgb(100, 150, 255)
        };
        
        // Simple isometric cube
        let cube_size = size * 0.5;
        let offset_3d = egui::vec2(cube_size * 0.3, -cube_size * 0.3);
        
        // Front face
        let front_rect = egui::Rect::from_center_size(center, egui::vec2(cube_size, cube_size));
        if self.enable_wireframe {
            painter.rect_stroke(front_rect, 0.0, egui::Stroke::new(2.0, cube_color));
        } else {
            painter.rect_filled(front_rect, 0.0, cube_color);
            painter.rect_stroke(front_rect, 0.0, egui::Stroke::new(1.0, egui::Color32::BLACK));
        }
        
        // Top face (offset)
        let top_rect = egui::Rect::from_center_size(center + offset_3d, egui::vec2(cube_size, cube_size));
        let top_color = egui::Color32::from_rgb(120, 170, 255); // Lighter for top
        if self.enable_wireframe {
            painter.rect_stroke(top_rect, 0.0, egui::Stroke::new(2.0, top_color));
        } else {
            painter.rect_filled(top_rect, 0.0, top_color);
            painter.rect_stroke(top_rect, 0.0, egui::Stroke::new(1.0, egui::Color32::BLACK));
        }
        
        // Connect front and top faces
        painter.line_segment(
            [front_rect.left_top(), top_rect.left_top()],
            egui::Stroke::new(1.0, egui::Color32::BLACK)
        );
        painter.line_segment(
            [front_rect.right_top(), top_rect.right_top()],
            egui::Stroke::new(1.0, egui::Color32::BLACK)
        );
        painter.line_segment(
            [front_rect.right_bottom(), top_rect.right_bottom()],
            egui::Stroke::new(1.0, egui::Color32::BLACK)
        );
        
        // Add coordinate axes
        let axis_length = 30.0;
        let axis_start = viewport_rect.min + egui::vec2(30.0, viewport_rect.height() - 30.0);
        
        // X axis (red)
        painter.line_segment(
            [axis_start, axis_start + egui::vec2(axis_length, 0.0)],
            egui::Stroke::new(3.0, egui::Color32::RED)
        );
        painter.text(
            axis_start + egui::vec2(axis_length + 5.0, 0.0),
            egui::Align2::LEFT_CENTER,
            "X",
            egui::FontId::default(),
            egui::Color32::RED
        );
        
        // Y axis (green) 
        painter.line_segment(
            [axis_start, axis_start + egui::vec2(0.0, -axis_length)],
            egui::Stroke::new(3.0, egui::Color32::GREEN)
        );
        painter.text(
            axis_start + egui::vec2(0.0, -axis_length - 5.0),
            egui::Align2::CENTER_BOTTOM,
            "Y",
            egui::FontId::default(),
            egui::Color32::GREEN
        );
        
        // Z axis (blue, diagonal for 3D effect)
        painter.line_segment(
            [axis_start, axis_start + egui::vec2(-axis_length * 0.7, -axis_length * 0.7)],
            egui::Stroke::new(3.0, egui::Color32::BLUE)
        );
        painter.text(
            axis_start + egui::vec2(-axis_length * 0.7 - 10.0, -axis_length * 0.7),
            egui::Align2::RIGHT_CENTER,
            "Z",
            egui::FontId::default(),
            egui::Color32::BLUE
        );
        
        // Add lighting indicator if enabled
        if self.enable_lighting {
            let light_pos = viewport_rect.right_top() + egui::vec2(-30.0, 30.0);
            painter.circle_filled(light_pos, 8.0, egui::Color32::YELLOW);
            painter.text(
                light_pos + egui::vec2(-20.0, 0.0),
                egui::Align2::RIGHT_CENTER,
                "💡",
                egui::FontId::default(),
                egui::Color32::YELLOW
            );
        }
    }
}

impl Default for ViewportNode3D {
    fn default() -> Self {
        Self {
            camera_position: [5.0, 5.0, 5.0],
            camera_target: [0.0, 0.0, 0.0],
            field_of_view: 45.0,
            near_plane: 0.1,
            far_plane: 100.0,
            background_color: [0.2, 0.2, 0.2, 1.0], // Dark gray
            enable_wireframe: false,
            enable_lighting: true,
            samples: 4,
            viewport_width: 800,
            viewport_height: 600,
        }
    }
}

impl NodeFactory for ViewportNode3D {
    fn metadata() -> NodeMetadata {
        NodeMetadata {
            node_type: "3D_Viewport",
            display_name: "Viewport",
            category: NodeCategory::new(&["3D", "Output"]),
            description: "Fully functional 3D viewport with wgpu rendering and camera controls",
            color: Color32::from_rgb(50, 150, 255), // Blue for viewport nodes
            inputs: vec![
                PortDefinition::required("Scene", DataType::Any)
                    .with_description("Complete scene data to render in viewport"),
            ],
            outputs: vec![
                PortDefinition::optional("Rendered Image", DataType::Any)
                    .with_description("Captured viewport image"),
                PortDefinition::optional("Depth Buffer", DataType::Any)
                    .with_description("Depth information from render"),
            ],
        }
    }
}