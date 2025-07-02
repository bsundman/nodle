//! Cube node functional operations - geometry generation logic

use crate::nodes::interface::{GeometryData, NodeData};

/// Pivot point options for cube generation
#[derive(Debug, Clone)]
pub enum PivotType {
    Center,
    Corner,
    Bottom,
}

impl Default for PivotType {
    fn default() -> Self {
        PivotType::Center
    }
}

/// Core cube data and functionality
#[derive(Debug, Clone)]
pub struct CubeGeometry {
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

impl Default for CubeGeometry {
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

impl CubeGeometry {
    /// Generate cube geometry based on current parameters
    pub fn generate_geometry(&self, node_id: usize) -> GeometryData {
        let size = self.size;
        let subdivisions = self.subdivisions;
        let pivot = &self.pivot;
        let generate_uvs = self.generate_uvs;
        let generate_normals = self.generate_normals;
        let [sx, sy, sz] = size;
        let [_subdiv_x, _subdiv_y, _subdiv_z] = subdivisions;
        
        // Calculate pivot offset
        let pivot_offset = match pivot {
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
        if generate_normals {
            // Simple face normals (not smooth)
            let _face_normals = [
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
        if generate_uvs {
            let cube_uvs = [
                [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], // Back face
                [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], // Front face
            ];
            uvs.extend_from_slice(&cube_uvs);
        }
        
        GeometryData {
            id: format!("cube_{}", node_id),
            vertices,
            indices,
            normals,
            uvs,
            material_id: None,
        }
    }
    
    /// Process input data and generate cube geometry
    pub fn process(&self, inputs: Vec<NodeData>, node_id: usize) -> Vec<NodeData> {
        // Generate the geometry
        let geometry = self.generate_geometry(node_id);
        
        // If there's a transform input, we would apply it here
        // For now, just return the geometry
        let _ = inputs; // Suppress unused warning
        vec![NodeData::Geometry(geometry)]
    }
}