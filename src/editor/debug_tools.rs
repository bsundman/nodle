//! Debug and performance monitoring tools for the node editor
//!
//! Handles performance tracking, debug information display, and development tools.

use egui::Ui;
use std::time::Instant;
use crate::nodes::NodeGraph;
use egui::Pos2;

/// Manages debug and performance monitoring features
pub struct DebugToolsManager {
    /// Whether to show performance information
    show_performance_info: bool,
    /// Frame time history for averaging
    frame_times: Vec<f32>,
    /// Last frame timestamp for delta calculation
    last_frame_time: Instant,
}

impl DebugToolsManager {
    /// Create a new debug tools manager
    pub fn new() -> Self {
        Self {
            show_performance_info: false,
            frame_times: Vec::new(),
            last_frame_time: Instant::now(),
        }
    }

    /// Toggle performance information display
    pub fn toggle_performance_info(&mut self) {
        self.show_performance_info = !self.show_performance_info;
    }

    /// Check if performance info should be shown
    pub fn should_show_performance_info(&self) -> bool {
        self.show_performance_info
    }

    /// Update frame time tracking
    pub fn update_frame_time(&mut self) {
        let current_time = Instant::now();
        let frame_time = current_time.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = current_time;

        self.frame_times.push(frame_time);
        if self.frame_times.len() > 60 { // Keep last 60 frames (1 second at 60fps)
            self.frame_times.remove(0);
        }
    }

    /// Get frame times for analysis
    pub fn get_frame_times(&self) -> &Vec<f32> {
        &self.frame_times
    }

    /// Render performance information panel
    pub fn render_performance_info(&self, ui: &mut Ui, use_gpu_rendering: bool, node_count: usize, menu_bar_height: f32) {
        if self.show_performance_info && !self.frame_times.is_empty() {
            let avg_frame_time = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
            let fps = 1.0 / avg_frame_time;
            let rendering_mode = if use_gpu_rendering { "GPU" } else { "CPU" };

            // Create window with menu bar constraint (using helper function)
            Self::create_window("Performance", ui.ctx(), menu_bar_height)
                .default_pos([10.0, 10.0])
                .default_size([200.0, 100.0])
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label(format!("FPS: {:.1}", fps));
                    ui.label(format!("Frame time: {:.2}ms", avg_frame_time * 1000.0));
                    ui.label(format!("Rendering: {}", rendering_mode));
                    ui.label(format!("Nodes: {}", node_count));
                    ui.separator();
                    ui.label("F1: Toggle performance info");
                    ui.label("F2: Add 10 nodes");
                    ui.label("F3: Add 25 nodes");
                    ui.label("F4: Stress test (5000 nodes + connections)");
                    ui.label("F5: Clear all nodes");
                    ui.label("F6: Toggle GPU/CPU rendering");
                });
        }
    }

    /// Create a window that automatically respects the menu bar constraint
    fn create_window<'a>(title: &'a str, ctx: &egui::Context, menu_bar_height: f32) -> egui::Window<'a> {
        egui::Window::new(title)
            .constrain_to(egui::Rect::from_min_size(
                egui::Pos2::new(0.0, menu_bar_height), 
                egui::Vec2::new(ctx.screen_rect().width(), ctx.screen_rect().height() - menu_bar_height)
            ))
    }

    /// Add benchmark nodes in a grid pattern for performance testing
    pub fn add_benchmark_nodes(graph: &mut NodeGraph, count: usize) {
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
                graph.add_node(node);
            }
        }
    }

    /// Add a large number of nodes with many connections for serious performance stress testing
    pub fn add_performance_stress_test(graph: &mut NodeGraph, count: usize) {
        let node_types = ["Add", "Subtract", "Multiply", "Divide", "AND", "OR", "NOT", "Constant", "Variable", "Print", "Debug"];
        
        // Calculate grid that fits in reasonable space with compact spacing
        let spacing = 80.0; // Tighter spacing for stress test
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
                let node_id = graph.add_node(node);
                node_ids.push(node_id);
            }
        }
        
        // Create many connections for performance testing
        let connection_count = (count / 2).min(500); // Create up to 500 connections
        
        for i in 0..connection_count {
            if i + 1 < node_ids.len() {
                let from_id = node_ids[i];
                let to_id = node_ids[i + 1];
                
                // Try to create a connection (may fail if ports don't match)
                let connection = crate::nodes::Connection::new(from_id, 0, to_id, 0);
                let _ = graph.add_connection(connection); // Ignore errors for stress test
            }
            
            // Also create some random long-distance connections
            if i % 10 == 0 && i + 20 < node_ids.len() {
                let from_id = node_ids[i];
                let to_id = node_ids[i + 20];
                let connection = crate::nodes::Connection::new(from_id, 0, to_id, 0);
                let _ = graph.add_connection(connection);
            }
        }
    }
}

impl Default for DebugToolsManager {
    fn default() -> Self {
        Self::new()
    }
}