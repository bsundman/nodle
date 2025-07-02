//! Extended USD engine operations for comprehensive node support

use super::usd_engine::{USDEngine, USDPrim, USDStage};

impl USDEngine {
    // Stage operations
    pub fn create_stage_to_file(&mut self, identifier: &str, file_path: &str) -> Result<USDStage, String> {
        // For now, create in memory and mark for file save
        let stage = self.create_stage(identifier)?;
        println!("Stage '{}' created for file: {}", identifier, file_path);
        Ok(stage)
    }
    
    pub fn set_default_prim(&mut self, stage_id: &str, prim_path: &str) -> Result<(), String> {
        if self.stages.contains_key(stage_id) {
            println!("Set default prim for stage '{}' to '{}'", stage_id, prim_path);
            Ok(())
        } else {
            Err(format!("Stage '{}' not found", stage_id))
        }
    }
    
    pub fn export_stage(&self, stage_id: &str, file_path: &str, format: &str) -> Result<(), String> {
        if self.stages.contains_key(stage_id) {
            println!("Exporting stage '{}' to '{}' as {}", stage_id, file_path, format);
            Ok(())
        } else {
            Err(format!("Stage '{}' not found", stage_id))
        }
    }
    
    pub fn clear_stage(&mut self, stage_id: &str) -> Result<(), String> {
        if self.stages.contains_key(stage_id) {
            // Remove all prims for this stage
            self.prims.retain(|k, _| !k.starts_with(&format!("{}:", stage_id)));
            println!("Cleared stage '{}'", stage_id);
            Ok(())
        } else {
            Err(format!("Stage '{}' not found", stage_id))
        }
    }
    
    // Geometry primitives
    pub fn create_cube(&mut self, stage_id: &str, prim_path: &str, size: f64) -> Result<USDPrim, String> {
        self.create_prim(stage_id, prim_path, "Cube", Some(format!("size: {}", size)))
    }
    
    pub fn create_cylinder(&mut self, stage_id: &str, prim_path: &str, radius: f64, height: f64) -> Result<USDPrim, String> {
        self.create_prim(stage_id, prim_path, "Cylinder", Some(format!("radius: {}, height: {}", radius, height)))
    }
    
    pub fn create_cone(&mut self, stage_id: &str, prim_path: &str, radius: f64, height: f64) -> Result<USDPrim, String> {
        self.create_prim(stage_id, prim_path, "Cone", Some(format!("radius: {}, height: {}", radius, height)))
    }
    
    pub fn create_plane(&mut self, stage_id: &str, prim_path: &str, width: f64, height: f64) -> Result<USDPrim, String> {
        self.create_prim(stage_id, prim_path, "Plane", Some(format!("width: {}, height: {}", width, height)))
    }
    
    pub fn create_capsule(&mut self, stage_id: &str, prim_path: &str, radius: f64, height: f64) -> Result<USDPrim, String> {
        self.create_prim(stage_id, prim_path, "Capsule", Some(format!("radius: {}, height: {}", radius, height)))
    }
    
    pub fn create_torus(&mut self, stage_id: &str, prim_path: &str, radius: f64, tube_radius: f64) -> Result<USDPrim, String> {
        self.create_prim(stage_id, prim_path, "Torus", Some(format!("radius: {}, tube_radius: {}", radius, tube_radius)))
    }
    
    pub fn create_mesh(&mut self, stage_id: &str, prim_path: &str) -> Result<USDPrim, String> {
        self.create_prim(stage_id, prim_path, "Mesh", None)
    }
    
    pub fn create_points(&mut self, stage_id: &str, prim_path: &str) -> Result<USDPrim, String> {
        self.create_prim(stage_id, prim_path, "Points", None)
    }
    
    pub fn create_curves(&mut self, stage_id: &str, prim_path: &str, curve_type: &str) -> Result<USDPrim, String> {
        self.create_prim(stage_id, prim_path, "BasisCurves", Some(format!("type: {}", curve_type)))
    }
    
    // Transform operations
    pub fn set_transform(&mut self, stage_id: &str, prim_path: &str, matrix: [[f64; 4]; 4]) -> Result<(), String> {
        if self.prim_exists(stage_id, prim_path) {
            println!("Set transform for '{}' on stage '{}'", prim_path, stage_id);
            Ok(())
        } else {
            Err(format!("Prim '{}' not found on stage '{}'", prim_path, stage_id))
        }
    }
    
    pub fn set_translation(&mut self, stage_id: &str, prim_path: &str, translation: [f64; 3]) -> Result<(), String> {
        if self.prim_exists(stage_id, prim_path) {
            println!("Set translation for '{}' to {:?}", prim_path, translation);
            Ok(())
        } else {
            Err(format!("Prim '{}' not found on stage '{}'", prim_path, stage_id))
        }
    }
    
    pub fn set_rotation(&mut self, stage_id: &str, prim_path: &str, rotation: [f64; 3]) -> Result<(), String> {
        if self.prim_exists(stage_id, prim_path) {
            println!("Set rotation for '{}' to {:?}", prim_path, rotation);
            Ok(())
        } else {
            Err(format!("Prim '{}' not found on stage '{}'", prim_path, stage_id))
        }
    }
    
    pub fn set_scale(&mut self, stage_id: &str, prim_path: &str, scale: [f64; 3]) -> Result<(), String> {
        if self.prim_exists(stage_id, prim_path) {
            println!("Set scale for '{}' to {:?}", prim_path, scale);
            Ok(())
        } else {
            Err(format!("Prim '{}' not found on stage '{}'", prim_path, stage_id))
        }
    }
    
