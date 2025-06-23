//! Nodle Application - Node-based visual programming editor

use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use nodle_core::{
    graph::{Connection, NodeGraph},
    math::{cubic_bezier_point, distance_to_line_segment},
    node::{Node, NodeId},
    port::PortId,
};
use std::collections::{HashMap, HashSet};

mod editor;
mod nodes;

use editor::NodeEditor;

/// Application entry point
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Nodle",
        options,
        Box::new(|_cc| Ok(Box::new(NodeEditor::new()))),
    )
}