//! CPU mesh creation and rendering logic

use egui::{Color32, Pos2, Rect, Vec2, Painter, Stroke};
use crate::nodes::Node;

/// Button color type for CPU rendering
#[derive(Debug, Clone, Copy)]
enum ButtonColorType {
    Green,
    Red,
}

/// Handles CPU-based mesh rendering for nodes and ports
pub struct MeshRenderer;

impl MeshRenderer {
    /// Create a new mesh renderer
    pub fn new() -> Self {
        Self
    }

    /// Render a complete node with all layers using CPU mesh generation
    /// This matches the exact rendering logic from the original editor
    pub fn render_node_complete_cpu(
        painter: &Painter,
        node: &Node,
        selected: bool,
        zoom: f32,
        transform_pos: impl Fn(Pos2) -> Pos2,
    ) {
        // Transform node rectangle
        let node_rect = node.get_rect();
        let transformed_rect = Rect::from_two_pos(
            transform_pos(node_rect.min), 
            transform_pos(node_rect.max)
        );
        
        // NODE BODY COMPONENTS
        let radius = 5.0 * zoom;
        
        // BACKGROUND: Inner gradient mesh - same for all nodes
        let (background_top_color, background_bottom_color) = (Color32::from_rgb(127, 127, 127), Color32::from_rgb(64, 64, 64));
        
        // BORDER: Outermost layer (1px larger than node rect, scaled by zoom)
        let border_expand = 1.0 * zoom;
        let border_rect = Rect::from_min_max(
            transformed_rect.min - Vec2::splat(border_expand),
            transformed_rect.max + Vec2::splat(border_expand),
        );
        let border_color = if selected {
            Color32::from_rgb(100, 150, 255) // Blue selection
        } else {
            Color32::from_rgb(64, 64, 64)    // Dark grey unselected
        };
        
        let border_mesh = Self::create_rounded_gradient_mesh_optimized(
            border_rect,
            radius, // Use same corner radius as base node
            border_color,
            border_color, // Solid color border
        );
        
        painter.add(egui::Shape::mesh(border_mesh));
        
        // BEVEL: Middle layer (same size as original node rect)
        let bevel_rect = transformed_rect;
        let (bevel_top_color, bevel_bottom_color) = (Color32::from_rgb(166, 166, 166), Color32::from_rgb(38, 38, 38));
        
        let bevel_mesh = Self::create_rounded_gradient_mesh_optimized(
            bevel_rect,
            radius,
            bevel_top_color,
            bevel_bottom_color,
        );
        
        painter.add(egui::Shape::mesh(bevel_mesh));
        
        // BACKGROUND: Inner gradient mesh (shrunk by 1px from bevel, scaled by zoom)
        let background_shrink_offset = 1.0 * zoom;
        let background_rect = Rect::from_min_max(
            transformed_rect.min + Vec2::splat(background_shrink_offset),
            transformed_rect.max - Vec2::splat(background_shrink_offset),
        );
        
        // Render main background
        let background_mesh = Self::create_rounded_gradient_mesh_optimized(
            background_rect,
            radius - background_shrink_offset, // Also shrink the radius
            background_top_color,
            background_bottom_color,
        );
        painter.add(egui::Shape::mesh(background_mesh));
        

        // Title - same for all nodes
        painter.text(
            transform_pos(node.position + Vec2::new(node.size.x / 2.0, 15.0)),
            egui::Align2::CENTER_CENTER,
            &node.title,
            egui::FontId::proportional(12.0 * zoom),
            Color32::WHITE,
        );
    }

    /// Render a port with all layers using CPU mesh generation
    /// This matches the exact port rendering logic from the original editor
    pub fn render_port_complete_cpu(
        painter: &Painter,
        port_pos: Pos2,
        is_input: bool,
        is_connecting: bool,
        zoom: f32,
        transform_pos: impl Fn(Pos2) -> Pos2,
    ) {
        let port_radius = 5.0 * zoom;
        let transformed_pos = transform_pos(port_pos);
        
        // Draw port border (2px larger) - blue if connecting, grey otherwise
        let port_border_color = if is_connecting {
            Color32::from_rgb(100, 150, 255) // Blue selection color
        } else {
            Color32::from_rgb(64, 64, 64) // Unselected node border color
        };
        
        painter.circle_filled(
            transformed_pos,
            port_radius + 2.0 * zoom,
            port_border_color,
        );
        
        // Draw port bevel (1px larger) - use node bevel bottom color
        painter.circle_filled(
            transformed_pos,
            port_radius + 1.0 * zoom,
            Color32::from_rgb(38, 38, 38), // Node bevel bottom color (0.15)
        );
        
        // Draw port background (main port)
        let port_bg_color = if is_input {
            Color32::from_rgb(70, 120, 90) // Darker green for input ports
        } else {
            Color32::from_rgb(120, 70, 70) // Darker red for output ports
        };
        
        painter.circle_filled(
            transformed_pos,
            port_radius,
            port_bg_color,
        );
    }

    /// Render a visibility toggle port with border and bevel outlines (no fill)
    pub fn render_visibility_port_cpu(
        painter: &Painter,
        port_pos: Pos2,
        is_visible: bool,
        zoom: f32,
        transform_pos: impl Fn(Pos2) -> Pos2,
    ) {
        let transformed_pos = transform_pos(port_pos);
        
        // Draw border outline (outer layer) - blue if enabled, grey if disabled
        let border_color = if is_visible {
            Color32::from_rgb(100, 150, 255) // Blue selection color when enabled
        } else {
            Color32::from_rgb(64, 64, 64) // Grey when disabled
        };
        
        let border_radius = 5.0 * zoom + 2.0 * zoom;
        painter.circle_stroke(
            transformed_pos,
            border_radius,
            Stroke::new(1.0 * zoom, border_color),
        );
        
        // Draw bevel outline (inner layer) - 1px smaller than border
        let bevel_radius = 5.0 * zoom + 1.0 * zoom;
        painter.circle_stroke(
            transformed_pos,
            bevel_radius,
            Stroke::new(1.0 * zoom, Color32::from_rgb(38, 38, 38)), // Bevel outline
        );
        
        // Add bigger dot for visible nodes only
        if is_visible {
            let dot_radius = 3.0 * zoom; // Bigger solid dot
            painter.circle_filled(
                transformed_pos,
                dot_radius,
                Color32::from_rgb(180, 180, 180), // Light grey dot for visibility
            );
        }
    }