    // Lighting operations
    pub fn create_distant_light(&mut self, stage_id: &str, prim_path: &str, intensity: f64) -> Result<USDPrim, String> {
        self.create_prim(stage_id, prim_path, "DistantLight", Some(format!("intensity: {}", intensity)))
    }
    
    pub fn create_rect_light(&mut self, stage_id: &str, prim_path: &str, width: f64, height: f64) -> Result<USDPrim, String> {
        self.create_prim(stage_id, prim_path, "RectLight", Some(format!("width: {}, height: {}", width, height)))
    }
    
    pub fn create_sphere_light(&mut self, stage_id: &str, prim_path: &str, radius: f64) -> Result<USDPrim, String> {
        self.create_prim(stage_id, prim_path, "SphereLight", Some(format!("radius: {}", radius)))
    }
    
    pub fn create_cylinder_light(&mut self, stage_id: &str, prim_path: &str, radius: f64, length: f64) -> Result<USDPrim, String> {
        self.create_prim(stage_id, prim_path, "CylinderLight", Some(format!("radius: {}, length: {}", radius, length)))
    }
    
    pub fn create_dome_light(&mut self, stage_id: &str, prim_path: &str, texture_file: Option<&str>) -> Result<USDPrim, String> {
        let info = texture_file.map(|t| format!("texture: {}", t)).unwrap_or_default();
        self.create_prim(stage_id, prim_path, "DomeLight", Some(info))
    }
    
    pub fn create_disk_light(&mut self, stage_id: &str, prim_path: &str, radius: f64) -> Result<USDPrim, String> {
        self.create_prim(stage_id, prim_path, "DiskLight", Some(format!("radius: {}", radius)))
    }
    
    // Material and shading
    pub fn create_material(&mut self, stage_id: &str, prim_path: &str) -> Result<USDPrim, String> {
        self.create_prim(stage_id, prim_path, "Material", None)
    }
    
    pub fn create_shader(&mut self, stage_id: &str, prim_path: &str, shader_type: &str) -> Result<USDPrim, String> {
        self.create_prim(stage_id, prim_path, "Shader", Some(format!("type: {}", shader_type)))
    }
    
    pub fn create_preview_surface(&mut self, stage_id: &str, prim_path: &str) -> Result<USDPrim, String> {
        self.create_prim(stage_id, prim_path, "PreviewSurface", None)
    }
    
    pub fn bind_material(&mut self, stage_id: &str, prim_path: &str, material_path: &str) -> Result<(), String> {
        if self.prim_exists(stage_id, prim_path) && self.prim_exists(stage_id, material_path) {
            println!("Bound material '{}' to '{}'", material_path, prim_path);
            Ok(())
        } else {
            Err("Prim or material not found".to_string())
        }
    }
    
    // Camera operations
    pub fn create_camera(&mut self, stage_id: &str, prim_path: &str) -> Result<USDPrim, String> {
        self.create_prim(stage_id, prim_path, "Camera", None)
    }
    
    pub fn set_camera_properties(&mut self, stage_id: &str, prim_path: &str, fov: f64, near: f64, far: f64) -> Result<(), String> {
        if self.prim_exists(stage_id, prim_path) {
            println!("Set camera properties: fov={}, near={}, far={}", fov, near, far);
            Ok(())
        } else {
            Err(format!("Camera '{}' not found", prim_path))
        }
    }
    
    // Attribute operations
    pub fn set_prim_purpose(&mut self, stage_id: &str, prim_path: &str, purpose: &str) -> Result<(), String> {
        if self.prim_exists(stage_id, prim_path) {
            println!("Set purpose for '{}' to '{}'", prim_path, purpose);
            Ok(())
        } else {
            Err(format!("Prim '{}' not found", prim_path))
        }
    }
    
    pub fn set_prim_visibility(&mut self, stage_id: &str, prim_path: &str, visibility: &str) -> Result<(), String> {
        if self.prim_exists(stage_id, prim_path) {
            println!("Set visibility for '{}' to '{}'", prim_path, visibility);
            Ok(())
        } else {
            Err(format!("Prim '{}' not found", prim_path))
        }
    }
    
    pub fn set_attribute(&mut self, stage_id: &str, prim_path: &str, attr_name: &str, value: &str) -> Result<(), String> {
        if self.prim_exists(stage_id, prim_path) {
            println!("Set attribute '{}' on '{}' to '{}'", attr_name, prim_path, value);
            Ok(())
        } else {
            Err(format!("Prim '{}' not found", prim_path))
        }
    }
    
    pub fn get_attribute(&self, stage_id: &str, prim_path: &str, attr_name: &str) -> Result<String, String> {
        if self.prim_exists(stage_id, prim_path) {
            Ok(format!("mock_value_for_{}", attr_name))
        } else {
            Err(format!("Prim '{}' not found", prim_path))
        }
    }
    
    // Helper functions
    fn create_prim(&mut self, stage_id: &str, prim_path: &str, prim_type: &str, info: Option<String>) -> Result<USDPrim, String> {
        if !self.stages.contains_key(stage_id) {
            return Err(format!("Stage '{}' not found", stage_id));
        }
        
        let prim = USDPrim {
            path: prim_path.to_string(),
            prim_type: prim_type.to_string(),
            stage_id: stage_id.to_string(),
        };
        
        let prim_key = format!("{}:{}", stage_id, prim_path);
        self.prims.insert(prim_key, prim.clone());
        
        let info_str = info.map(|i| format!(" ({})", i)).unwrap_or_default();
        println!("Created USD {} at '{}'{}", prim_type, prim_path, info_str);
        
        Ok(prim)
    }
    
    fn prim_exists(&self, stage_id: &str, prim_path: &str) -> bool {
        let prim_key = format!("{}:{}", stage_id, prim_path);
        self.prims.contains_key(&prim_key)
    }
}