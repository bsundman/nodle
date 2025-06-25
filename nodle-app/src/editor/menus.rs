//! Context menu system for node creation

use egui::{Pos2, Color32, Rect, Vec2};
use crate::context::ContextManager;

/// Manages context menus and submenus for node creation
#[derive(Debug, Clone)]
pub struct MenuManager {
    open_submenu: Option<String>,
    submenu_pos: Option<Pos2>,
}

impl MenuManager {
    /// Creates a new menu manager
    pub fn new() -> Self {
        Self {
            open_submenu: None,
            submenu_pos: None,
        }
    }

    /// Reset menu state (close submenus)
    pub fn reset(&mut self) {
        self.open_submenu = None;
        self.submenu_pos = None;
    }

    /// Render the context menu and return selected node type
    /// Returns (selected_node_type, menu_response, submenu_response)
    pub fn render_context_menu(
        &mut self, 
        ui: &mut egui::Ui, 
        menu_screen_pos: Pos2,
        context_manager: &ContextManager
    ) -> (Option<String>, egui::Response, Option<egui::Response>) {
        let mut selected_node_type = None;
        let popup_id = egui::Id::new("context_menu");

        // Render main menu
        let menu_response = egui::Area::new(popup_id)
            .fixed_pos(menu_screen_pos)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .fill(Color32::from_rgb(28, 28, 28))
                    .show(ui, |ui| {
                        // Calculate menu width based on category names
                        let mut categories = vec!["Create Node:", "Math", "Logic", "Data", "Output"];
                        
                        // Add MaterialX to width calculation if active
                        if let Some(context) = context_manager.get_active_context() {
                            if context.id() == "materialx" {
                                categories.push("MaterialX");
                            }
                        }
                        
                        let text_width = categories.iter()
                            .map(|text| {
                                let galley = ui.fonts(|f| f.layout_no_wrap(text.to_string(), egui::FontId::default(), Color32::WHITE));
                                galley.rect.width()
                            })
                            .fold(0.0, f32::max);
                        let menu_width = (text_width + ui.spacing().button_padding.x * 2.0 + 20.0).max(120.0);
                        ui.set_min_width(menu_width);
                        ui.set_max_width(menu_width);

                        ui.label("Create Node:");
                        ui.separator();

                        // Show MaterialX items if MaterialX context is active
                        if let Some(context) = context_manager.get_active_context() {
                            if context.id() == "materialx" {
                                self.render_submenu_item(ui, "MaterialX", menu_width);
                            }
                        }
                        
                        self.render_submenu_item(ui, "Math", menu_width);
                        self.render_submenu_item(ui, "Logic", menu_width);
                        self.render_submenu_item(ui, "Data", menu_width);
                        self.render_submenu_item(ui, "Output", menu_width);
                    })
                    .inner
            });

        // Render submenu if one is open
        let submenu_response = if let (Some(submenu_name), Some(submenu_screen_pos)) = 
            (self.open_submenu.clone(), self.submenu_pos) {
            
            let submenu_id = egui::Id::new(format!("submenu_{}", submenu_name));
            Some(egui::Area::new(submenu_id)
                .fixed_pos(submenu_screen_pos)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style())
                        .fill(Color32::from_rgb(28, 28, 28))
                        .show(ui, |ui| {
                            selected_node_type = self.render_submenu_content(ui, &submenu_name);
                        })
                        .inner
                }).response)
        } else {
            None
        };

        (selected_node_type, menu_response.response, submenu_response)
    }

    /// Render a submenu item with full-width highlighting and arrow
    fn render_submenu_item(&mut self, ui: &mut egui::Ui, text: &str, menu_width: f32) {
        let desired_size = Vec2::new(menu_width, ui.spacing().button_padding.y * 2.0 + ui.text_style_height(&egui::TextStyle::Body));
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
        
        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);
            
            // Fill background on hover
            if response.hovered() {
                ui.painter().rect_filled(rect, 0.0, visuals.bg_fill);
                self.open_submenu = Some(text.to_string());
                self.submenu_pos = Some(Pos2::new(rect.right(), rect.top()));
            }
            
            // Draw text
            ui.painter().text(
                rect.left_center() + egui::vec2(ui.spacing().button_padding.x, 0.0),
                egui::Align2::LEFT_CENTER,
                text,
                egui::FontId::default(),
                visuals.text_color(),
            );
            
            // Draw arrow
            ui.painter().text(
                rect.right_center() - egui::vec2(ui.spacing().button_padding.x, 0.0),
                egui::Align2::RIGHT_CENTER,
                "▶",
                egui::FontId::default(),
                visuals.text_color(),
            );
        }
    }

    /// Render submenu content and return selected node type
    fn render_submenu_content(&self, ui: &mut egui::Ui, submenu_name: &str) -> Option<String> {
        // Get node items for this category  
        let node_items = match submenu_name {
            "MaterialX" => vec!["Noise", "Texture", "Mix", "Standard Surface", "3D View", "2D View"],
            "Math" => vec!["Add", "Subtract", "Multiply", "Divide"],
            "Logic" => vec!["AND", "OR", "NOT"],
            "Data" => vec!["Constant", "Variable"],
            "Output" => vec!["Print", "Debug"],
            _ => vec![],
        };

        // Calculate submenu width using actual text measurement
        let text_width = node_items.iter()
            .map(|text| {
                let galley = ui.fonts(|f| f.layout_no_wrap(text.to_string(), egui::FontId::default(), Color32::WHITE));
                galley.rect.width()
            })
            .fold(0.0, f32::max);
        let submenu_width = (text_width + ui.spacing().button_padding.x * 4.0).max(140.0);
        ui.set_min_width(submenu_width);
        ui.set_max_width(submenu_width);

        // Draw submenu items
        for node_type in node_items {
            if self.render_submenu_node_item(ui, node_type, submenu_width) {
                return Some(node_type.to_string());
            }
        }
        
        None
    }

    /// Render a node item in submenu
    fn render_submenu_node_item(&self, ui: &mut egui::Ui, text: &str, submenu_width: f32) -> bool {
        let desired_size = Vec2::new(submenu_width, ui.spacing().button_padding.y * 2.0 + ui.text_style_height(&egui::TextStyle::Body));
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
        
        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);
            
            // Fill background on hover
            if response.hovered() {
                ui.painter().rect_filled(rect, 0.0, visuals.bg_fill);
            }
            
            // Draw text
            ui.painter().text(
                rect.left_center() + egui::vec2(ui.spacing().button_padding.x, 0.0),
                egui::Align2::LEFT_CENTER,
                text,
                egui::FontId::default(),
                visuals.text_color(),
            );
        }
        
        response.clicked()
    }

    /// Check if mouse is in buffer area between menu and submenu
    pub fn is_mouse_in_menu_buffer(&self, mouse_pos: Pos2, menu_rect: Rect, submenu_rect: Rect) -> bool {
        let buffer_rect = Rect::from_two_pos(
            menu_rect.right_top(),
            submenu_rect.left_bottom()
        );
        buffer_rect.contains(mouse_pos)
    }

    /// Handle mouse movement for submenu management
    pub fn handle_mouse_movement(
        &mut self, 
        mouse_pos: Option<Pos2>, 
        menu_rect: Rect, 
        submenu_rect: Option<Rect>
    ) {
        if let Some(pos) = mouse_pos {
            if let Some(sub_rect) = submenu_rect {
                // Close submenu if mouse moves away from both menu and submenu
                if !menu_rect.contains(pos) && !sub_rect.contains(pos) {
                    // Check if mouse is in buffer area
                    if !self.is_mouse_in_menu_buffer(pos, menu_rect, sub_rect) {
                        self.open_submenu = None;
                        self.submenu_pos = None;
                    }
                }
            }
        }
    }

}

impl Default for MenuManager {
    fn default() -> Self {
        Self::new()
    }
}