//! Cylinder node logic implementation

use crate::nodes::interface::NodeData;
use crate::nodes::Node;
use crate::workspaces::three_d::usd::usd_engine::{USDSceneData, USDMeshGeometry};
use glam::{Mat4, Vec3, Vec2};
use std::f32::consts::PI;

pub struct CylinderLogic {
    mode: String,
    radius: f32,
    height: f32,
    subdivisions_axis: i32,
    subdivisions_caps: i32,
    subdivisions_height: i32,
    smooth_normals: bool,
    generate_uvs: bool,
    node_id: crate::nodes::NodeId,
}

impl CylinderLogic {
    pub fn from_node(node: &Node) -> Self {
        Self {
            mode: node.parameters.get("mode")
                .and_then(|d| if let NodeData::String(s) = d { Some(s.clone()) } else { None })
                .unwrap_or_else(|| "primitive".to_string()),
            radius: node.parameters.get("radius")
                .and_then(|d| if let NodeData::Float(f) = d { Some(*f) } else { None })
                .unwrap_or(1.0),
            height: node.parameters.get("height")
                .and_then(|d| if let NodeData::Float(f) = d { Some(*f) } else { None })
                .unwrap_or(2.0),
            subdivisions_axis: node.parameters.get("subdivisions_axis")
                .and_then(|d| if let NodeData::Integer(i) = d { Some(*i) } else { None })
                .unwrap_or(20),
            subdivisions_caps: node.parameters.get("subdivisions_caps")
                .and_then(|d| if let NodeData::Integer(i) = d { Some(*i) } else { None })
                .unwrap_or(1),
            subdivisions_height: node.parameters.get("subdivisions_height")
                .and_then(|d| if let NodeData::Integer(i) = d { Some(*i) } else { None })
                .unwrap_or(1),
            smooth_normals: node.parameters.get("smooth_normals")
                .and_then(|d| if let NodeData::Boolean(b) = d { Some(*b) } else { None })
                .unwrap_or(true),
            generate_uvs: node.parameters.get("generate_uvs")
                .and_then(|d| if let NodeData::Boolean(b) = d { Some(*b) } else { None })
                .unwrap_or(true),
            node_id: node.id,
        }
    }
    
    pub fn process(&mut self, _inputs: Vec<NodeData>) -> Vec<NodeData> {
        // Generate USD scene data based on mode
        let scene_data = if self.mode == "primitive" {
            self.generate_primitive_cylinder_scene()
        } else {
            self.generate_mesh_cylinder_scene()
        };
        
        vec![NodeData::USDSceneData(scene_data)]
    }
    
    fn generate_primitive_cylinder_scene(&self) -> USDSceneData {
        // For primitive mode, create a USD scene with a procedural cylinder primitive
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        
        // Generate basic cylinder geometry for primitive representation
        let segments = 12; // Lower resolution for primitive mode
        let half_height = self.height / 2.0;
        
        // Generate vertices for cylinder sides
        for i in 0..=segments {
            let angle = 2.0 * PI * i as f32 / segments as f32;
            let cos_a = angle.cos();
            let sin_a = angle.sin();
            
            // Bottom ring
            vertices.push(Vec3::new(self.radius * cos_a, -half_height, self.radius * sin_a));
            normals.push(Vec3::new(cos_a, 0.0, sin_a));
            if self.generate_uvs {
                uvs.push(Vec2::new(i as f32 / segments as f32, 0.0));
            }
            
            // Top ring
            vertices.push(Vec3::new(self.radius * cos_a, half_height, self.radius * sin_a));
            normals.push(Vec3::new(cos_a, 0.0, sin_a));
            if self.generate_uvs {
                uvs.push(Vec2::new(i as f32 / segments as f32, 1.0));
            }
        }
        
        // Generate indices for cylinder sides
        for i in 0..segments {
            let bottom_current = i * 2;
            let bottom_next = (i + 1) * 2;
            let top_current = bottom_current + 1;
            let top_next = bottom_next + 1;
            
            // Triangle 1 (counter-clockwise when looking from outside)
            indices.push(bottom_current as u32);
            indices.push(bottom_next as u32);
            indices.push(top_current as u32);
            
            // Triangle 2
            indices.push(bottom_next as u32);
            indices.push(top_next as u32);
            indices.push(top_current as u32);
        }
        
        // Add center vertices for caps
        let center_bottom = vertices.len() as u32;
        vertices.push(Vec3::new(0.0, -half_height, 0.0));
        normals.push(Vec3::new(0.0, -1.0, 0.0));
        if self.generate_uvs {
            uvs.push(Vec2::new(0.5, 0.5));
        }
        
        let center_top = vertices.len() as u32;
        vertices.push(Vec3::new(0.0, half_height, 0.0));
        normals.push(Vec3::new(0.0, 1.0, 0.0));
        if self.generate_uvs {
            uvs.push(Vec2::new(0.5, 0.5));
        }
        
        // Generate indices for caps
        for i in 0..segments {
            let bottom_current = i * 2;
            let bottom_next = (i + 1) * 2;
            let top_current = bottom_current + 1;
            let top_next = bottom_next + 1;
            
            // Bottom cap (looking up from below, counter-clockwise)
            indices.push(center_bottom);
            indices.push(bottom_current as u32);
            indices.push(bottom_next as u32);
            
            // Top cap (looking down from above, counter-clockwise)
            indices.push(center_top);
            indices.push(top_next as u32);
            indices.push(top_current as u32);
        }
        
        USDSceneData {
            stage_path: format!("procedural://cylinder_node_{}", self.node_id),
            meshes: vec![
                USDMeshGeometry {
                    prim_path: "/Cylinder".to_string(),
                    vertices,
                    indices,
                    normals,
                    uvs,
                    vertex_colors: None,
                    transform: Mat4::IDENTITY,
                }
            ],
            lights: vec![],
            materials: vec![],
            up_axis: "Y".to_string(),
        }
    }
    
