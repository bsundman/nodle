//! Reverse node logic for 3D modifications
//! 
//! Provides comprehensive reversal operations for USD geometry data including:
//! - Normal reversal
//! - Face winding order reversal  
//! - Point order reversal
//! - Axis mirroring
//! - UV coordinate reversal

use crate::nodes::interface::NodeData;
use crate::workspaces::three_d::usd::usd_engine::USDMeshGeometry;
use glam::{Vec3, Mat4};

#[derive(Debug, Clone, PartialEq)]
pub enum MirrorAxis {
    X,
    Y, 
    Z,
    XY,
    XZ,
    YZ,
    None,
}

impl Default for MirrorAxis {
    fn default() -> Self {
        MirrorAxis::None
    }
}

/// Reverse node processing logic
#[derive(Debug, Clone)]
pub struct ReverseLogic {
    pub reverse_normals: bool,
    pub reverse_face_winding: bool,
    pub reverse_point_order: bool,
    pub mirror_axis: MirrorAxis,
    pub reverse_uvs_u: bool,
    pub reverse_uvs_v: bool,
    pub flip_vertex_colors: bool,
    pub invert_transforms: bool,
}

impl Default for ReverseLogic {
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

impl ReverseLogic {
    /// Process USD scene data with reverse operations
    pub fn process(&self, inputs: Vec<NodeData>) -> Vec<NodeData> {
        // Check if we have USD scene data input
        if inputs.is_empty() {
            return vec![NodeData::None];
        }

        match &inputs[0] {
            NodeData::USDSceneData(usd_scene_data) => {
                println!("🔄 Reverse: Processing USD scene with {} meshes", usd_scene_data.meshes.len());
                
                // Clone the scene data for modification
                let mut modified_scene = usd_scene_data.clone();
                
                // Apply reverse operations to each mesh
                for mesh in &mut modified_scene.meshes {
                    self.apply_reverse_operations(mesh);
                }
                
                println!("✅ Reverse: Applied reverse operations to {} meshes", modified_scene.meshes.len());
                println!("✅ Reverse: Output stage path: {}", modified_scene.stage_path);
                vec![NodeData::USDSceneData(modified_scene)]
            }
            _ => {
                println!("⚠️ Reverse: Input is not USD scene data, passing through");
                inputs
            }
        }
    }
    
    /// Apply all enabled reverse operations to a single mesh
    fn apply_reverse_operations(&self, mesh: &mut USDMeshGeometry) {
        println!("🔄 Reverse: Processing mesh '{}'", mesh.prim_path);
        
        // 1. Mirror geometry along specified axis
        if self.mirror_axis != MirrorAxis::None {
            self.apply_axis_mirroring(mesh);
        }
        
        // 2. Reverse normals
        if self.reverse_normals {
            self.reverse_mesh_normals(mesh);
        }
        
        // 3. Reverse face winding order
        if self.reverse_face_winding {
            self.reverse_face_winding_order(mesh);
        }
        
        // 4. Reverse point order in faces
        if self.reverse_point_order {
            self.reverse_point_order_in_faces(mesh);
        }
        
        // 5. Reverse UV coordinates
        if self.reverse_uvs_u || self.reverse_uvs_v {
            self.reverse_uv_coordinates(mesh);
        }
        
        // 6. Flip vertex colors (invert RGB values)
        if self.flip_vertex_colors {
            self.flip_vertex_colors(mesh);
        }
        
        // 7. Invert transform matrices
        if self.invert_transforms {
            self.invert_mesh_transform(mesh);
        }
    }
    
    /// Mirror geometry along specified axis
    fn apply_axis_mirroring(&self, mesh: &mut USDMeshGeometry) {
        let mirror_matrix = self.get_mirror_matrix();
        
        // Transform vertices
        for vertex in &mut mesh.vertices {
            *vertex = mirror_matrix.transform_point3(*vertex);
        }
        
        // Transform normals (use transpose inverse for normals)
        let normal_matrix = mirror_matrix.inverse().transpose();
        for normal in &mut mesh.normals {
            *normal = normal_matrix.transform_vector3(*normal).normalize();
        }
        
        // Update mesh transform
        mesh.transform = mirror_matrix * mesh.transform;
        
        println!("🪞 Reverse: Applied {:?} axis mirroring", self.mirror_axis);
    }
    
