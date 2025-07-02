//! Sphere node functional operations - geometry generation logic

use crate::nodes::interface::{GeometryData, NodeData};
use std::f32::consts::PI;

/// Sphere type options for different sphere configurations
#[derive(Debug, Clone)]
pub enum SphereType {
    Full,
    Hemisphere,
    Quarter,
    Custom,
}

impl Default for SphereType {
    fn default() -> Self {
        SphereType::Full
    }
}

/// Core sphere data and functionality
#[derive(Debug, Clone)]
pub struct SphereGeometry {
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

impl Default for SphereGeometry {
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

impl SphereGeometry {
    /// Generate sphere geometry based on current parameters
    pub fn generate_geometry(&self, node_id: usize) -> GeometryData {
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
            id: format!("sphere_{}", node_id),
            vertices,
            indices,
            normals,
            uvs,
            material_id: None,
        }
    }
    
    /// Update sphere parameters based on type
    pub fn update_for_type(&mut self) {
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
    
    /// Process input data and generate sphere geometry
    pub fn process(&self, inputs: Vec<NodeData>, node_id: usize) -> Vec<NodeData> {
        // Generate the geometry
        let geometry = self.generate_geometry(node_id);
        
        // If there's a transform input, we would apply it here
        // For now, just return the geometry
        let _ = inputs; // Suppress unused warning
        vec![NodeData::Geometry(geometry)]
    }
}