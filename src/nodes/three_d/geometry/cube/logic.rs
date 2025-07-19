//! Cube node logic implementation

use crate::nodes::interface::NodeData;
use crate::nodes::Node;
use crate::workspaces::three_d::usd::usd_engine::{USDSceneData, USDMeshGeometry};
use glam::{Mat4, Vec3, Vec2};

pub struct CubeLogic {
    mode: String,
    size_x: f32,
    size_y: f32,
    size_z: f32,
    subdivisions_x: i32,
    subdivisions_y: i32,
    subdivisions_z: i32,
    smooth_normals: bool,
    generate_uvs: bool,
    node_id: crate::nodes::NodeId,
}

impl CubeLogic {
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
            size_z: node.parameters.get("size_z")
                .and_then(|d| if let NodeData::Float(f) = d { Some(*f) } else { None })
                .unwrap_or(2.0),
            subdivisions_x: node.parameters.get("subdivisions_x")
                .and_then(|d| if let NodeData::Integer(i) = d { Some(*i) } else { None })
                .unwrap_or(1),
            subdivisions_y: node.parameters.get("subdivisions_y")
                .and_then(|d| if let NodeData::Integer(i) = d { Some(*i) } else { None })
                .unwrap_or(1),
            subdivisions_z: node.parameters.get("subdivisions_z")
                .and_then(|d| if let NodeData::Integer(i) = d { Some(*i) } else { None })
                .unwrap_or(1),
            smooth_normals: node.parameters.get("smooth_normals")
                .and_then(|d| if let NodeData::Boolean(b) = d { Some(*b) } else { None })
                .unwrap_or(false),
            generate_uvs: node.parameters.get("generate_uvs")
                .and_then(|d| if let NodeData::Boolean(b) = d { Some(*b) } else { None })
                .unwrap_or(true),
            node_id: node.id,
        }
    }
    
    pub fn process(&mut self, _inputs: Vec<NodeData>) -> Vec<NodeData> {
        // Generate USD scene data based on mode
        let scene_data = if self.mode == "primitive" {
            self.generate_primitive_cube_scene()
        } else {
            self.generate_mesh_cube_scene()
        };
        
        println!("📦 CubeLogic::process generated USD scene with stage_path: '{}'", scene_data.stage_path);
        vec![NodeData::USDSceneData(scene_data)]
    }
    
    fn generate_primitive_cube_scene(&self) -> USDSceneData {
        // For primitive mode, create a USD scene with a procedural cube primitive
        USDSceneData {
            stage_path: format!("procedural://cube_node_{}", self.node_id),
            meshes: vec![
                USDMeshGeometry {
                    prim_path: "/Cube".to_string(),
                    vertices: vec![
                        Vec3::new(-self.size_x/2.0, -self.size_y/2.0, -self.size_z/2.0),
                        Vec3::new(self.size_x/2.0, -self.size_y/2.0, -self.size_z/2.0),
                        Vec3::new(self.size_x/2.0, self.size_y/2.0, -self.size_z/2.0),
                        Vec3::new(-self.size_x/2.0, self.size_y/2.0, -self.size_z/2.0),
                        Vec3::new(-self.size_x/2.0, -self.size_y/2.0, self.size_z/2.0),
                        Vec3::new(self.size_x/2.0, -self.size_y/2.0, self.size_z/2.0),
                        Vec3::new(self.size_x/2.0, self.size_y/2.0, self.size_z/2.0),
                        Vec3::new(-self.size_x/2.0, self.size_y/2.0, self.size_z/2.0),
                    ],
                    indices: vec![
                        // Front face (z = +half_z)
                        4, 5, 6, 4, 6, 7,
                        // Back face (z = -half_z)
                        1, 0, 3, 1, 3, 2,
                        // Left face (x = -half_x)
                        0, 4, 7, 0, 7, 3,
                        // Right face (x = +half_x)
                        5, 1, 2, 5, 2, 6,
                        // Bottom face (y = -half_y)
                        0, 1, 5, 0, 5, 4,
                        // Top face (y = +half_y)
                        3, 7, 6, 3, 6, 2,
                    ],
                    normals: vec![
                        Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, -1.0),
                        Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0),
                    ],
                    uvs: vec![
                        Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0),
                        Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0),
                    ],
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
    
    fn generate_mesh_cube_scene(&self) -> USDSceneData {
        // Generate subdivided cube mesh with proper tessellation
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        
        // Generate cube with subdivisions
        let half_x = self.size_x / 2.0;
        let half_y = self.size_y / 2.0;
        let half_z = self.size_z / 2.0;
        
        // For now, create a simple subdivided cube (simplified implementation)
        // This would be expanded to generate proper tessellation for all faces
        
        // Generate vertices for all 8 corners
        vertices.push(Vec3::new(-half_x, -half_y, -half_z));
        vertices.push(Vec3::new(half_x, -half_y, -half_z));
        vertices.push(Vec3::new(half_x, half_y, -half_z));
        vertices.push(Vec3::new(-half_x, half_y, -half_z));
        vertices.push(Vec3::new(-half_x, -half_y, half_z));
        vertices.push(Vec3::new(half_x, -half_y, half_z));
        vertices.push(Vec3::new(half_x, half_y, half_z));
        vertices.push(Vec3::new(-half_x, half_y, half_z));
        
        // Generate indices for all faces (counter-clockwise winding)
        indices.extend(vec![
            // Front face (z = +half_z)
            4, 5, 6, 4, 6, 7,
            // Back face (z = -half_z)
            1, 0, 3, 1, 3, 2,
            // Left face (x = -half_x)
            0, 4, 7, 0, 7, 3,
            // Right face (x = +half_x)
            5, 1, 2, 5, 2, 6,
            // Bottom face (y = -half_y)
            0, 1, 5, 0, 5, 4,
            // Top face (y = +half_y)
            3, 7, 6, 3, 6, 2,
        ]);
        
        // Generate normals
        normals.extend(vec![
            Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0),
        ]);
        
        // Generate UVs
        uvs.extend(vec![
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0),
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0),
        ]);
        
        USDSceneData {
            stage_path: format!("procedural://cube_mesh_node_{}", self.node_id),
            meshes: vec![
                USDMeshGeometry {
                    prim_path: "/CubeMesh".to_string(),
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