    /// Render port name on hover using CPU rendering
    pub fn render_port_name_on_hover(
        painter: &Painter,
        port_pos: Pos2,
        port_name: &str,
        is_input: bool,
        mouse_world_pos: Option<Pos2>,
        zoom: f32,
        transform_pos: impl Fn(Pos2) -> Pos2,
    ) {
        let hover_radius = 10.0; // Radius for hover detection (larger than visual port)
        
        if let Some(mouse_world_pos) = mouse_world_pos {
            if (port_pos - mouse_world_pos).length() < hover_radius {
                let text_offset = if is_input {
                    Vec2::new(0.0, -15.0) // Input ports: text above
                } else {
                    Vec2::new(0.0, 15.0)  // Output ports: text below
                };
                
                let text_align = if is_input {
                    egui::Align2::CENTER_BOTTOM
                } else {
                    egui::Align2::CENTER_TOP
                };
                
                painter.text(
                    transform_pos(port_pos + text_offset),
                    text_align,
                    port_name,
                    egui::FontId::proportional(10.0 * zoom),
                    Color32::from_gray(255), // Brighter when hovering
                );
            }
        }
    }

    /// Legacy render function for backwards compatibility
    pub fn render_node_cpu(
        &self,
        painter: &Painter,
        node: &Node,
        selected: bool,
        _zoom: f32,
    ) -> Rect {
        let rect = node.get_rect();
        let corner_radius = 5.0;
        
        // Create 3-layer rendering: border, bevel, background
        let border_color = if selected {
            Color32::from_rgb(100, 150, 255) // Blue when selected
        } else {
            Color32::from_rgb(64, 64, 64) // Grey when not selected
        };
        
        // Border layer (outermost)
        let border_rect = Rect::from_min_size(
            rect.min - Vec2::splat(1.0),
            rect.size() + Vec2::splat(2.0)
        );
        let border_mesh = Self::create_rounded_gradient_mesh_optimized(
            border_rect,
            corner_radius,
            border_color,
            border_color,
        );
        painter.add(egui::Shape::mesh(border_mesh));
        
        // Bevel layer (middle)
        let bevel_rect = Rect::from_min_size(
            rect.min,
            rect.size()
        );
        let bevel_top = Color32::from_gray((255.0 * 0.65) as u8);
        let bevel_bottom = Color32::from_gray((255.0 * 0.15) as u8);
        let bevel_mesh = Self::create_rounded_gradient_mesh_optimized(
            bevel_rect,
            corner_radius,
            bevel_top,
            bevel_bottom,
        );
        painter.add(egui::Shape::mesh(bevel_mesh));
        
        // Background layer (innermost)
        let bg_rect = Rect::from_min_size(
            rect.min + Vec2::splat(3.0),
            rect.size() - Vec2::splat(6.0)
        );
        let bg_top = Color32::from_gray((255.0 * 0.5) as u8);
        let bg_bottom = Color32::from_gray((255.0 * 0.25) as u8);
        let bg_mesh = Self::create_rounded_gradient_mesh_optimized(
            bg_rect,
            corner_radius,
            bg_top,
            bg_bottom,
        );
        painter.add(egui::Shape::mesh(bg_mesh));
        
        // Render node title
        let text_pos = Pos2::new(
            rect.center().x,
            rect.center().y
        );
        painter.text(
            text_pos,
            egui::Align2::CENTER_CENTER,
            &node.title,
            egui::FontId::default(),
            Color32::WHITE,
        );
        
        rect
    }

    /// Render a port using CPU mesh generation
    pub fn render_port_cpu(
        &self,
        painter: &Painter,
        port_pos: Pos2,
        is_input: bool,
        is_connecting: bool,
        zoom: f32,
    ) {
        let radius = 5.0 * zoom;
        
        // Port colors
        let border_color = if is_connecting {
            Color32::from_rgb(100, 150, 255) // Blue when connecting
        } else {
            Color32::from_rgb(64, 64, 64) // Grey normally
        };
        
        let background_color = if is_input {
            Color32::from_rgb(100, 255, 100) // Green for input
        } else {
            Color32::from_rgb(255, 100, 100) // Red for output
        };
        
        // Render port as circle
        painter.circle_filled(port_pos, radius + 1.0, border_color);
        painter.circle_filled(port_pos, radius, background_color);
    }

