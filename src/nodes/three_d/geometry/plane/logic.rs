//! Plane node logic implementation

use crate::nodes::interface::NodeData;
use crate::nodes::Node;
use crate::workspaces::three_d::usd::usd_engine::{USDSceneData, USDMeshGeometry};
use glam::{Mat4, Vec3, Vec2};

pub struct PlaneLogic {
    mode: String,
    size_x: f32,
    size_y: f32,
    subdivisions_x: i32,
    subdivisions_y: i32,
    smooth_normals: bool,
    generate_uvs: bool,
    node_id: crate::nodes::NodeId,
}

impl PlaneLogic {
    pub fn from_node(node: &Node) -> Self {
        Self {
            mode: node.parameters.get("mode")
                .and_then(|d| if let NodeData::String(s) = d { Some(s.clone()) } else { None })
                .unwrap_or_else(|| "primitive".to_string()),
            size_x: node.parameters.get("size_x")
                .and_then(|d| if let NodeData::Float(f) = d { Some(*f) } else { None })
                .unwrap_or(2.0),
            size_y: node.parameters.get("size_y")
                .and_then(|d| if let NodeData::Float(f) = d { Some(*f) } else { None })
                .unwrap_or(2.0),
            subdivisions_x: node.parameters.get("subdivisions_x")
                .and_then(|d| if let NodeData::Integer(i) = d { Some(*i) } else { None })
                .unwrap_or(1),
            subdivisions_y: node.parameters.get("subdivisions_y")
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
            self.generate_primitive_plane_scene()
        } else {
            self.generate_mesh_plane_scene()
        };
        
        vec![NodeData::USDSceneData(scene_data)]
    }
    
    fn generate_primitive_plane_scene(&self) -> USDSceneData {
        // For primitive mode, create a USD scene with a procedural plane primitive
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        
        // Generate basic plane geometry for primitive representation
        let half_x = self.size_x / 2.0;
        let half_y = self.size_y / 2.0;
        
        // Create a simple quad
        vertices.extend(vec![
            Vec3::new(-half_x, 0.0, -half_y), // Bottom-left
            Vec3::new(half_x, 0.0, -half_y),  // Bottom-right
            Vec3::new(half_x, 0.0, half_y),   // Top-right
            Vec3::new(-half_x, 0.0, half_y),  // Top-left
        ]);
        
        normals.extend(vec![
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ]);
        
        if self.generate_uvs {
            uvs.extend(vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 1.0),
            ]);
        }
        
        // Create two triangles for the quad (counter-clockwise winding)
        indices.extend(vec![
            0, 2, 1, // First triangle (counter-clockwise when looking down at Y-up plane)
            0, 3, 2, // Second triangle (counter-clockwise when looking down at Y-up plane)
        ]);
        
        USDSceneData {
            stage_path: format!("procedural://plane_node_{}", self.node_id),
            meshes: vec![
                USDMeshGeometry {
                    prim_path: "/Plane".to_string(),
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
    
    fn generate_mesh_plane_scene(&self) -> USDSceneData {
        // Generate high-resolution plane mesh with proper tessellation
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        
        let half_x = self.size_x / 2.0;
        let half_y = self.size_y / 2.0;
        
        // Generate vertices with subdivisions
        for y in 0..=self.subdivisions_y {
            for x in 0..=self.subdivisions_x {
                let pos_x = -half_x + (x as f32 / self.subdivisions_x as f32) * self.size_x;
                let pos_z = -half_y + (y as f32 / self.subdivisions_y as f32) * self.size_y;
                
                vertices.push(Vec3::new(pos_x, 0.0, pos_z));
                
                if self.smooth_normals {
                    normals.push(Vec3::new(0.0, 1.0, 0.0));
                } else {
                    normals.push(Vec3::new(0.0, 1.0, 0.0));
                }
                
                if self.generate_uvs {
                    let u = x as f32 / self.subdivisions_x as f32;
                    let v = y as f32 / self.subdivisions_y as f32;
                    uvs.push(Vec2::new(u, v));
                }
            }
        }
        
        // Generate indices for the subdivided plane
        for y in 0..self.subdivisions_y {
            for x in 0..self.subdivisions_x {
                let bottom_left = y * (self.subdivisions_x + 1) + x;
                let bottom_right = bottom_left + 1;
                let top_left = bottom_left + (self.subdivisions_x + 1);
                let top_right = top_left + 1;
                
                // First triangle (counter-clockwise when looking down at Y-up plane)
                indices.push(bottom_left as u32);
                indices.push(top_right as u32);
                indices.push(bottom_right as u32);
                
                // Second triangle (counter-clockwise when looking down at Y-up plane)
                indices.push(bottom_left as u32);
                indices.push(top_left as u32);
                indices.push(top_right as u32);
            }
        }
        
        USDSceneData {
            stage_path: format!("procedural://plane_mesh_node_{}", self.node_id),
            meshes: vec![
                USDMeshGeometry {
                    prim_path: "/PlaneMesh".to_string(),
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