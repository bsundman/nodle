//! Workspace menu system for node creation

use egui::{Pos2, Color32, Rect, Vec2};
use crate::workspace::{WorkspaceManager, WorkspaceMenuItem};
use crate::editor::navigation::NavigationManager;

/// Standard menu styling for consistency across all menus
pub fn apply_menu_style(ui: &mut egui::Ui) {
    // Dark background like context menu
    ui.style_mut().visuals.window_fill = Color32::from_rgb(28, 28, 28);
    ui.style_mut().visuals.extreme_bg_color = Color32::from_rgb(28, 28, 28);
    ui.style_mut().visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(28, 28, 28);
    
    // Hover color
    ui.style_mut().visuals.widgets.hovered.bg_fill = Color32::from_rgb(48, 48, 48);
    
    // Text color
    ui.style_mut().visuals.widgets.noninteractive.fg_stroke.color = Color32::from_gray(200);
    ui.style_mut().visuals.widgets.hovered.fg_stroke.color = Color32::WHITE;
    
    // Consistent item spacing - set to 0 for tight menu layout
    ui.style_mut().spacing.item_spacing.y = 0.0;
}

/// Render a complete menu using Area/Frame - SHARED by ALL menus
pub fn render_shared_menu<F>(
    ui_ctx: &egui::Context,
    menu_id: &str,
    position: Pos2,
    menu_items: Vec<(&str, bool)>, // (text, show_arrow)
    render_callback: F
) -> (Option<String>, egui::Response)
where
    F: FnOnce(&mut egui::Ui, &[(&str, bool)], f32) -> Option<String>,
{
    let popup_id = egui::Id::new(menu_id);
    
    let menu_response = egui::Area::new(popup_id)
        .fixed_pos(position)
        .show(ui_ctx, |ui| {
            egui::Frame::popup(ui.style())
                .fill(Color32::from_rgb(28, 28, 28))
                .show(ui, |ui| {
                    // Apply consistent menu styling
                    apply_menu_style(ui);
                    
                    // Calculate menu width based on text
                    let text_width = menu_items.iter()
                        .map(|(text, _)| {
                            let galley = ui.fonts(|f| f.layout_no_wrap(text.to_string(), egui::FontId::default(), Color32::WHITE));
                            galley.rect.width()
                        })
                        .fold(0.0, f32::max);
                    let menu_width = (text_width + ui.spacing().button_padding.x * 2.0 + 20.0).max(120.0);
                    ui.set_min_width(menu_width);
                    ui.set_max_width(menu_width);
                    
                    // Call the specific menu content renderer
                    render_callback(ui, &menu_items, menu_width)
                })
                .inner
        });
    
    (menu_response.inner, menu_response.response)
}

/// Render a menu item with full-width hover highlighting - SHARED by ALL menus
/// Returns true if clicked
pub fn render_menu_item(ui: &mut egui::Ui, text: &str, menu_width: f32) -> bool {
    render_menu_item_with_arrow(ui, text, menu_width, false).0 // Return only clicked
}

/// Render a menu item and return both clicked and hovered states
pub fn render_menu_item_full(ui: &mut egui::Ui, text: &str, menu_width: f32) -> (bool, bool) {
    render_menu_item_with_arrow(ui, text, menu_width, false)
}

/// Render a menu item with optional arrow - SHARED base function for ALL menu styling
/// Returns (clicked, hovered)
pub fn render_menu_item_with_arrow(ui: &mut egui::Ui, text: &str, menu_width: f32, show_arrow: bool) -> (bool, bool) {
    let desired_size = Vec2::new(menu_width, ui.spacing().button_padding.y * 2.0 + ui.text_style_height(&egui::TextStyle::Body));
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        
        // Fill background on hover - extend PAST item limits to hit parent container edges
        if response.hovered() {
            // Get parent container bounds and extend highlight beyond item boundaries
            let container_rect = ui.max_rect();
            // Extend highlight to go PAST the menu item's natural boundaries
            let highlight_rect = Rect::from_min_max(
                Pos2::new(container_rect.min.x - 7.0, rect.min.y + ui.spacing().button_padding.y / 2.0), // Extend left past container
                Pos2::new(container_rect.max.x + 7.0, rect.max.y - ui.spacing().button_padding.y / 2.0)  // Extend right past container
            );
            ui.painter().rect_filled(highlight_rect, 0.0, Color32::from_rgb(48, 48, 48));
        }
        
        // Draw text
        ui.painter().text(
            rect.left_center() + egui::vec2(ui.spacing().button_padding.x, 0.0),
            egui::Align2::LEFT_CENTER,
            text,
            egui::FontId::default(),
            visuals.text_color(),
        );
        
        // Draw arrow if requested
        if show_arrow {
            ui.painter().text(
                rect.right_center() - egui::vec2(ui.spacing().button_padding.x, 0.0),
                egui::Align2::RIGHT_CENTER,
                "▶",
                egui::FontId::default(),
                visuals.text_color(),
            );
        }
    }
    
    (response.clicked(), response.hovered())
}

