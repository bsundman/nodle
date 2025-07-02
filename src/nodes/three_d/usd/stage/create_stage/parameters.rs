//! Create Stage node parameter interface

use crate::nodes::interface::{build_parameter_ui, NodeData, ParameterChange};
use crate::nodes::Node;

/// USD Create Stage node with parameter controls
#[derive(Default)]
pub struct CreateStageNode;

impl CreateStageNode {
    /// Build the parameter interface
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("Create Stage");
        ui.separator();
        
        // Stage identifier parameter
        if let Some(change) = build_parameter_ui(
            ui,
            "identifier",
            "Stage ID",
            node.parameters.get("identifier").cloned().unwrap_or(NodeData::String("default".to_string())),
            |ui, value| {
                if let NodeData::String(ref s) = value {
                    let mut text = s.clone();
                    let response = ui.text_edit_singleline(&mut text);
                    if response.changed() {
                        return Some(NodeData::String(text));
                    }
                }
                None
            }
        ) {
            node.parameters.insert("identifier".to_string(), change.clone());
            changes.push(ParameterChange {
                parameter: "identifier".to_string(),
                value: change,
            });
        }
        
        // In-memory vs file-backed option
        if let Some(change) = build_parameter_ui(
            ui,
            "in_memory",
            "In Memory",
            node.parameters.get("in_memory").cloned().unwrap_or(NodeData::Boolean(true)),
            |ui, value| {
                if let NodeData::Boolean(ref b) = value {
                    let mut checked = *b;
                    let response = ui.checkbox(&mut checked, "Create in-memory stage");
                    if response.changed() {
                        return Some(NodeData::Boolean(checked));
                    }
                }
                None
            }
        ) {
            node.parameters.insert("in_memory".to_string(), change.clone());
            changes.push(ParameterChange {
                parameter: "in_memory".to_string(),
                value: change,
            });
        }
        
        // File path (if not in-memory)
        let in_memory = matches!(
            node.parameters.get("in_memory"),
            Some(NodeData::Boolean(true)) | None
        );
        
        if !in_memory {
            if let Some(change) = build_parameter_ui(
                ui,
                "file_path",
                "File Path",
                node.parameters.get("file_path").cloned().unwrap_or(NodeData::String("stage.usda".to_string())),
                |ui, value| {
                    if let NodeData::String(ref s) = value {
                        let mut text = s.clone();
                        let response = ui.text_edit_singleline(&mut text);
                        if response.changed() {
                            return Some(NodeData::String(text));
                        }
                    }
                    None
                }
            ) {
                node.parameters.insert("file_path".to_string(), change.clone());
                changes.push(ParameterChange {
                    parameter: "file_path".to_string(),
                    value: change,
                });
            }
        }
        
        // Default prim path
        if let Some(change) = build_parameter_ui(
            ui,
            "default_prim",
            "Default Prim",
            node.parameters.get("default_prim").cloned().unwrap_or(NodeData::String("/World".to_string())),
            |ui, value| {
                if let NodeData::String(ref s) = value {
                    let mut text = s.clone();
                    let response = ui.text_edit_singleline(&mut text);
                    if response.changed() {
                        return Some(NodeData::String(text));
                    }
                }
                None
            }
        ) {
            node.parameters.insert("default_prim".to_string(), change.clone());
            changes.push(ParameterChange {
                parameter: "default_prim".to_string(),
                value: change,
            });
        }
        
        changes
    }
}