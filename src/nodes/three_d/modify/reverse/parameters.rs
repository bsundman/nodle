//! Reverse node parameters using Pattern A interface

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::Node;
use super::logic::{ReverseLogic, MirrorAxis};

/// Reverse node with Pattern A interface
#[derive(Debug, Clone)]
pub struct ReverseNode {
    pub reverse_normals: bool,
    pub reverse_face_winding: bool,
    pub reverse_point_order: bool,
    pub mirror_axis: MirrorAxis,
    pub reverse_uvs_u: bool,
    pub reverse_uvs_v: bool,
    pub flip_vertex_colors: bool,
    pub invert_transforms: bool,
}

impl Default for ReverseNode {
    fn default() -> Self {
        Self {
            reverse_normals: false,
            reverse_face_winding: false,
            reverse_point_order: false,
            mirror_axis: MirrorAxis::None,
            reverse_uvs_u: false,
            reverse_uvs_v: false,
            flip_vertex_colors: false,
            invert_transforms: false,
        }
    }
}

impl ReverseNode {
    /// Pattern A: build_interface method that renders UI and returns parameter changes
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        ui.heading("Reverse Operations");
        ui.separator();
        
        // Section 1: Geometry Reversal
        ui.vertical(|ui| {
            ui.heading("Geometry");
            ui.add_space(5.0);
            
            // Reverse Normals
            let mut reverse_normals = node.parameters.get("reverse_normals")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false);
            
            if ui.checkbox(&mut reverse_normals, "Reverse Normals").changed() {
                node.parameters.insert("reverse_normals".to_string(), NodeData::Boolean(reverse_normals));
                changes.push(ParameterChange {
                    parameter: "reverse_normals".to_string(),
                    value: NodeData::Boolean(reverse_normals),
                });
            }
            
            ui.label("   ↳ Invert all normal vectors (flips lighting)");
            ui.add_space(3.0);
            
            // Reverse Face Winding
            let mut reverse_face_winding = node.parameters.get("reverse_face_winding")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false);
            
            if ui.checkbox(&mut reverse_face_winding, "Reverse Face Winding").changed() {
                node.parameters.insert("reverse_face_winding".to_string(), NodeData::Boolean(reverse_face_winding));
                changes.push(ParameterChange {
                    parameter: "reverse_face_winding".to_string(),
                    value: NodeData::Boolean(reverse_face_winding),
                });
            }
            
            ui.label("   ↳ Change triangle winding order (affects culling)");
            ui.add_space(3.0);
            