    /// Create a rounded rectangle mesh with vertical gradient using complex grid algorithm
    /// Performance note: This creates many vertices with triangle fans for smooth corners
    pub fn create_rounded_gradient_mesh(
        rect: Rect,
        radius: f32,
        top_color: Color32,
        bottom_color: Color32,
    ) -> egui::Mesh {
        let mut mesh = egui::Mesh::default();
        
        // Optimized approach: 4x4 grid (16 vertices) that avoids corner radius areas
        let _grid_size = 4; // 4x4 = 16 vertices total
        
        // Function to interpolate color based on Y position
        let interpolate_color = |y: f32| -> Color32 {
            let t = (y - rect.top()) / rect.height();
            Color32::from_rgb(
                (top_color.r() as f32 * (1.0 - t) + bottom_color.r() as f32 * t) as u8,
                (top_color.g() as f32 * (1.0 - t) + bottom_color.g() as f32 * t) as u8,
                (top_color.b() as f32 * (1.0 - t) + bottom_color.b() as f32 * t) as u8,
            )
        };
        
        // Function to check if a point is inside the rounded rectangle
        // Extended to get as close as possible to the corners
        let is_inside_rounded_rect = |x: f32, y: f32| -> bool {
            // First check if we're inside the basic rectangle bounds
            if x < rect.left() || x > rect.right() || y < rect.top() || y > rect.bottom() {
                return false;
            }
            
            // Check corners - slightly more permissive to fill remaining gaps
            let tolerance = 0.5; // Small tolerance to push grid closer to edges
            if x < rect.left() + radius && y < rect.top() + radius {
                // Top-left corner
                let dx = x - (rect.left() + radius);
                let dy = y - (rect.top() + radius);
                dx * dx + dy * dy <= (radius + tolerance) * (radius + tolerance)
            } else if x > rect.right() - radius && y < rect.top() + radius {
                // Top-right corner
                let dx = x - (rect.right() - radius);
                let dy = y - (rect.top() + radius);
                dx * dx + dy * dy <= (radius + tolerance) * (radius + tolerance)
            } else if x > rect.right() - radius && y > rect.bottom() - radius {
                // Bottom-right corner
                let dx = x - (rect.right() - radius);
                let dy = y - (rect.bottom() - radius);
                dx * dx + dy * dy <= (radius + tolerance) * (radius + tolerance)
            } else if x < rect.left() + radius && y > rect.bottom() - radius {
                // Bottom-left corner
                let dx = x - (rect.left() + radius);
                let dy = y - (rect.bottom() - radius);
                dx * dx + dy * dy <= (radius + tolerance) * (radius + tolerance)
            } else {
                // Inside the main rectangle area
                true
            }
        };
        
        // Create a non-uniform grid with high density at edges/corners
        let mut vertex_grid = Vec::new();
        
        // Define non-uniform column positions - dense at edges, sparse in middle
        let mut col_positions = Vec::new();
        let edge_density = 12; // High density near edges
        let middle_density = 3; // Low density in middle
        
        // Left edge high density (0 to 0.3)
        for i in 0..=edge_density {
            let t = i as f32 / edge_density as f32;
            col_positions.push(t * 0.3);
        }
        
        // Middle low density (0.3 to 0.7) 
        for i in 1..middle_density {
            let t = i as f32 / middle_density as f32;
            col_positions.push(0.3 + t * 0.4);
        }
        
        // Right edge high density (0.7 to 1.0)
        for i in 0..=edge_density {
            let t = i as f32 / edge_density as f32;
            col_positions.push(0.7 + t * 0.3);
        }
        
        // Define non-uniform row positions - extra dense at edges, minimal in middle  
        let mut row_positions = Vec::new();
        let top_bottom_density = 16; // Even higher density for top/bottom edges
        let vertical_middle_density = 2; // Even sparser middle
        
        // Top edge extra high density (0 to 0.25) - smaller edge zone, higher density
        for i in 0..=top_bottom_density {
            let t = i as f32 / top_bottom_density as f32;
            row_positions.push(t * 0.25);
        }
        
        // Middle minimal density (0.25 to 0.75)
        for i in 1..vertical_middle_density {
            let t = i as f32 / vertical_middle_density as f32;
            row_positions.push(0.25 + t * 0.5);
        }
        
        // Bottom edge extra high density (0.75 to 1.0) - smaller edge zone, higher density
        for i in 0..=top_bottom_density {
            let t = i as f32 / top_bottom_density as f32;
            row_positions.push(0.75 + t * 0.25);
        }
        
        // Create vertices using non-uniform grid
        for (_row_idx, &row_t) in row_positions.iter().enumerate() {
            let mut row_vertices = Vec::new();
            let y = rect.top() + row_t * rect.height();
            
            for (_col_idx, &col_t) in col_positions.iter().enumerate() {
                let x = rect.left() + col_t * rect.width();
                
                if is_inside_rounded_rect(x, y) {
                    let color = interpolate_color(y);
                    mesh.colored_vertex(egui::pos2(x, y), color);
                    row_vertices.push(Some(mesh.vertices.len() - 1));
                } else {
                    row_vertices.push(None);
                }
            }
            vertex_grid.push(row_vertices);
        }
        
        // Update grid dimensions for triangle creation
        let grid_height = row_positions.len() - 1;
        let grid_width = col_positions.len() - 1;
        
        // Add triangle fans for each corner underneath the grid to fill gaps
        let segments = 16; // Increased for smoother corner curves
        
        // Top-left corner triangle fan
        let tl_center_x = rect.left() + radius;
        let tl_center_y = rect.top() + radius;
        let tl_center_color = interpolate_color(tl_center_y);
        mesh.colored_vertex(egui::pos2(tl_center_x, tl_center_y), tl_center_color);
        let tl_center_idx = mesh.vertices.len() - 1;
        
        let mut tl_perimeter = Vec::new();
        for i in 0..=segments {
            let angle = std::f32::consts::PI + (i as f32 / segments as f32) * (std::f32::consts::PI / 2.0);
            let x = tl_center_x + radius * angle.cos();
            let y = tl_center_y + radius * angle.sin();
            let color = interpolate_color(y);
            mesh.colored_vertex(egui::pos2(x, y), color);
            tl_perimeter.push(mesh.vertices.len() - 1);
        }
        
        for i in 0..segments {
            mesh.add_triangle(
                tl_center_idx as u32,
                tl_perimeter[i] as u32,
                tl_perimeter[i + 1] as u32,
            );
        }
        
        // Top-right corner triangle fan
        let tr_center_x = rect.right() - radius;
        let tr_center_y = rect.top() + radius;
        let tr_center_color = interpolate_color(tr_center_y);
        mesh.colored_vertex(egui::pos2(tr_center_x, tr_center_y), tr_center_color);
        let tr_center_idx = mesh.vertices.len() - 1;
        
        let mut tr_perimeter = Vec::new();
        for i in 0..=segments {
            let angle = 1.5 * std::f32::consts::PI + (i as f32 / segments as f32) * (std::f32::consts::PI / 2.0);
            let x = tr_center_x + radius * angle.cos();
            let y = tr_center_y + radius * angle.sin();
            let color = interpolate_color(y);
            mesh.colored_vertex(egui::pos2(x, y), color);
            tr_perimeter.push(mesh.vertices.len() - 1);
        }
        
        for i in 0..segments {
            mesh.add_triangle(
                tr_center_idx as u32,
                tr_perimeter[i] as u32,
                tr_perimeter[i + 1] as u32,
            );
        }
        
        // Bottom-right corner triangle fan
        let br_center_x = rect.right() - radius;
        let br_center_y = rect.bottom() - radius;
        let br_center_color = interpolate_color(br_center_y);
        mesh.colored_vertex(egui::pos2(br_center_x, br_center_y), br_center_color);
        let br_center_idx = mesh.vertices.len() - 1;
        
        let mut br_perimeter = Vec::new();
        for i in 0..=segments {
            let angle = 0.0 + (i as f32 / segments as f32) * (std::f32::consts::PI / 2.0);
            let x = br_center_x + radius * angle.cos();
            let y = br_center_y + radius * angle.sin();
            let color = interpolate_color(y);
            mesh.colored_vertex(egui::pos2(x, y), color);
            br_perimeter.push(mesh.vertices.len() - 1);
        }
        
        for i in 0..segments {
            mesh.add_triangle(
                br_center_idx as u32,
                br_perimeter[i] as u32,
                br_perimeter[i + 1] as u32,
            );
        }
        
        // Bottom-left corner triangle fan
        let bl_center_x = rect.left() + radius;
        let bl_center_y = rect.bottom() - radius;
        let bl_center_color = interpolate_color(bl_center_y);
        mesh.colored_vertex(egui::pos2(bl_center_x, bl_center_y), bl_center_color);
        let bl_center_idx = mesh.vertices.len() - 1;
        
        let mut bl_perimeter = Vec::new();
        for i in 0..=segments {
            let angle = 0.5 * std::f32::consts::PI + (i as f32 / segments as f32) * (std::f32::consts::PI / 2.0);
            let x = bl_center_x + radius * angle.cos();
            let y = bl_center_y + radius * angle.sin();
            let color = interpolate_color(y);
            mesh.colored_vertex(egui::pos2(x, y), color);
            bl_perimeter.push(mesh.vertices.len() - 1);
        }
        
        for i in 0..segments {
            mesh.add_triangle(
                bl_center_idx as u32,
                bl_perimeter[i] as u32,
                bl_perimeter[i + 1] as u32,
            );
        }
        
        // Create triangles from the grid
        for row in 0..grid_height {
            for col in 0..grid_width {
                if let (Some(v0), Some(v1), Some(v2), Some(v3)) = (
                    vertex_grid[row][col],
                    vertex_grid[row][col + 1],
                    vertex_grid[row + 1][col],
                    vertex_grid[row + 1][col + 1],
                ) {
                    // Create two triangles for this grid cell
                    mesh.add_triangle(v0 as u32, v2 as u32, v1 as u32);
                    mesh.add_triangle(v1 as u32, v2 as u32, v3 as u32);
                }
            }
        }
        
        mesh
    }

