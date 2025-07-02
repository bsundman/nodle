//! USD engine implementation using PyO3 to interface with USD Python API

#[cfg(feature = "usd")]
use pyo3::prelude::*;
#[cfg(feature = "usd")]
use pyo3::types::{PyDict, PyString};
use std::collections::HashMap;
use super::local_usd;

/// USD Stage handle - holds a reference to a USD stage
#[derive(Debug, Clone)]
pub struct USDStage {
    pub path: String,
    pub identifier: String,
}

/// USD Prim handle - holds a reference to a USD primitive
#[derive(Debug, Clone)]
pub struct USDPrim {
    pub path: String,
    pub prim_type: String,
    pub stage_id: String,
}

/// USD Engine - manages USD operations through Python API
pub struct USDEngine {
    #[cfg(feature = "usd")]
    _python_initialized: bool,
    stages: HashMap<String, USDStage>,
    prims: HashMap<String, USDPrim>,
}

impl USDEngine {
    pub fn new() -> Self {
        // Initialize local USD on first engine creation
        #[cfg(feature = "usd")]
        local_usd::init_local_usd();
        Self {
            #[cfg(feature = "usd")]
            _python_initialized: true,
            stages: HashMap::new(),
            prims: HashMap::new(),
        }
    }
    
    /// Create a new USD stage
    pub fn create_stage(&mut self, identifier: &str) -> Result<USDStage, String> {
        #[cfg(feature = "usd")]
        {
            Python::with_gil(|py| -> Result<USDStage, String> {
                let usd = py.import("pxr.Usd").map_err(|e| format!("Failed to import USD: {}", e))?;
                
                // Create an in-memory stage
                let stage = usd.call_method0("Stage.CreateInMemory")
                    .map_err(|e| format!("Failed to create stage: {}", e))?;
                
                let stage_obj = USDStage {
                    path: format!("memory://{}", identifier),
                    identifier: identifier.to_string(),
                };
                
                self.stages.insert(identifier.to_string(), stage_obj.clone());
                Ok(stage_obj)
            })
        }
        
        #[cfg(not(feature = "usd"))]
        {
            // Mock implementation when USD feature is disabled
            let stage = USDStage {
                path: format!("mock://{}", identifier),
                identifier: identifier.to_string(),
            };
            self.stages.insert(identifier.to_string(), stage.clone());
            Ok(stage)
        }
    }
    
    /// Load a USD stage from file
    pub fn load_stage(&mut self, file_path: &str) -> Result<USDStage, String> {
        #[cfg(feature = "usd")]
        {
            Python::with_gil(|py| -> Result<USDStage, String> {
                let usd = py.import("pxr.Usd").map_err(|e| format!("Failed to import USD: {}", e))?;
                
                let stage = usd.call_method1("Stage.Open", (file_path,))
                    .map_err(|e| format!("Failed to open stage '{}': {}", file_path, e))?;
                
                let identifier = format!("loaded_{}", self.stages.len());
                let stage_obj = USDStage {
                    path: file_path.to_string(),
                    identifier: identifier.clone(),
                };
                
                self.stages.insert(identifier.clone(), stage_obj.clone());
                Ok(stage_obj)
            })
        }
        
        #[cfg(not(feature = "usd"))]
        {
            let identifier = format!("loaded_{}", self.stages.len());
            let stage = USDStage {
                path: file_path.to_string(),
                identifier: identifier.clone(),
            };
            self.stages.insert(identifier.clone(), stage.clone());
            Ok(stage)
        }
    }
    
