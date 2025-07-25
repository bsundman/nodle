//! Rotate node functional operations - rotation logic

use crate::nodes::interface::NodeData;

/// Core rotate data and functionality
#[derive(Debug, Clone)]
pub struct RotateLogic {
    /// Rotation angles in degrees
    pub rotation: [f32; 3],
    /// Whether to use world or local space
    pub use_world_space: bool,
    /// Rotation order (XYZ, XZY, YXZ, etc.)
    pub rotation_order: RotationOrder,
    /// Whether to use degrees or radians
    pub use_degrees: bool,
}

#[derive(Debug, Clone)]
pub enum RotationOrder {
    XYZ,
    XZY,
    YXZ,
    YZX,
    ZXY,
    ZYX,
}

impl Default for RotateLogic {
    fn default() -> Self {
        Self {
            rotation: [0.0, 0.0, 0.0],
            use_world_space: true,
            rotation_order: RotationOrder::XYZ,
            use_degrees: true,
        }
    }
}

impl RotateLogic {
    /// Process input data and perform rotation
    pub fn process(&self, inputs: Vec<NodeData>) -> Vec<NodeData> {
        let mut rotation = self.rotation;
        
        // Extract rotation vector from inputs if provided
        if inputs.len() >= 2 {
            if let NodeData::Vector3(vec) = inputs[1] {
                rotation = vec;
            }
        }
        
        // Transform the geometry (simplified for now)
        // In a real implementation, this would apply rotation matrices
        if inputs.len() >= 1 {
            // Pass through the geometry with applied rotation
            vec![inputs[0].clone()]
        } else {
            vec![NodeData::Vector3(rotation)]
        }
    }
    
    /// Get the current rotation matrix
    pub fn get_rotation_matrix(&self) -> [[f32; 4]; 4] {
        let (x, y, z) = if self.use_degrees {
            (
                self.rotation[0].to_radians(),
                self.rotation[1].to_radians(),
                self.rotation[2].to_radians(),
            )
        } else {
            (self.rotation[0], self.rotation[1], self.rotation[2])
        };
        
        let (cx, sx) = (x.cos(), x.sin());
        let (cy, sy) = (y.cos(), y.sin());
        let (cz, sz) = (z.cos(), z.sin());
        
        // XYZ rotation order
        match self.rotation_order {
            RotationOrder::XYZ => [
                [cy * cz, -cy * sz, sy, 0.0],
                [cx * sz + sx * sy * cz, cx * cz - sx * sy * sz, -sx * cy, 0.0],
                [sx * sz - cx * sy * cz, sx * cz + cx * sy * sz, cx * cy, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            _ => {
                // Simplified - just return identity for other orders
                [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ]
            }
        }
    }
    
    /// Convert degrees to radians
    pub fn to_radians(&self) -> [f32; 3] {
        if self.use_degrees {
            [
                self.rotation[0].to_radians(),
                self.rotation[1].to_radians(),
                self.rotation[2].to_radians(),
            ]
        } else {
            self.rotation
        }
    }
}