    /// Create a rounded rectangle mesh with vertical gradient using optimized 16-vertex grid
    /// Performance note: This creates exactly 16 vertices and 18 triangles per node
    pub fn create_rounded_gradient_mesh_optimized(
        rect: Rect,
        radius: f32,
        top_color: Color32,
        bottom_color: Color32,
    ) -> egui::Mesh {
        let mut mesh = egui::Mesh::default();
        
        // Optimized approach: 4x4 grid (16 vertices) that avoids corner radius areas
        let grid_size = 4; // 4x4 = 16 vertices total
        
        // Function to interpolate color based on Y position
        let interpolate_color = |y: f32| -> Color32 {
            let t = (y - rect.top()) / rect.height();
            Color32::from_rgb(
                (top_color.r() as f32 * (1.0 - t) + bottom_color.r() as f32 * t) as u8,
                (top_color.g() as f32 * (1.0 - t) + bottom_color.g() as f32 * t) as u8,
                (top_color.b() as f32 * (1.0 - t) + bottom_color.b() as f32 * t) as u8,
            )
        };
        
        // Create 4x4 grid with vertices positioned at corner radius boundaries
        let mut vertices = Vec::new();
        
        // Define the x positions: left edge, left+radius, right-radius, right edge
        let x_positions = [
            rect.left(),                    // Column 0: left edge
            rect.left() + radius,           // Column 1: left + radius
            rect.right() - radius,          // Column 2: right - radius  
            rect.right(),                   // Column 3: right edge
        ];
        
        // Define the y positions: top edge, top+radius, bottom-radius, bottom edge
        let y_positions = [
            rect.top(),                     // Row 0: top edge
            rect.top() + radius,            // Row 1: top + radius
            rect.bottom() - radius,         // Row 2: bottom - radius
            rect.bottom(),                  // Row 3: bottom edge
        ];
        
        for row in 0..grid_size {
            for col in 0..grid_size {
                let x = x_positions[col];
                let y = y_positions[row];
                
                // All vertices use gradient color - transparency will be applied per triangle
                let color = interpolate_color(y);
                
                mesh.colored_vertex(egui::pos2(x, y), color);
                vertices.push(mesh.vertices.len() - 1);
            }
        }
        
        // Create triangles from the 4x4 grid, making corner polygons transparent
        for row in 0..(grid_size - 1) {
            for col in 0..(grid_size - 1) {
                let top_left_idx = vertices[row * grid_size + col];
                let top_right_idx = vertices[row * grid_size + col + 1];
                let bottom_left_idx = vertices[(row + 1) * grid_size + col];
                let bottom_right_idx = vertices[(row + 1) * grid_size + col + 1];
                
                // Check if this is a corner polygon that should be transparent
                let is_corner_polygon = (row == 0 && col == 0) ||                    // Top-left (vertices 1,2,5,6)
                                       (row == 0 && col == grid_size - 2) ||         // Top-right (vertices 3,4,7,8)
                                       (row == grid_size - 2 && col == 0) ||         // Bottom-left (vertices 9,10,13,14)
                                       (row == grid_size - 2 && col == grid_size - 2); // Bottom-right (vertices 11,12,15,16)
                
                if is_corner_polygon {
                    // For corner polygons, create transparent versions by adding transparent vertices
                    // at the same positions but with transparent color
                    let tl_pos = egui::pos2(x_positions[col], y_positions[row]);
                    let tr_pos = egui::pos2(x_positions[col + 1], y_positions[row]);
                    let bl_pos = egui::pos2(x_positions[col], y_positions[row + 1]);
                    let br_pos = egui::pos2(x_positions[col + 1], y_positions[row + 1]);
                    
                    mesh.colored_vertex(tl_pos, Color32::TRANSPARENT);
                    let tl_transparent = mesh.vertices.len() - 1;
                    
                    mesh.colored_vertex(tr_pos, Color32::TRANSPARENT);
                    let tr_transparent = mesh.vertices.len() - 1;
                    
                    mesh.colored_vertex(bl_pos, Color32::TRANSPARENT);
                    let bl_transparent = mesh.vertices.len() - 1;
                    
                    mesh.colored_vertex(br_pos, Color32::TRANSPARENT);
                    let br_transparent = mesh.vertices.len() - 1;
                    
                    // Create the transparent triangles
                    mesh.add_triangle(tl_transparent as u32, bl_transparent as u32, tr_transparent as u32);
                    mesh.add_triangle(tr_transparent as u32, bl_transparent as u32, br_transparent as u32);
                } else {
                    // Normal triangles for non-corner polygons
                    mesh.add_triangle(top_left_idx as u32, bottom_left_idx as u32, top_right_idx as u32);
                    mesh.add_triangle(top_right_idx as u32, bottom_left_idx as u32, bottom_right_idx as u32);
                }
            }
        }
        
        // Create top-left rounded corner triangle fan
        // Center: vertex 6 (row=1, col=1) at position (left + radius, top + radius)
        let center_vertex_6 = vertices[1 * grid_size + 1]; // Row 1, Col 1 - vertex 6
        let center_pos = egui::pos2(x_positions[1], y_positions[1]); // left + radius, top + radius
        
        // Create arc vertices for the rounded corner
        let segments = 6; // 6 segments for optimal balance of smoothness and performance
        let mut arc_vertices = Vec::new();
        
        for i in 0..=segments {
            let angle = std::f32::consts::PI + (i as f32 / segments as f32) * (std::f32::consts::PI / 2.0);
            let x = center_pos.x + radius * angle.cos();
            let y = center_pos.y + radius * angle.sin();
            let color = interpolate_color(y);
            
            mesh.colored_vertex(egui::pos2(x, y), color);
            arc_vertices.push(mesh.vertices.len() - 1);
        }
        
        // Create triangle fan from center to arc vertices
        for i in 0..segments {
            mesh.add_triangle(
                center_vertex_6 as u32,
                arc_vertices[i] as u32,
                arc_vertices[i + 1] as u32,
            );
        }
        
        // Top-right triangle fan: converges on vertex 7 (row=1, col=2)
        let center_vertex_7 = vertices[1 * grid_size + 2]; // Row 1, Col 2 - vertex 7
        let center_pos_tr = egui::pos2(x_positions[2], y_positions[1]); // right - radius, top + radius
        
        let mut arc_vertices_tr = Vec::new();
        for i in 0..=segments {
            let angle = 1.5 * std::f32::consts::PI + (i as f32 / segments as f32) * (std::f32::consts::PI / 2.0);
            let x = center_pos_tr.x + radius * angle.cos();
            let y = center_pos_tr.y + radius * angle.sin();
            let color = interpolate_color(y);
            
            mesh.colored_vertex(egui::pos2(x, y), color);
            arc_vertices_tr.push(mesh.vertices.len() - 1);
        }
        
        for i in 0..segments {
            mesh.add_triangle(
                center_vertex_7 as u32,
                arc_vertices_tr[i] as u32,
                arc_vertices_tr[i + 1] as u32,
            );
        }
        
        // Bottom-right triangle fan: converges on vertex 11 (row=2, col=2)
        let center_vertex_11 = vertices[2 * grid_size + 2]; // Row 2, Col 2 - vertex 11
        let center_pos_br = egui::pos2(x_positions[2], y_positions[2]); // right - radius, bottom - radius
        
        let mut arc_vertices_br = Vec::new();
        for i in 0..=segments {
            let angle = 0.0 + (i as f32 / segments as f32) * (std::f32::consts::PI / 2.0);
            let x = center_pos_br.x + radius * angle.cos();
            let y = center_pos_br.y + radius * angle.sin();
            let color = interpolate_color(y);
            
            mesh.colored_vertex(egui::pos2(x, y), color);
            arc_vertices_br.push(mesh.vertices.len() - 1);
        }
        
        for i in 0..segments {
            mesh.add_triangle(
                center_vertex_11 as u32,
                arc_vertices_br[i] as u32,
                arc_vertices_br[i + 1] as u32,
            );
        }
        
        // Bottom-left triangle fan: converges on vertex 10 (row=2, col=1)
        let center_vertex_10 = vertices[2 * grid_size + 1]; // Row 2, Col 1 - vertex 10
        let center_pos_bl = egui::pos2(x_positions[1], y_positions[2]); // left + radius, bottom - radius
        
        let mut arc_vertices_bl = Vec::new();
        for i in 0..=segments {
            let angle = 0.5 * std::f32::consts::PI + (i as f32 / segments as f32) * (std::f32::consts::PI / 2.0);
            let x = center_pos_bl.x + radius * angle.cos();
            let y = center_pos_bl.y + radius * angle.sin();
            let color = interpolate_color(y);
            
            mesh.colored_vertex(egui::pos2(x, y), color);
            arc_vertices_bl.push(mesh.vertices.len() - 1);
        }
        
        for i in 0..segments {
            mesh.add_triangle(
                center_vertex_10 as u32,
                arc_vertices_bl[i] as u32,
                arc_vertices_bl[i + 1] as u32,
            );
        }
        
        mesh
    }

