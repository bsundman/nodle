//! Cone node logic implementation

use crate::nodes::interface::NodeData;
use crate::nodes::Node;
use crate::workspaces::three_d::usd::usd_engine::{USDSceneData, USDMeshGeometry};
use glam::{Mat4, Vec3, Vec2};
use std::f32::consts::PI;

pub struct ConeLogic {
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

impl ConeLogic {
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
            self.generate_primitive_cone_scene()
        } else {
            self.generate_mesh_cone_scene()
        };
        
        vec![NodeData::USDSceneData(scene_data)]
    }
    
    fn generate_primitive_cone_scene(&self) -> USDSceneData {
        // For primitive mode, create a USD scene with a procedural cone primitive
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        
        // Generate basic cone geometry for primitive representation
        let segments = 12; // Lower resolution for primitive mode
        let half_height = self.height / 2.0;
        
        // Add apex vertex
        vertices.push(Vec3::new(0.0, half_height, 0.0));
        normals.push(Vec3::new(0.0, 1.0, 0.0));
        if self.generate_uvs {
            uvs.push(Vec2::new(0.5, 1.0));
        }
        
        // Generate vertices for base ring
        for i in 0..=segments {
            let angle = 2.0 * PI * i as f32 / segments as f32;
            let cos_a = angle.cos();
            let sin_a = angle.sin();
            
            // Base ring vertex
            vertices.push(Vec3::new(self.radius * cos_a, -half_height, self.radius * sin_a));
            
            // Calculate normal for cone side
            let side_normal = Vec3::new(cos_a, 0.5, sin_a).normalize();
            normals.push(side_normal);
            
            if self.generate_uvs {
                uvs.push(Vec2::new(i as f32 / segments as f32, 0.0));
            }
        }
        
        // Generate indices for cone sides
        for i in 0..segments {
            let base_current = i + 1;
            let base_next = (i + 1) % segments + 1;
            
            // Triangle from apex to base edge (counter-clockwise when looking from outside)
            indices.push(0); // apex
            indices.push(base_next as u32);
            indices.push(base_current as u32);
        }
        
        // Add center vertex for base cap
        let center_base = vertices.len() as u32;
        vertices.push(Vec3::new(0.0, -half_height, 0.0));
        normals.push(Vec3::new(0.0, -1.0, 0.0));
        if self.generate_uvs {
            uvs.push(Vec2::new(0.5, 0.5));
        }
        
        // Generate indices for base cap
        for i in 0..segments {
            let base_current = i + 1;
            let base_next = (i + 1) % segments + 1;
            
            // Triangle for base cap (looking up from below, counter-clockwise)
            indices.push(center_base);
            indices.push(base_current as u32);
            indices.push(base_next as u32);
        }
        
        USDSceneData {
            stage_path: format!("procedural://cone_node_{}", self.node_id),
            meshes: vec![
                USDMeshGeometry {
                    prim_path: "/Cone".to_string(),
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
    
    fn generate_mesh_cone_scene(&self) -> USDSceneData {
        // Generate high-resolution cone mesh with proper tessellation
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        
        let half_height = self.height / 2.0;
        
        // Add apex vertex
        vertices.push(Vec3::new(0.0, half_height, 0.0));
        normals.push(Vec3::new(0.0, 1.0, 0.0));
        if self.generate_uvs {
            uvs.push(Vec2::new(0.5, 1.0));
        }
        
        // Generate vertices for cone sides with subdivisions
        for h in 1..=self.subdivisions_height {
            let y = half_height - (h as f32 / self.subdivisions_height as f32) * self.height;
            let r = self.radius * (h as f32 / self.subdivisions_height as f32);
            let v = 1.0 - (h as f32 / self.subdivisions_height as f32);
            
            for i in 0..=self.subdivisions_axis {
                let angle = 2.0 * PI * i as f32 / self.subdivisions_axis as f32;
                let cos_a = angle.cos();
                let sin_a = angle.sin();
                
                vertices.push(Vec3::new(r * cos_a, y, r * sin_a));
                
                if self.smooth_normals {
                    // Calculate smooth normal for cone surface
                    let side_normal = Vec3::new(cos_a, 0.5, sin_a).normalize();
                    normals.push(side_normal);
                } else {
                    normals.push(Vec3::new(cos_a, 0.5, sin_a).normalize());
                }
                
                if self.generate_uvs {
                    uvs.push(Vec2::new(i as f32 / self.subdivisions_axis as f32, v));
                }
            }
        }
        
        // Generate indices for cone sides
        for h in 0..self.subdivisions_height {
            if h == 0 {
                // Connect apex to first ring
                for i in 0..self.subdivisions_axis {
                    let current = 1 + i;
                    let next = 1 + (i + 1) % (self.subdivisions_axis + 1);
                    
                    indices.push(0); // apex
                    indices.push(current as u32);
                    indices.push(next as u32);
                }
            } else {
                // Connect rings
                for i in 0..self.subdivisions_axis {
                    let current = 1 + (h - 1) * (self.subdivisions_axis + 1) + i;
                    let next = current + self.subdivisions_axis + 1;
                    
                    // Triangle 1
                    indices.push(current as u32);
                    indices.push((current + 1) as u32);
                    indices.push((next + 1) as u32);
                    
                    // Triangle 2
                    indices.push(current as u32);
                    indices.push((next + 1) as u32);
                    indices.push(next as u32);
                }
            }
        }
        
        // Add center vertex for base cap
        let center_base = vertices.len() as u32;
        vertices.push(Vec3::new(0.0, -half_height, 0.0));
        normals.push(Vec3::new(0.0, -1.0, 0.0));
        if self.generate_uvs {
            uvs.push(Vec2::new(0.5, 0.5));
        }
        
        // Generate indices for base cap
        let base_ring_start = 1 + (self.subdivisions_height - 1) * (self.subdivisions_axis + 1);
        for i in 0..self.subdivisions_axis {
            let current = base_ring_start + i;
            let next = base_ring_start + (i + 1) % (self.subdivisions_axis + 1);
            
            // Triangle for base cap
            indices.push(center_base);
            indices.push(next as u32);
            indices.push(current as u32);
        }
        
        USDSceneData {
            stage_path: format!("procedural://cone_mesh_node_{}", self.node_id),
            meshes: vec![
                USDMeshGeometry {
                    prim_path: "/ConeMesh".to_string(),
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