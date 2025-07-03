//! CPU rendering for the 2D canvas node editor

use egui::{Color32, Pos2, Rect, Vec2, Painter, Stroke};
use crate::nodes::Node;
use crate::theme;

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
        let radius = theme::dimensions().corner_radius * zoom;
        
        // BACKGROUND: Inner gradient mesh - same for all nodes
        let (background_top_color, background_bottom_color) = (theme::colors().node_bg_top, theme::colors().node_bg_bottom);
        
        // BORDER: Outermost layer (1px larger than node rect, scaled by zoom)
        let border_expand = theme::dimensions().border_width * zoom;
        let border_rect = Rect::from_min_max(
            transformed_rect.min - Vec2::splat(border_expand),
            transformed_rect.max + Vec2::splat(border_expand),
        );
        let border_color = if selected {
            theme::colors().selection_blue // Blue selection
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
        let port_radius = theme::dimensions().corner_radius * zoom;
        let transformed_pos = transform_pos(port_pos);
        
        // Draw port border (2px larger) - blue if connecting, grey otherwise
        let port_border_color = if is_connecting {
            theme::colors().selection_blue // Blue selection color
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
            theme::colors().port_input // Darker green for input ports
        } else {
            theme::colors().port_output // Darker red for output ports
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
            theme::colors().selection_blue // Blue selection color when enabled
        } else {
            Color32::from_rgb(64, 64, 64) // Grey when disabled
        };
        
        let border_radius = theme::dimensions().corner_radius * zoom + 2.0 * zoom;
        painter.circle_stroke(
            transformed_pos,
            border_radius,
            Stroke::new(1.0 * zoom, border_color),
        );
        
        // Draw bevel outline (inner layer) - 1px smaller than border
        let bevel_radius = theme::dimensions().corner_radius * zoom + 1.0 * zoom;
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
}

impl Default for MeshRenderer {
    fn default() -> Self {
        Self::new()
    }
}