    /// Create a simple rectangular gradient mesh (much faster than complex rounded mesh)
    pub fn create_simple_gradient_mesh(
        rect: Rect,
        top_color: Color32,
        bottom_color: Color32,
    ) -> egui::Mesh {
        let mut mesh = egui::Mesh::default();
        
        // Simple 2x2 grid for gradient
        let positions = [
            rect.left_top(),    // Top-left
            rect.right_top(),   // Top-right
            rect.left_bottom(), // Bottom-left
            rect.right_bottom() // Bottom-right
        ];
        
        let colors = [
            top_color,    // Top-left
            top_color,    // Top-right
            bottom_color, // Bottom-left
            bottom_color  // Bottom-right
        ];
        
        // Add vertices
        for (pos, color) in positions.iter().zip(colors.iter()) {
            mesh.colored_vertex(*pos, *color);
        }
        
        // Add triangles (two triangles make a rectangle)
        mesh.add_triangle(0, 1, 2); // Top-left triangle
        mesh.add_triangle(1, 3, 2); // Bottom-right triangle
        
        mesh
    }

    /// Mask the corners of a rectangle to create rounded appearance
    pub fn mask_corners_for_rounded_rect(
        painter: &egui::Painter,
        rect: Rect,
        radius: f32,
        bg_color: Color32,
    ) {
        let corner_size = radius;
        
        // Top-left corner mask
        let tl_rect = Rect::from_min_size(rect.min, Vec2::splat(corner_size));
        Self::draw_corner_mask(painter, tl_rect, bg_color, 0); // Top-left quadrant
        
        // Top-right corner mask
        let tr_rect = Rect::from_min_size(
            Pos2::new(rect.right() - corner_size, rect.top()),
            Vec2::splat(corner_size)
        );
        Self::draw_corner_mask(painter, tr_rect, bg_color, 1); // Top-right quadrant
        
        // Bottom-left corner mask
        let bl_rect = Rect::from_min_size(
            Pos2::new(rect.left(), rect.bottom() - corner_size),
            Vec2::splat(corner_size)
        );
        Self::draw_corner_mask(painter, bl_rect, bg_color, 2); // Bottom-left quadrant
        
        // Bottom-right corner mask
        let br_rect = Rect::from_min_size(
            Pos2::new(rect.right() - corner_size, rect.bottom() - corner_size),
            Vec2::splat(corner_size)
        );
        Self::draw_corner_mask(painter, br_rect, bg_color, 3); // Bottom-right quadrant
    }

