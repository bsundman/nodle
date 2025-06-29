//! Cube node with interface panel for parameter control

use egui::Color32;
use crate::nodes::{
    Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition,
    NodeInterfacePanel, InterfaceParameter, NodeData, GeometryData, SceneData,
};
use crate::{interface_float, interface_vector3, interface_enum};

/// Cube geometry node with interface panel
#[derive(Debug, Clone)]
pub struct CubeNodeWithInterface {
    /// Cube dimensions
    pub size: [f32; 3],
    /// Number of subdivisions for each axis
    pub subdivisions: [i32; 3],
    /// Pivot point for the cube
    pub pivot: PivotType,
    /// Whether to generate UVs
    pub generate_uvs: bool,
    /// Whether to generate normals
    pub generate_normals: bool,
}

#[derive(Debug, Clone)]
pub enum PivotType {
    Center,
    Corner,
    Bottom,
}

impl Default for CubeNodeWithInterface {
    fn default() -> Self {
        Self {
            size: [1.0, 1.0, 1.0],
            subdivisions: [1, 1, 1],
            pivot: PivotType::Center,
            generate_uvs: true,
            generate_normals: true,
        }
    }
}

impl CubeNodeWithInterface {
    /// Generate cube geometry based on current parameters
    fn generate_geometry(&self) -> GeometryData {
        let [sx, sy, sz] = self.size;
        let [subdiv_x, subdiv_y, subdiv_z] = self.subdivisions;
        
        // Calculate pivot offset
        let pivot_offset = match self.pivot {
            PivotType::Center => [0.0, 0.0, 0.0],
            PivotType::Corner => [-sx * 0.5, -sy * 0.5, -sz * 0.5],
            PivotType::Bottom => [0.0, -sy * 0.5, 0.0],
        };
        
        // Generate vertices for a subdivided cube
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        
        // For simplicity, generate a basic cube
        // In a real implementation, this would generate proper subdivisions
        let half_size = [sx * 0.5, sy * 0.5, sz * 0.5];
        
        // Cube vertices (8 corners)
        let cube_vertices = [
            [-half_size[0], -half_size[1], -half_size[2]], // 0: left-bottom-back
            [ half_size[0], -half_size[1], -half_size[2]], // 1: right-bottom-back
            [ half_size[0],  half_size[1], -half_size[2]], // 2: right-top-back
            [-half_size[0],  half_size[1], -half_size[2]], // 3: left-top-back
            [-half_size[0], -half_size[1],  half_size[2]], // 4: left-bottom-front
            [ half_size[0], -half_size[1],  half_size[2]], // 5: right-bottom-front
            [ half_size[0],  half_size[1],  half_size[2]], // 6: right-top-front
            [-half_size[0],  half_size[1],  half_size[2]], // 7: left-top-front
        ];
        
        // Apply pivot offset and add vertices
        for vertex in &cube_vertices {
            vertices.push([
                vertex[0] + pivot_offset[0],
                vertex[1] + pivot_offset[1],
                vertex[2] + pivot_offset[2],
            ]);
        }
        
        // Cube face indices (12 triangles, 2 per face)
        let cube_indices = [
            // Back face
            0, 1, 2, 0, 2, 3,
            // Front face  
            4, 6, 5, 4, 7, 6,
            // Left face
            0, 3, 7, 0, 7, 4,
            // Right face
            1, 5, 6, 1, 6, 2,
            // Bottom face
            0, 4, 5, 0, 5, 1,
            // Top face
            3, 2, 6, 3, 6, 7,
        ];
        
        indices.extend_from_slice(&cube_indices);
        
        // Generate normals if requested
        if self.generate_normals {
            // Simple face normals (not smooth)
            let face_normals = [
                [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], // Back face
                [0.0, 0.0,  1.0], [0.0, 0.0,  1.0], // Front face
                [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], // Left face
                [1.0, 0.0, 0.0], [1.0, 0.0, 0.0],   // Right face
                [0.0, -1.0, 0.0], [0.0, -1.0, 0.0], // Bottom face
                [0.0, 1.0, 0.0], [0.0, 1.0, 0.0],   // Top face
            ];
            
            // Assign normals to vertices (simplified)
            for _ in 0..8 {
                normals.push([0.0, 1.0, 0.0]); // Default up normal
            }
        }
        
        // Generate UVs if requested
        if self.generate_uvs {
            let cube_uvs = [
                [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], // Back face
                [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], // Front face
            ];
            uvs.extend_from_slice(&cube_uvs);
        }
        
        GeometryData {
            id: format!("cube_{}", std::ptr::addr_of!(*self) as usize),
            vertices,
            indices,
            normals,
            uvs,
            material_id: None,
        }
    }
}

