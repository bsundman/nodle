//! Sphere node with interface panel for parameter control

use egui::Color32;
use crate::nodes::{
    Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition,
    NodeInterfacePanel, InterfaceParameter, NodeData, GeometryData,
};
use crate::{interface_float, interface_enum};
use std::f32::consts::PI;

/// Sphere geometry node with interface panel
#[derive(Debug, Clone)]
pub struct SphereNodeWithInterface {
    /// Sphere radius
    pub radius: f32,
    /// Number of longitudinal segments (around the sphere)
    pub rings: i32,
    /// Number of latitudinal segments (from pole to pole)
    pub segments: i32,
    /// Start and end angles for partial spheres
    pub phi_start: f32,
    pub phi_length: f32,
    pub theta_start: f32,
    pub theta_length: f32,
    /// Whether to generate UVs
    pub generate_uvs: bool,
    /// Whether to generate normals
    pub generate_normals: bool,
    /// Sphere type
    pub sphere_type: SphereType,
}

#[derive(Debug, Clone)]
pub enum SphereType {
    Full,
    Hemisphere,
    Quarter,
    Custom,
}

impl Default for SphereNodeWithInterface {
    fn default() -> Self {
        Self {
            radius: 1.0,
            rings: 16,
            segments: 32,
            phi_start: 0.0,
            phi_length: 2.0 * PI,
            theta_start: 0.0,
            theta_length: PI,
            generate_uvs: true,
            generate_normals: true,
            sphere_type: SphereType::Full,
        }
    }
}

impl SphereNodeWithInterface {
    /// Generate sphere geometry based on current parameters
    fn generate_geometry(&self) -> GeometryData {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        
        // Generate sphere vertices
        for i in 0..=self.rings {
            let theta = self.theta_start + (i as f32 / self.rings as f32) * self.theta_length;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();
            
            for j in 0..=self.segments {
                let phi = self.phi_start + (j as f32 / self.segments as f32) * self.phi_length;
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();
                
                // Vertex position
                let x = self.radius * sin_theta * cos_phi;
                let y = self.radius * cos_theta;
                let z = self.radius * sin_theta * sin_phi;
                
                vertices.push([x, y, z]);
                
                // Normal (normalized position for a sphere)
                if self.generate_normals {
                    normals.push([sin_theta * cos_phi, cos_theta, sin_theta * sin_phi]);
                }
                
                // UV coordinates
                if self.generate_uvs {
                    let u = j as f32 / self.segments as f32;
                    let v = i as f32 / self.rings as f32;
                    uvs.push([u, v]);
                }
            }
        }
        
        // Generate indices
        for i in 0..self.rings {
            for j in 0..self.segments {
                let first = i * (self.segments + 1) + j;
                let second = first + self.segments + 1;
                
                // First triangle
                indices.push(first as u32);
                indices.push(second as u32);
                indices.push((first + 1) as u32);
                
                // Second triangle
                indices.push(second as u32);
                indices.push((second + 1) as u32);
                indices.push((first + 1) as u32);
            }
        }
        
        GeometryData {
            id: format!("sphere_{}", std::ptr::addr_of!(*self) as usize),
            vertices,
            indices,
            normals,
            uvs,
            material_id: None,
        }
    }
    
    /// Update sphere parameters based on type
    fn update_for_type(&mut self) {
        match self.sphere_type {
            SphereType::Full => {
                self.phi_start = 0.0;
                self.phi_length = 2.0 * PI;
                self.theta_start = 0.0;
                self.theta_length = PI;
            }
            SphereType::Hemisphere => {
                self.phi_start = 0.0;
                self.phi_length = 2.0 * PI;
                self.theta_start = 0.0;
                self.theta_length = PI * 0.5;
            }
            SphereType::Quarter => {
                self.phi_start = 0.0;
                self.phi_length = PI * 0.5;
                self.theta_start = 0.0;
                self.theta_length = PI * 0.5;
            }
            SphereType::Custom => {
                // Keep current values
            }
        }
    }
}