    /// Draw a corner mask for the specified quadrant (0=TL, 1=TR, 2=BL, 3=BR)
    pub fn draw_corner_mask(
        painter: &egui::Painter,
        corner_rect: Rect,
        bg_color: Color32,
        quadrant: u8,
    ) {
        let center = match quadrant {
            0 => corner_rect.max,         // Top-left: circle center at bottom-right
            1 => corner_rect.left_bottom(), // Top-right: circle center at bottom-left
            2 => corner_rect.right_top(),   // Bottom-left: circle center at top-right
            3 => corner_rect.min,         // Bottom-right: circle center at top-left
            _ => corner_rect.center(),
        };
        
        let radius = corner_rect.width(); // Should be same as height
        
        // Create a mesh to fill the corner area outside the circle
        let mut mask_mesh = egui::Mesh::default();
        let segments = 8;
        
        // Add corner rectangle vertices
        mask_mesh.colored_vertex(corner_rect.min, bg_color);
        mask_mesh.colored_vertex(corner_rect.right_top(), bg_color);
        mask_mesh.colored_vertex(corner_rect.left_bottom(), bg_color);
        mask_mesh.colored_vertex(corner_rect.max, bg_color);
        
        // Add circle arc vertices
        let start_angle = match quadrant {
            0 => std::f32::consts::PI,           // Top-left
            1 => 1.5 * std::f32::consts::PI,     // Top-right
            2 => 0.5 * std::f32::consts::PI,     // Bottom-left
            3 => 0.0,                            // Bottom-right
            _ => 0.0,
        };
        
        for i in 0..=segments {
            let angle = start_angle + (i as f32 / segments as f32) * (std::f32::consts::PI / 2.0);
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            mask_mesh.colored_vertex(Pos2::new(x, y), bg_color);
        }
        
        // Create triangles to fill the area outside the circle
        // This is a simplified approach - in practice you'd want more precise masking
        mask_mesh.add_triangle(0, 1, 2);
        mask_mesh.add_triangle(1, 2, 3);
        
        painter.add(egui::Shape::mesh(mask_mesh));
    }

