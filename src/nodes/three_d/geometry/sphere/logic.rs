//! Sphere node logic implementation

use crate::nodes::interface::NodeData;
use crate::nodes::Node;
use crate::workspaces::three_d::usd::usd_engine::{USDSceneData, USDMeshGeometry};
use glam::{Mat4, Vec3, Vec2};
use std::f32::consts::PI;

pub struct SphereLogic {
    mode: String,
    radius: f32,
    rings: i32,
    segments: i32,
    smooth_normals: bool,
    generate_uvs: bool,
    node_id: crate::nodes::NodeId,
}

impl SphereLogic {
    pub fn from_node(node: &Node) -> Self {
        Self {
            mode: node.parameters.get("mode")
                .and_then(|d| if let NodeData::String(s) = d { Some(s.clone()) } else { None })
                .unwrap_or_else(|| "primitive".to_string()),
            radius: node.parameters.get("radius")
                .and_then(|d| if let NodeData::Float(f) = d { Some(*f) } else { None })
                .unwrap_or(1.0),
            rings: node.parameters.get("rings")
                .and_then(|d| if let NodeData::Integer(i) = d { Some(*i) } else { None })
                .unwrap_or(16),
            segments: node.parameters.get("segments")
                .and_then(|d| if let NodeData::Integer(i) = d { Some(*i) } else { None })
                .unwrap_or(20),
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
            self.generate_primitive_sphere_scene()
        } else {
            self.generate_mesh_sphere_scene()
        };
        
        println!("🔵 SphereLogic::process generated USD scene with stage_path: '{}'", scene_data.stage_path);
        vec![NodeData::USDSceneData(scene_data)]
    }
    
    fn generate_primitive_sphere_scene(&self) -> USDSceneData {
        // For primitive mode, create a USD scene with a procedural sphere primitive
        // Generate simple sphere geometry for primitive representation
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        
        // Generate basic sphere vertices using spherical coordinates
        let rings = 8; // Lower resolution for primitive mode
        let segments = 12;
        
        for ring in 0..=rings {
            let theta = PI * ring as f32 / rings as f32;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();
            
            for segment in 0..=segments {
                let phi = 2.0 * PI * segment as f32 / segments as f32;
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();
                
                let x = self.radius * sin_theta * cos_phi;
                let y = self.radius * cos_theta;
                let z = self.radius * sin_theta * sin_phi;
                
                vertices.push(Vec3::new(x, y, z));
                normals.push(Vec3::new(x, y, z).normalize());
                
                if self.generate_uvs {
                    let u = segment as f32 / segments as f32;
                    let v = ring as f32 / rings as f32;
                    uvs.push(Vec2::new(u, v));
                }
            }
        }
        
        // Generate indices
        for ring in 0..rings {
            for segment in 0..segments {
                let current = ring * (segments + 1) + segment;
                let next = current + segments + 1;
                
                // Create two triangles for each quad
                indices.push(current as u32);
                indices.push((current + 1) as u32);
                indices.push((next + 1) as u32);
                
                indices.push(current as u32);
                indices.push((next + 1) as u32);
                indices.push(next as u32);
            }
        }
        
        USDSceneData {
            stage_path: format!("procedural://sphere_node_{}", self.node_id),
            meshes: vec![
                USDMeshGeometry {
                    prim_path: "/Sphere".to_string(),
                    vertices,
                    indices,
                    normals,
                    uvs,
                    vertex_colors: None,
                    transform: Mat4::IDENTITY,
                    primvars: vec![],
                }
            ],
            lights: vec![],
            materials: vec![],
            up_axis: "Y".to_string(),
        }
    }
    
    fn generate_mesh_sphere_scene(&self) -> USDSceneData {
        // Generate high-resolution sphere mesh with proper tessellation
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        
        // Generate sphere vertices using spherical coordinates
        for ring in 0..=self.rings {
            let theta = PI * ring as f32 / self.rings as f32;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();
            
            for segment in 0..=self.segments {
                let phi = 2.0 * PI * segment as f32 / self.segments as f32;
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();
                
                let x = self.radius * sin_theta * cos_phi;
                let y = self.radius * cos_theta;
                let z = self.radius * sin_theta * sin_phi;
                
                vertices.push(Vec3::new(x, y, z));
                
                // Generate normals
                if self.smooth_normals {
                    normals.push(Vec3::new(x, y, z).normalize());
                } else {
                    // For flat shading, we'd need to compute face normals
                    normals.push(Vec3::new(x, y, z).normalize());
                }
                
                // Generate UVs
                if self.generate_uvs {
                    let u = segment as f32 / self.segments as f32;
                    let v = ring as f32 / self.rings as f32;
                    uvs.push(Vec2::new(u, v));
                }
            }
        }
        
        // Generate sphere indices
        for ring in 0..self.rings {
            for segment in 0..self.segments {
                let current = ring * (self.segments + 1) + segment;
                let next = current + self.segments + 1;
                
                // Create two triangles for each quad
                indices.push(current as u32);
                indices.push((current + 1) as u32);
                indices.push((next + 1) as u32);
                
                indices.push(current as u32);
                indices.push((next + 1) as u32);
                indices.push(next as u32);
            }
        }
        
        USDSceneData {
            stage_path: format!("procedural://sphere_mesh_node_{}", self.node_id),
            meshes: vec![
                USDMeshGeometry {
                    prim_path: "/SphereMesh".to_string(),
                    vertices,
                    indices,
                    normals,
                    uvs,
                    vertex_colors: None,
                    transform: Mat4::IDENTITY,
                    primvars: vec![],
                }
            ],
            lights: vec![],
            materials: vec![],
            up_axis: "Y".to_string(),
        }
    }
}