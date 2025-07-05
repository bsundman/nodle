//! Viewport interface - simplified for USD separation test

use egui::Ui;
use crate::nodes::{interface::ParameterChange, Node};
use crate::gpu::ViewportRenderCallback;

#[derive(Default, Debug)]
pub struct ViewportNode {
    // Simplified viewport node - USD functionality moved to plugin
}

impl ViewportNode {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn load_usd_scene(&mut self, _scene_data: &str) {
        // USD scene loading now handled by plugin
    }
}

pub fn build_viewport_interface() -> Vec<ParameterChange> {
    // Simplified viewport interface - USD functionality moved to plugin
    Vec::new()
}

pub fn build_interface(node: &mut ViewportNode, ui: &mut Ui) -> Vec<ParameterChange> {
    ui.label("Viewport Node");
    ui.label("(USD viewport functionality now handled by plugin)");
    Vec::new()
}