impl NodeFactory for CubeNodeWithInterface {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "3D_CubeInterface",
            "Cube (Interface)",
            NodeCategory::new(&["3D", "Geometry"]),
            "Creates a cube primitive with interface panel controls"
        )
        .with_color(Color32::from_rgb(100, 150, 200)) // Blue tint for geometry
        .with_icon("🟫")
        .with_inputs(vec![
            // Optional transform input for positioning
            PortDefinition::optional("Transform", DataType::Any)
                .with_description("Optional transform matrix to position the cube"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Geometry", DataType::Any)
                .with_description("Generated cube geometry"),
        ])
        .with_tags(vec!["3d", "geometry", "primitive", "cube", "interface"])
        .with_processing_cost(crate::nodes::ProcessingCost::Medium)
        .with_workspace_compatibility(vec!["3d", "modeling"])
    }
}

impl NodeInterfacePanel for CubeNodeWithInterface {
    fn get_parameters(&self) -> Vec<(&'static str, InterfaceParameter)> {
        vec![
            ("Size X", interface_float!(self.size[0], 0.1, 10.0, 0.1)),
            ("Size Y", interface_float!(self.size[1], 0.1, 10.0, 0.1)),
            ("Size Z", interface_float!(self.size[2], 0.1, 10.0, 0.1)),
            ("Subdiv X", InterfaceParameter::Integer { 
                value: self.subdivisions[0], min: 1, max: 20 
            }),
            ("Subdiv Y", InterfaceParameter::Integer { 
                value: self.subdivisions[1], min: 1, max: 20 
            }),
            ("Subdiv Z", InterfaceParameter::Integer { 
                value: self.subdivisions[2], min: 1, max: 20 
            }),
            ("Pivot", interface_enum!(
                match self.pivot {
                    PivotType::Center => 0,
                    PivotType::Corner => 1,
                    PivotType::Bottom => 2,
                },
                "Center", "Corner", "Bottom"
            )),
            ("Generate UVs", InterfaceParameter::Boolean { 
                value: self.generate_uvs 
            }),
            ("Generate Normals", InterfaceParameter::Boolean { 
                value: self.generate_normals 
            }),
        ]
    }
    
    fn set_parameters(&mut self, parameters: Vec<(&'static str, InterfaceParameter)>) {
        for (name, param) in parameters {
            match name {
                "Size X" => if let Some(val) = param.get_float() { self.size[0] = val; },
                "Size Y" => if let Some(val) = param.get_float() { self.size[1] = val; },
                "Size Z" => if let Some(val) = param.get_float() { self.size[2] = val; },
                "Subdiv X" => if let InterfaceParameter::Integer { value, .. } = param { 
                    self.subdivisions[0] = value; 
                },
                "Subdiv Y" => if let InterfaceParameter::Integer { value, .. } = param { 
                    self.subdivisions[1] = value; 
                },
                "Subdiv Z" => if let InterfaceParameter::Integer { value, .. } = param { 
                    self.subdivisions[2] = value; 
                },
                "Pivot" => if let InterfaceParameter::Enum { value, .. } = param {
                    self.pivot = match value {
                        0 => PivotType::Center,
                        1 => PivotType::Corner,
                        2 => PivotType::Bottom,
                        _ => PivotType::Center,
                    };
                },
                "Generate UVs" => if let Some(val) = param.get_bool() { 
                    self.generate_uvs = val; 
                },
                "Generate Normals" => if let Some(val) = param.get_bool() { 
                    self.generate_normals = val; 
                },
                _ => {}
            }
        }
    }
    
    fn process(&self, inputs: Vec<NodeData>) -> Vec<NodeData> {
        // Generate the geometry
        let geometry = self.generate_geometry();
        
        // If there's a transform input, we would apply it here
        // For now, just return the geometry
        vec![NodeData::Geometry(geometry)]
    }
    
    fn panel_title(&self) -> String {
        format!("Cube Parameters")
    }
    
    fn render_custom_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        
        ui.label("Quick Size Presets:");
        ui.horizontal(|ui| {
            if ui.button("Unit Cube").clicked() {
                self.size = [1.0, 1.0, 1.0];
                changed = true;
            }
            if ui.button("Thin Panel").clicked() {
                self.size = [2.0, 0.1, 1.0];
                changed = true;
            }
            if ui.button("Tall Tower").clicked() {
                self.size = [0.5, 3.0, 0.5];
                changed = true;
            }
        });
        
        ui.separator();
        
        ui.label("Quick Subdivision Presets:");
        ui.horizontal(|ui| {
            if ui.button("Low Detail").clicked() {
                self.subdivisions = [1, 1, 1];
                changed = true;
            }
            if ui.button("Medium Detail").clicked() {
                self.subdivisions = [3, 3, 3];
                changed = true;
            }
            if ui.button("High Detail").clicked() {
                self.subdivisions = [8, 8, 8];
                changed = true;
            }
        });
        
        changed
    }
}