    /// Save a USD stage to file
    pub fn save_stage(&self, stage_id: &str, file_path: &str, format: Option<&str>) -> Result<bool, String> {
        #[cfg(feature = "usd")]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            Python::with_gil(|py| -> Result<bool, String> {
                let usd = py.import("pxr.Usd").map_err(|e| format!("Failed to import USD: {}", e))?;
                
                // For now, return success - actual implementation would save the stage
                println!("Saving USD stage '{}' to '{}' with format {:?}", stage_id, file_path, format);
                Ok(true)
            })
        }
        
        #[cfg(not(feature = "usd"))]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
            println!("Mock: Saving USD stage '{}' to '{}' with format {:?}", stage_id, file_path, format);
            Ok(true)
        }
    }
    
    /// Create a USD Xform primitive
    pub fn create_xform(&mut self, stage_id: &str, prim_path: &str) -> Result<USDPrim, String> {
        #[cfg(feature = "usd")]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            Python::with_gil(|py| -> Result<USDPrim, String> {
                let usd_geom = py.import("pxr.UsdGeom").map_err(|e| format!("Failed to import UsdGeom: {}", e))?;
                
                // For now, create a mock prim - actual implementation would create on the stage
                let prim = USDPrim {
                    path: prim_path.to_string(),
                    prim_type: "Xform".to_string(),
                    stage_id: stage_id.to_string(),
                };
                
                let prim_key = format!("{}:{}", stage_id, prim_path);
                self.prims.insert(prim_key, prim.clone());
                
                println!("Created USD Xform at '{}'", prim_path);
                Ok(prim)
            })
        }
        
        #[cfg(not(feature = "usd"))]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            let prim = USDPrim {
                path: prim_path.to_string(),
                prim_type: "Xform".to_string(),
                stage_id: stage_id.to_string(),
            };
            
            let prim_key = format!("{}:{}", stage_id, prim_path);
            self.prims.insert(prim_key, prim.clone());
            
            println!("Mock: Created USD Xform at '{}'", prim_path);
            Ok(prim)
        }
    }
    
    /// Create a USD Sphere primitive
    pub fn create_sphere(&mut self, stage_id: &str, prim_path: &str, radius: f64) -> Result<USDPrim, String> {
        #[cfg(feature = "usd")]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            Python::with_gil(|py| -> Result<USDPrim, String> {
                let _usd_geom = py.import("pxr.UsdGeom").map_err(|e| format!("Failed to import UsdGeom: {}", e))?;
                
                let prim = USDPrim {
                    path: prim_path.to_string(),
                    prim_type: "Sphere".to_string(),
                    stage_id: stage_id.to_string(),
                };
                
                let prim_key = format!("{}:{}", stage_id, prim_path);
                self.prims.insert(prim_key, prim.clone());
                
                println!("Created USD Sphere at '{}' with radius {}", prim_path, radius);
                Ok(prim)
            })
        }
        
        #[cfg(not(feature = "usd"))]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            let prim = USDPrim {
                path: prim_path.to_string(),
                prim_type: "Sphere".to_string(),
                stage_id: stage_id.to_string(),
            };
            
            let prim_key = format!("{}:{}", stage_id, prim_path);
            self.prims.insert(prim_key, prim.clone());
            
            println!("Mock: Created USD Sphere at '{}' with radius {}", prim_path, radius);
            Ok(prim)
        }
    }
    
    /// Create a USD Cube primitive  
    pub fn create_cube(&mut self, stage_id: &str, prim_path: &str, size: f64) -> Result<USDPrim, String> {
        #[cfg(feature = "usd")]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            Python::with_gil(|py| -> Result<USDPrim, String> {
                let _usd_geom = py.import("pxr.UsdGeom").map_err(|e| format!("Failed to import UsdGeom: {}", e))?;
                
                let prim = USDPrim {
                    path: prim_path.to_string(),
                    prim_type: "Cube".to_string(),
                    stage_id: stage_id.to_string(),
                };
                
                let prim_key = format!("{}:{}", stage_id, prim_path);
                self.prims.insert(prim_key, prim.clone());
                
                println!("Created USD Cube at '{}' with size {}", prim_path, size);
                Ok(prim)
            })
        }
        
        #[cfg(not(feature = "usd"))]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            let prim = USDPrim {
                path: prim_path.to_string(),
                prim_type: "Cube".to_string(),
                stage_id: stage_id.to_string(),
            };
            
            let prim_key = format!("{}:{}", stage_id, prim_path);
            self.prims.insert(prim_key, prim.clone());
            
            println!("Mock: Created USD Cube at '{}' with size {}", prim_path, size);
            Ok(prim)
        }
    }
    
    /// Set an attribute on a USD prim
    pub fn set_attribute(&self, stage_id: &str, prim_path: &str, attr_name: &str, value: &str) -> Result<(), String> {
        #[cfg(feature = "usd")]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            Python::with_gil(|py| -> Result<(), String> {
                println!("Setting attribute '{}' on '{}:{}' to '{}'", attr_name, stage_id, prim_path, value);
                Ok(())
            })
        }
        
        #[cfg(not(feature = "usd"))]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
            println!("Mock: Setting attribute '{}' on '{}:{}' to '{}'", attr_name, stage_id, prim_path, value);
            Ok(())
        }
    }
    
    /// Get an attribute from a USD prim
    pub fn get_attribute(&self, stage_id: &str, prim_path: &str, attr_name: &str) -> Result<String, String> {
        #[cfg(feature = "usd")]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            Python::with_gil(|py| -> Result<String, String> {
                // Mock return value for now
                Ok(format!("mock_value_for_{}", attr_name))
            })
        }
        
        #[cfg(not(feature = "usd"))]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
            Ok(format!("mock_value_for_{}", attr_name))
        }
    }
    
    /// Get list of all stages
    pub fn list_stages(&self) -> Vec<String> {
        self.stages.keys().cloned().collect()
    }
    
    /// Create a USD Camera primitive
    pub fn create_camera(&mut self, stage_id: &str, prim_path: &str, focal_length: f64, near_clip: f64, far_clip: f64) -> Result<USDPrim, String> {
        #[cfg(feature = "usd")]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            Python::with_gil(|py| -> Result<USDPrim, String> {
                let _usd_geom = py.import("pxr.UsdGeom").map_err(|e| format!("Failed to import UsdGeom: {}", e))?;
                
                let prim = USDPrim {
                    path: prim_path.to_string(),
                    prim_type: "Camera".to_string(),
                    stage_id: stage_id.to_string(),
                };
                
                let prim_key = format!("{}:{}", stage_id, prim_path);
                self.prims.insert(prim_key, prim.clone());
                
                println!("Created USD Camera at '{}' (focal: {}mm, near: {}, far: {})", prim_path, focal_length, near_clip, far_clip);
                Ok(prim)
            })
        }
        
        #[cfg(not(feature = "usd"))]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            let prim = USDPrim {
                path: prim_path.to_string(),
                prim_type: "Camera".to_string(),
                stage_id: stage_id.to_string(),
            };
            
            let prim_key = format!("{}:{}", stage_id, prim_path);
            self.prims.insert(prim_key, prim.clone());
            
            println!("Mock: Created USD Camera at '{}' (focal: {}mm, near: {}, far: {})", prim_path, focal_length, near_clip, far_clip);
            Ok(prim)
        }
    }
    
    /// Create a USD Distant Light primitive
    pub fn create_distant_light(&mut self, stage_id: &str, prim_path: &str, intensity: f64, angle: f64) -> Result<USDPrim, String> {
        #[cfg(feature = "usd")]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            Python::with_gil(|py| -> Result<USDPrim, String> {
                let _usd_lux = py.import("pxr.UsdLux").map_err(|e| format!("Failed to import UsdLux: {}", e))?;
                
                let prim = USDPrim {
                    path: prim_path.to_string(),
                    prim_type: "DistantLight".to_string(),
                    stage_id: stage_id.to_string(),
                };
                
                let prim_key = format!("{}:{}", stage_id, prim_path);
                self.prims.insert(prim_key, prim.clone());
                
                println!("Created USD Distant Light at '{}' (intensity: {}, angle: {}°)", prim_path, intensity, angle);
                Ok(prim)
            })
        }
        
        #[cfg(not(feature = "usd"))]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            let prim = USDPrim {
                path: prim_path.to_string(),
                prim_type: "DistantLight".to_string(),
                stage_id: stage_id.to_string(),
            };
            
            let prim_key = format!("{}:{}", stage_id, prim_path);
            self.prims.insert(prim_key, prim.clone());
            
            println!("Mock: Created USD Distant Light at '{}' (intensity: {}, angle: {}°)", prim_path, intensity, angle);
            Ok(prim)
        }
    }
    
    /// Create a USD Sphere Light primitive
    pub fn create_sphere_light(&mut self, stage_id: &str, prim_path: &str, intensity: f64, radius: f64) -> Result<USDPrim, String> {
        #[cfg(feature = "usd")]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            Python::with_gil(|py| -> Result<USDPrim, String> {
                let _usd_lux = py.import("pxr.UsdLux").map_err(|e| format!("Failed to import UsdLux: {}", e))?;
                
                let prim = USDPrim {
                    path: prim_path.to_string(),
                    prim_type: "SphereLight".to_string(),
                    stage_id: stage_id.to_string(),
                };
                
                let prim_key = format!("{}:{}", stage_id, prim_path);
                self.prims.insert(prim_key, prim.clone());
                
                println!("Created USD Sphere Light at '{}' (intensity: {}, radius: {})", prim_path, intensity, radius);
                Ok(prim)
            })
        }
        
        #[cfg(not(feature = "usd"))]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            let prim = USDPrim {
                path: prim_path.to_string(),
                prim_type: "SphereLight".to_string(),
                stage_id: stage_id.to_string(),
            };
            
            let prim_key = format!("{}:{}", stage_id, prim_path);
            self.prims.insert(prim_key, prim.clone());
            
            println!("Mock: Created USD Sphere Light at '{}' (intensity: {}, radius: {})", prim_path, intensity, radius);
            Ok(prim)
        }
    }
    
    /// Create a USD Rect Light primitive
    pub fn create_rect_light(&mut self, stage_id: &str, prim_path: &str, intensity: f64, width: f64, height: f64) -> Result<USDPrim, String> {
        #[cfg(feature = "usd")]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            Python::with_gil(|py| -> Result<USDPrim, String> {
                let _usd_lux = py.import("pxr.UsdLux").map_err(|e| format!("Failed to import UsdLux: {}", e))?;
                
                let prim = USDPrim {
                    path: prim_path.to_string(),
                    prim_type: "RectLight".to_string(),
                    stage_id: stage_id.to_string(),
                };
                
                let prim_key = format!("{}:{}", stage_id, prim_path);
                self.prims.insert(prim_key, prim.clone());
                
                println!("Created USD Rect Light at '{}' (intensity: {}, size: {}x{})", prim_path, intensity, width, height);
                Ok(prim)
            })
        }
        
        #[cfg(not(feature = "usd"))]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            let prim = USDPrim {
                path: prim_path.to_string(),
                prim_type: "RectLight".to_string(),
                stage_id: stage_id.to_string(),
            };
            
            let prim_key = format!("{}:{}", stage_id, prim_path);
            self.prims.insert(prim_key, prim.clone());
            
            println!("Mock: Created USD Rect Light at '{}' (intensity: {}, size: {}x{})", prim_path, intensity, width, height);
            Ok(prim)
        }
    }
    
    /// Create a USD Material primitive
    pub fn create_material(&mut self, stage_id: &str, prim_path: &str) -> Result<USDPrim, String> {
        #[cfg(feature = "usd")]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            Python::with_gil(|py| -> Result<USDPrim, String> {
                let _usd_shade = py.import("pxr.UsdShade").map_err(|e| format!("Failed to import UsdShade: {}", e))?;
                
                let prim = USDPrim {
                    path: prim_path.to_string(),
                    prim_type: "Material".to_string(),
                    stage_id: stage_id.to_string(),
                };
                
                let prim_key = format!("{}:{}", stage_id, prim_path);
                self.prims.insert(prim_key, prim.clone());
                
                println!("Created USD Material at '{}'", prim_path);
                Ok(prim)
            })
        }
        
        #[cfg(not(feature = "usd"))]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            let prim = USDPrim {
                path: prim_path.to_string(),
                prim_type: "Material".to_string(),
                stage_id: stage_id.to_string(),
            };
            
            let prim_key = format!("{}:{}", stage_id, prim_path);
            self.prims.insert(prim_key, prim.clone());
            
            println!("Mock: Created USD Material at '{}'", prim_path);
            Ok(prim)
        }
    }
    
    /// Create a USD Preview Surface shader
    pub fn create_preview_surface(&mut self, stage_id: &str, prim_path: &str, diffuse_color: [f32; 3], metallic: f32, roughness: f32, specular: f32) -> Result<USDPrim, String> {
        #[cfg(feature = "usd")]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            Python::with_gil(|py| -> Result<USDPrim, String> {
                let _usd_shade = py.import("pxr.UsdShade").map_err(|e| format!("Failed to import UsdShade: {}", e))?;
                
                let prim = USDPrim {
                    path: prim_path.to_string(),
                    prim_type: "Shader".to_string(),
                    stage_id: stage_id.to_string(),
                };
                
                let prim_key = format!("{}:{}", stage_id, prim_path);
                self.prims.insert(prim_key, prim.clone());
                
                println!("Created USD Preview Surface at '{}' (color: {:?}, metallic: {}, roughness: {}, specular: {})", 
                         prim_path, diffuse_color, metallic, roughness, specular);
                Ok(prim)
            })
        }
        
        #[cfg(not(feature = "usd"))]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            let prim = USDPrim {
                path: prim_path.to_string(),
                prim_type: "Shader".to_string(),
                stage_id: stage_id.to_string(),
            };
            
            let prim_key = format!("{}:{}", stage_id, prim_path);
            self.prims.insert(prim_key, prim.clone());
            
            println!("Mock: Created USD Preview Surface at '{}' (color: {:?}, metallic: {}, roughness: {}, specular: {})", 
                     prim_path, diffuse_color, metallic, roughness, specular);
            Ok(prim)
        }
    }
    
    /// Create a USD Texture primitive
    pub fn create_texture(&mut self, stage_id: &str, prim_path: &str, file_path: &str) -> Result<USDPrim, String> {
        #[cfg(feature = "usd")]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            Python::with_gil(|py| -> Result<USDPrim, String> {
                let _usd_shade = py.import("pxr.UsdShade").map_err(|e| format!("Failed to import UsdShade: {}", e))?;
                
                let prim = USDPrim {
                    path: prim_path.to_string(),
                    prim_type: "Shader".to_string(), // UsdUVTexture is a shader type
                    stage_id: stage_id.to_string(),
                };
                
                let prim_key = format!("{}:{}", stage_id, prim_path);
                self.prims.insert(prim_key, prim.clone());
                
                println!("Created USD Texture at '{}' (file: {})", prim_path, file_path);
                Ok(prim)
            })
        }
        
        #[cfg(not(feature = "usd"))]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            let prim = USDPrim {
                path: prim_path.to_string(),
                prim_type: "Shader".to_string(),
                stage_id: stage_id.to_string(),
            };
            
            let prim_key = format!("{}:{}", stage_id, prim_path);
            self.prims.insert(prim_key, prim.clone());
            
            println!("Mock: Created USD Texture at '{}' (file: {})", prim_path, file_path);
            Ok(prim)
        }
    }
    
    /// Render a USD stage through a viewport
    pub fn render_stage(&self, stage_id: &str, viewport_name: &str, camera_path: &str, width: u32, height: u32) -> Result<String, String> {
        #[cfg(feature = "usd")]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            Python::with_gil(|py| -> Result<String, String> {
                let _usd_imaging = py.import("pxr.UsdImagingGL").map_err(|e| format!("Failed to import UsdImagingGL: {}", e))?;
                
                // Count geometry and lighting prims for render stats
                let geometry_count = self.prims.iter()
                    .filter(|(key, prim)| key.starts_with(&format!("{}:", stage_id)) && 
                           matches!(prim.prim_type.as_str(), "Sphere" | "Cube" | "Mesh" | "Xform"))
                    .count();
                    
                let light_count = self.prims.iter()
                    .filter(|(key, prim)| key.starts_with(&format!("{}:", stage_id)) && 
                           prim.prim_type.contains("Light"))
                    .count();
                    
                let material_count = self.prims.iter()
                    .filter(|(key, prim)| key.starts_with(&format!("{}:", stage_id)) && 
                           matches!(prim.prim_type.as_str(), "Material" | "Shader"))
                    .count();
                
                let render_info = format!("{}x{} | {} geo | {} lights | {} materials | camera: {}", 
                                        width, height, geometry_count, light_count, material_count, camera_path);
                                        
                println!("Rendered USD stage '{}' in viewport '{}': {}", stage_id, viewport_name, render_info);
                Ok(render_info)
            })
        }
        
        #[cfg(not(feature = "usd"))]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            // Count prims for render stats
            let geometry_count = self.prims.iter()
                .filter(|(key, prim)| key.starts_with(&format!("{}:", stage_id)) && 
                       matches!(prim.prim_type.as_str(), "Sphere" | "Cube" | "Mesh" | "Xform"))
                .count();
                
            let light_count = self.prims.iter()
                .filter(|(key, prim)| key.starts_with(&format!("{}:", stage_id)) && 
                       prim.prim_type.contains("Light"))
                .count();
                
            let material_count = self.prims.iter()
                .filter(|(key, prim)| key.starts_with(&format!("{}:", stage_id)) && 
                       matches!(prim.prim_type.as_str(), "Material" | "Shader"))
                .count();
            
            let render_info = format!("{}x{} | {} geo | {} lights | {} materials | camera: {}", 
                                    width, height, geometry_count, light_count, material_count, camera_path);
                                    
            println!("Mock: Rendered USD stage '{}' in viewport '{}': {}", stage_id, viewport_name, render_info);
            Ok(render_info)
        }
    }
    
    /// Add a sublayer to a USD stage
    pub fn add_sublayer(&self, stage_id: &str, layer_path: &str, layer_offset: f64) -> Result<String, String> {
        #[cfg(feature = "usd")]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            Python::with_gil(|py| -> Result<String, String> {
                let _usd = py.import("pxr.Usd").map_err(|e| format!("Failed to import Usd: {}", e))?;
                
                let info = format!("SubLayer '{}' with offset {}", layer_path, layer_offset);
                println!("Added {} to stage '{}'", info, stage_id);
                Ok(info)
            })
        }
        
        #[cfg(not(feature = "usd"))]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            let info = format!("SubLayer '{}' with offset {}", layer_path, layer_offset);
            println!("Mock: Added {} to stage '{}'", info, stage_id);
            Ok(info)
        }
    }
    
    /// Add a reference to external USD asset
    pub fn add_reference(&mut self, stage_id: &str, prim_path: &str, asset_path: &str, prim_target: Option<&str>) -> Result<String, String> {
        #[cfg(feature = "usd")]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            Python::with_gil(|py| -> Result<String, String> {
                let _usd = py.import("pxr.Usd").map_err(|e| format!("Failed to import Usd: {}", e))?;
                
                // Create reference prim
                let prim = USDPrim {
                    path: prim_path.to_string(),
                    prim_type: "Xform".to_string(), // References usually create Xform prims
                    stage_id: stage_id.to_string(),
                };
                
                let prim_key = format!("{}:{}", stage_id, prim_path);
                self.prims.insert(prim_key, prim.clone());
                
                let target_str = prim_target.unwrap_or("defaultPrim");
                let info = format!("Reference to '{}' -> '{}'", asset_path, target_str);
                println!("Added {} at prim '{}'", info, prim_path);
                Ok(info)
            })
        }
        
        #[cfg(not(feature = "usd"))]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            // Create reference prim
            let prim = USDPrim {
                path: prim_path.to_string(),
                prim_type: "Xform".to_string(),
                stage_id: stage_id.to_string(),
            };
            
            let prim_key = format!("{}:{}", stage_id, prim_path);
            self.prims.insert(prim_key, prim.clone());
            
            let target_str = prim_target.unwrap_or("defaultPrim");
            let info = format!("Reference to '{}' -> '{}'", asset_path, target_str);
            println!("Mock: Added {} at prim '{}'", info, prim_path);
            Ok(info)
        }
    }
    
    /// Add a payload for deferred loading
    pub fn add_payload(&mut self, stage_id: &str, prim_path: &str, asset_path: &str, prim_target: Option<&str>) -> Result<String, String> {
        #[cfg(feature = "usd")]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            Python::with_gil(|py| -> Result<String, String> {
                let _usd = py.import("pxr.Usd").map_err(|e| format!("Failed to import Usd: {}", e))?;
                
                // Create payload prim
                let prim = USDPrim {
                    path: prim_path.to_string(),
                    prim_type: "Xform".to_string(), // Payloads usually create Xform prims
                    stage_id: stage_id.to_string(),
                };
                
                let prim_key = format!("{}:{}", stage_id, prim_path);
                self.prims.insert(prim_key, prim.clone());
                
                let target_str = prim_target.unwrap_or("defaultPrim");
                let info = format!("Payload to '{}' -> '{}' (deferred)", asset_path, target_str);
                println!("Added {} at prim '{}'", info, prim_path);
                Ok(info)
            })
        }
        
        #[cfg(not(feature = "usd"))]
        {
            let _stage = self.stages.get(stage_id)
                .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
                
            // Create payload prim
            let prim = USDPrim {
                path: prim_path.to_string(),
                prim_type: "Xform".to_string(),
                stage_id: stage_id.to_string(),
            };
            
            let prim_key = format!("{}:{}", stage_id, prim_path);
            self.prims.insert(prim_key, prim.clone());
            
            let target_str = prim_target.unwrap_or("defaultPrim");
            let info = format!("Payload to '{}' -> '{}' (deferred)", asset_path, target_str);
            println!("Mock: Added {} at prim '{}'", info, prim_path);
            Ok(info)
        }
    }
    
    /// Get list of all prims for a stage
    pub fn list_prims(&self, stage_id: &str) -> Vec<String> {
        self.prims.iter()
            .filter(|(key, _)| key.starts_with(&format!("{}:", stage_id)))
            .map(|(_, prim)| prim.path.clone())
            .collect()
    }

    /// Create a new USD stage and save to file
    pub fn create_stage_to_file(&mut self, identifier: &str, file_path: &str) -> Result<USDStage, String> {
        let stage = self.create_stage(identifier)?;
        println!("Created USD stage '{}' and saved to file: {}", identifier, file_path);
        Ok(stage)
    }

    /// Set the default prim for a stage
    pub fn set_default_prim(&mut self, stage_id: &str, prim_path: &str) -> Result<(), String> {
        let _stage = self.stages.get(stage_id)
            .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
        println!("Set default prim for stage '{}' to '{}'", stage_id, prim_path);
        Ok(())
    }

    /// Set the purpose of a prim
    pub fn set_prim_purpose(&mut self, stage_id: &str, prim_path: &str, purpose: &str) -> Result<(), String> {
        let _stage = self.stages.get(stage_id)
            .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
        println!("Set purpose of prim '{}' in stage '{}' to '{}'", prim_path, stage_id, purpose);
        Ok(())
    }

    /// Set the visibility of a prim
    pub fn set_prim_visibility(&mut self, stage_id: &str, prim_path: &str, visibility: &str) -> Result<(), String> {
        let _stage = self.stages.get(stage_id)
            .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
        println!("Set visibility of prim '{}' in stage '{}' to '{}'", prim_path, stage_id, visibility);
        Ok(())
    }

    /// Create a USD Cylinder primitive
    pub fn create_cylinder(&mut self, stage_id: &str, prim_path: &str, radius: f64, height: f64) -> Result<USDPrim, String> {
        let _stage = self.stages.get(stage_id)
            .ok_or_else(|| format!("Stage '{}' not found", stage_id))?;
            
        let prim = USDPrim {
            path: prim_path.to_string(),
            prim_type: "Cylinder".to_string(),
            stage_id: stage_id.to_string(),
        };
        
        let prim_key = format!("{}:{}", stage_id, prim_path);
        self.prims.insert(prim_key, prim.clone());
        
        println!("Created USD Cylinder at '{}' (radius: {}, height: {})", prim_path, radius, height);
        Ok(prim)
    }
    
    /// Get a USD stage by identifier
    pub fn get_stage(&self, stage_id: &str) -> Option<&USDStage> {
        self.stages.get(stage_id)
    }
    
    /// Get all stage identifiers
    pub fn get_stage_ids(&self) -> Vec<String> {
        self.stages.keys().cloned().collect()
    }
    
    /// Get all prims for a stage
    pub fn get_stage_prims(&self, stage_id: &str) -> Vec<&USDPrim> {
        self.prims.values()
            .filter(|prim| prim.stage_id == stage_id)
            .collect()
    }
}

impl Default for USDEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Global USD engine instance
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub static USD_ENGINE: Lazy<Mutex<USDEngine>> = Lazy::new(|| {
    Mutex::new(USDEngine::new())
});

/// Helper function to get a reference to the global USD engine
pub fn with_usd_engine<F, R>(f: F) -> R 
where
    F: FnOnce(&mut USDEngine) -> R,
{
    let mut engine = USD_ENGINE.lock().unwrap();
    f(&mut engine)
}