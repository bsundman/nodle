//! Scenegraph node parameter definitions

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;

/// Scenegraph node for displaying USD scene hierarchy in a tree view
#[derive(Debug, Clone)]
pub struct ScenegraphNode;

impl ScenegraphNode {
    /// Build the parameter interface for this node
    pub fn build_interface(node: &Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("Scene Graph Viewer");
        ui.separator();
        
        ui.label("Connect a USD data source to view the scene hierarchy.");
        ui.label("The tree view will display:");
        ui.label("• Geometry objects");
        ui.label("• Lights");
        ui.label("• Materials");
        ui.label("• Cameras");
        
        ui.separator();
        ui.label("Use the tree panel to explore the scene structure.");
        
        changes
    }
}