    /// Get mirror transformation matrix for the specified axis
    fn get_mirror_matrix(&self) -> Mat4 {
        match self.mirror_axis {
            MirrorAxis::X => Mat4::from_scale(Vec3::new(-1.0, 1.0, 1.0)),
            MirrorAxis::Y => Mat4::from_scale(Vec3::new(1.0, -1.0, 1.0)),
            MirrorAxis::Z => Mat4::from_scale(Vec3::new(1.0, 1.0, -1.0)),
            MirrorAxis::XY => Mat4::from_scale(Vec3::new(-1.0, -1.0, 1.0)),
            MirrorAxis::XZ => Mat4::from_scale(Vec3::new(-1.0, 1.0, -1.0)),
            MirrorAxis::YZ => Mat4::from_scale(Vec3::new(1.0, -1.0, -1.0)),
            MirrorAxis::None => Mat4::IDENTITY,
        }
    }
    
    /// Reverse all normal vectors
    fn reverse_mesh_normals(&self, mesh: &mut USDMeshGeometry) {
        for normal in &mut mesh.normals {
            *normal = -*normal;
        }
        println!("🔄 Reverse: Reversed {} normal vectors", mesh.normals.len());
    }
    
    /// Reverse face winding order (swap triangle indices)
    fn reverse_face_winding_order(&self, mesh: &mut USDMeshGeometry) {
        let mut faces_reversed = 0;
        
        // Process triangles (groups of 3 indices)
        for triangle in mesh.indices.chunks_mut(3) {
            if triangle.len() == 3 {
                triangle.swap(1, 2); // ABC -> ACB (reverse winding)
                faces_reversed += 1;
            }
        }
        
        println!("🔄 Reverse: Reversed winding order for {} faces", faces_reversed);
    }
    
    /// Reverse point order within each face (different from winding order)
    fn reverse_point_order_in_faces(&self, mesh: &mut USDMeshGeometry) {
        let mut faces_processed = 0;
        
        // Process triangles (groups of 3 indices)  
        for triangle in mesh.indices.chunks_mut(3) {
            if triangle.len() == 3 {
                triangle.reverse(); // ABC -> CBA (reverse point order)
                faces_processed += 1;
            }
        }
        
        println!("🔄 Reverse: Reversed point order for {} faces", faces_processed);
    }
    
    /// Reverse UV coordinates
    fn reverse_uv_coordinates(&self, mesh: &mut USDMeshGeometry) {
        for uv in &mut mesh.uvs {
            if self.reverse_uvs_u {
                uv.x = 1.0 - uv.x; // Flip U coordinate
            }
            if self.reverse_uvs_v {
                uv.y = 1.0 - uv.y; // Flip V coordinate
            }
        }
        
        let operations = match (self.reverse_uvs_u, self.reverse_uvs_v) {
            (true, true) => "U and V",
            (true, false) => "U",
            (false, true) => "V",
            (false, false) => "none",
        };
        
        println!("🔄 Reverse: Reversed {} UV coordinates for {} vertices", operations, mesh.uvs.len());
    }
    
    /// Flip vertex colors (invert RGB values)
    fn flip_vertex_colors(&self, mesh: &mut USDMeshGeometry) {
        if let Some(vertex_colors) = &mut mesh.vertex_colors {
            let vertex_count = vertex_colors.len();
            for color in vertex_colors.iter_mut() {
                color.x = 1.0 - color.x; // Invert R
                color.y = 1.0 - color.y; // Invert G
                color.z = 1.0 - color.z; // Invert B
            }
            println!("🔄 Reverse: Flipped vertex colors for {} vertices", vertex_count);
        } else {
            println!("🔄 Reverse: No vertex colors to flip");
        }
    }
    
    /// Invert mesh transform matrix
    fn invert_mesh_transform(&self, mesh: &mut USDMeshGeometry) {
        mesh.transform = mesh.transform.inverse();
        println!("🔄 Reverse: Inverted mesh transform matrix");
    }
}