    /// Create a rounded rectangle border mesh with vertical gradient
    pub fn create_rounded_gradient_border_mesh(
        rect: Rect,
        radius: f32,
        thickness: f32,
        top_color: Color32,
        bottom_color: Color32,
    ) -> egui::Mesh {
        let mut mesh = egui::Mesh::default();
        
        let segments = 8; // Number of segments for rounded corners
        let inner_rect = Rect::from_min_max(
            rect.min + Vec2::splat(thickness),
            rect.max - Vec2::splat(thickness),
        );
        let inner_radius = (radius - thickness).max(0.0);
        
        // Function to interpolate color based on Y position
        let interpolate_color = |y: f32| -> Color32 {
            let t = (y - rect.top()) / rect.height();
            Color32::from_rgb(
                (top_color.r() as f32 * (1.0 - t) + bottom_color.r() as f32 * t) as u8,
                (top_color.g() as f32 * (1.0 - t) + bottom_color.g() as f32 * t) as u8,
                (top_color.b() as f32 * (1.0 - t) + bottom_color.b() as f32 * t) as u8,
            )
        };
        
        let mut outer_vertices = Vec::new();
        let mut inner_vertices = Vec::new();
        
        // Create vertices around the perimeter
        // Top-left corner
        for i in 0..=segments {
            let angle = std::f32::consts::PI + (i as f32 / segments as f32) * (std::f32::consts::PI / 2.0);
            let outer_x = rect.left() + radius + radius * angle.cos();
            let outer_y = rect.top() + radius + radius * angle.sin();
            let color = interpolate_color(outer_y);
            mesh.colored_vertex(egui::pos2(outer_x, outer_y), color);
            outer_vertices.push(mesh.vertices.len() - 1);
            
            let inner_x = inner_rect.left() + inner_radius + inner_radius * angle.cos();
            let inner_y = inner_rect.top() + inner_radius + inner_radius * angle.sin();
            mesh.colored_vertex(egui::pos2(inner_x, inner_y), color);
            inner_vertices.push(mesh.vertices.len() - 1);
        }
        
        // Top edge
        let steps = 4;
        for i in 1..steps {
            let t = i as f32 / steps as f32;
            let x = rect.left() + radius + t * (rect.width() - 2.0 * radius);
            mesh.colored_vertex(egui::pos2(x, rect.top()), top_color);
            outer_vertices.push(mesh.vertices.len() - 1);
            mesh.colored_vertex(egui::pos2(x, inner_rect.top()), top_color);
            inner_vertices.push(mesh.vertices.len() - 1);
        }
        
        // Top-right corner
        for i in 0..=segments {
            let angle = 1.5 * std::f32::consts::PI + (i as f32 / segments as f32) * (std::f32::consts::PI / 2.0);
            let outer_x = rect.right() - radius + radius * angle.cos();
            let outer_y = rect.top() + radius + radius * angle.sin();
            let color = interpolate_color(outer_y);
            mesh.colored_vertex(egui::pos2(outer_x, outer_y), color);
            outer_vertices.push(mesh.vertices.len() - 1);
            
            let inner_x = inner_rect.right() - inner_radius + inner_radius * angle.cos();
            let inner_y = inner_rect.top() + inner_radius + inner_radius * angle.sin();
            mesh.colored_vertex(egui::pos2(inner_x, inner_y), color);
            inner_vertices.push(mesh.vertices.len() - 1);
        }
        
        // Right edge
        for i in 1..steps {
            let t = i as f32 / steps as f32;
            let y = rect.top() + radius + t * (rect.height() - 2.0 * radius);
            let color = interpolate_color(y);
            mesh.colored_vertex(egui::pos2(rect.right(), y), color);
            outer_vertices.push(mesh.vertices.len() - 1);
            mesh.colored_vertex(egui::pos2(inner_rect.right(), y), color);
            inner_vertices.push(mesh.vertices.len() - 1);
        }
        
        // Bottom-right corner
        for i in 0..=segments {
            let angle = 0.0 + (i as f32 / segments as f32) * (std::f32::consts::PI / 2.0);
            let outer_x = rect.right() - radius + radius * angle.cos();
            let outer_y = rect.bottom() - radius + radius * angle.sin();
            let color = interpolate_color(outer_y);
            mesh.colored_vertex(egui::pos2(outer_x, outer_y), color);
            outer_vertices.push(mesh.vertices.len() - 1);
            
            let inner_x = inner_rect.right() - inner_radius + inner_radius * angle.cos();
            let inner_y = inner_rect.bottom() - inner_radius + inner_radius * angle.sin();
            mesh.colored_vertex(egui::pos2(inner_x, inner_y), color);
            inner_vertices.push(mesh.vertices.len() - 1);
        }
        
        // Bottom edge
        for i in 1..steps {
            let t = 1.0 - (i as f32 / steps as f32); // Reverse direction
            let x = rect.right() - radius - t * (rect.width() - 2.0 * radius);
            mesh.colored_vertex(egui::pos2(x, rect.bottom()), bottom_color);
            outer_vertices.push(mesh.vertices.len() - 1);
            mesh.colored_vertex(egui::pos2(x, inner_rect.bottom()), bottom_color);
            inner_vertices.push(mesh.vertices.len() - 1);
        }
        
        // Bottom-left corner
        for i in 0..=segments {
            let angle = 0.5 * std::f32::consts::PI + (i as f32 / segments as f32) * (std::f32::consts::PI / 2.0);
            let outer_x = rect.left() + radius + radius * angle.cos();
            let outer_y = rect.bottom() - radius + radius * angle.sin();
            let color = interpolate_color(outer_y);
            mesh.colored_vertex(egui::pos2(outer_x, outer_y), color);
            outer_vertices.push(mesh.vertices.len() - 1);
            
            let inner_x = inner_rect.left() + inner_radius + inner_radius * angle.cos();
            let inner_y = inner_rect.bottom() - inner_radius + inner_radius * angle.sin();
            mesh.colored_vertex(egui::pos2(inner_x, inner_y), color);
            inner_vertices.push(mesh.vertices.len() - 1);
        }
        
        // Left edge
        for i in 1..steps {
            let t = i as f32 / steps as f32;
            let y = rect.bottom() - radius - t * (rect.height() - 2.0 * radius);
            let color = interpolate_color(y);
            mesh.colored_vertex(egui::pos2(rect.left(), y), color);
            outer_vertices.push(mesh.vertices.len() - 1);
            mesh.colored_vertex(egui::pos2(inner_rect.left(), y), color);
            inner_vertices.push(mesh.vertices.len() - 1);
        }
        
        // Create triangles for the border (between outer and inner vertices)
        for i in 0..outer_vertices.len() {
            let next_i = (i + 1) % outer_vertices.len();
            
            // Two triangles per quad
            mesh.add_triangle(
                outer_vertices[i] as u32,
                inner_vertices[i] as u32,
                outer_vertices[next_i] as u32,
            );
            mesh.add_triangle(
                inner_vertices[i] as u32,
                inner_vertices[next_i] as u32,
                outer_vertices[next_i] as u32,
            );
        }
        
        mesh
    }
    
