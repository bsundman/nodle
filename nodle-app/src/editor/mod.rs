//! Node editor implementation

use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use egui_wgpu;
use nodle_core::{
    graph::NodeGraph,
    node::NodeId,
    port::PortId,
};
use std::collections::{HashMap, HashSet};
use crate::context::{ContextManager, ContextMenuItem};
use crate::contexts::materialx::MaterialXContext;
use crate::gpu_renderer::NodeRenderCallback;

/// Main application state for the node editor
pub struct NodeEditor {
    graph: NodeGraph,
    connecting_from: Option<(NodeId, PortId, bool)>, // (node_id, port_id, is_input)
    selected_nodes: HashSet<NodeId>,
    selected_connection: Option<usize>,
    context_menu_pos: Option<Pos2>,
    open_submenu: Option<String>, // Track which submenu is open
    submenu_pos: Option<Pos2>,    // Position for the submenu
    pan_offset: Vec2,
    zoom: f32,
    box_selection_start: Option<Pos2>,
    box_selection_end: Option<Pos2>,
    drag_offsets: HashMap<NodeId, Vec2>,
    context_manager: ContextManager,
    // Performance tracking
    show_performance_info: bool,
    frame_times: Vec<f32>,
    last_frame_time: std::time::Instant,
    // GPU rendering toggle
    use_gpu_rendering: bool,
}

