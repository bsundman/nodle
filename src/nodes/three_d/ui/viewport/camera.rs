//! Camera system for USD viewport with Maya-style navigation

use glam::{Mat4, Vec3};
use bytemuck::{Pod, Zeroable};

/// 3D Vertex structure for rendering
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

/// 3D Camera with Maya-style navigation
#[derive(Debug, Clone)]
pub struct Camera3D {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect: f32,
    
    // Maya-style navigation state
    pub orbit_sensitivity: f32,
    pub pan_sensitivity: f32,
    pub zoom_sensitivity: f32,
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            position: Vec3::new(5.0, 5.0, 5.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov: 45.0_f32.to_radians(),
            near: 0.1,
            far: 100.0,
            aspect: 1.0,
            orbit_sensitivity: 0.5,   // Responsive orbiting
            pan_sensitivity: 1.0,     // Responsive panning
            zoom_sensitivity: 1.0,    // Responsive zooming
        }
    }
}

impl Camera3D {
    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.position, self.target, self.up);
        let proj = Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far);
        proj * view
    }
    
    /// Maya-style orbit around target
    pub fn orbit(&mut self, delta_x: f32, delta_y: f32) {
        let offset = self.position - self.target;
        let radius = offset.length();
        
        // Convert to spherical coordinates
        let mut theta = offset.z.atan2(offset.x); // Azimuth
        let mut phi = (offset.y / radius).acos(); // Elevation
        
        // Apply rotation
        theta += delta_x * self.orbit_sensitivity;
        phi += delta_y * self.orbit_sensitivity;
        
        // Clamp phi to avoid gimbal lock
        phi = phi.clamp(0.01, std::f32::consts::PI - 0.01);
        
        // Convert back to cartesian
        let new_offset = Vec3::new(
            radius * phi.sin() * theta.cos(),
            radius * phi.cos(),
            radius * phi.sin() * theta.sin(),
        );
        
        self.position = self.target + new_offset;
    }
    
    /// Maya-style pan (move target and position together)
    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        let forward = (self.target - self.position).normalize();
        let right = forward.cross(self.up).normalize();
        let up = right.cross(forward).normalize();
        
        let pan_vector = right * delta_x * self.pan_sensitivity 
                        + up * delta_y * self.pan_sensitivity;
        
        self.position += pan_vector;
        self.target += pan_vector;
    }
    
    /// Maya-style zoom (move camera closer/farther from target)
    pub fn zoom(&mut self, delta: f32) {
        let direction = (self.target - self.position).normalize();
        let distance = (self.target - self.position).length();
        let new_distance = (distance + delta * self.zoom_sensitivity).max(0.1);
        
        self.position = self.target - direction * new_distance;
    }
    
    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
    
    /// Convert screen delta to world space movement for 1:1 pan
    pub fn screen_to_world_pan(&self, screen_delta_x: f32, screen_delta_y: f32, viewport_height: f32) -> Vec3 {
        // Calculate the vertical field of view extent at the target distance
        let distance = (self.target - self.position).length();
        let fov_height = 2.0 * distance * (self.fov / 2.0).tan();
        
        // Scale factor to convert screen pixels to world units
        let world_per_pixel = fov_height / viewport_height;
        
        // Calculate camera coordinate system
        let forward = (self.target - self.position).normalize();
        let right = forward.cross(self.up).normalize();
        let up = right.cross(forward).normalize();
        
        // Convert screen deltas to world space movement (fixed Y-axis)
        right * (screen_delta_x * world_per_pixel) + up * (screen_delta_y * world_per_pixel)
    }
    
    /// Get a ray from camera through screen position (normalized 0-1)
    pub fn screen_to_ray(&self, screen_x: f32, screen_y: f32) -> (Vec3, Vec3) {
        // Convert from screen space (0,1) to NDC (-1,1)
        let ndc_x = screen_x * 2.0 - 1.0;
        let ndc_y = 1.0 - screen_y * 2.0; // Flip Y
        
        // Create inverse view-projection matrix
        let view_proj = self.build_view_projection_matrix();
        let inv_view_proj = view_proj.inverse();
        
        // Transform points from NDC to world space
        let near_point = inv_view_proj.project_point3(Vec3::new(ndc_x, ndc_y, -1.0));
        let far_point = inv_view_proj.project_point3(Vec3::new(ndc_x, ndc_y, 1.0));
        
        let ray_origin = near_point;
        let ray_direction = (far_point - near_point).normalize();
        
        (ray_origin, ray_direction)
    }
    
    /// Orbit around a specific point in 3D space
    pub fn orbit_around_point(&mut self, pivot: Vec3, delta_x: f32, delta_y: f32) {
        // Calculate offset from pivot to current camera position
        let position_offset = self.position - pivot;
        let radius = position_offset.length();
        
        if radius < 0.001 {
            return; // Too close to pivot
        }
        
        // Convert to spherical coordinates
        let mut theta = position_offset.z.atan2(position_offset.x);
        let mut phi = (position_offset.y / radius).acos();
        
        // Apply rotation
        theta += delta_x * self.orbit_sensitivity;
        phi += delta_y * self.orbit_sensitivity;
        
        // Clamp phi to avoid gimbal lock
        phi = phi.clamp(0.01, std::f32::consts::PI - 0.01);
        
        // Convert back to cartesian for new position
        let new_position_offset = Vec3::new(
            radius * phi.sin() * theta.cos(),
            radius * phi.cos(),
            radius * phi.sin() * theta.sin(),
        );
        
        // Update camera position and make it look at the pivot point
        self.position = pivot + new_position_offset;
        self.target = pivot;
    }
    
    /// Zoom towards a specific point
    pub fn zoom_to_point(&mut self, target_point: Vec3, delta: f32) {
        let direction = (target_point - self.position).normalize();
        let distance = (target_point - self.position).length();
        
        // Scale zoom amount based on distance and increase sensitivity
        let zoom_amount = delta * self.zoom_sensitivity * distance * 2.0;
        
        // Calculate new position (moving towards target point)
        let new_position = self.position + direction * zoom_amount;
        
        // Ensure we don't zoom past the target point or too close
        let new_distance = (target_point - new_position).length();
        if new_distance > 0.1 {
            self.position = new_position;
            // Update target to maintain the camera's look direction
            let target_direction = (self.target - (self.position - direction * zoom_amount)).normalize();
            self.target = self.position + target_direction * (self.target - self.position).length();
        }
    }
    
    /// Ray-triangle intersection test using Möller-Trumbore algorithm
    pub fn ray_triangle_intersect(&self, ray_origin: Vec3, ray_direction: Vec3, v0: Vec3, v1: Vec3, v2: Vec3) -> Option<f32> {
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let h = ray_direction.cross(edge2);
        let a = edge1.dot(h);
        
        // Ray is parallel to triangle
        if a > -0.00001 && a < 0.00001 {
            return None;
        }
        
        let f = 1.0 / a;
        let s = ray_origin - v0;
        let u = f * s.dot(h);
        
        if u < 0.0 || u > 1.0 {
            return None;
        }
        
        let q = s.cross(edge1);
        let v = f * ray_direction.dot(q);
        
        if v < 0.0 || u + v > 1.0 {
            return None;
        }
        
        let t = f * edge2.dot(q);
        
        if t > 0.00001 {
            Some(t)
        } else {
            None
        }
    }
    
    /// Find the closest intersection point with scene geometry (only in front of camera)
    pub fn find_closest_intersection(&self, ray_origin: Vec3, ray_direction: Vec3, geometries: &[super::usd_rendering::USDGeometry]) -> Option<Vec3> {
        let mut closest_distance = f32::INFINITY;
        let mut closest_point = None;
        
        if geometries.is_empty() {
            return None;
        }
        
        for geometry in geometries {
            if !geometry.visibility {
                continue;
            }
            
            // Transform vertices by geometry transform
            let transform = geometry.transform;
            
            // Test intersection with each triangle
            for triangle in geometry.indices.chunks(3) {
                if triangle.len() != 3 {
                    continue;
                }
                
                let v0_local = Vec3::from(geometry.vertices[triangle[0] as usize].position);
                let v1_local = Vec3::from(geometry.vertices[triangle[1] as usize].position);
                let v2_local = Vec3::from(geometry.vertices[triangle[2] as usize].position);
                
                // Transform vertices to world space
                let v0 = transform.transform_point3(v0_local);
                let v1 = transform.transform_point3(v1_local);
                let v2 = transform.transform_point3(v2_local);
                
                if let Some(distance) = self.ray_triangle_intersect(ray_origin, ray_direction, v0, v1, v2) {
                    // Only accept intersections in front of camera (positive distance)
                    if distance > 0.1 && distance < closest_distance {
                        closest_distance = distance;
                        closest_point = Some(ray_origin + ray_direction * distance);
                    }
                }
            }
        }
        
        closest_point
    }
    
    /// Find the best orbit pivot point for mouse position using proper ray casting
    pub fn find_orbit_pivot(&self, mouse_x: f32, mouse_y: f32, geometries: &[super::usd_rendering::USDGeometry]) -> Vec3 {
        let (ray_origin, ray_direction) = self.screen_to_ray(mouse_x, mouse_y);
        
        // First try to find exact intersection with scene geometry
        if let Some(intersection_point) = self.find_closest_intersection(ray_origin, ray_direction, geometries) {
            return intersection_point;
        }
        
        // No direct intersection - use a reasonable default distance
        // Use current target distance as a sensible fallback
        let fallback_distance = (self.target - self.position).length();
        let fallback_point = ray_origin + ray_direction * fallback_distance;
        
        fallback_point
    }
}