    /// Render a circular radial button for Viewport nodes
    /// Positioned with equal distance from left, top, and bottom edges
    pub fn render_viewport_radial_button(
        painter: &Painter,
        node: &Node,
        zoom: f32,
        transform_pos: &impl Fn(Pos2) -> Pos2,
    ) {
        // Calculate button size and position
        let button_radius = 10.0 * zoom; // Larger than port radius (5.0)
        let margin = button_radius; // Equal distance from edges
        
        // Left button position: equal distance from left, top, and bottom edges
        let left_button_center = Pos2::new(
            node.position.x + margin + button_radius, // Left edge + margin + radius
            node.position.y + node.size.y / 2.0,     // Centered vertically
        );
        
        // Right button position: equal distance from right, top, and bottom edges
        let right_button_center = Pos2::new(
            node.position.x + node.size.x - margin - button_radius, // Right edge - margin - radius
            node.position.y + node.size.y / 2.0,                   // Centered vertically
        );
        
        let left_transformed_center = transform_pos(left_button_center);
        let right_transformed_center = transform_pos(right_button_center);
        
        // Draw button overlays - adjacent on the left side  
        let button_width = 10.0 * zoom;
        
        // Left button overlay (green) if active - first 10px  
        if node.button_states[0] {
            let left_button_rect = Rect::from_min_size(
                transform_pos(node.position), 
                Vec2::new(button_width, node.size.y * zoom)
            );
            painter.rect_filled(left_button_rect, 0.0, Color32::from_rgb(0, 255, 0));
        }
        
        // Right button overlay (red) if active - second 10px (adjacent to green)
        if node.button_states[1] {
            let right_button_rect = Rect::from_min_size(
                transform_pos(Pos2::new(node.position.x + 10.0, node.position.y)), 
                Vec2::new(button_width, node.size.y * zoom)
            );
            painter.rect_filled(right_button_rect, 0.0, Color32::from_rgb(255, 0, 0));
        }
    }
    
    /// Create a circular mesh with colorized radial gradient matching the GPU implementation
    fn render_colorized_radial_gradient_circle(
        painter: &Painter,
        center: Pos2,
        radius: f32,
        color_type: ButtonColorType,
        is_active: bool,
    ) {
        use egui::epaint::Mesh;
        
        // Use hardcoded working colors - don't overcomplicate
        let (center_color, outer_color) = match color_type {
            ButtonColorType::Green => {
                if is_active {
                    (Color32::from_rgb(120, 200, 120), Color32::from_rgb(60, 120, 60))
                } else {
                    (Color32::from_rgb(90, 160, 90), Color32::from_rgb(45, 90, 45))
                }
            }
            ButtonColorType::Red => {
                if is_active {
                    (Color32::from_rgb(200, 120, 120), Color32::from_rgb(120, 60, 60))
                } else {
                    (Color32::from_rgb(160, 90, 90), Color32::from_rgb(90, 45, 45))
                }
            }
        };
        
        let mut mesh = Mesh::default();
        let segments = 32; // Smooth circle
        
        // Center vertex (bright)
        mesh.colored_vertex(center, center_color);
        let center_idx = 0;
        
        // Outer ring vertices (darker)
        let mut outer_indices = Vec::new();
        
        for i in 0..segments {
            let angle = (i as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            mesh.colored_vertex(Pos2::new(x, y), outer_color);
            outer_indices.push(mesh.vertices.len() - 1);
        }
        
        // Create triangles from center to outer ring
        for i in 0..segments {
            let next_i = (i + 1) % segments;
            mesh.add_triangle(
                center_idx as u32,
                outer_indices[i] as u32,
                outer_indices[next_i] as u32,
            );
        }
        
        painter.add(egui::Shape::mesh(mesh));
    }
    
    /// Colorize the gradient top color (CPU version matching GPU)
    fn colorize_gradient_top_cpu(base_color: Color32, color: ButtonColorType) -> Color32 {
        let luminance = base_color.r() as f32 / 255.0; // Use red channel as luminance since it's grey
        match color {
            ButtonColorType::Green => {
                // Green tint with much stronger colorization and brightness boost (matches GPU)
                Color32::from_rgb(
                    (luminance * 0.3 * 255.0) as u8,  // Much darker red
                    ((luminance * 1.5 + 0.2) * 255.0).min(255.0) as u8, // Significantly boosted green + brightness
                    (luminance * 0.3 * 255.0) as u8,  // Much darker blue
                )
            }
            ButtonColorType::Red => {
                // Red tint with much stronger colorization and brightness boost (matches GPU)
                Color32::from_rgb(
                    ((luminance * 1.5 + 0.2) * 255.0).min(255.0) as u8, // Significantly boosted red + brightness
                    (luminance * 0.2 * 255.0) as u8,  // Much darker green
                    (luminance * 0.2 * 255.0) as u8,  // Much darker blue
                )
            }
        }
    }
    
    /// Colorize the gradient bottom color (CPU version matching GPU)
    fn colorize_gradient_bottom_cpu(base_color: Color32, color: ButtonColorType) -> Color32 {
        let luminance = base_color.r() as f32 / 255.0; // Use red channel as luminance since it's grey
        match color {
            ButtonColorType::Green => {
                // Green tint with much stronger colorization (much darker for contrast) - matches GPU
                Color32::from_rgb(
                    (luminance * 0.1 * 255.0) as u8,  // Very dark red
                    (luminance * 0.8 * 255.0) as u8,  // Reduced green (darker base)
                    (luminance * 0.1 * 255.0) as u8,  // Very dark blue
                )
            }
            ButtonColorType::Red => {
                // Red tint with much stronger colorization (much darker for contrast) - matches GPU
                Color32::from_rgb(
                    (luminance * 0.8 * 255.0) as u8,  // Reduced red (darker base)
                    (luminance * 0.1 * 255.0) as u8,  // Very dark green
                    (luminance * 0.1 * 255.0) as u8,  // Very dark blue
                )
            }
        }
    }
    
    /// Legacy function for backwards compatibility
    fn render_radial_gradient_circle(
        painter: &Painter,
        center: Pos2,
        radius: f32,
    ) {
        // Use green colorization as default
        Self::render_colorized_radial_gradient_circle(painter, center, radius, ButtonColorType::Green, true);
    }
}

impl Default for MeshRenderer {
    fn default() -> Self {
        Self::new()
    }
}