//! Plane node functional operations - geometry generation logic

use crate::nodes::interface::{GeometryData, NodeData};

/// Core plane data and functionality
#[derive(Debug, Clone)]
pub struct PlaneGeometry {
    /// Plane dimensions
    pub width: f32,
    pub height: f32,
    /// Number of subdivisions for each axis
    pub width_segments: i32,
    pub height_segments: i32,
    /// Whether to generate UVs
    pub generate_uvs: bool,
    /// Whether to generate normals
    pub generate_normals: bool,
}

impl Default for PlaneGeometry {
    fn default() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
            width_segments: 1,
            height_segments: 1,
            generate_uvs: true,
            generate_normals: true,
        }
    }
}

impl PlaneGeometry {
    /// Generate plane geometry based on current parameters
    pub fn generate_geometry(&self, node_id: usize) -> GeometryData {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        
        let half_width = self.width * 0.5;
        let half_height = self.height * 0.5;
        
        // Generate vertices
        for i in 0..=self.height_segments {
            let y = -half_height + (i as f32 / self.height_segments as f32) * self.height;
            for j in 0..=self.width_segments {
                let x = -half_width + (j as f32 / self.width_segments as f32) * self.width;
                
                // Vertex position (plane lies in XY plane)
                vertices.push([x, y, 0.0]);
                
                // Normal (always pointing up in Z direction for a flat plane)
                if self.generate_normals {
                    normals.push([0.0, 0.0, 1.0]);
                }
                
                // UV coordinates
                if self.generate_uvs {
                    let u = j as f32 / self.width_segments as f32;
                    let v = i as f32 / self.height_segments as f32;
                    uvs.push([u, v]);
                }
            }
        }
        
        // Generate indices
        for i in 0..self.height_segments {
            for j in 0..self.width_segments {
                let first = i * (self.width_segments + 1) + j;
                let second = first + self.width_segments + 1;
                
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
            id: format!("plane_{}", node_id),
            vertices,
            indices,
            normals,
            uvs,
            material_id: None,
        }
    }
    
    /// Process input data and generate plane geometry
    pub fn process(&self, inputs: Vec<NodeData>, node_id: usize) -> Vec<NodeData> {
        // Generate the geometry
        let geometry = self.generate_geometry(node_id);
        
        // If there's a transform input, we would apply it here
        // For now, just return the geometry
        let _ = inputs; // Suppress unused warning
        vec![NodeData::Geometry(geometry)]
    }
}