    fn generate_mesh_cylinder_scene(&self) -> USDSceneData {
        // Generate high-resolution cylinder mesh with proper tessellation
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        
        let half_height = self.height / 2.0;
        
        // Generate vertices for cylinder sides with subdivisions
        for h in 0..=self.subdivisions_height {
            let y = -half_height + (h as f32 / self.subdivisions_height as f32) * self.height;
            let v = h as f32 / self.subdivisions_height as f32;
            
            for i in 0..=self.subdivisions_axis {
                let angle = 2.0 * PI * i as f32 / self.subdivisions_axis as f32;
                let cos_a = angle.cos();
                let sin_a = angle.sin();
                
                vertices.push(Vec3::new(self.radius * cos_a, y, self.radius * sin_a));
                
                if self.smooth_normals {
                    normals.push(Vec3::new(cos_a, 0.0, sin_a));
                } else {
                    normals.push(Vec3::new(cos_a, 0.0, sin_a));
                }
                
                if self.generate_uvs {
                    uvs.push(Vec2::new(i as f32 / self.subdivisions_axis as f32, v));
                }
            }
        }
        
        // Generate indices for cylinder sides
        for h in 0..self.subdivisions_height {
            for i in 0..self.subdivisions_axis {
                let current = h * (self.subdivisions_axis + 1) + i;
                let next = current + self.subdivisions_axis + 1;
                
                // Triangle 1 (counter-clockwise when looking from outside)
                indices.push(current as u32);
                indices.push(next as u32);
                indices.push((current + 1) as u32);
                
                // Triangle 2
                indices.push(next as u32);
                indices.push((next + 1) as u32);
                indices.push((current + 1) as u32);
            }
        }
        
        // Add caps with subdivisions
        let bottom_center = vertices.len() as u32;
        vertices.push(Vec3::new(0.0, -half_height, 0.0));
        normals.push(Vec3::new(0.0, -1.0, 0.0));
        if self.generate_uvs {
            uvs.push(Vec2::new(0.5, 0.5));
        }
        
        let top_center = vertices.len() as u32;
        vertices.push(Vec3::new(0.0, half_height, 0.0));
        normals.push(Vec3::new(0.0, 1.0, 0.0));
        if self.generate_uvs {
            uvs.push(Vec2::new(0.5, 0.5));
        }
        
        // Generate indices for caps
        for i in 0..self.subdivisions_axis {
            let current = i;
            let next = (i + 1) % (self.subdivisions_axis + 1);
            let top_current = (self.subdivisions_height * (self.subdivisions_axis + 1)) + i;
            let top_next = (self.subdivisions_height * (self.subdivisions_axis + 1)) + next;
            
            // Bottom cap (looking up from below, counter-clockwise)
            indices.push(bottom_center);
            indices.push(current as u32);
            indices.push(next as u32);
            
            // Top cap (looking down from above, counter-clockwise)
            indices.push(top_center);
            indices.push(top_next as u32);
            indices.push(top_current as u32);
        }
        
        USDSceneData {
            stage_path: format!("procedural://cylinder_mesh_node_{}", self.node_id),
            meshes: vec![
                USDMeshGeometry {
                    prim_path: "/CylinderMesh".to_string(),
                    vertices,
                    indices,
                    normals,
                    uvs,
                    vertex_colors: None,
                    transform: Mat4::IDENTITY,
                }
            ],
            lights: vec![],
            materials: vec![],
            up_axis: "Y".to_string(),
        }
    }
}