            // Reverse Point Order
            let mut reverse_point_order = node.parameters.get("reverse_point_order")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false);
            
            if ui.checkbox(&mut reverse_point_order, "Reverse Point Order").changed() {
                node.parameters.insert("reverse_point_order".to_string(), NodeData::Boolean(reverse_point_order));
                changes.push(ParameterChange { 
                    parameter: "reverse_point_order".to_string(), 
                    value: NodeData::Boolean(reverse_point_order),
                });
            }
            
            ui.label("   ↳ Reverse vertex order within faces");
        });
        
        ui.add_space(10.0);
        ui.separator();
        
        // Section 2: Axis Mirroring
        ui.vertical(|ui| {
            ui.heading("Axis Mirroring");
            ui.add_space(5.0);
            
            let current_mirror = node.parameters.get("mirror_axis")
                .and_then(|v| if let NodeData::String(s) = v { 
                    Some(match s.as_str() {
                        "X" => MirrorAxis::X,
                        "Y" => MirrorAxis::Y,
                        "Z" => MirrorAxis::Z,
                        "XY" => MirrorAxis::XY,
                        "XZ" => MirrorAxis::XZ,
                        "YZ" => MirrorAxis::YZ,
                        _ => MirrorAxis::None,
                    })
                } else { None })
                .unwrap_or(MirrorAxis::None);
            
            let mut selected_mirror = match current_mirror {
                MirrorAxis::X => "X",
                MirrorAxis::Y => "Y", 
                MirrorAxis::Z => "Z",
                MirrorAxis::XY => "XY",
                MirrorAxis::XZ => "XZ",
                MirrorAxis::YZ => "YZ",
                MirrorAxis::None => "None",
            };
            
            egui::ComboBox::from_id_salt("mirror_axis")
                .selected_text(selected_mirror)
                .width(120.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut selected_mirror, "None", "None");
                    ui.selectable_value(&mut selected_mirror, "X", "X Axis");
                    ui.selectable_value(&mut selected_mirror, "Y", "Y Axis");
                    ui.selectable_value(&mut selected_mirror, "Z", "Z Axis");
                    ui.selectable_value(&mut selected_mirror, "XY", "XY Plane");
                    ui.selectable_value(&mut selected_mirror, "XZ", "XZ Plane");
                    ui.selectable_value(&mut selected_mirror, "YZ", "YZ Plane");
                });
            
            // Check if selection changed
            let new_mirror_string = selected_mirror.to_string();
            let current_string = match current_mirror {
                MirrorAxis::X => "X",
                MirrorAxis::Y => "Y",
                MirrorAxis::Z => "Z", 
                MirrorAxis::XY => "XY",
                MirrorAxis::XZ => "XZ",
                MirrorAxis::YZ => "YZ",
                MirrorAxis::None => "None",
            };
            
            if new_mirror_string != current_string {
                node.parameters.insert("mirror_axis".to_string(), NodeData::String(new_mirror_string.clone()));
                changes.push(ParameterChange {
                    parameter: "mirror_axis".to_string(),
                    value: NodeData::String(new_mirror_string),
                });
            }
            
            ui.label("   ↳ Mirror geometry along selected axis/plane");
        });
        
        ui.add_space(10.0);
        ui.separator();
        
        // Section 3: UV Coordinates
        ui.vertical(|ui| {
            ui.heading("UV Coordinates");
            ui.add_space(5.0);
            
            // Reverse U coordinates
            let mut reverse_uvs_u = node.parameters.get("reverse_uvs_u")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false);
            
            if ui.checkbox(&mut reverse_uvs_u, "Reverse U Coordinates").changed() {
                node.parameters.insert("reverse_uvs_u".to_string(), NodeData::Boolean(reverse_uvs_u));
                changes.push(ParameterChange {
                    parameter: "reverse_uvs_u".to_string(),
                    value: NodeData::Boolean(reverse_uvs_u),
                });
            }
            
            ui.label("   ↳ Flip U texture coordinates (horizontal flip)");
            ui.add_space(3.0);
            
            // Reverse V coordinates
            let mut reverse_uvs_v = node.parameters.get("reverse_uvs_v")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false);
            
            if ui.checkbox(&mut reverse_uvs_v, "Reverse V Coordinates").changed() {
                node.parameters.insert("reverse_uvs_v".to_string(), NodeData::Boolean(reverse_uvs_v));
                changes.push(ParameterChange {
                    parameter: "reverse_uvs_v".to_string(),
                    value: NodeData::Boolean(reverse_uvs_v),
                });
            }
            
            ui.label("   ↳ Flip V texture coordinates (vertical flip)");
        });
        
        ui.add_space(10.0);
        ui.separator();
        
        // Section 4: Advanced Options
        ui.vertical(|ui| {
            ui.heading("Advanced");
            ui.add_space(5.0);
            
            // Flip Vertex Colors
            let mut flip_vertex_colors = node.parameters.get("flip_vertex_colors")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false);
            
            if ui.checkbox(&mut flip_vertex_colors, "Flip Vertex Colors").changed() {
                node.parameters.insert("flip_vertex_colors".to_string(), NodeData::Boolean(flip_vertex_colors));
                changes.push(ParameterChange {
                    parameter: "flip_vertex_colors".to_string(),
                    value: NodeData::Boolean(flip_vertex_colors),
                });
            }
            
            ui.label("   ↳ Invert RGB values of vertex colors");
            ui.add_space(3.0);
            
            // Invert Transforms
            let mut invert_transforms = node.parameters.get("invert_transforms")
                .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
                .unwrap_or(false);
            
            if ui.checkbox(&mut invert_transforms, "Invert Transform Matrix").changed() {
                node.parameters.insert("invert_transforms".to_string(), NodeData::Boolean(invert_transforms));
                changes.push(ParameterChange {
                    parameter: "invert_transforms".to_string(),
                    value: NodeData::Boolean(invert_transforms),
                });
            }
            
            ui.label("   ↳ Invert mesh transformation matrix");
        });
        
        ui.add_space(10.0);
        ui.separator();
        
        // Summary information
        ui.vertical(|ui| {
            ui.heading("Info");
            ui.label("This node applies various reversal operations to USD geometry data.");
            ui.label("Operations are applied in the order listed above.");
            ui.label("Use with caution as some combinations may produce unexpected results.");
        });
        
        changes
    }
    
    /// Process the reverse node with the given inputs
    pub fn process_node(node: &Node, inputs: Vec<NodeData>) -> Vec<NodeData> {
        // Extract parameters from node
        let reverse_normals = node.parameters.get("reverse_normals")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
            
        let reverse_face_winding = node.parameters.get("reverse_face_winding")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
            
        let reverse_point_order = node.parameters.get("reverse_point_order")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
            
        let mirror_axis = node.parameters.get("mirror_axis")
            .and_then(|v| if let NodeData::String(s) = v { 
                Some(match s.as_str() {
                    "X" => MirrorAxis::X,
                    "Y" => MirrorAxis::Y,
                    "Z" => MirrorAxis::Z,
                    "XY" => MirrorAxis::XY,
                    "XZ" => MirrorAxis::XZ,
                    "YZ" => MirrorAxis::YZ,
                    _ => MirrorAxis::None,
                })
            } else { None })
            .unwrap_or(MirrorAxis::None);
            
        let reverse_uvs_u = node.parameters.get("reverse_uvs_u")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
            
        let reverse_uvs_v = node.parameters.get("reverse_uvs_v")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
            
        let flip_vertex_colors = node.parameters.get("flip_vertex_colors")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
            
        let invert_transforms = node.parameters.get("invert_transforms")
            .and_then(|v| if let NodeData::Boolean(b) = v { Some(*b) } else { None })
            .unwrap_or(false);
        
        // Create logic instance and process
        let logic = ReverseLogic {
            reverse_normals,
            reverse_face_winding,
            reverse_point_order,
            mirror_axis,
            reverse_uvs_u,
            reverse_uvs_v,
            flip_vertex_colors,
            invert_transforms,
        };
        
        logic.process(inputs)
    }
}