impl NodeFactory for SphereNodeWithInterface {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "3D_SphereInterface",
            "Sphere (Interface)",
            NodeCategory::new(&["3D", "Geometry"]),
            "Creates a sphere primitive with interface panel controls"
        )
        .with_color(Color32::from_rgb(100, 150, 200)) // Blue tint for geometry
        .with_icon("🔵")
        .with_inputs(vec![
            // Optional transform input for positioning
            PortDefinition::optional("Transform", DataType::Any)
                .with_description("Optional transform matrix to position the sphere"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Geometry", DataType::Any)
                .with_description("Generated sphere geometry"),
        ])
        .with_tags(vec!["3d", "geometry", "primitive", "sphere", "interface"])
        .with_processing_cost(crate::nodes::ProcessingCost::Medium)
        .with_workspace_compatibility(vec!["3d", "modeling"])
    }
}

impl NodeInterfacePanel for SphereNodeWithInterface {
    fn get_parameters(&self) -> Vec<(&'static str, InterfaceParameter)> {
        vec![
            ("Radius", interface_float!(self.radius, 0.1, 10.0, 0.1)),
            ("Rings", InterfaceParameter::Integer { 
                value: self.rings, min: 3, max: 64 
            }),
            ("Segments", InterfaceParameter::Integer { 
                value: self.segments, min: 3, max: 128 
            }),
            ("Type", interface_enum!(
                match self.sphere_type {
                    SphereType::Full => 0,
                    SphereType::Hemisphere => 1,
                    SphereType::Quarter => 2,
                    SphereType::Custom => 3,
                },
                "Full", "Hemisphere", "Quarter", "Custom"
            )),
            ("Phi Start", interface_float!(self.phi_start, 0.0, 2.0 * PI, 0.1)),
            ("Phi Length", interface_float!(self.phi_length, 0.0, 2.0 * PI, 0.1)),
            ("Theta Start", interface_float!(self.theta_start, 0.0, PI, 0.1)),
            ("Theta Length", interface_float!(self.theta_length, 0.0, PI, 0.1)),
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
                "Radius" => if let Some(val) = param.get_float() { self.radius = val; },
                "Rings" => if let InterfaceParameter::Integer { value, .. } = param { 
                    self.rings = value; 
                },
                "Segments" => if let InterfaceParameter::Integer { value, .. } = param { 
                    self.segments = value; 
                },
                "Type" => if let InterfaceParameter::Enum { value, .. } = param {
                    let new_type = match value {
                        0 => SphereType::Full,
                        1 => SphereType::Hemisphere,
                        2 => SphereType::Quarter,
                        3 => SphereType::Custom,
                        _ => SphereType::Full,
                    };
                    if !matches!(self.sphere_type, SphereType::Custom) || value != 3 {
                        self.sphere_type = new_type;
                        self.update_for_type();
                    }
                },
                "Phi Start" => if let Some(val) = param.get_float() { 
                    self.phi_start = val;
                    self.sphere_type = SphereType::Custom;
                },
                "Phi Length" => if let Some(val) = param.get_float() { 
                    self.phi_length = val;
                    self.sphere_type = SphereType::Custom;
                },
                "Theta Start" => if let Some(val) = param.get_float() { 
                    self.theta_start = val;
                    self.sphere_type = SphereType::Custom;
                },
                "Theta Length" => if let Some(val) = param.get_float() { 
                    self.theta_length = val;
                    self.sphere_type = SphereType::Custom;
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
        format!("Sphere Parameters")
    }
    
    fn render_custom_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        
        ui.label("Quality Presets:");
        ui.horizontal(|ui| {
            if ui.button("Low").clicked() {
                self.rings = 8;
                self.segments = 16;
                changed = true;
            }
            if ui.button("Medium").clicked() {
                self.rings = 16;
                self.segments = 32;
                changed = true;
            }
            if ui.button("High").clicked() {
                self.rings = 32;
                self.segments = 64;
                changed = true;
            }
            if ui.button("Ultra").clicked() {
                self.rings = 64;
                self.segments = 128;
                changed = true;
            }
        });
        
        ui.separator();
        
        ui.label("Size Presets:");
        ui.horizontal(|ui| {
            if ui.button("Marble").clicked() {
                self.radius = 0.05;
                changed = true;
            }
            if ui.button("Ball").clicked() {
                self.radius = 0.5;
                changed = true;
            }
            if ui.button("Planet").clicked() {
                self.radius = 5.0;
                changed = true;
            }
        });
        
        changed
    }
}