impl NodeEditor {
    /// Create a rounded rectangle mesh with vertical gradient using optimized 16-vertex grid
    /// Performance note: This creates exactly 16 vertices and 18 triangles per node
    fn create_rounded_gradient_mesh(
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
    fn create_rounded_gradient_mesh_optimized(
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
        
        // Function to check if a point is in a corner that should be transparent
        let is_in_corner = |x: f32, y: f32| -> bool {
            // Check if point is in a corner radius area
            if x < rect.left() + radius && y < rect.top() + radius {
                // Top-left corner
                let dx = x - (rect.left() + radius);
                let dy = y - (rect.top() + radius);
                dx * dx + dy * dy > radius * radius
            } else if x > rect.right() - radius && y < rect.top() + radius {
                // Top-right corner
                let dx = x - (rect.right() - radius);
                let dy = y - (rect.top() + radius);
                dx * dx + dy * dy > radius * radius
            } else if x > rect.right() - radius && y > rect.bottom() - radius {
                // Bottom-right corner
                let dx = x - (rect.right() - radius);
                let dy = y - (rect.bottom() - radius);
                dx * dx + dy * dy > radius * radius
            } else if x < rect.left() + radius && y > rect.bottom() - radius {
                // Bottom-left corner
                let dx = x - (rect.left() + radius);
                let dy = y - (rect.bottom() - radius);
                dx * dx + dy * dy > radius * radius
            } else {
                false // Not in any corner area
            }
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
    fn create_simple_gradient_mesh(
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
    fn mask_corners_for_rounded_rect(
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
    fn draw_corner_mask(
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
    fn create_rounded_gradient_border_mesh(
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

    pub fn new() -> Self {
        let mut context_manager = ContextManager::new();
        
        // Register MaterialX context
        context_manager.register_context(Box::new(MaterialXContext::new()));
        
        let mut editor = Self {
            graph: NodeGraph::new(),
            connecting_from: None,
            selected_nodes: HashSet::new(),
            selected_connection: None,
            context_menu_pos: None,
            open_submenu: None,
            submenu_pos: None,
            pan_offset: Vec2::ZERO,
            zoom: 1.0,
            box_selection_start: None,
            box_selection_end: None,
            drag_offsets: HashMap::new(),
            context_manager,
            // Performance tracking
            show_performance_info: false,
            frame_times: Vec::new(),
            last_frame_time: std::time::Instant::now(),
            // GPU rendering
            use_gpu_rendering: true, // Start with GPU rendering enabled
        };

        // Start with empty node graph - use F2/F3/F4 to add test nodes

        editor
    }
    
    /// Get the menu structure based on the active context
    fn get_menu_structure(&self) -> Vec<ContextMenuItem> {
        let mut menu_items = Vec::new();
        
        // Add context-specific menus if a context is active
        if let Some(context) = self.context_manager.get_active_context() {
            menu_items.extend(context.get_menu_structure());
        }
        
        // Always add generic node categories
        menu_items.push(ContextMenuItem::Category {
            name: "Math".to_string(),
            items: vec![
                ContextMenuItem::Node { name: "Add".to_string(), node_type: "Add".to_string() },
                ContextMenuItem::Node { name: "Subtract".to_string(), node_type: "Subtract".to_string() },
                ContextMenuItem::Node { name: "Multiply".to_string(), node_type: "Multiply".to_string() },
                ContextMenuItem::Node { name: "Divide".to_string(), node_type: "Divide".to_string() },
            ],
        });
        
        menu_items.push(ContextMenuItem::Category {
            name: "Logic".to_string(),
            items: vec![
                ContextMenuItem::Node { name: "AND".to_string(), node_type: "AND".to_string() },
                ContextMenuItem::Node { name: "OR".to_string(), node_type: "OR".to_string() },
                ContextMenuItem::Node { name: "NOT".to_string(), node_type: "NOT".to_string() },
            ],
        });
        
        menu_items.push(ContextMenuItem::Category {
            name: "Data".to_string(),
            items: vec![
                ContextMenuItem::Node { name: "Constant".to_string(), node_type: "Constant".to_string() },
                ContextMenuItem::Node { name: "Variable".to_string(), node_type: "Variable".to_string() },
            ],
        });
        
        menu_items.push(ContextMenuItem::Category {
            name: "Output".to_string(),
            items: vec![
                ContextMenuItem::Node { name: "Print".to_string(), node_type: "Print".to_string() },
                ContextMenuItem::Node { name: "Debug".to_string(), node_type: "Debug".to_string() },
            ],
        });
        
        menu_items
    }
    
    /// Check if a node should be marked as incompatible
    fn is_node_incompatible(&self, node: &nodle_core::Node) -> bool {
        if let Some(context) = self.context_manager.get_active_context() {
            // Check if it's a context-specific node
            if node.title.starts_with("Noise") || node.title.starts_with("Texture") || 
               node.title.starts_with("Mix") || node.title.starts_with("Standard Surface") ||
               node.title.contains("3D View") || node.title.contains("2D View") {
                // This is a MaterialX node, always compatible in MaterialX context
                return false;
            }
            
            // Check if generic node is compatible with this context
            !context.is_generic_node_compatible(&node.title)
        } else {
            false // No context active, all nodes are compatible
        }
    }

    fn zoom_at_point(&mut self, screen_point: Pos2, zoom_delta: f32) {
        // Convert screen point to world coordinates before zoom
        let world_point = Pos2::new(
            (screen_point.x - self.pan_offset.x) / self.zoom,
            (screen_point.y - self.pan_offset.y) / self.zoom,
        );

        // Apply zoom
        let new_zoom = (self.zoom + zoom_delta).clamp(0.1, 3.0);

        // Calculate new pan offset to keep the world point under the mouse
        let new_pan_offset = Vec2::new(
            screen_point.x - world_point.x * new_zoom,
            screen_point.y - world_point.y * new_zoom,
        );

        self.zoom = new_zoom;
        self.pan_offset = new_pan_offset;
    }

    fn create_node(&mut self, node_type: &str, position: Pos2) {
        // Map MaterialX display names to internal node types
        let internal_node_type = match node_type {
            "Noise" => "MaterialX_Noise",
            "Texture" => "MaterialX_Texture", 
            "Mix" => "MaterialX_Mix",
            "Standard Surface" => "MaterialX_StandardSurface",
            "3D View" => "MaterialX_3DView",
            "2D View" => "MaterialX_2DView",
            _ => node_type, // Use original name for generic nodes
        };
        
        // Try context-specific nodes first
        if let Some(context) = self.context_manager.get_active_context() {
            if let Some(node) = crate::NodeRegistry::create_context_node(context, internal_node_type, position) {
                self.graph.add_node(node);
                return;
            }
        }
        
        // Fall back to generic nodes
        if let Some(node) = crate::NodeRegistry::create_node(internal_node_type, position) {
            self.graph.add_node(node);
        }
    }

    /// Add benchmark nodes in a grid pattern for performance testing
    fn add_benchmark_nodes(&mut self, count: usize) {
        println!("Adding {} benchmark nodes", count);
        
        let node_types = ["Add", "Subtract", "Multiply", "Divide", "AND", "OR", "NOT", "Constant", "Variable", "Print", "Debug"];
        let spacing = 120.0;
        let grid_cols = (count as f32).sqrt().ceil() as usize;
        
        for i in 0..count {
            let col = i % grid_cols;
            let row = i / grid_cols;
            let x = 50.0 + col as f32 * spacing;
            let y = 100.0 + row as f32 * spacing;
            let node_type = node_types[i % node_types.len()];
            
            if let Some(node) = crate::NodeRegistry::create_node(node_type, Pos2::new(x, y)) {
                self.graph.add_node(node);
            }
        }
    }

    /// Add a large number of nodes with many connections for serious performance stress testing
    fn add_performance_stress_test(&mut self, count: usize) {
        println!("Creating performance stress test with {} nodes and many connections", count);
        
        let start_time = std::time::Instant::now();
        let node_types = ["Add", "Subtract", "Multiply", "Divide", "AND", "OR", "NOT", "Constant", "Variable", "Print", "Debug"];
        
        // Calculate grid that fits in reasonable space with compact spacing
        let spacing = 80.0; // Tighter spacing for 1000 nodes
        let grid_cols = (count as f32).sqrt().ceil() as usize;
        
        // Create all nodes first
        let mut node_ids = Vec::new();
        for i in 0..count {
            let col = i % grid_cols;
            let row = i / grid_cols;
            let x = 50.0 + col as f32 * spacing;
            let y = 100.0 + row as f32 * spacing;
            let node_type = node_types[i % node_types.len()];
            
            if let Some(node) = crate::NodeRegistry::create_node(node_type, Pos2::new(x, y)) {
                let node_id = self.graph.add_node(node);
                node_ids.push(node_id);
            }
        }
        
        let node_creation_time = start_time.elapsed();
        println!("Created {} nodes in {:?}", node_ids.len(), node_creation_time);
        
        // Create many connections for performance testing
        let connection_start = std::time::Instant::now();
        let connection_count = (count / 2).min(500); // Create up to 500 connections
        
        for i in 0..connection_count {
            if i + 1 < node_ids.len() {
                let from_id = node_ids[i];
                let to_id = node_ids[i + 1];
                
                // Try to create a connection (may fail if ports don't match)
                let connection = nodle_core::Connection::new(from_id, 0, to_id, 0);
                let _ = self.graph.add_connection(connection); // Ignore errors for stress test
            }
            
            // Also create some random long-distance connections
            if i % 10 == 0 && i + 20 < node_ids.len() {
                let from_id = node_ids[i];
                let to_id = node_ids[i + 20];
                let connection = nodle_core::Connection::new(from_id, 0, to_id, 0);
                let _ = self.graph.add_connection(connection);
            }
        }
        
        let connection_time = connection_start.elapsed();
        let total_time = start_time.elapsed();
        
        println!("Stress test complete: {} nodes, {} connections", 
                 self.graph.nodes.len(), self.graph.connections.len());
        println!("Total time: {:?} (nodes: {:?}, connections: {:?})", 
                 total_time, node_creation_time, connection_time);
    }

}

impl eframe::App for NodeEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request repaint for smooth updates
        ctx.request_repaint();

        // Track frame time for performance monitoring
        let current_time = std::time::Instant::now();
        let frame_time = current_time.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = current_time;
        
        self.frame_times.push(frame_time);
        if self.frame_times.len() > 60 { // Keep last 60 frames (1 second at 60fps)
            self.frame_times.remove(0);
        }
        
        // Set dark theme for window decorations
        ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(egui::SystemTheme::Dark));

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(22, 27, 34)))
            .show(ctx, |ui| {
            // Add padding around the top menu bar
            ui.add_space(8.0); // Top padding
            
            // Draw menu bar background
            let menu_height = 40.0; // Approximate height for menu bar
            let menu_rect = egui::Rect::from_min_size(
                egui::Pos2::new(0.0, 8.0), 
                egui::Vec2::new(ui.available_width(), menu_height)
            );
            ui.painter().rect_filled(
                menu_rect,
                0.0,
                Color32::from_rgb(28, 28, 28), // Standard background color
            );
            
            ui.horizontal(|ui| {
                ui.add_space(12.0); // Left padding
                
                // Context selection
                ui.label("Context:");
                
                // Collect context information to avoid borrowing issues
                let current_context_name = self.context_manager.get_active_context()
                    .map(|c| c.display_name())
                    .unwrap_or("None");
                let context_names: Vec<String> = self.context_manager.get_contexts()
                    .iter()
                    .map(|c| c.display_name().to_string())
                    .collect();
                
                let mut selected_context = self.context_manager.get_active_context().map(|_| 0);
                egui::ComboBox::from_label("")
                    .selected_text(current_context_name)
                    .show_ui(ui, |ui| {
                        if ui.selectable_value(&mut selected_context, None, "None").clicked() {
                            self.context_manager.set_active_context(None);
                        }
                        
                        for (i, context_name) in context_names.iter().enumerate() {
                            if ui.selectable_value(&mut selected_context, Some(i), context_name).clicked() {
                                self.context_manager.set_active_context(Some(i));
                            }
                        }
                    });
                
                ui.separator();
                ui.label(format!("Zoom: {:.1}x", self.zoom));
                ui.label(format!(
                    "Pan: ({:.0}, {:.0})",
                    self.pan_offset.x, self.pan_offset.y
                ));
                
                ui.add_space(12.0); // Right padding
            });
            ui.add_space(8.0); // Bottom padding

            let response = ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());
            let painter = ui.painter();

            // Draw node graph background
            painter.rect_filled(
                response.rect,
                0.0,
                Color32::from_rgb(28, 28, 28), // Standard background color
            );

            // Apply zoom and pan transforms
            let zoom = self.zoom;
            let pan_offset = self.pan_offset;

            let transform_pos = |pos: Pos2| -> Pos2 {
                Pos2::new(pos.x * zoom + pan_offset.x, pos.y * zoom + pan_offset.y)
            };

            let inverse_transform_pos = |pos: Pos2| -> Pos2 {
                Pos2::new((pos.x - pan_offset.x) / zoom, (pos.y - pan_offset.y) / zoom)
            };

            // Handle pan and zoom
            if ui.input(|i| i.pointer.middle_down()) && response.dragged() {
                self.pan_offset += response.drag_delta();
            }

            // Handle zoom with mouse wheel
            let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
            if scroll_delta != 0.0 {
                if let Some(mouse_pos) = response.hover_pos() {
                    self.zoom_at_point(mouse_pos, scroll_delta * 0.001);
                }
            }

            // Handle input
            if let Some(screen_pos) = response.interact_pointer_pos() {
                let pos = inverse_transform_pos(screen_pos);
                // Skip node interaction if we're panning
                let is_panning = ui.input(|i| i.pointer.middle_down());

                if !is_panning {
                    // Handle clicks (not just drags)
                    if response.clicked() {
                        let mut clicked_node = None;
                        let mut clicked_port = false;
                        let mut new_connection: Option<nodle_core::Connection> = None;
                        let mut new_connecting_from: Option<(nodle_core::NodeId, usize, bool)> = None;

                        // First check if we clicked on a port
                        for (node_id, node) in &self.graph.nodes {
                            // Check output ports
                            for (port_idx, port) in node.outputs.iter().enumerate() {
                                if (port.position - pos).length() < 10.0 {
                                    clicked_port = true;
                                    // If we already have a connection in progress, try to complete it
                                    if let Some((from_node, from_port, from_is_input)) =
                                        self.connecting_from
                                    {
                                        if from_is_input && *node_id != from_node {
                                            // Connecting from input to output
                                            new_connection = Some(nodle_core::Connection::new(*node_id, port_idx, from_node, from_port));
                                            new_connecting_from = None;
                                        } else {
                                            // Start new connection from this output
                                            new_connecting_from = Some((*node_id, port_idx, false));
                                        }
                                    } else {
                                        // Start new connection from this output
                                        new_connecting_from = Some((*node_id, port_idx, false));
                                    }
                                    break;
                                }
                            }
                            if clicked_port {
                                break;
                            }

                            // Check input ports
                            for (port_idx, port) in node.inputs.iter().enumerate() {
                                if (port.position - pos).length() < 10.0 {
                                    clicked_port = true;
                                    // If we already have a connection in progress, try to complete it
                                    if let Some((from_node, from_port, from_is_input)) =
                                        self.connecting_from
                                    {
                                        if !from_is_input && *node_id != from_node {
                                            // Connecting from output to input
                                            new_connection = Some(nodle_core::Connection::new(from_node, from_port, *node_id, port_idx));
                                            new_connecting_from = None;
                                        } else {
                                            // Start new connection from this input
                                            new_connecting_from = Some((*node_id, port_idx, true));
                                        }
                                    } else {
                                        // Start new connection from this input
                                        new_connecting_from = Some((*node_id, port_idx, true));
                                    }
                                    break;
                                }
                            }
                            if clicked_port {
                                break;
                            }
                        }

                        // Apply any connection changes after the loop
                        let connection_made = new_connection.is_some();
                        if let Some(connection) = new_connection {
                            let _ = self.graph.add_connection(connection);
                        }
                        if let Some(connecting) = new_connecting_from {
                            self.connecting_from = Some(connecting);
                        } else if connection_made {
                            self.connecting_from = None;
                        }

                        // If not clicking on a port, check for node
                        if !clicked_port {
                            for (id, node) in &self.graph.nodes {
                                if node.get_rect().contains(pos) {
                                    clicked_node = Some(*id);
                                    break;
                                }
                            }

                            if let Some(node_id) = clicked_node {
                                // Handle multi-selection with Ctrl/Cmd
                                if ui.input(|i| i.modifiers.ctrl || i.modifiers.command) {
                                    if self.selected_nodes.contains(&node_id) {
                                        self.selected_nodes.remove(&node_id);
                                    } else {
                                        self.selected_nodes.insert(node_id);
                                    }
                                } else {
                                    // Single selection - clear others and select this one
                                    self.selected_nodes.clear();
                                    self.selected_nodes.insert(node_id);
                                }
                                self.selected_connection = None;
                            } else {
                                // Clicked on empty space, deselect all
                                self.selected_nodes.clear();
                                self.selected_connection = None;
                                // Cancel any ongoing connection
                                self.connecting_from = None;
                            }
                        }
                    }

                    // Handle drag start for connections, node movement and box selection
                    if response.drag_started() {
                        let mut dragging_port = false;
                        let mut dragging_selected = false;
                        let mut clicked_node_id = None;

                        // First check if we're starting to drag from a port
                        for (node_id, node) in &self.graph.nodes {
                            // Check output ports
                            for (port_idx, port) in node.outputs.iter().enumerate() {
                                if (port.position - pos).length() < 10.0 {
                                    dragging_port = true;
                                    self.connecting_from = Some((*node_id, port_idx, false));
                                    break;
                                }
                            }
                            if dragging_port {
                                break;
                            }

                            // Check input ports
                            for (port_idx, port) in node.inputs.iter().enumerate() {
                                if (port.position - pos).length() < 10.0 {
                                    dragging_port = true;
                                    self.connecting_from = Some((*node_id, port_idx, true));
                                    break;
                                }
                            }
                            if dragging_port {
                                break;
                            }
                        }

                        // If not dragging from a port, handle node dragging
                        if !dragging_port {
                            // Check if we're starting to drag a currently selected node
                            for &node_id in &self.selected_nodes {
                                if let Some(node) = self.graph.nodes.get(&node_id) {
                                    if node.get_rect().contains(pos) {
                                        // Calculate drag offsets for all selected nodes
                                        self.drag_offsets.clear();
                                        for &selected_id in &self.selected_nodes {
                                            if let Some(selected_node) =
                                                self.graph.nodes.get(&selected_id)
                                            {
                                                self.drag_offsets.insert(
                                                    selected_id,
                                                    selected_node.position - pos,
                                                );
                                            }
                                        }
                                        dragging_selected = true;
                                        break;
                                    }
                                }
                            }

                            // If not dragging a selected node, check if we clicked on any node
                            if !dragging_selected {
                                for (node_id, node) in &self.graph.nodes {
                                    if node.get_rect().contains(pos) {
                                        clicked_node_id = Some(*node_id);
                                        break;
                                    }
                                }

                                if let Some(node_id) = clicked_node_id {
                                    // Select the node and start dragging it
                                    self.selected_nodes.clear();
                                    self.selected_nodes.insert(node_id);

                                    // Set up drag offset for this node
                                    self.drag_offsets.clear();
                                    if let Some(node) = self.graph.nodes.get(&node_id) {
                                        self.drag_offsets.insert(node_id, node.position - pos);
                                    }
                                } else {
                                    // Start box selection if not on any node and using left mouse button
                                    if ui.input(|i| i.pointer.primary_down()) {
                                        self.box_selection_start = Some(pos);
                                        self.box_selection_end = Some(pos);
                                    }
                                }
                            }
                        }
                    }

                    // Handle dragging
                    if response.dragged() {
                        if !self.drag_offsets.is_empty() {
                            // Drag all selected nodes
                            for (&node_id, &offset) in &self.drag_offsets {
                                if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                                    node.position = pos + offset;
                                }
                            }
                        } else if self.box_selection_start.is_some() {
                            // Update box selection
                            self.box_selection_end = Some(pos);
                        }
                    }

                    // Handle connection completion
                    if response.drag_stopped() {
                        if let Some((from_node, from_port, from_is_input)) = self.connecting_from {
                            // Check if we released on a port
                            let mut drag_connection: Option<nodle_core::Connection> = None;
                            for (node_id, node) in &self.graph.nodes {
                                if from_is_input {
                                    // Connecting from input, look for output
                                    for (port_idx, port) in node.outputs.iter().enumerate() {
                                        if (port.position - pos).length() < 10.0
                                            && *node_id != from_node
                                        {
                                            drag_connection = Some(nodle_core::Connection::new(*node_id, port_idx, from_node, from_port));
                                            break;
                                        }
                                    }
                                } else {
                                    // Connecting from output, look for input
                                    for (port_idx, port) in node.inputs.iter().enumerate() {
                                        if (port.position - pos).length() < 10.0
                                            && *node_id != from_node
                                        {
                                            drag_connection = Some(nodle_core::Connection::new(from_node, from_port, *node_id, port_idx));
                                            break;
                                        }
                                    }
                                }
                                if drag_connection.is_some() {
                                    break;
                                }
                            }
                            
                            // Apply connection after loop
                            if let Some(connection) = drag_connection {
                                let _ = self.graph.add_connection(connection);
                            }
                        }
                        // Cancel connection if we're releasing and not connecting
                        self.connecting_from = None;
                    }
                }

                if response.drag_stopped() {
                    self.drag_offsets.clear();

                    // Complete box selection
                    if let (Some(start), Some(end)) =
                        (self.box_selection_start, self.box_selection_end)
                    {
                        let selection_rect = Rect::from_two_pos(start, end);

                        // Handle multi-selection with Ctrl/Cmd
                        if !ui.input(|i| i.modifiers.ctrl || i.modifiers.command) {
                            self.selected_nodes.clear();
                        }

                        // Find all nodes in selection box
                        for (node_id, node) in &self.graph.nodes {
                            if selection_rect.intersects(node.get_rect()) {
                                self.selected_nodes.insert(*node_id);
                            }
                        }

                        self.box_selection_start = None;
                        self.box_selection_end = None;
                    }
                }
            }

            // Handle keyboard input
            if ui.input(|i| i.key_pressed(egui::Key::Delete)) {
                if !self.selected_nodes.is_empty() {
                    // Delete all selected nodes
                    for &node_id in &self.selected_nodes {
                        self.graph.remove_node(node_id);
                    }
                    self.selected_nodes.clear();
                } else if let Some(conn_idx) = self.selected_connection {
                    self.graph.remove_connection(conn_idx);
                    self.selected_connection = None;
                }
            }

            // Handle ESC key to cancel connections
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.connecting_from = None;
            }

            // Handle F1 to toggle performance info
            if ui.input(|i| i.key_pressed(egui::Key::F1)) {
                self.show_performance_info = !self.show_performance_info;
            }

            // Handle F2-F4 to add different numbers of nodes
            if ui.input(|i| i.key_pressed(egui::Key::F2)) {
                self.add_benchmark_nodes(10);
            }
            if ui.input(|i| i.key_pressed(egui::Key::F3)) {
                self.add_benchmark_nodes(25);
            }
            if ui.input(|i| i.key_pressed(egui::Key::F4)) {
                self.add_performance_stress_test(1000);
            }

            // Handle F5 to clear all nodes
            if ui.input(|i| i.key_pressed(egui::Key::F5)) {
                self.graph.nodes.clear();
                self.graph.connections.clear();
                self.selected_nodes.clear();
                self.connecting_from = None;
                println!("Cleared all nodes");
            }

            // Handle F6 to toggle GPU/CPU rendering
            if ui.input(|i| i.key_pressed(egui::Key::F6)) {
                self.use_gpu_rendering = !self.use_gpu_rendering;
                println!("Switched to {} rendering", if self.use_gpu_rendering { "GPU" } else { "CPU" });
            }

            // Handle right-click for context menu first (before other input handling)
            if response.secondary_clicked() {
                if let Some(screen_pos) = response.interact_pointer_pos() {
                    let pos = inverse_transform_pos(screen_pos);
                    let mut clicked_on_node = false;
                    for (id, node) in &self.graph.nodes {
                        if node.get_rect().contains(pos) {
                            self.selected_nodes.clear();
                            self.selected_nodes.insert(*id);
                            clicked_on_node = true;
                            break;
                        }
                    }

                    if !clicked_on_node {
                        // Right-clicked on empty space, show context menu
                        self.context_menu_pos = Some(screen_pos);
                    }
                }
            }

            // Show context menu with submenus
            if let Some(menu_screen_pos) = self.context_menu_pos {
                let menu_world_pos = inverse_transform_pos(menu_screen_pos);
                let popup_id = egui::Id::new("context_menu");

                let menu_response =
                    egui::Area::new(popup_id)
                        .fixed_pos(menu_screen_pos)
                        .show(ui.ctx(), |ui| {
                            egui::Frame::popup(ui.style())
                                .fill(Color32::from_rgb(28, 28, 28)) // Standard background color
                                .show(ui, |ui| {
                                    // Calculate menu width based on category names
                                    let mut categories = vec!["Create Node:", "Math", "Logic", "Data", "Output"];
                                    
                                    // Add MaterialX to width calculation if active
                                    if let Some(context) = self.context_manager.get_active_context() {
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
                                    let menu_width = (text_width + ui.spacing().button_padding.x * 2.0 + 20.0).max(120.0); // +20 for arrow
                                    ui.set_min_width(menu_width);
                                    ui.set_max_width(menu_width);

                                    ui.label("Create Node:");
                                    ui.separator();

                                    // Helper function to create menu items with full-width highlighting and arrow
                                    let submenu_item = |ui: &mut egui::Ui, text: &str, open_submenu: &mut Option<String>, submenu_pos: &mut Option<Pos2>| -> bool {
                                        let desired_size = egui::Vec2::new(menu_width, ui.spacing().button_padding.y * 2.0 + ui.text_style_height(&egui::TextStyle::Body));
                                        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
                                        
                                        if ui.is_rect_visible(rect) {
                                            let visuals = ui.style().interact(&response);
                                            
                                            // Fill background on hover
                                            if response.hovered() {
                                                ui.painter().rect_filled(rect, 0.0, visuals.bg_fill);
                                                *open_submenu = Some(text.to_string());
                                                *submenu_pos = Some(Pos2::new(rect.right(), rect.top()));
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
                                        
                                        response.clicked()
                                    };

                                    // Category menu items
                                    // Show MaterialX items if MaterialX context is active
                                    if let Some(context) = self.context_manager.get_active_context() {
                                        if context.id() == "materialx" {
                                            submenu_item(ui, "MaterialX", &mut self.open_submenu, &mut self.submenu_pos);
                                        }
                                    }
                                    
                                    submenu_item(ui, "Math", &mut self.open_submenu, &mut self.submenu_pos);
                                    submenu_item(ui, "Logic", &mut self.open_submenu, &mut self.submenu_pos);
                                    submenu_item(ui, "Data", &mut self.open_submenu, &mut self.submenu_pos);
                                    submenu_item(ui, "Output", &mut self.open_submenu, &mut self.submenu_pos);
                                })
                                .inner
                        });

                // Show submenu if one is open
                if let (Some(submenu_name), Some(submenu_screen_pos)) = (self.open_submenu.clone(), self.submenu_pos) {
                    let submenu_id = egui::Id::new(format!("submenu_{}", submenu_name));
                    
                    let submenu_response = egui::Area::new(submenu_id)
                        .fixed_pos(submenu_screen_pos)
                        .show(ui.ctx(), |ui| {
                            egui::Frame::popup(ui.style())
                                .fill(Color32::from_rgb(28, 28, 28)) // Standard background color
                                .show(ui, |ui| {
                                    // Get node items for this category  
                                    let node_items = match submenu_name.as_str() {
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
                                    let submenu_width = (text_width + ui.spacing().button_padding.x * 4.0).max(140.0); // Extra padding and larger minimum
                                    ui.set_min_width(submenu_width);
                                    ui.set_max_width(submenu_width);

                                    // Helper for submenu items
                                    let submenu_node_item = |ui: &mut egui::Ui, text: &str| -> bool {
                                        let desired_size = egui::Vec2::new(submenu_width, ui.spacing().button_padding.y * 2.0 + ui.text_style_height(&egui::TextStyle::Body));
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
                                    };

                                    // Draw submenu items
                                    for node_type in node_items {
                                        if submenu_node_item(ui, node_type) {
                                            self.create_node(node_type, menu_world_pos);
                                            self.context_menu_pos = None;
                                            self.open_submenu = None;
                                            self.submenu_pos = None;
                                        }
                                    }
                                })
                                .inner
                        });

                    // Close submenu if mouse moves away from both main menu and submenu
                    if let Some(mouse_pos) = ui.input(|i| i.pointer.interact_pos()) {
                        if !menu_response.response.rect.contains(mouse_pos) && 
                           !submenu_response.response.rect.contains(mouse_pos) {
                            // Add a small delay/buffer area between menu and submenu
                            let buffer_rect = egui::Rect::from_two_pos(
                                menu_response.response.rect.right_top(),
                                submenu_response.response.rect.left_bottom()
                            );
                            if !buffer_rect.contains(mouse_pos) {
                                self.open_submenu = None;
                                self.submenu_pos = None;
                            }
                        }
                    }
                }

                // Close entire menu if clicked outside all menu areas
                if ui.input(|i| i.pointer.primary_clicked()) {
                    if let Some(click_pos) = ui.input(|i| i.pointer.interact_pos()) {
                        let clicked_outside = !menu_response.response.rect.contains(click_pos);
                        
                        // Also check submenu if open
                        if let Some(_) = &self.open_submenu {
                            // We need to get the submenu rect, but it's already been computed above
                            // For now, we'll handle this in the submenu interaction logic
                        }
                        
                        if clicked_outside {
                            self.context_menu_pos = None;
                            self.open_submenu = None;
                            self.submenu_pos = None;
                        }
                    }
                }

                // Close on Escape key
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.context_menu_pos = None;
                    self.open_submenu = None;
                    self.submenu_pos = None;
                }
            }

            // Update port positions
            self.graph.update_all_port_positions();

            // Draw nodes - GPU vs CPU rendering
            if self.use_gpu_rendering && !self.graph.nodes.is_empty() {
                println!("GPU: Using actual GPU callback rendering for {} node bodies", self.graph.nodes.len());
                
                // GPU mode indicator  
                painter.rect_filled(
                    Rect::from_min_size(Pos2::new(10.0, 50.0), Vec2::new(100.0, 20.0)),
                    0.0,
                    Color32::BLUE
                );
                painter.text(
                    Pos2::new(60.0, 60.0),
                    egui::Align2::CENTER_CENTER,
                    "GPU-SHADER",
                    egui::FontId::proportional(12.0),
                    Color32::WHITE,
                );
                
                // Calculate viewport bounds for GPU callback
                let viewport_rect = response.rect;
                
                // Create GPU callback for node body rendering  
                // Use the full screen size, not just the response rect size, to match GPU viewport
                let screen_size = Vec2::new(
                    ui.ctx().screen_rect().width(),
                    ui.ctx().screen_rect().height()
                );
                let gpu_callback = NodeRenderCallback::new(
                    self.graph.nodes.clone(),
                    &self.selected_nodes,
                    self.connecting_from,
                    self.pan_offset,
                    self.zoom,
                    screen_size,
                );
                
                // Add the GPU paint callback using egui_wgpu::Callback - this will trigger prepare() and paint() methods
                painter.add(egui_wgpu::Callback::new_paint_callback(
                    viewport_rect,
                    gpu_callback,
                ));
                
                // Render node titles using CPU (GPU handles node bodies and ports)
                for (node_id, node) in &self.graph.nodes {
                    // Node titles (CPU-rendered text)
                    painter.text(
                        transform_pos(node.position + Vec2::new(node.size.x / 2.0, 15.0)),
                        egui::Align2::CENTER_CENTER,
                        &node.title,
                        egui::FontId::proportional(12.0 * self.zoom),
                        Color32::WHITE,
                    );
                    
                    // Port names on hover (CPU-rendered text)
                    let mouse_pos = response.hover_pos().map(|pos| inverse_transform_pos(pos));
                    if let Some(mouse_world_pos) = mouse_pos {
                        // Input port names
                        for input in &node.inputs {
                            if (input.position - mouse_world_pos).length() < 10.0 {
                                painter.text(
                                    transform_pos(input.position - Vec2::new(0.0, 15.0)),
                                    egui::Align2::CENTER_BOTTOM,
                                    &input.name,
                                    egui::FontId::proportional(10.0 * self.zoom),
                                    Color32::WHITE,
                                );
                            }
                        }
                        
                        // Output port names
                        for output in &node.outputs {
                            if (output.position - mouse_world_pos).length() < 10.0 {
                                painter.text(
                                    transform_pos(output.position + Vec2::new(0.0, 15.0)),
                                    egui::Align2::CENTER_TOP,
                                    &output.name,
                                    egui::FontId::proportional(10.0 * self.zoom),
                                    Color32::WHITE,
                                );
                            }
                        }
                    }
                }
                
            } else if !self.graph.nodes.is_empty() {
                // CPU rendering path - fallback mode
                for (node_id, node) in &self.graph.nodes {
                    // Transform node rectangle
                    let node_rect = node.get_rect();
                    let transformed_rect =
                        Rect::from_two_pos(transform_pos(node_rect.min), transform_pos(node_rect.max));
                    

                    // NODE BODY COMPONENTS
                    let radius = 5.0 * zoom;
                    
                    // BACKGROUND: Inner gradient mesh (0.5 to 0.25)
                    let background_top_color = Color32::from_rgb(127, 127, 127); // 0.5 grey
                    let background_bottom_color = Color32::from_rgb(64, 64, 64); // 0.25 grey
                    
                    // BEVEL: Outer gradient mesh with grid and fans (0.65 to 0.15)
                    let bevel_rect = transformed_rect; // Same size as original rect
                    let bevel_top_color = Color32::from_rgb(166, 166, 166); // 0.65 grey  
                    let bevel_bottom_color = Color32::from_rgb(38, 38, 38); // 0.15 grey
                    
                    let bevel_mesh = Self::create_rounded_gradient_mesh_optimized(
                        bevel_rect,
                        radius,
                        bevel_top_color,
                        bevel_bottom_color,
                    );
                    
                    painter.add(egui::Shape::mesh(bevel_mesh));
                    
                    // BORDER: Selection stroke
                    let is_selected = self.selected_nodes.contains(&node_id);
                    let border_color = if is_selected {
                        Color32::from_rgb(100, 150, 255) // Blue for selected
                    } else {
                        Color32::from_rgb(64, 64, 64) // 0.25 grey for unselected
                    };
                    
                    painter.rect_stroke(
                        transformed_rect,
                        radius,
                        Stroke::new(1.0 * zoom, border_color),
                    );
                    
                    // BACKGROUND: Inner gradient mesh with grid and fans (0.5 to 0.25)
                    // Shrunk by 2px to fit inside border
                    let background_shrink_offset = 2.0;
                    let background_rect = Rect::from_min_max(
                        transformed_rect.min + Vec2::splat(background_shrink_offset),
                        transformed_rect.max - Vec2::splat(background_shrink_offset),
                    );
                    let background_mesh = Self::create_rounded_gradient_mesh_optimized(
                        background_rect,
                        radius - background_shrink_offset, // Also shrink the radius
                        background_top_color,
                        background_bottom_color,
                    );
                    
                    painter.add(egui::Shape::mesh(background_mesh));

                    // Title
                    painter.text(
                        transform_pos(node.position + Vec2::new(node.size.x / 2.0, 15.0)),
                        egui::Align2::CENTER_CENTER,
                        &node.title,
                        egui::FontId::proportional(12.0 * zoom),
                        Color32::WHITE,
                    );

                    // Draw ports
                    let port_radius = 5.0 * zoom;
                    let hover_radius = 10.0; // Radius for hover detection (larger than visual port)

                    // Get mouse position for hover detection
                    let mouse_pos = response.hover_pos().map(|pos| inverse_transform_pos(pos));

                    // Input ports (on top)
                    for (port_idx, input) in node.inputs.iter().enumerate() {
                        // Check if this port is being used for an active connection
                        let is_connecting_port = if let Some((from_node, from_port, from_is_input)) = self.connecting_from {
                            from_node == *node_id && from_port == port_idx && from_is_input
                        } else {
                            false
                        };
                        
                        // Draw port border (2px larger) - blue if connecting, grey otherwise
                        let port_border_color = if is_connecting_port {
                            Color32::from_rgb(100, 150, 255) // Blue selection color
                        } else {
                            Color32::from_rgb(64, 64, 64) // Unselected node border color
                        };
                        
                        painter.circle_filled(
                            transform_pos(input.position),
                            port_radius + 2.0 * zoom,
                            port_border_color,
                        );
                        
                        // Draw port bevel (1px larger) - use node bevel bottom color
                        painter.circle_filled(
                            transform_pos(input.position),
                            port_radius + 1.0 * zoom,
                            Color32::from_rgb(38, 38, 38), // Node bevel bottom color (0.15)
                        );
                        
                        // Draw port background (main port)
                        painter.circle_filled(
                            transform_pos(input.position),
                            port_radius,
                            Color32::from_rgb(70, 120, 90), // Darker green for input ports
                        );
                        
                        // Only draw port name if hovering over it
                        if let Some(mouse_world_pos) = mouse_pos {
                            if (input.position - mouse_world_pos).length() < hover_radius {
                                painter.text(
                                    transform_pos(input.position - Vec2::new(0.0, 15.0)),
                                    egui::Align2::CENTER_BOTTOM,
                                    &input.name,
                                    egui::FontId::proportional(10.0 * zoom),
                                    Color32::from_gray(255), // Brighter when hovering
                                );
                            }
                        }
                    }

                    // Output ports (on bottom)
                    for (port_idx, output) in node.outputs.iter().enumerate() {
                        // Check if this port is being used for an active connection
                        let is_connecting_port = if let Some((from_node, from_port, from_is_input)) = self.connecting_from {
                            from_node == *node_id && from_port == port_idx && !from_is_input
                        } else {
                            false
                        };
                        
                        // Draw port border (2px larger) - blue if connecting, grey otherwise
                        let port_border_color = if is_connecting_port {
                            Color32::from_rgb(100, 150, 255) // Blue selection color
                        } else {
                            Color32::from_rgb(64, 64, 64) // Unselected node border color
                        };
                        
                        painter.circle_filled(
                            transform_pos(output.position),
                            port_radius + 2.0 * zoom,
                            port_border_color,
                        );
                        
                        // Draw port bevel (1px larger) - use node bevel bottom color
                        painter.circle_filled(
                            transform_pos(output.position),
                            port_radius + 1.0 * zoom,
                            Color32::from_rgb(38, 38, 38), // Node bevel bottom color (0.15)
                        );
                        
                        // Draw port background (main port)
                        painter.circle_filled(
                            transform_pos(output.position),
                            port_radius,
                            Color32::from_rgb(120, 70, 70), // Darker red for output ports
                        );
                        
                        // Only draw port name if hovering over it
                        if let Some(mouse_world_pos) = mouse_pos {
                            if (output.position - mouse_world_pos).length() < hover_radius {
                                painter.text(
                                    transform_pos(output.position + Vec2::new(0.0, 15.0)),
                                    egui::Align2::CENTER_TOP,
                                    &output.name,
                                    egui::FontId::proportional(10.0 * zoom),
                                    Color32::from_gray(255), // Brighter when hovering
                                );
                            }
                        }
                    }
                }
            } // End of CPU rendering mode
            

            // Draw connections
            for (idx, connection) in self.graph.connections.iter().enumerate() {
                if let (Some(from_node), Some(to_node)) = (
                    self.graph.nodes.get(&connection.from_node),
                    self.graph.nodes.get(&connection.to_node),
                ) {
                    if let (Some(from_port), Some(to_port)) = (
                        from_node.outputs.get(connection.from_port),
                        to_node.inputs.get(connection.to_port),
                    ) {
                        let from_pos = from_port.position;
                        let to_pos = to_port.position;

                        // Transform connection positions
                        let transformed_from = transform_pos(from_pos);
                        let transformed_to = transform_pos(to_pos);

                        // Draw bezier curve (vertical flow: top to bottom)
                        let vertical_distance = (transformed_to.y - transformed_from.y).abs();
                        let control_offset = if vertical_distance > 10.0 {
                            vertical_distance * 0.4
                        } else {
                            60.0 * zoom // Minimum offset for short connections
                        };

                        let points = [
                            transformed_from,
                            transformed_from + Vec2::new(0.0, control_offset),
                            transformed_to - Vec2::new(0.0, control_offset),
                            transformed_to,
                        ];

                        // Highlight selected connection
                        let (stroke_width, stroke_color) = if Some(idx) == self.selected_connection
                        {
                            (4.0 * zoom, Color32::from_rgb(88, 166, 255)) // Blue accent for selected
                        } else {
                            (2.0 * zoom, Color32::from_rgb(100, 110, 120)) // Darker gray for normal
                        };

                        painter.add(egui::Shape::CubicBezier(egui::epaint::CubicBezierShape {
                            points,
                            closed: false,
                            fill: Color32::TRANSPARENT,
                            stroke: Stroke::new(stroke_width, stroke_color).into(),
                        }));
                    }
                }
            }

            // Draw current connection being made
            if let Some((from_node, from_port, from_is_input)) = self.connecting_from {
                if let Some(mouse_pos) = response.hover_pos() {
                    if let Some(node) = self.graph.nodes.get(&from_node) {
                        let from_pos = if from_is_input {
                            node.inputs[from_port].position
                        } else {
                            node.outputs[from_port].position
                        };

                        let transformed_from = transform_pos(from_pos);
                        let transformed_to = mouse_pos;

                        // Draw bezier curve for connection preview (vertical flow)
                        // Use fixed control offset to prevent popping when curve goes horizontal
                        let control_offset = 60.0 * zoom;

                        // Control points should aim in the correct direction based on port type
                        let from_control = if from_is_input {
                            transformed_from - Vec2::new(0.0, control_offset) // Input ports: aim up
                        } else {
                            transformed_from + Vec2::new(0.0, control_offset) // Output ports: aim down
                        };
                        
                        let to_control = if from_is_input {
                            transformed_to + Vec2::new(0.0, control_offset) // When connecting from input: aim up from mouse
                        } else {
                            transformed_to - Vec2::new(0.0, control_offset) // When connecting from output: aim down to mouse
                        };

                        let points = [
                            transformed_from,
                            from_control,
                            to_control,
                            transformed_to,
                        ];

                        painter.add(egui::Shape::CubicBezier(egui::epaint::CubicBezierShape {
                            points,
                            closed: false,
                            fill: Color32::TRANSPARENT,
                            stroke: Stroke::new(2.0 * zoom, Color32::from_rgb(100, 180, 255))
                                .into(),
                        }));
                    }
                }
            }

            // Draw box selection
            if let (Some(start), Some(end)) = (self.box_selection_start, self.box_selection_end) {
                let selection_rect = Rect::from_two_pos(transform_pos(start), transform_pos(end));

                // Draw selection box background
                painter.rect_filled(
                    selection_rect,
                    0.0,
                    Color32::from_rgba_unmultiplied(100, 150, 255, 30),
                );

                // Draw selection box border
                painter.rect_stroke(
                    selection_rect,
                    0.0,
                    Stroke::new(1.0 * zoom, Color32::from_rgb(100, 150, 255)),
                );
            }

            // Performance info overlay
            if self.show_performance_info && !self.frame_times.is_empty() {
                let avg_frame_time = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
                let fps = 1.0 / avg_frame_time;
                let rendering_mode = if self.use_gpu_rendering { "GPU" } else { "CPU" };
                
                egui::Window::new("Performance")
                    .default_pos([10.0, 10.0])
                    .default_size([200.0, 100.0])
                    .resizable(false)
                    .show(ui.ctx(), |ui| {
                        ui.label(format!("FPS: {:.1}", fps));
                        ui.label(format!("Frame time: {:.2}ms", avg_frame_time * 1000.0));
                        ui.label(format!("Rendering: {}", rendering_mode));
                        ui.label(format!("Nodes: {}", self.graph.nodes.len()));
                        ui.separator();
                        ui.label("F1: Toggle performance info");
                        ui.label("F2: Add 10 nodes");
                        ui.label("F3: Add 25 nodes");
                        ui.label("F4: Stress test (1000 nodes + connections)");
                        ui.label("F5: Clear all nodes");
                        ui.label("F6: Toggle GPU/CPU rendering");
                    });
            }
        });
    }
}