/// Manages workspace menus and submenus for node creation
#[derive(Debug, Clone)]
pub struct MenuManager {
    open_submenu: Option<String>,
    submenu_pos: Option<Pos2>,
    submenu_close_timer: Option<std::time::Instant>,
    // Support for hierarchical menu navigation
    submenu_path: Vec<String>, // Track the current path in the menu hierarchy
    // Support for multiple nested submenus
    nested_submenus: Vec<(String, Pos2)>, // Track multiple open submenus with their positions
}

impl MenuManager {
    /// Creates a new menu manager
    pub fn new() -> Self {
        Self {
            open_submenu: None,
            submenu_pos: None,
            submenu_close_timer: None,
            submenu_path: Vec::new(),
            nested_submenus: Vec::new(),
        }
    }

    /// Reset menu state (close submenus)
    pub fn reset(&mut self) {
        self.open_submenu = None;
        self.submenu_pos = None;
        self.submenu_close_timer = None;
        self.submenu_path.clear();
        self.nested_submenus.clear();
    }

    /// Render the workspace menu and return selected node type
    /// Returns (selected_node_type, menu_response, submenu_response)
    pub fn render_workspace_menu(
        &mut self, 
        ui: &mut egui::Ui, 
        menu_screen_pos: Pos2,
        workspace_manager: &WorkspaceManager,
        navigation: &NavigationManager
    ) -> (Option<String>, egui::Response, Option<egui::Response>) {
        let mut selected_node_type = None;
        let popup_id = egui::Id::new("workspace_menu");

        // Render main menu
        let menu_response = egui::Area::new(popup_id)
            .fixed_pos(menu_screen_pos)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .fill(Color32::from_rgb(28, 28, 28))
                    .show(ui, |ui| {
                        // Apply consistent menu styling - CRITICAL for matching file menu
                        apply_menu_style(ui);
                        
                        // Get workspace-aware menu structure based on current navigation path
                        let menu_structure = workspace_manager.get_menu_for_path(&navigation.current_path);
                        
                        // Calculate menu width based on category names
                        let category_names: Vec<String> = menu_structure.iter()
                            .map(|item| match item {
                                WorkspaceMenuItem::Category { name, .. } => name.clone(),
                                WorkspaceMenuItem::Node { name, .. } => name.clone(),
                                WorkspaceMenuItem::Workspace { name, .. } => name.clone(),
                            })
                            .collect();
                        
                        let text_width = category_names.iter()
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

                        // Track if any item is currently being hovered
                        let mut any_item_hovered = false;
                        
                        // Render the workspace-aware menu structure
                        for menu_item in menu_structure {
                            match menu_item {
                                WorkspaceMenuItem::Category { name, .. } => {
                                    let was_hovered = self.render_submenu_item(ui, &name, menu_width);
                                    if was_hovered {
                                        any_item_hovered = true;
                                    }
                                }
                                WorkspaceMenuItem::Workspace { name, .. } => {
                                    let was_hovered = self.render_submenu_item(ui, &name, menu_width);
                                    if was_hovered {
                                        any_item_hovered = true;
                                    }
                                }
                                WorkspaceMenuItem::Node { name, node_type } => {
                                    // Direct nodes at top level (workspaces)
                                    if render_menu_item(ui, &name, menu_width) {
                                        selected_node_type = Some(node_type);
                                    }
                                }
                            }
                        }
                        
                        // Start close timer if no item is hovered, but don't immediately close
                        if !any_item_hovered && self.open_submenu.is_some() && self.submenu_close_timer.is_none() {
                            self.submenu_close_timer = Some(std::time::Instant::now());
                        }
                        
                        // Check if close timer has expired (300ms delay)
                        if let Some(timer) = self.submenu_close_timer {
                            if timer.elapsed().as_millis() > 300 {
                                self.open_submenu = None;
                                self.submenu_pos = None;
                                self.submenu_close_timer = None;
                                // Also close all nested submenus when main submenu closes
                                self.nested_submenus.clear();
                            }
                        }
                    })
                    .inner
            });

        // Render primary submenu if one is open
        let submenu_response = if let (Some(submenu_name), Some(submenu_screen_pos)) = 
            (self.open_submenu.clone(), self.submenu_pos) {
            
            let submenu_id = egui::Id::new(format!("submenu_{}", submenu_name));
            let response = egui::Area::new(submenu_id)
                .fixed_pos(submenu_screen_pos)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style())
                        .fill(Color32::from_rgb(28, 28, 28))
                        .show(ui, |ui| {
                            // Apply consistent menu styling - CRITICAL for matching file menu
                            apply_menu_style(ui);
                            
                            selected_node_type = self.render_submenu_content(ui, &submenu_name, workspace_manager, navigation);
                        })
                        .inner
                }).response;
            Some(response)
        } else {
            None
        };

        // Render all nested submenus and track if any are being hovered
        let mut nested_submenu_hovered = false;
        for (i, (nested_name, nested_pos)) in self.nested_submenus.iter().enumerate() {
            let nested_id = egui::Id::new(format!("nested_submenu_{}_{}", i, nested_name));
            let area_response = egui::Area::new(nested_id)
                .fixed_pos(*nested_pos)
                .interactable(true)  // Ensure the area is interactable
                .show(ui.ctx(), |ui| {
                    let frame_response = egui::Frame::popup(ui.style())
                        .fill(Color32::from_rgb(28, 28, 28))
                        .show(ui, |ui| {
                            // Apply consistent menu styling
                            apply_menu_style(ui);
                            
                            // Render nested submenu content
                            if let Some(node_type) = self.render_nested_submenu_content(ui, nested_name, workspace_manager, navigation) {
                                selected_node_type = Some(node_type);
                            }
                        });
                    
                    // Return the frame response for better hover detection
                    frame_response.response
                });
            
            // Check if this nested submenu is being hovered - check both area and inner response
            if area_response.response.hovered() || area_response.inner.hovered() {
                nested_submenu_hovered = true;
                self.submenu_close_timer = None; // Cancel any pending close
            }
        }
        
        // Handle nested submenu closing with proper usability - use a delay to allow mouse movement
        let main_submenu_hovered = submenu_response.as_ref().map(|r| r.hovered()).unwrap_or(false);
        
        // Check if mouse is in a safe path between main submenu and nested submenu
        let mouse_in_safe_path = if let Some(submenu_resp) = &submenu_response {
            if !self.nested_submenus.is_empty() {
                // Create a triangular "safe zone" between the main submenu and nested submenu
                let main_rect = submenu_resp.rect;
                let mouse_pos = ui.ctx().input(|i| i.pointer.hover_pos()).unwrap_or_default();
                
                // If mouse is moving towards the nested submenu, give extra grace time
                mouse_pos.x > main_rect.right() && 
                mouse_pos.y >= main_rect.top() && 
                mouse_pos.y <= main_rect.bottom()
            } else {
                false
            }
        } else {
            false
        };
        
        // Additional hover check - see if the mouse is inside any of the menu areas
        let mouse_in_any_menu = ui.ctx().input(|i| {
            if let Some(mouse_pos) = i.pointer.hover_pos() {
                // Check main menu
                if menu_response.response.rect.contains(mouse_pos) {
                    return true;
                }
                // Check submenu
                if let Some(sub_resp) = &submenu_response {
                    if sub_resp.rect.contains(mouse_pos) {
                        return true;
                    }
                }
                // Check all nested submenus
                // Note: We don't have direct access to their rects here, but nested_submenu_hovered should cover this
                false
            } else {
                false
            }
        });

        if main_submenu_hovered || nested_submenu_hovered || mouse_in_safe_path || mouse_in_any_menu {
            // Reset close timer if hovering over any menu or in safe path
            self.submenu_close_timer = None;
        } else {
            // Start close timer if not already started
            if self.submenu_close_timer.is_none() {
                self.submenu_close_timer = Some(std::time::Instant::now());
            }
            
            // Only close nested submenus after a delay to allow mouse movement
            if let Some(timer) = self.submenu_close_timer {
                if timer.elapsed().as_millis() > 800 { // Increased to 800ms grace period for better UX
                    self.nested_submenus.clear();
                }
            }
        }

        (selected_node_type, menu_response.response, submenu_response)
    }

    /// Render a submenu item with full-width highlighting and arrow - USES SHARED HELPER
    /// Returns true if this item is currently hovered
    fn render_submenu_item(&mut self, ui: &mut egui::Ui, text: &str, menu_width: f32) -> bool {
        // Use the exact same shared helper function and get proper hover detection
        let (clicked, hovered) = render_menu_item_with_arrow(ui, text, menu_width, true);
        
        // Check for hover to open submenu - use the actual hover state from the response
        if hovered {
            // Calculate submenu position based on the menu item that was just rendered
            let current_pos = ui.next_widget_position();
            let item_height = ui.spacing().button_padding.y * 2.0 + ui.text_style_height(&egui::TextStyle::Body);
            let item_top = current_pos.y - item_height; // Go back to the top of the item we just rendered
            
            self.open_submenu = Some(text.to_string());
            self.submenu_pos = Some(Pos2::new(current_pos.x + menu_width, item_top));
            self.submenu_close_timer = None; // Cancel any close timer
        }
        
        hovered
    }

    /// Render submenu content and return selected node type
    fn render_submenu_content(&mut self, ui: &mut egui::Ui, submenu_name: &str, workspace_manager: &WorkspaceManager, navigation: &NavigationManager) -> Option<String> {
        // Get workspace-aware menu structure and find the matching category
        let menu_structure = workspace_manager.get_menu_for_path(&navigation.current_path);
        let mut menu_items = Vec::new();
        
        // Find the category that matches the submenu name by searching recursively
        // If we're in a nested path, use the path to find the right category
        let search_name = if self.submenu_path.len() > 1 {
            &self.submenu_path[self.submenu_path.len() - 1]
        } else {
            submenu_name
        };
        
        if let Some(category_items) = self.find_category_items(&menu_structure, search_name) {
            // Extract both node types and nested categories from the category items
            for item in category_items {
                match item {
                    WorkspaceMenuItem::Node { name, node_type } => {
                        menu_items.push((name, node_type, false)); // false = no arrow
                    }
                    WorkspaceMenuItem::Workspace { name, .. } => {
                        menu_items.push((name.clone(), format!("SUBWORKSPACE:{}", name), false));
                    }
                    WorkspaceMenuItem::Category { name: sub_name, .. } => {
                        // Nested category - render with arrow to indicate submenu
                        menu_items.push((sub_name.clone(), format!("CATEGORY:{}", sub_name), true)); // true = show arrow
                    }
                }
            }
        }

        // Calculate submenu width using actual text measurement
        let text_width = menu_items.iter()
            .map(|(display_name, _, _)| {
                let galley = ui.fonts(|f| f.layout_no_wrap(display_name.to_string(), egui::FontId::default(), Color32::WHITE));
                galley.rect.width()
            })
            .fold(0.0, f32::max);
        let submenu_width = (text_width + ui.spacing().button_padding.x * 4.0).max(140.0);
        ui.set_min_width(submenu_width);
        ui.set_max_width(submenu_width);

        // Track which nested submenus should remain open
        let mut currently_hovered_nested = None;
        
        // Draw submenu items
        for (display_name, node_type, show_arrow) in menu_items {
            if show_arrow {
                // This is a nested category - handle as submenu navigation
                let (clicked, hovered) = render_menu_item_with_arrow(ui, &display_name, submenu_width, true);
                
                if hovered {
                    // Open nested submenu - calculate position for the next level
                    let current_pos = ui.next_widget_position();
                    let item_height = ui.spacing().button_padding.y * 2.0 + ui.text_style_height(&egui::TextStyle::Body);
                    let item_top = current_pos.y - item_height;
                    
                    // Only clear and replace if this is a different submenu than what's currently open
                    let needs_update = self.nested_submenus.is_empty() || 
                        !self.nested_submenus.iter().any(|(name, _)| name == &display_name);
                    
                    if needs_update {
                        // Clear all existing nested submenus and add only the new one
                        self.nested_submenus.clear();
                        
                        // Add the new nested submenu
                        let nested_pos = Pos2::new(current_pos.x + submenu_width, item_top);
                        self.nested_submenus.push((display_name.clone(), nested_pos));
                    }
                    
                    self.submenu_close_timer = None;
                    currently_hovered_nested = Some(display_name.clone());
                }
                
                // Don't return the category identifier for clicks - let hover handle navigation
            } else {
                // This is a regular node - render normally
                if self.render_submenu_node_item(ui, &display_name, submenu_width) {
                    return Some(node_type);
                }
            }
        }
        
        // If no nested submenu is being hovered over in this frame, clear all nested submenus
        if currently_hovered_nested.is_none() {
            // Don't clear immediately - check if any nested submenu itself is being hovered
            // This will be handled in the main render method
        }
        
        None
    }

    /// Render nested submenu content and return selected node type
    fn render_nested_submenu_content(&self, ui: &mut egui::Ui, submenu_name: &str, workspace_manager: &WorkspaceManager, navigation: &NavigationManager) -> Option<String> {
        // Get workspace-aware menu structure and find the matching category
        let menu_structure = workspace_manager.get_menu_for_path(&navigation.current_path);
        let mut menu_items = Vec::new();
        
        // Find the category that matches the submenu name by searching recursively
        if let Some(category_items) = self.find_category_items(&menu_structure, submenu_name) {
            // Extract only node types from the category items (no further nesting for now)
            for item in category_items {
                match item {
                    WorkspaceMenuItem::Node { name, node_type } => {
                        menu_items.push((name, node_type));
                    }
                    WorkspaceMenuItem::Workspace { name, .. } => {
                        menu_items.push((name.clone(), format!("SUBWORKSPACE:{}", name)));
                    }
                    // For nested categories, we could add further nesting here if needed
                    WorkspaceMenuItem::Category { .. } => {
                        // Skip nested categories in nested submenus for now to avoid infinite nesting
                    }
                }
            }
        }

        // Calculate submenu width using actual text measurement
        let text_width = menu_items.iter()
            .map(|(display_name, _)| {
                let galley = ui.fonts(|f| f.layout_no_wrap(display_name.to_string(), egui::FontId::default(), Color32::WHITE));
                galley.rect.width()
            })
            .fold(0.0, f32::max);
        let submenu_width = (text_width + ui.spacing().button_padding.x * 4.0).max(140.0);
        ui.set_min_width(submenu_width);
        ui.set_max_width(submenu_width);

        // Track if any item is hovered to maintain menu visibility
        let mut any_item_hovered = false;

        // Draw submenu items
        for (display_name, node_type) in menu_items {
            let (clicked, hovered) = render_menu_item_full(ui, &display_name, submenu_width);
            if hovered {
                any_item_hovered = true;
            }
            if clicked {
                return Some(node_type);
            }
        }
        
        // If any item in this submenu is hovered, ensure we don't close the menu
        if any_item_hovered {
            // This is handled by the parent, but we make sure hover is detected
            ui.ctx().request_repaint(); // Ensure UI updates
        }
        
        None
    }

    /// Helper method to find category items by name, searching recursively
    fn find_category_items(&self, menu_structure: &[WorkspaceMenuItem], category_name: &str) -> Option<Vec<WorkspaceMenuItem>> {
        for menu_item in menu_structure {
            match menu_item {
                WorkspaceMenuItem::Category { name, items } if name == category_name => {
                    return Some(items.clone());
                }
                WorkspaceMenuItem::Category { items, .. } => {
                    // Recursively search in nested categories
                    if let Some(found_items) = self.find_category_items(items, category_name) {
                        return Some(found_items);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Render a node item in submenu - USES SHARED STYLING
    fn render_submenu_node_item(&self, ui: &mut egui::Ui, text: &str, submenu_width: f32) -> bool {
        render_menu_item(ui, text, submenu_width)
    }

    /// Render a category item in submenu with arrow - USES SHARED STYLING
    fn render_submenu_category_item(&self, ui: &mut egui::Ui, text: &str, submenu_width: f32) -> bool {
        let (clicked, _) = render_menu_item_with_arrow(ui, text, submenu_width, true);
        clicked
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
                // Check if mouse is in menu, submenu, or buffer area
                let in_menu = menu_rect.contains(pos);
                let in_submenu = sub_rect.contains(pos);
                let in_buffer = self.is_mouse_in_menu_buffer(pos, menu_rect, sub_rect);
                
                if in_menu || in_submenu || in_buffer {
                    // Cancel close timer - mouse is in a valid area
                    self.submenu_close_timer = None;
                } else {
                    // Start close timer if not already started
                    if self.submenu_close_timer.is_none() {
                        self.submenu_close_timer = Some(std::time::Instant::now());
                    }
                    
                    // Close immediately if timer has expired (300ms)
                    if let Some(timer) = self.submenu_close_timer {
                        if timer.elapsed().as_millis() > 300 {
                            self.open_submenu = None;
                            self.submenu_pos = None;
                            self.submenu_close_timer = None;
                        }
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