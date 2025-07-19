//! Capsule node logic implementation - simple cylinder + hemispheres approach

use crate::nodes::interface::NodeData;
use crate::nodes::Node;
use crate::workspaces::three_d::usd::usd_engine::{USDSceneData, USDMeshGeometry};
use glam::{Mat4, Vec3, Vec2};
use std::f32::consts::PI;

pub struct CapsuleLogic {
    mode: String,
    radius: f32,
    height: f32,
    subdivisions_axis: i32,
    subdivisions_height: i32,
    subdivisions_caps: i32,
    smooth_normals: bool,
    generate_uvs: bool,
    node_id: crate::nodes::NodeId,
}

impl CapsuleLogic {
    pub fn from_node(node: &Node) -> Self {
        Self {
            mode: node.parameters.get("mode")
                .and_then(|d| if let NodeData::String(s) = d { Some(s.clone()) } else { None })
                .unwrap_or_else(|| "primitive".to_string()),
            radius: node.parameters.get("radius")
                .and_then(|d| if let NodeData::Float(f) = d { Some(*f) } else { None })
                .unwrap_or(0.5),
            height: node.parameters.get("height")
                .and_then(|d| if let NodeData::Float(f) = d { Some(*f) } else { None })
                .unwrap_or(1.0),
            subdivisions_axis: node.parameters.get("subdivisions_axis")
                .and_then(|d| if let NodeData::Integer(i) = d { Some(*i) } else { None })
                .unwrap_or(16),
            subdivisions_height: node.parameters.get("subdivisions_height")
                .and_then(|d| if let NodeData::Integer(i) = d { Some(*i) } else { None })
                .unwrap_or(1),
            subdivisions_caps: node.parameters.get("subdivisions_caps")
                .and_then(|d| if let NodeData::Integer(i) = d { Some(*i) } else { None })
                .unwrap_or(8),
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
        let scene_data = self.generate_capsule_geometry();
        vec![NodeData::USDSceneData(scene_data)]
    }
    
    fn generate_capsule_geometry(&self) -> USDSceneData {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        
        // Use appropriate resolution based on mode
        let radial_segments = if self.mode == "primitive" { 12 } else { self.subdivisions_axis.max(3) as usize };
        let height_segments = if self.mode == "primitive" { 1 } else { self.subdivisions_height.max(1) as usize };
        let hemisphere_rings = if self.mode == "primitive" { 4 } else { self.subdivisions_caps.max(2) as usize };
        
        // A capsule is: bottom hemisphere + cylinder + top hemisphere
        // The height parameter represents the cylinder body height (not including caps)
        // Total capsule height = cylinder_height + 2 * radius
        let cylinder_height = self.height;
        let total_height = self.height + 2.0 * self.radius;
        let half_total_height = total_height / 2.0;
        
        println!("🏗️ Generating capsule: total_height={}, cylinder_height={}, radius={}", 
                 total_height, cylinder_height, self.radius);
        
        // === BOTTOM HEMISPHERE ===
        // Bottom pole
        vertices.push(Vec3::new(0.0, -half_total_height, 0.0));
        normals.push(Vec3::new(0.0, -1.0, 0.0));
        if self.generate_uvs {
            uvs.push(Vec2::new(0.5, 0.0));
        }
        let bottom_pole_idx = 0;
        
        // Bottom hemisphere rings (from bottom pole up to equator)
        let bottom_hemisphere_start = vertices.len();
        for ring in 1..=hemisphere_rings {
            // Angle from bottom pole (0) to equator (PI/2)
            let phi = PI * 0.5 * ring as f32 / hemisphere_rings as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();
            
            // Y position: start at bottom pole and work up to cylinder bottom
            let y = -half_total_height + self.radius * (1.0 - cos_phi);
            
            for segment in 0..radial_segments {
                let theta = 2.0 * PI * segment as f32 / radial_segments as f32;
                let sin_theta = theta.sin();
                let cos_theta = theta.cos();
                
                let x = self.radius * sin_phi * cos_theta;
                let z = self.radius * sin_phi * sin_theta;
                
                vertices.push(Vec3::new(x, y, z));
                
                // Normal for hemisphere points outward from sphere center at (0, -half_total_height + radius, 0)
                let sphere_center_y = -half_total_height + self.radius;
                let normal = Vec3::new(x, y - sphere_center_y, z).normalize();
                normals.push(normal);
                
                if self.generate_uvs {
                    let u = segment as f32 / radial_segments as f32;
                    let v = ring as f32 / (hemisphere_rings * 2 + height_segments) as f32;
                    uvs.push(Vec2::new(u, v));
                }
            }
        }
        
        // === CYLINDER BODY ===
        let cylinder_start_idx = vertices.len();
        if cylinder_height > 0.0 {
            for height_ring in 0..=height_segments {
                // Y ranges from cylinder bottom to cylinder top
                let y = -cylinder_height / 2.0 + (cylinder_height * height_ring as f32 / height_segments as f32);
                
                for segment in 0..radial_segments {
                    let theta = 2.0 * PI * segment as f32 / radial_segments as f32;
                    let sin_theta = theta.sin();
                    let cos_theta = theta.cos();
                    
                    let x = self.radius * cos_theta;
                    let z = self.radius * sin_theta;
                    
                    vertices.push(Vec3::new(x, y, z));
                    
                    // Cylinder normal points radially outward
                    normals.push(Vec3::new(cos_theta, 0.0, sin_theta));
                    
                    if self.generate_uvs {
                        let u = segment as f32 / radial_segments as f32;
                        let v = (hemisphere_rings + height_ring * height_segments) as f32 / (hemisphere_rings * 2 + height_segments) as f32;
                        uvs.push(Vec2::new(u, v));
                    }
                }
            }
        }
        
        // === TOP HEMISPHERE ===
        let top_hemisphere_start = vertices.len();
        for ring in 1..=hemisphere_rings {
            // Angle from equator (PI/2) to top pole (PI)
            let phi = PI * 0.5 + PI * 0.5 * ring as f32 / hemisphere_rings as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();
            
            // Y position: start at cylinder top and work up to top pole  
            let y = half_total_height - self.radius * (1.0 + cos_phi);
            
            for segment in 0..radial_segments {
                let theta = 2.0 * PI * segment as f32 / radial_segments as f32;
                let sin_theta = theta.sin();
                let cos_theta = theta.cos();
                
                let x = self.radius * sin_phi * cos_theta;
                let z = self.radius * sin_phi * sin_theta;
                
                vertices.push(Vec3::new(x, y, z));
                
                // Normal for hemisphere points outward from sphere center at (0, half_total_height - radius, 0)
                let sphere_center_y = half_total_height - self.radius;
                let normal = Vec3::new(x, y - sphere_center_y, z).normalize();
                normals.push(normal);
                
                if self.generate_uvs {
                    let u = segment as f32 / radial_segments as f32;
                    let v = (hemisphere_rings + height_segments + ring) as f32 / (hemisphere_rings * 2 + height_segments) as f32;
                    uvs.push(Vec2::new(u, v));
                }
            }
        }
        
        // Top pole
        vertices.push(Vec3::new(0.0, half_total_height, 0.0));
        normals.push(Vec3::new(0.0, 1.0, 0.0));
        if self.generate_uvs {
            uvs.push(Vec2::new(0.5, 1.0));
        }
        let top_pole_idx = vertices.len() - 1;
        
        println!("🏗️ Generated {} vertices for capsule", vertices.len());
        
        // === GENERATE TRIANGLES ===
        
        // Bottom hemisphere triangles
        // Connect bottom pole to first ring (counter-clockwise when viewed from outside)
        let first_ring_start = bottom_hemisphere_start;
        for segment in 0..radial_segments {
            let next_segment = (segment + 1) % radial_segments;
            
            // For bottom hemisphere, triangles should face outward
            // Correct winding order: pole, current, next
            indices.extend([
                bottom_pole_idx as u32,
                (first_ring_start + segment) as u32,
                (first_ring_start + next_segment) as u32,
            ]);
        }
        
        // Connect hemisphere rings
        for ring in 0..(hemisphere_rings - 1) {
            let current_ring_start = bottom_hemisphere_start + ring * radial_segments;
            let next_ring_start = bottom_hemisphere_start + (ring + 1) * radial_segments;
            
            for segment in 0..radial_segments {
                let next_segment = (segment + 1) % radial_segments;
                
                let v0 = current_ring_start + segment;
                let v1 = current_ring_start + next_segment;
                let v2 = next_ring_start + next_segment;
                let v3 = next_ring_start + segment;
                
                // Two triangles per quad (reversed winding)
                indices.extend([v0 as u32, v2 as u32, v1 as u32]);
                indices.extend([v0 as u32, v3 as u32, v2 as u32]);
            }
        }
        
        // Connect bottom hemisphere to cylinder or top hemisphere
        if cylinder_height > 0.0 {
            // Connect bottom hemisphere to cylinder bottom
            let bottom_hemisphere_last_ring = bottom_hemisphere_start + (hemisphere_rings - 1) * radial_segments;
            let cylinder_bottom_ring = cylinder_start_idx;
            
            for segment in 0..radial_segments {
                let next_segment = (segment + 1) % radial_segments;
                
                let v0 = bottom_hemisphere_last_ring + segment;
                let v1 = bottom_hemisphere_last_ring + next_segment;
                let v2 = cylinder_bottom_ring + next_segment;
                let v3 = cylinder_bottom_ring + segment;
                
                // Two triangles per quad (reversed winding)
                indices.extend([v0 as u32, v2 as u32, v1 as u32]);
                indices.extend([v0 as u32, v3 as u32, v2 as u32]);
            }
        } else {
            // No cylinder: connect bottom hemisphere directly to top hemisphere
            let bottom_hemisphere_last_ring = bottom_hemisphere_start + (hemisphere_rings - 1) * radial_segments;
            let top_hemisphere_first_ring = top_hemisphere_start;
            
            for segment in 0..radial_segments {
                let next_segment = (segment + 1) % radial_segments;
                
                let v0 = bottom_hemisphere_last_ring + segment;
                let v1 = bottom_hemisphere_last_ring + next_segment;
                let v2 = top_hemisphere_first_ring + next_segment;
                let v3 = top_hemisphere_first_ring + segment;
                
                // Two triangles per quad (reversed winding)
                indices.extend([v0 as u32, v2 as u32, v1 as u32]);
                indices.extend([v0 as u32, v3 as u32, v2 as u32]);
            }
        }
        
        // Cylinder triangles (if cylinder exists)
        if cylinder_height > 0.0 && height_segments > 0 {
            for height_ring in 0..height_segments {
                let current_ring_start = cylinder_start_idx + height_ring * radial_segments;
                let next_ring_start = cylinder_start_idx + (height_ring + 1) * radial_segments;
                
                for segment in 0..radial_segments {
                    let next_segment = (segment + 1) % radial_segments;
                    
                    let v0 = current_ring_start + segment;
                    let v1 = current_ring_start + next_segment;
                    let v2 = next_ring_start + next_segment;
                    let v3 = next_ring_start + segment;
                    
                    // Two triangles per quad (reversed winding)
                    indices.extend([v0 as u32, v2 as u32, v1 as u32]);
                    indices.extend([v0 as u32, v3 as u32, v2 as u32]);
                }
            }
        }
        
        // Connect cylinder to top hemisphere (if cylinder exists)
        if cylinder_height > 0.0 {
            let cylinder_top_ring = cylinder_start_idx + height_segments * radial_segments;
            let top_hemisphere_first_ring = top_hemisphere_start;
            
            for segment in 0..radial_segments {
                let next_segment = (segment + 1) % radial_segments;
                
                let v0 = cylinder_top_ring + segment;
                let v1 = cylinder_top_ring + next_segment;
                let v2 = top_hemisphere_first_ring + next_segment;
                let v3 = top_hemisphere_first_ring + segment;
                
                // Two triangles per quad (reversed winding)
                indices.extend([v0 as u32, v2 as u32, v1 as u32]);
                indices.extend([v0 as u32, v3 as u32, v2 as u32]);
            }
        }
        
        // Top hemisphere triangles
        for ring in 0..(hemisphere_rings - 1) {
            let current_ring_start = top_hemisphere_start + ring * radial_segments;
            let next_ring_start = top_hemisphere_start + (ring + 1) * radial_segments;
            
            for segment in 0..radial_segments {
                let next_segment = (segment + 1) % radial_segments;
                
                let v0 = current_ring_start + segment;
                let v1 = current_ring_start + next_segment;
                let v2 = next_ring_start + next_segment;
                let v3 = next_ring_start + segment;
                
                // Two triangles per quad (reversed winding)
                indices.extend([v0 as u32, v2 as u32, v1 as u32]);
                indices.extend([v0 as u32, v3 as u32, v2 as u32]);
            }
        }
        
        // Connect top hemisphere to top pole
        let last_ring_start = top_hemisphere_start + (hemisphere_rings - 1) * radial_segments;
        for segment in 0..radial_segments {
            let next_segment = (segment + 1) % radial_segments;
            
            // Reversed winding order
            indices.extend([
                (last_ring_start + segment) as u32,
                (last_ring_start + next_segment) as u32,
                top_pole_idx as u32,
            ]);
        }
        
        println!("🏗️ Generated {} triangles for capsule", indices.len() / 3);
        
        let stage_path = if self.mode == "primitive" {
            format!("procedural://capsule_node_{}", self.node_id)
        } else {
            format!("procedural://capsule_mesh_node_{}", self.node_id)
        };
        
        USDSceneData {
            stage_path,
            meshes: vec![
                USDMeshGeometry {
                    prim_path: "/Capsule".to_string(),
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