//! USD engine implementation for 3D workspace using PyO3 to interface with USD Python API

#[cfg(feature = "usd")]
use pyo3::prelude::*;
#[cfg(feature = "usd")]
use pyo3::types::{PyDict, PyList, PyString};
#[cfg(feature = "usd")]
use numpy::{PyArray1, PyArray2, PyArrayMethods};
use std::collections::HashMap;
use glam::{Mat4, Vec3, Vec2};
use serde::{Serialize, Deserialize};

/// USD Stage handle - holds a reference to a USD stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct USDStage {
    pub path: String,
    pub identifier: String,
}

/// USD Primvar with interpolation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct USDPrimvar {
    pub name: String,
    pub interpolation: String,  // "vertex", "faceVarying", "uniform", "constant"
    pub data_type: String,      // "float3", "float2", "float", "int", etc.
    pub values: PrimvarValues,
    pub indices: Option<Vec<u32>>,  // For indexed primvars
}

/// Primvar value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrimvarValues {
    Float(Vec<f32>),
    Float2(Vec<Vec2>),
    Float3(Vec<Vec3>),
    Int(Vec<i32>),
    String(Vec<String>),
}

/// General USD attribute (all prim attributes, not just primvars)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct USDAttribute {
    pub name: String,
    pub value_type: String,      // "float3", "string", "token", "bool", "matrix4d", etc.
    pub value: AttributeValue,
    pub is_custom: bool,         // True for custom attributes, false for built-in
    pub metadata: std::collections::HashMap<String, String>, // Any metadata on the attribute
}

/// USD attribute value types (more comprehensive than PrimvarValues)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeValue {
    // Scalar types
    Bool(bool),
    Int(i32),
    Float(f32),
    Double(f64),
    String(String),
    Token(String),
    Asset(String),
    
    // Vector types
    Float2(Vec2),
    Float3(Vec3),
    Color3f(Vec3),
    Normal3f(Vec3),
    Point3f(Vec3),
    Vector3f(Vec3),
    TexCoord2f(Vec2),
    
    // Matrix types
    Matrix4d(glam::Mat4),
    
    // Array types
    BoolArray(Vec<bool>),
    IntArray(Vec<i32>),
    FloatArray(Vec<f32>),
    DoubleArray(Vec<f64>),
    StringArray(Vec<String>),
    TokenArray(Vec<String>),
    AssetArray(Vec<String>),
    Float2Array(Vec<Vec2>),
    Float3Array(Vec<Vec3>),
    Color3fArray(Vec<Vec3>),
    Normal3fArray(Vec<Vec3>),
    Point3fArray(Vec<Vec3>),
    Vector3fArray(Vec<Vec3>),
    TexCoord2fArray(Vec<Vec2>),
    Matrix4dArray(Vec<glam::Mat4>),
    
    // Special USD types
    Relationship(Vec<String>), // USD relationships (paths to other prims)
    TimeSamples(Vec<(f64, String)>), // Time-varying values (time, value) pairs
    
    // Fallback for unknown types
    Unknown(String),
}

/// USD Geometry extracted from USD mesh prims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct USDMeshGeometry {
    pub prim_path: String,
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub vertex_colors: Option<Vec<Vec3>>,  // Optional vertex colors from displayColor
    pub transform: Mat4,
    pub primvars: Vec<USDPrimvar>,  // All primvars with their interpolation types
    pub attributes: Vec<USDAttribute>,     // ALL USD prim attributes (built-in + custom)
}

/// USD Light extracted from UsdLux prims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct USDLightData {
    pub prim_path: String,
    pub light_type: String,
    pub intensity: f32,
    pub color: Vec3,
    pub transform: Mat4,
}

/// USD Material extracted from UsdShade prims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct USDMaterialData {
    pub prim_path: String,
    pub diffuse_color: Vec3,
    pub metallic: f32,
    pub roughness: f32,
}

/// Lightweight USD metadata for scenegraph tree display (no geometry data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct USDScenegraphMetadata {
    pub stage_path: String,
    pub meshes: Vec<USDMeshMetadata>,
    pub lights: Vec<USDLightMetadata>,
    pub materials: Vec<USDMaterialMetadata>,
    pub up_axis: String,
    pub total_vertices: usize,
    pub total_triangles: usize,
}

/// Lightweight mesh metadata for scenegraph display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct USDMeshMetadata {
    pub prim_path: String,
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub has_normals: bool,
    pub has_uvs: bool,
    pub has_colors: bool,
    pub material_binding: Option<String>,
}

/// Lightweight light metadata for scenegraph display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct USDLightMetadata {
    pub prim_path: String,
    pub light_type: String,
    pub intensity: f32,
}

/// Lightweight material metadata for scenegraph display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct USDMaterialMetadata {
    pub prim_path: String,
    pub has_diffuse_texture: bool,
    pub has_normal_texture: bool,
    pub has_metallic_roughness: bool,
}

/// USD Scene extracted from a stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct USDSceneData {
    pub stage_path: String,
    pub meshes: Vec<USDMeshGeometry>,
    pub lights: Vec<USDLightData>,
    pub materials: Vec<USDMaterialData>,
    pub up_axis: String, // USD up axis: "Y", "Z", etc.
}

/// USD Engine for 3D workspace - manages USD operations through Python API
pub struct USDEngine {
    #[cfg(feature = "usd")]
    _python_initialized: bool,
    stages: HashMap<String, USDStage>,
}

impl USDEngine {
    pub fn new() -> Self {
        // Set up Python environment for embedded Python
        std::env::set_var("PYTHONHOME", "./vendor/python-runtime/python");
        std::env::set_var("PYTHONPATH", "./vendor/python-runtime/python/lib/python3.9/site-packages:./vendor/python-runtime/python/lib/python3.9");
        std::env::set_var("PYTHONDONTWRITEBYTECODE", "1");
        
        Self {
            #[cfg(feature = "usd")]
            _python_initialized: true,
            stages: HashMap::new(),
        }
    }
}

/// Helper function to extract AttributeValue from Python object
#[cfg(feature = "usd")]
fn extract_attribute_value(value_obj: &pyo3::PyObject, py: Python) -> AttributeValue {
    use pyo3::prelude::*;
    
    // The value_obj should be a dictionary with one key indicating the type
    if let Ok(value_dict) = value_obj.extract::<std::collections::HashMap<String, pyo3::PyObject>>(py) {
        if let Some((key, val)) = value_dict.iter().next() {
            match key.as_str() {
                "Bool" => {
                    if let Ok(v) = val.extract::<bool>(py) {
                        return AttributeValue::Bool(v);
                    }
                },
                "Int" => {
                    if let Ok(v) = val.extract::<i32>(py) {
                        return AttributeValue::Int(v);
                    }
                },
                "Float" => {
                    if let Ok(v) = val.extract::<f32>(py) {
                        return AttributeValue::Float(v);
                    }
                },
                "Double" => {
                    if let Ok(v) = val.extract::<f64>(py) {
                        return AttributeValue::Double(v);
                    }
                },
                "String" => {
                    if let Ok(v) = val.extract::<String>(py) {
                        return AttributeValue::String(v);
                    }
                },
                "Token" => {
                    if let Ok(v) = val.extract::<String>(py) {
                        return AttributeValue::Token(v);
                    }
                },
                "Asset" => {
                    if let Ok(v) = val.extract::<String>(py) {
                        return AttributeValue::Asset(v);
                    }
                },
                "Float2" => {
                    if let Ok(v) = val.extract::<Vec<f32>>(py) {
                        if v.len() >= 2 {
                            return AttributeValue::Float2(Vec2::new(v[0], v[1]));
                        }
                    }
                },
                "Float3" => {
                    if let Ok(v) = val.extract::<Vec<f32>>(py) {
                        if v.len() >= 3 {
                            return AttributeValue::Float3(Vec3::new(v[0], v[1], v[2]));
                        }
                    }
                },
                "Color3f" => {
                    if let Ok(v) = val.extract::<Vec<f32>>(py) {
                        if v.len() >= 3 {
                            return AttributeValue::Color3f(Vec3::new(v[0], v[1], v[2]));
                        }
                    }
                },
                "Normal3f" => {
                    if let Ok(v) = val.extract::<Vec<f32>>(py) {
                        if v.len() >= 3 {
                            return AttributeValue::Normal3f(Vec3::new(v[0], v[1], v[2]));
                        }
                    }
                },
                "Point3f" => {
                    if let Ok(v) = val.extract::<Vec<f32>>(py) {
                        if v.len() >= 3 {
                            return AttributeValue::Point3f(Vec3::new(v[0], v[1], v[2]));
                        }
                    }
                },
                "Vector3f" => {
                    if let Ok(v) = val.extract::<Vec<f32>>(py) {
                        if v.len() >= 3 {
                            return AttributeValue::Vector3f(Vec3::new(v[0], v[1], v[2]));
                        }
                    }
                },
                "Matrix4d" => {
                    if let Ok(v) = val.extract::<Vec<f32>>(py) {
                        if v.len() >= 16 {
                            let matrix = glam::Mat4::from_cols_array(&[
                                v[0], v[1], v[2], v[3],
                                v[4], v[5], v[6], v[7],
                                v[8], v[9], v[10], v[11],
                                v[12], v[13], v[14], v[15],
                            ]);
                            return AttributeValue::Matrix4d(matrix);
                        }
                    }
                },
                "BoolArray" => {
                    if let Ok(v) = val.extract::<Vec<bool>>(py) {
                        return AttributeValue::BoolArray(v);
                    }
                },
                "IntArray" => {
                    if let Ok(v) = val.extract::<Vec<i32>>(py) {
                        return AttributeValue::IntArray(v);
                    }
                },
                "FloatArray" => {
                    if let Ok(v) = val.extract::<Vec<f32>>(py) {
                        return AttributeValue::FloatArray(v);
                    }
                },
                "StringArray" => {
                    if let Ok(v) = val.extract::<Vec<String>>(py) {
                        return AttributeValue::StringArray(v);
                    }
                },
                "TokenArray" => {
                    if let Ok(v) = val.extract::<Vec<String>>(py) {
                        return AttributeValue::TokenArray(v);
                    }
                },
                "Float3Array" => {
                    if let Ok(v) = val.extract::<Vec<Vec<f32>>>(py) {
                        let vec3_array: Vec<Vec3> = v.into_iter()
                            .filter(|arr| arr.len() >= 3)
                            .map(|arr| Vec3::new(arr[0], arr[1], arr[2]))
                            .collect();
                        return AttributeValue::Float3Array(vec3_array);
                    }
                },
                "Color3fArray" => {
                    if let Ok(v) = val.extract::<Vec<Vec<f32>>>(py) {
                        let vec3_array: Vec<Vec3> = v.into_iter()
                            .filter(|arr| arr.len() >= 3)
                            .map(|arr| Vec3::new(arr[0], arr[1], arr[2]))
                            .collect();
                        return AttributeValue::Color3fArray(vec3_array);
                    }
                },
                "Unknown" => {
                    if let Ok(v) = val.extract::<String>(py) {
                        return AttributeValue::Unknown(v);
                    }
                },
                _ => {}
            }
        }
    }
    
    // Fallback
    AttributeValue::Unknown("Failed to extract value".to_string())
}

impl USDEngine {
    /// Extract lightweight metadata from full USD scene data for scenegraph display
    pub fn extract_scenegraph_metadata(scene_data: &USDSceneData) -> USDScenegraphMetadata {
        let mut total_vertices = 0;
        let mut total_triangles = 0;
        
        let meshes = scene_data.meshes.iter().map(|mesh| {
            let vertex_count = mesh.vertices.len();
            let triangle_count = mesh.indices.len() / 3;
            
            total_vertices += vertex_count;
            total_triangles += triangle_count;
            
            USDMeshMetadata {
                prim_path: mesh.prim_path.clone(),
                vertex_count,
                triangle_count,
                has_normals: !mesh.normals.is_empty(),
                has_uvs: !mesh.uvs.is_empty(),
                has_colors: mesh.vertex_colors.is_some(),
                material_binding: None, // TODO: Extract material binding from USD data
            }
        }).collect();
        
        let lights = scene_data.lights.iter().map(|light| {
            USDLightMetadata {
                prim_path: light.prim_path.clone(),
                light_type: light.light_type.clone(),
                intensity: light.intensity,
            }
        }).collect();
        
        let materials = scene_data.materials.iter().map(|material| {
            USDMaterialMetadata {
                prim_path: material.prim_path.clone(),
                has_diffuse_texture: false, // TODO: Extract texture info from USD data
                has_normal_texture: false,
                has_metallic_roughness: true,
            }
        }).collect();
        
        USDScenegraphMetadata {
            stage_path: scene_data.stage_path.clone(),
            meshes,
            lights,
            materials,
            up_axis: scene_data.up_axis.clone(),
            total_vertices,
            total_triangles,
        }
    }

    /// Extract attribute value from Python object to Rust enum
    #[cfg(feature = "usd")]
    fn extract_attribute_value(value_obj: &pyo3::PyObject, py: Python) -> AttributeValue {
        use pyo3::types::PyDict;
        
        // The Python code structures values as {'Bool': value}, {'Float': value}, etc.
        if let Ok(value_dict) = value_obj.extract::<HashMap<String, pyo3::PyObject>>(py) {
            for (key, val) in value_dict {
                match key.as_str() {
                    "Bool" => {
                        if let Ok(b) = val.extract::<bool>(py) {
                            return AttributeValue::Bool(b);
                        }
                    },
                    "Int" => {
                        if let Ok(i) = val.extract::<i32>(py) {
                            return AttributeValue::Int(i);
                        }
                    },
                    "Float" => {
                        if let Ok(f) = val.extract::<f32>(py) {
                            return AttributeValue::Float(f);
                        }
                    },
                    "Double" => {
                        if let Ok(d) = val.extract::<f64>(py) {
                            return AttributeValue::Double(d);
                        }
                    },
                    "String" => {
                        if let Ok(s) = val.extract::<String>(py) {
                            return AttributeValue::String(s);
                        }
                    },
                    "Token" => {
                        if let Ok(s) = val.extract::<String>(py) {
                            return AttributeValue::Token(s);
                        }
                    },
                    "Asset" => {
                        if let Ok(s) = val.extract::<String>(py) {
                            return AttributeValue::Asset(s);
                        }
                    },
                    "Float2" => {
                        if let Ok(vec) = val.extract::<Vec<f32>>(py) {
                            if vec.len() >= 2 {
                                return AttributeValue::Float2(Vec2::new(vec[0], vec[1]));
                            }
                        }
                    },
                    "Float3" => {
                        if let Ok(vec) = val.extract::<Vec<f32>>(py) {
                            if vec.len() >= 3 {
                                return AttributeValue::Float3(Vec3::new(vec[0], vec[1], vec[2]));
                            }
                        }
                    },
                    "Color3f" => {
                        if let Ok(vec) = val.extract::<Vec<f32>>(py) {
                            if vec.len() >= 3 {
                                return AttributeValue::Color3f(Vec3::new(vec[0], vec[1], vec[2]));
                            }
                        }
                    },
                    "Normal3f" => {
                        if let Ok(vec) = val.extract::<Vec<f32>>(py) {
                            if vec.len() >= 3 {
                                return AttributeValue::Normal3f(Vec3::new(vec[0], vec[1], vec[2]));
                            }
                        }
                    },
                    "Point3f" => {
                        if let Ok(vec) = val.extract::<Vec<f32>>(py) {
                            if vec.len() >= 3 {
                                return AttributeValue::Point3f(Vec3::new(vec[0], vec[1], vec[2]));
                            }
                        }
                    },
                    "Vector3f" => {
                        if let Ok(vec) = val.extract::<Vec<f32>>(py) {
                            if vec.len() >= 3 {
                                return AttributeValue::Vector3f(Vec3::new(vec[0], vec[1], vec[2]));
                            }
                        }
                    },
                    "TexCoord2f" => {
                        if let Ok(vec) = val.extract::<Vec<f32>>(py) {
                            if vec.len() >= 2 {
                                return AttributeValue::TexCoord2f(Vec2::new(vec[0], vec[1]));
                            }
                        }
                    },
                    "Matrix4d" => {
                        if let Ok(matrix_array) = val.extract::<Vec<f32>>(py) {
                            if matrix_array.len() >= 16 {
                                return AttributeValue::Matrix4d(Mat4::from_cols_array(&[
                                    matrix_array[0], matrix_array[1], matrix_array[2], matrix_array[3],
                                    matrix_array[4], matrix_array[5], matrix_array[6], matrix_array[7],
                                    matrix_array[8], matrix_array[9], matrix_array[10], matrix_array[11],
                                    matrix_array[12], matrix_array[13], matrix_array[14], matrix_array[15],
                                ]));
                            }
                        }
                    },
                    // Array types
                    "BoolArray" => {
                        if let Ok(arr) = val.extract::<Vec<bool>>(py) {
                            return AttributeValue::BoolArray(arr);
                        }
                    },
                    "IntArray" => {
                        if let Ok(arr) = val.extract::<Vec<i32>>(py) {
                            return AttributeValue::IntArray(arr);
                        }
                    },
                    "FloatArray" => {
                        if let Ok(arr) = val.extract::<Vec<f32>>(py) {
                            return AttributeValue::FloatArray(arr);
                        }
                    },
                    "DoubleArray" => {
                        if let Ok(arr) = val.extract::<Vec<f64>>(py) {
                            return AttributeValue::DoubleArray(arr);
                        }
                    },
                    "StringArray" => {
                        if let Ok(arr) = val.extract::<Vec<String>>(py) {
                            return AttributeValue::StringArray(arr);
                        }
                    },
                    "TokenArray" => {
                        if let Ok(arr) = val.extract::<Vec<String>>(py) {
                            return AttributeValue::TokenArray(arr);
                        }
                    },
                    "AssetArray" => {
                        if let Ok(arr) = val.extract::<Vec<String>>(py) {
                            return AttributeValue::AssetArray(arr);
                        }
                    },
                    "Float2Array" => {
                        if let Ok(arr) = val.extract::<Vec<Vec<f32>>>(py) {
                            let vec2_arr = arr.into_iter()
                                .filter(|v| v.len() >= 2)
                                .map(|v| Vec2::new(v[0], v[1]))
                                .collect();
                            return AttributeValue::Float2Array(vec2_arr);
                        }
                    },
                    "Float3Array" => {
                        if let Ok(arr) = val.extract::<Vec<Vec<f32>>>(py) {
                            let vec3_arr = arr.into_iter()
                                .filter(|v| v.len() >= 3)
                                .map(|v| Vec3::new(v[0], v[1], v[2]))
                                .collect();
                            return AttributeValue::Float3Array(vec3_arr);
                        }
                    },
                    "Color3fArray" => {
                        if let Ok(arr) = val.extract::<Vec<Vec<f32>>>(py) {
                            let vec3_arr = arr.into_iter()
                                .filter(|v| v.len() >= 3)
                                .map(|v| Vec3::new(v[0], v[1], v[2]))
                                .collect();
                            return AttributeValue::Color3fArray(vec3_arr);
                        }
                    },
                    "Normal3fArray" => {
                        if let Ok(arr) = val.extract::<Vec<Vec<f32>>>(py) {
                            let vec3_arr = arr.into_iter()
                                .filter(|v| v.len() >= 3)
                                .map(|v| Vec3::new(v[0], v[1], v[2]))
                                .collect();
                            return AttributeValue::Normal3fArray(vec3_arr);
                        }
                    },
                    "Point3fArray" => {
                        if let Ok(arr) = val.extract::<Vec<Vec<f32>>>(py) {
                            let vec3_arr = arr.into_iter()
                                .filter(|v| v.len() >= 3)
                                .map(|v| Vec3::new(v[0], v[1], v[2]))
                                .collect();
                            return AttributeValue::Point3fArray(vec3_arr);
                        }
                    },
                    "Vector3fArray" => {
                        if let Ok(arr) = val.extract::<Vec<Vec<f32>>>(py) {
                            let vec3_arr = arr.into_iter()
                                .filter(|v| v.len() >= 3)
                                .map(|v| Vec3::new(v[0], v[1], v[2]))
                                .collect();
                            return AttributeValue::Vector3fArray(vec3_arr);
                        }
                    },
                    "TexCoord2fArray" => {
                        if let Ok(arr) = val.extract::<Vec<Vec<f32>>>(py) {
                            let vec2_arr = arr.into_iter()
                                .filter(|v| v.len() >= 2)
                                .map(|v| Vec2::new(v[0], v[1]))
                                .collect();
                            return AttributeValue::TexCoord2fArray(vec2_arr);
                        }
                    },
                    "Matrix4dArray" => {
                        if let Ok(arr) = val.extract::<Vec<Vec<f32>>>(py) {
                            let mat4_arr = arr.into_iter()
                                .filter(|m| m.len() >= 16)
                                .map(|m| Mat4::from_cols_array(&[
                                    m[0], m[1], m[2], m[3],
                                    m[4], m[5], m[6], m[7],
                                    m[8], m[9], m[10], m[11],
                                    m[12], m[13], m[14], m[15],
                                ]))
                                .collect();
                            return AttributeValue::Matrix4dArray(mat4_arr);
                        }
                    },
                    "Unknown" => {
                        if let Ok(s) = val.extract::<String>(py) {
                            return AttributeValue::Unknown(s);
                        }
                    },
                    _ => {
                        // Unknown key, try to convert to string
                        if let Ok(s) = val.extract::<String>(py) {
                            return AttributeValue::Unknown(format!("{}:{}", key, s));
                        }
                    }
                }
            }
        }
        
        // Fallback: convert the whole object to string
        if let Ok(s) = value_obj.extract::<String>(py) {
            AttributeValue::Unknown(s)
        } else {
            AttributeValue::Unknown("Failed to extract attribute value".to_string())
        }
    }

    /// Load a USD stage from file and extract scene data
    pub fn load_stage(&mut self, file_path: &str) -> Result<USDSceneData, String> {
        println!("🎬 🎬 🎬 USDEngine: REAL USD LOADING CALLED for {}", file_path);
        
        #[cfg(feature = "usd")]
        {
            let start_time = std::time::Instant::now();
            
            // Process all meshes in Python and return packed data
            let scene_data = Python::with_gil(|py| -> Result<USDSceneData, String> {
                println!("🎬 USDEngine: Opening USD stage with Python...");
                
                
                // Execute the Python function with optimized pure Python
                py.run(c"def extract_all_meshes(stage_path):
    import math
    from pxr import Usd, UsdGeom
    
    # Open stage once
    stage = Usd.Stage.Open(stage_path)
    if not stage:
        return None
    
    # Get the up axis from the stage
    up_axis = 'Z'  # Default to Z-up
    if hasattr(stage, 'GetMetadata'):
        metadata = stage.GetMetadata('upAxis')
        if metadata:
            up_axis = metadata
    
    meshes = []
    
    # Traverse once and collect all mesh data
    for prim in stage.Traverse():
        prim_type = prim.GetTypeName()
        
        if prim_type == 'Mesh':
            mesh = UsdGeom.Mesh(prim)
            prim_path = str(prim.GetPath())
            
            # Get all attributes at once
            points_attr = mesh.GetPointsAttr()
            indices_attr = mesh.GetFaceVertexIndicesAttr() 
            counts_attr = mesh.GetFaceVertexCountsAttr()
            display_color_attr = mesh.GetDisplayColorAttr()  # Get vertex colors
            
            if not (points_attr and indices_attr and counts_attr):
                continue
                
            # Extract raw data
            points = points_attr.Get()
            face_indices = indices_attr.Get()
            face_counts = counts_attr.Get()
            
            # Extract vertex colors if available
            vertex_colors = None
            if display_color_attr and display_color_attr.HasValue():
                colors = display_color_attr.Get()
                if colors:
                    vertex_colors = [[float(c[0]), float(c[1]), float(c[2])] for c in colors]
            
            if not (points and face_indices and face_counts):
                continue
            
            # Convert to lists for processing
            vertices = [[float(p[0]), float(p[1]), float(p[2])] for p in points]
            indices = [int(i) for i in face_indices]
            counts = [int(c) for c in face_counts]
            
            # Extract all primvars with their interpolation types
            primvars = []
            primvars_api = UsdGeom.PrimvarsAPI(mesh)
            
            # Debug: print all primvars found (commented out due to Rust syntax conflicts)
            
            for primvar in primvars_api.GetPrimvars():
                primvar_name = primvar.GetPrimvarName()
                interpolation = primvar.GetInterpolation()
                
                # Skip if no value
                if not primvar.HasValue():
                    continue
                
                # Get the value
                value = primvar.Get()
                if value is None:
                    continue
                
                # Get type information
                type_name = primvar.GetTypeName()
                
                # Check if it's indexed
                indices_attr = primvar.GetIndicesAttr()
                primvar_indices = None
                if indices_attr and indices_attr.HasValue():
                    primvar_indices = [int(i) for i in indices_attr.Get()]
                
                # Convert value to appropriate format
                primvar_data = {
                    'name': primvar_name,
                    'interpolation': interpolation,
                    'type_name': str(type_name),
                    'is_indexed': primvar_indices is not None,
                }
                
                # Convert values based on type
                if 'float3' in str(type_name) or 'point3' in str(type_name) or 'normal3' in str(type_name) or 'vector3' in str(type_name):
                    if hasattr(value, '__iter__'):
                        primvar_data['values'] = [[float(v[0]), float(v[1]), float(v[2])] for v in value]
                    else:
                        primvar_data['values'] = [[float(value[0]), float(value[1]), float(value[2])]]
                    primvar_data['data_type'] = 'float3'
                elif 'float2' in str(type_name) or 'texCoord2' in str(type_name):
                    if hasattr(value, '__iter__'):
                        primvar_data['values'] = [[float(v[0]), float(v[1])] for v in value]
                    else:
                        primvar_data['values'] = [[float(value[0]), float(value[1])]]
                    primvar_data['data_type'] = 'float2'
                elif 'float' in str(type_name) or 'double' in str(type_name):
                    if hasattr(value, '__iter__'):
                        primvar_data['values'] = [float(v) for v in value]
                    else:
                        primvar_data['values'] = [float(value)]
                    primvar_data['data_type'] = 'float'
                elif 'int' in str(type_name):
                    if hasattr(value, '__iter__'):
                        primvar_data['values'] = [int(v) for v in value]
                    else:
                        primvar_data['values'] = [int(value)]
                    primvar_data['data_type'] = 'int'
                elif 'string' in str(type_name):
                    if hasattr(value, '__iter__'):
                        primvar_data['values'] = [str(v) for v in value]
                    else:
                        primvar_data['values'] = [str(value)]
                    primvar_data['data_type'] = 'string'
                else:
                    # Unknown type, convert to string
                    primvar_data['values'] = [str(value)]
                    primvar_data['data_type'] = 'string'
                
                if primvar_indices:
                    primvar_data['indices'] = primvar_indices
                
                # Debug: Processing primvar (commented out due to Rust syntax conflicts)
                primvars.append(primvar_data)
            
            # Extract ALL USD prim attributes (not just primvars)
            all_attributes = []
            
            # Get all attributes on this prim
            for attr in prim.GetAttributes():
                attr_name = attr.GetName()
                
                # Skip if no value or already covered by primvars/geometry
                if not attr.HasValue():
                    continue
                    
                # Skip attributes we already handle elsewhere
                if attr_name in ['points', 'faceVertexIndices', 'faceVertexCounts', 'normals', 'primvars:displayColor']:
                    continue
                    
                # Skip primvars (we handle those separately)
                if attr_name.startswith('primvars:'):
                    continue
                
                # Get attribute info
                value = attr.Get()
                if value is None:
                    continue
                    
                type_name = str(attr.GetTypeName())
                is_custom = attr.IsCustom()
                
                # Convert value to appropriate format
                attr_data = {
                    'name': attr_name,
                    'value_type': type_name,
                    'is_custom': is_custom,
                    'metadata': {}  # TODO: extract metadata if needed
                }
                
                # Convert values based on USD type
                try:
                    if type_name == 'bool':
                        attr_data['value'] = {'Bool': bool(value)}
                    elif type_name in ['int', 'uint', 'int64', 'uint64']:
                        attr_data['value'] = {'Int': int(value)}
                    elif type_name == 'float':
                        attr_data['value'] = {'Float': float(value)}
                    elif type_name == 'double':
                        attr_data['value'] = {'Double': float(value)}
                    elif type_name in ['string', 'token']:
                        attr_data['value'] = {'String' if type_name == 'string' else 'Token': str(value)}
                    elif type_name == 'asset':
                        attr_data['value'] = {'Asset': str(value)}
                    elif type_name in ['float2', 'double2']:
                        if hasattr(value, '__len__') and len(value) >= 2:
                            attr_data['value'] = {'Float2': [float(value[0]), float(value[1])]}
                        else:
                            attr_data['value'] = {'Float2': [0.0, 0.0]}
                    elif type_name in ['float3', 'double3', 'point3f', 'point3d', 'vector3f', 'vector3d', 'normal3f', 'normal3d', 'color3f', 'color3d']:
                        if hasattr(value, '__len__') and len(value) >= 3:
                            if 'color' in type_name.lower():
                                attr_data['value'] = {'Color3f': [float(value[0]), float(value[1]), float(value[2])]}
                            elif 'normal' in type_name.lower():
                                attr_data['value'] = {'Normal3f': [float(value[0]), float(value[1]), float(value[2])]}
                            elif 'point' in type_name.lower():
                                attr_data['value'] = {'Point3f': [float(value[0]), float(value[1]), float(value[2])]}
                            elif 'vector' in type_name.lower():
                                attr_data['value'] = {'Vector3f': [float(value[0]), float(value[1]), float(value[2])]}
                            else:
                                attr_data['value'] = {'Float3': [float(value[0]), float(value[1]), float(value[2])]}
                        else:
                            attr_data['value'] = {'Float3': [0.0, 0.0, 0.0]}
                    elif type_name in ['matrix4d', 'matrix4f']:
                        # Convert matrix to flat array
                        if hasattr(value, 'GetArray'):
                            matrix_array = list(value.GetArray())
                        else:
                            matrix_array = [1.0,0.0,0.0,0.0, 0.0,1.0,0.0,0.0, 0.0,0.0,1.0,0.0, 0.0,0.0,0.0,1.0]
                        attr_data['value'] = {'Matrix4d': matrix_array}
                    # Array types
                    elif '[]' in type_name or type_name.endswith('Array'):
                        base_type = type_name.replace('[]', '').replace('Array', '')
                        if hasattr(value, '__iter__'):
                            if base_type in ['bool']:
                                attr_data['value'] = {'BoolArray': [bool(v) for v in value]}
                            elif base_type in ['int', 'uint']:
                                attr_data['value'] = {'IntArray': [int(v) for v in value]}
                            elif base_type in ['float', 'double']:
                                attr_data['value'] = {'FloatArray': [float(v) for v in value]}
                            elif base_type in ['string', 'token']:
                                attr_data['value'] = {'StringArray' if base_type == 'string' else 'TokenArray': [str(v) for v in value]}
                            elif base_type in ['float3', 'point3f', 'vector3f', 'normal3f', 'color3f']:
                                if 'color' in base_type:
                                    attr_data['value'] = {'Color3fArray': [[float(v[0]), float(v[1]), float(v[2])] for v in value]}
                                elif 'normal' in base_type:
                                    attr_data['value'] = {'Normal3fArray': [[float(v[0]), float(v[1]), float(v[2])] for v in value]}
                                elif 'point' in base_type:
                                    attr_data['value'] = {'Point3fArray': [[float(v[0]), float(v[1]), float(v[2])] for v in value]}
                                elif 'vector' in base_type:
                                    attr_data['value'] = {'Vector3fArray': [[float(v[0]), float(v[1]), float(v[2])] for v in value]}
                                else:
                                    attr_data['value'] = {'Float3Array': [[float(v[0]), float(v[1]), float(v[2])] for v in value]}
                            else:
                                attr_data['value'] = {'StringArray': [str(v) for v in value]}
                        else:
                            attr_data['value'] = {'StringArray': [str(value)]}
                    else:
                        # Unknown type, convert to string
                        attr_data['value'] = {'Unknown': str(value)}
                        
                    all_attributes.append(attr_data)
                    
                except Exception as e:
                    # If conversion fails, store as unknown
                    attr_data['value'] = {'Unknown': f'Error: {str(e)}'}
                    all_attributes.append(attr_data)
            
            # Fast triangulation in Python
            triangles = []
            face_start = 0
            
            for face_count in counts:
                face_verts = indices[face_start:face_start + face_count]
                
                # Fan triangulation
                if face_count == 3:
                    triangles.extend(face_verts)
                elif face_count == 4:
                    # Quad -> 2 triangles
                    triangles.extend([face_verts[0], face_verts[1], face_verts[2]])
                    triangles.extend([face_verts[0], face_verts[2], face_verts[3]])
                elif face_count > 4:
                    # N-gon fan
                    for i in range(1, face_count - 1):
                        triangles.extend([face_verts[0], face_verts[i], face_verts[i + 1]])
                        
                face_start += face_count
            
            # Normal calculation using pure Python
            normals = [[0.0, 0.0, 0.0] for _ in vertices]
            normal_counts = [0 for _ in vertices]
            
            # Calculate normals for each triangle
            for i in range(0, len(triangles), 3):
                if i + 2 < len(triangles):
                    i0, i1, i2 = triangles[i], triangles[i+1], triangles[i+2]
                    if i0 < len(vertices) and i1 < len(vertices) and i2 < len(vertices):
                        v0, v1, v2 = vertices[i0], vertices[i1], vertices[i2]
                        
                        # Calculate edges
                        edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]]
                        edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]]
                        
                        # Cross product
                        normal = [
                            edge1[1] * edge2[2] - edge1[2] * edge2[1],
                            edge1[2] * edge2[0] - edge1[0] * edge2[2],
                            edge1[0] * edge2[1] - edge1[1] * edge2[0]
                        ]
                        
                        # Normalize
                        length = math.sqrt(normal[0]**2 + normal[1]**2 + normal[2]**2)
                        if length > 0:
                            normal = [normal[0]/length, normal[1]/length, normal[2]/length]
                            
                            # Accumulate at vertices
                            for idx in [i0, i1, i2]:
                                normals[idx][0] += normal[0]
                                normals[idx][1] += normal[1]
                                normals[idx][2] += normal[2]
                                normal_counts[idx] += 1
            
            # Average and normalize accumulated normals
            for i in range(len(normals)):
                if normal_counts[i] > 0:
                    normals[i] = [normals[i][0]/normal_counts[i], 
                                 normals[i][1]/normal_counts[i], 
                                 normals[i][2]/normal_counts[i]]
                    length = math.sqrt(normals[i][0]**2 + normals[i][1]**2 + normals[i][2]**2)
                    if length > 0:
                        normals[i] = [normals[i][0]/length, normals[i][1]/length, normals[i][2]/length]
                else:
                    normals[i] = [0.0, 1.0, 0.0]  # Default up
            
            # Simple UV mapping
            uvs = []
            for vertex in vertices:
                u = (vertex[0] + 1.0) * 0.5  # X -> U
                v = (vertex[2] + 1.0) * 0.5  # Z -> V
                uvs.append([u, v])
            
            # Convert to numpy arrays for efficient PyO3 transfer
            import numpy as np
            vertices_array = np.array(vertices, dtype=np.float32)
            indices_array = np.array(triangles, dtype=np.uint32)
            normals_array = np.array(normals, dtype=np.float32)
            uvs_array = np.array(uvs, dtype=np.float32)
            
            # Pack mesh data with numpy arrays
            mesh_data = {
                'prim_path': prim_path,
                'vertices': vertices_array,
                'indices': indices_array,
                'normals': normals_array,
                'uvs': uvs_array,
                'primvars': primvars,  # Include all primvars with their interpolation
                'attributes': all_attributes  # Include ALL USD prim attributes
            }
            
            # Add vertex colors if available
            if vertex_colors:
                vertex_colors_array = np.array(vertex_colors, dtype=np.float32)
                mesh_data['vertex_colors'] = vertex_colors_array
            
            # Debug: Final counts (commented out due to Rust syntax conflicts)
            meshes.append(mesh_data)
            
        elif prim_type == 'Cube':
            cube = UsdGeom.Cube(prim)
            prim_path = str(prim.GetPath())
            
            # Get cube size
            size = 2.0  # Default size
            if cube.GetSizeAttr() and cube.GetSizeAttr().HasValue():
                size = float(cube.GetSizeAttr().Get())
            half_size = size / 2.0
            
            # Generate cube vertices
            vertices = [
                [-half_size, -half_size, -half_size], [half_size, -half_size, -half_size],
                [half_size, half_size, -half_size], [-half_size, half_size, -half_size],
                [-half_size, -half_size, half_size], [half_size, -half_size, half_size],
                [half_size, half_size, half_size], [-half_size, half_size, half_size]
            ]
            
            # Cube face indices (6 faces, 4 vertices each)
            face_indices = [
                0, 1, 2, 3,  # Front
                4, 7, 6, 5,  # Back
                0, 4, 5, 1,  # Bottom
                3, 2, 6, 7,  # Top
                0, 3, 7, 4,  # Left
                1, 5, 6, 2   # Right
            ]
            face_counts = [4, 4, 4, 4, 4, 4]
            
            # Triangulate faces with correct winding order for coordinate system
            triangles = []
            face_start = 0
            for face_count in face_counts:
                face_verts = face_indices[face_start:face_start + face_count]
                # Quad to triangles
                # Always use reversed winding order for procedural primitives
                triangles.extend([face_verts[0], face_verts[2], face_verts[1]])
                triangles.extend([face_verts[0], face_verts[3], face_verts[2]])
                face_start += face_count
                
            # Calculate normals
            normals = [[0.0, 0.0, 0.0] for _ in vertices]
            normal_counts = [0 for _ in vertices]
            
            for i in range(0, len(triangles), 3):
                i0, i1, i2 = triangles[i:i+3]
                v0, v1, v2 = vertices[i0], vertices[i1], vertices[i2]
                
                # Calculate face normal
                edge1 = [v1[j] - v0[j] for j in range(3)]
                edge2 = [v2[j] - v0[j] for j in range(3)]
                normal = [
                    edge1[1] * edge2[2] - edge1[2] * edge2[1],
                    edge1[2] * edge2[0] - edge1[0] * edge2[2],
                    edge1[0] * edge2[1] - edge1[1] * edge2[0]
                ]
                length = math.sqrt(normal[0]**2 + normal[1]**2 + normal[2]**2)
                if length > 0:
                    normal = [normal[j]/length for j in range(3)]
                    
                    # Accumulate at vertices
                    for idx in [i0, i1, i2]:
                        normals[idx][0] += normal[0]
                        normals[idx][1] += normal[1]
                        normals[idx][2] += normal[2]
                        normal_counts[idx] += 1
            
            # Normalize
            for i in range(len(normals)):
                if normal_counts[i] > 0:
                    normals[i] = [normals[i][j]/normal_counts[i] for j in range(3)]
                    length = math.sqrt(sum(n**2 for n in normals[i]))
                    if length > 0:
                        normals[i] = [normals[i][j]/length for j in range(3)]
                else:
                    normals[i] = [0.0, 1.0, 0.0]
            
            # UV mapping
            uvs = []
            for vertex in vertices:
                u = (vertex[0] + half_size) / size
                v = (vertex[2] + half_size) / size
                uvs.append([u, v])
            
            # Convert to numpy arrays
            import numpy as np
            mesh_data = {
                'prim_path': prim_path,
                'vertices': np.array(vertices, dtype=np.float32),
                'indices': np.array(triangles, dtype=np.uint32),
                'normals': np.array(normals, dtype=np.float32),
                'uvs': np.array(uvs, dtype=np.float32),
                'primvars': []  # Procedural geometry has no primvars
            }
            meshes.append(mesh_data)
            
        elif prim_type == 'Sphere':
            sphere = UsdGeom.Sphere(prim)
            prim_path = str(prim.GetPath())
            
            # Get sphere radius
            radius = 1.0  # Default radius
            if sphere.GetRadiusAttr() and sphere.GetRadiusAttr().HasValue():
                radius = float(sphere.GetRadiusAttr().Get())
            
            # Generate sphere geometry
            u_res = 20  # longitude resolution
            v_res = 16  # latitude resolution
            
            vertices = []
            indices = []
            
            # Generate vertices
            for v in range(v_res + 1):
                theta = math.pi * v / v_res  # 0 to pi
                for u in range(u_res):
                    phi = 2 * math.pi * u / u_res  # 0 to 2pi
                    
                    x = radius * math.sin(theta) * math.cos(phi)
                    y = radius * math.cos(theta)
                    z = radius * math.sin(theta) * math.sin(phi)
                    
                    vertices.append([x, y, z])
            
            # Generate triangulated indices
            triangles = []
            for v in range(v_res):
                for u in range(u_res):
                    # Current quad vertices
                    v0 = v * u_res + u
                    v1 = v * u_res + (u + 1) % u_res
                    v2 = (v + 1) * u_res + (u + 1) % u_res
                    v3 = (v + 1) * u_res + u
                    
                    # Add quad as two triangles
                    # Use original winding order for sphere
                    triangles.extend([v0, v1, v2])
                    triangles.extend([v0, v2, v3])
            
            # Calculate normals (for sphere, normals are just normalized positions)
            normals = []
            for vertex in vertices:
                length = math.sqrt(vertex[0]**2 + vertex[1]**2 + vertex[2]**2)
                if length > 0:
                    normals.append([vertex[0]/length, vertex[1]/length, vertex[2]/length])
                else:
                    normals.append([0.0, 1.0, 0.0])
            
            # UV mapping
            uvs = []
            for vertex in vertices:
                u = 0.5 + math.atan2(vertex[2], vertex[0]) / (2 * math.pi)
                v = 0.5 - math.asin(vertex[1] / radius) / math.pi
                uvs.append([u, v])
            
            # Convert to numpy arrays
            import numpy as np
            mesh_data = {
                'prim_path': prim_path,
                'vertices': np.array(vertices, dtype=np.float32),
                'indices': np.array(triangles, dtype=np.uint32),
                'normals': np.array(normals, dtype=np.float32),
                'uvs': np.array(uvs, dtype=np.float32),
                'primvars': []  # Procedural geometry has no primvars
            }
            meshes.append(mesh_data)
            
        elif prim_type == 'Cylinder':
            cylinder = UsdGeom.Cylinder(prim)
            prim_path = str(prim.GetPath())
            
            # Get cylinder properties
            radius = 1.0
            height = 2.0
            
            if cylinder.GetRadiusAttr() and cylinder.GetRadiusAttr().HasValue():
                radius = float(cylinder.GetRadiusAttr().Get())
            if cylinder.GetHeightAttr() and cylinder.GetHeightAttr().HasValue():
                height = float(cylinder.GetHeightAttr().Get())
            
            # Generate cylinder geometry
            u_res = 20  # circumference resolution
            v_res = 2   # height segments
            
            vertices = []
            
            # Generate side vertices
            for v in range(v_res + 1):
                y = height * (v / v_res - 0.5)  # -height/2 to height/2
                for u in range(u_res):
                    angle = 2 * math.pi * u / u_res
                    x = radius * math.cos(angle)
                    z = radius * math.sin(angle)
                    vertices.append([x, y, z])
            
            # Add center vertices for caps
            cap_bottom_center = len(vertices)
            vertices.append([0.0, -height/2, 0.0])
            cap_top_center = len(vertices)
            vertices.append([0.0, height/2, 0.0])
            
            # Generate side triangles with correct winding order for coordinate system
            triangles = []
            for v in range(v_res):
                for u in range(u_res):
                    v0 = v * u_res + u
                    v1 = v * u_res + (u + 1) % u_res
                    v2 = (v + 1) * u_res + (u + 1) % u_res
                    v3 = (v + 1) * u_res + u
                    
                    # Always use reversed winding order for procedural primitives
                    triangles.extend([v0, v2, v1])
                    triangles.extend([v0, v3, v2])
            
            # Add bottom cap triangles 
            for u in range(u_res):
                u1 = (u + 1) % u_res
                # Always use reversed winding order for procedural primitives
                triangles.extend([cap_bottom_center, u, u1])
            
            # Add top cap triangles
            top_ring_start = v_res * u_res
            for u in range(u_res):
                u1 = (u + 1) % u_res
                # Always use reversed winding order for procedural primitives
                triangles.extend([cap_top_center, top_ring_start + u1, top_ring_start + u])
            
            # Calculate normals
            normals = []
            for i, vertex in enumerate(vertices):
                if i < len(vertices) - 2:  # Side vertices
                    # For cylinder sides, normal is just normalized XZ position
                    normal = [vertex[0], 0.0, vertex[2]]
                    length = math.sqrt(normal[0]**2 + normal[2]**2)
                    if length > 0:
                        normals.append([normal[0]/length, 0.0, normal[2]/length])
                    else:
                        normals.append([1.0, 0.0, 0.0])
                elif i == cap_bottom_center:  # Bottom cap center
                    normals.append([0.0, -1.0, 0.0])
                else:  # Top cap center
                    normals.append([0.0, 1.0, 0.0])
            
            # UV mapping
            uvs = []
            for i, vertex in enumerate(vertices):
                if i < len(vertices) - 2:  # Side vertices
                    v_idx = i // u_res
                    u_idx = i % u_res
                    u = u_idx / u_res
                    v = v_idx / v_res
                else:  # Cap centers
                    u, v = 0.5, 0.5
                uvs.append([u, v])
            
            # Convert to numpy arrays
            import numpy as np
            mesh_data = {
                'prim_path': prim_path,
                'vertices': np.array(vertices, dtype=np.float32),
                'indices': np.array(triangles, dtype=np.uint32),
                'normals': np.array(normals, dtype=np.float32),
                'uvs': np.array(uvs, dtype=np.float32),
                'primvars': []  # Procedural geometry has no primvars
            }
            meshes.append(mesh_data)
        
        elif prim_type == 'Cone':
            cone = UsdGeom.Cone(prim)
            prim_path = str(prim.GetPath())
            
            # Get cone properties
            radius = 1.0
            height = 2.0
            
            if cone.GetRadiusAttr() and cone.GetRadiusAttr().HasValue():
                radius = float(cone.GetRadiusAttr().Get())
            if cone.GetHeightAttr() and cone.GetHeightAttr().HasValue():
                height = float(cone.GetHeightAttr().Get())
            
            # Generate cone geometry
            u_res = 20  # circumference resolution
            
            vertices = []
            
            # Add apex vertex
            vertices.append([0.0, height/2, 0.0])
            
            # Add base circle vertices
            for u in range(u_res):
                angle = 2 * math.pi * u / u_res
                x = radius * math.cos(angle)
                z = radius * math.sin(angle)
                vertices.append([x, -height/2, z])
            
            # Add base center
            base_center = len(vertices)
            vertices.append([0.0, -height/2, 0.0])
            
            # Generate triangles
            triangles = []
            
            # Side triangles (apex to base)
            for u in range(u_res):
                u1 = (u + 1) % u_res
                # Always use reversed winding order for procedural primitives
                triangles.extend([0, u1 + 1, u + 1])
            
            # Base triangles
            for u in range(u_res):
                u1 = (u + 1) % u_res
                # Always use reversed winding order for procedural primitives
                triangles.extend([base_center, u + 1, u1 + 1])
            
            # Calculate normals
            normals = []
            # Apex normal (average of side face normals)
            normals.append([0.0, 0.8, 0.0])  # Pointing mostly up
            
            # Base circle normals (side faces)
            for u in range(u_res):
                angle = 2 * math.pi * u / u_res
                # Cone side normal points outward and slightly up
                nx = math.cos(angle) * 0.8
                nz = math.sin(angle) * 0.8
                ny = 0.6  # Upward component
                length = math.sqrt(nx**2 + ny**2 + nz**2)
                normals.append([nx/length, ny/length, nz/length])
            
            # Base center normal
            normals.append([0.0, -1.0, 0.0])
            
            # UV mapping
            uvs = []
            # Apex UV
            uvs.append([0.5, 1.0])
            
            # Base circle UVs
            for u in range(u_res):
                angle = 2 * math.pi * u / u_res
                u_coord = 0.5 + 0.4 * math.cos(angle)
                v_coord = 0.5 + 0.4 * math.sin(angle)
                uvs.append([u_coord, 0.0])
            
            # Base center UV
            uvs.append([0.5, 0.0])
            
            # Convert to numpy arrays
            import numpy as np
            mesh_data = {
                'prim_path': prim_path,
                'vertices': np.array(vertices, dtype=np.float32),
                'indices': np.array(triangles, dtype=np.uint32),
                'normals': np.array(normals, dtype=np.float32),
                'uvs': np.array(uvs, dtype=np.float32),
                'primvars': []  # Procedural geometry has no primvars
            }
            meshes.append(mesh_data)
        
        elif prim_type == 'Capsule':
            capsule = UsdGeom.Capsule(prim)
            prim_path = str(prim.GetPath())
            
            # Get capsule properties
            radius = 1.0
            height = 2.0
            
            if capsule.GetRadiusAttr() and capsule.GetRadiusAttr().HasValue():
                radius = float(capsule.GetRadiusAttr().Get())
            if capsule.GetHeightAttr() and capsule.GetHeightAttr().HasValue():
                height = float(capsule.GetHeightAttr().Get())
            
            # Generate capsule geometry from scratch
            u_res = 20  # circumference resolution
            v_res = 16  # vertical resolution for hemispheres
            
            vertices = []
            triangles = []
            
            # Capsule is a cylinder with hemisphere caps
            # Total height includes the hemispheres
            cylinder_height = max(0, height - 2 * radius)
            
            # Generate vertices for the complete capsule
            # Start from bottom pole and work up to top pole
            
            # Bottom pole (south pole)
            bottom_pole_idx = len(vertices)
            vertices.append([0.0, -height/2, 0.0])
            
            # Bottom hemisphere rings (from bottom pole upward)
            for v in range(1, v_res//2):
                theta = math.pi * v / v_res  # angle from south pole
                y = -height/2 + radius * (1 - math.cos(theta))
                
                for u in range(u_res):
                    phi = 2 * math.pi * u / u_res
                    x = radius * math.sin(theta) * math.cos(phi)
                    z = radius * math.sin(theta) * math.sin(phi)
                    vertices.append([x, y, z])
            
            # Cylinder body rings (if any)
            cylinder_start_idx = len(vertices)
            if cylinder_height > 0:
                # Bottom ring of cylinder
                y = -cylinder_height/2
                for u in range(u_res):
                    phi = 2 * math.pi * u / u_res
                    x = radius * math.cos(phi)
                    z = radius * math.sin(phi)
                    vertices.append([x, y, z])
                
                # Top ring of cylinder
                y = cylinder_height/2
                for u in range(u_res):
                    phi = 2 * math.pi * u / u_res
                    x = radius * math.cos(phi)
                    z = radius * math.sin(phi)
                    vertices.append([x, y, z])
            
            # Top hemisphere rings (from equator to top pole)
            for v in range(v_res//2 + 1, v_res):
                theta = math.pi * v / v_res  # angle from south pole
                y = height/2 - radius * (1 + math.cos(theta))
                
                for u in range(u_res):
                    phi = 2 * math.pi * u / u_res
                    x = radius * math.sin(theta) * math.cos(phi)
                    z = radius * math.sin(theta) * math.sin(phi)
                    vertices.append([x, y, z])
            
            # Top pole (north pole)
            top_pole_idx = len(vertices)
            vertices.append([0.0, height/2, 0.0])
            
            # Generate triangles with proper connections
            current_ring_start = 1  # Start after bottom pole
            
            # Bottom hemisphere triangles
            # Connect bottom pole to first ring (keep pole triangles as is)
            for u in range(u_res):
                u1 = (u + 1) % u_res
                triangles.extend([bottom_pole_idx, current_ring_start + u, current_ring_start + u1])
            
            # Connect bottom hemisphere rings
            for ring in range(v_res//2 - 2):
                for u in range(u_res):
                    u1 = (u + 1) % u_res
                    v0 = current_ring_start + ring * u_res + u
                    v1 = current_ring_start + ring * u_res + u1
                    v2 = current_ring_start + (ring + 1) * u_res + u1
                    v3 = current_ring_start + (ring + 1) * u_res + u
                    
                    # Use reversed winding order for hemisphere body
                    triangles.extend([v0, v2, v1])
                    triangles.extend([v0, v3, v2])
            
            # Connect bottom hemisphere to cylinder (if cylinder exists)
            if cylinder_height > 0:
                # Last hemisphere ring to first cylinder ring
                last_hemisphere_ring = current_ring_start + (v_res//2 - 2) * u_res
                cylinder_bottom_ring = cylinder_start_idx
                
                for u in range(u_res):
                    u1 = (u + 1) % u_res
                    v0 = last_hemisphere_ring + u
                    v1 = last_hemisphere_ring + u1
                    v2 = cylinder_bottom_ring + u1
                    v3 = cylinder_bottom_ring + u
                    
                    # Use reversed winding order
                    triangles.extend([v0, v2, v1])
                    triangles.extend([v0, v3, v2])
                
                # Cylinder body triangles
                for u in range(u_res):
                    u1 = (u + 1) % u_res
                    bottom_ring = cylinder_start_idx + u
                    bottom_ring_next = cylinder_start_idx + u1
                    top_ring = cylinder_start_idx + u_res + u
                    top_ring_next = cylinder_start_idx + u_res + u1
                    
                    # Use reversed winding order
                    triangles.extend([bottom_ring, top_ring_next, bottom_ring_next])
                    triangles.extend([bottom_ring, top_ring, top_ring_next])
                
                # Connect cylinder to top hemisphere
                cylinder_top_ring = cylinder_start_idx + u_res
                top_hemisphere_first_ring = cylinder_start_idx + 2 * u_res
                
                for u in range(u_res):
                    u1 = (u + 1) % u_res
                    v0 = cylinder_top_ring + u
                    v1 = cylinder_top_ring + u1
                    v2 = top_hemisphere_first_ring + u1
                    v3 = top_hemisphere_first_ring + u
                    
                    # Use reversed winding order
                    triangles.extend([v0, v2, v1])
                    triangles.extend([v0, v3, v2])
                
                current_ring_start = cylinder_start_idx + 2 * u_res
            else:
                # No cylinder - connect hemispheres directly
                last_bottom_ring = current_ring_start + (v_res//2 - 2) * u_res
                first_top_ring = current_ring_start + (v_res//2 - 1) * u_res
                
                for u in range(u_res):
                    u1 = (u + 1) % u_res
                    v0 = last_bottom_ring + u
                    v1 = last_bottom_ring + u1
                    v2 = first_top_ring + u1
                    v3 = first_top_ring + u
                    
                    # Use reversed winding order
                    triangles.extend([v0, v2, v1])
                    triangles.extend([v0, v3, v2])
                
                current_ring_start = first_top_ring
            
            # Top hemisphere triangles
            # Connect top hemisphere rings
            hemisphere_rings = v_res//2 - 1
            for ring in range(hemisphere_rings - 1):
                for u in range(u_res):
                    u1 = (u + 1) % u_res
                    v0 = current_ring_start + ring * u_res + u
                    v1 = current_ring_start + ring * u_res + u1
                    v2 = current_ring_start + (ring + 1) * u_res + u1
                    v3 = current_ring_start + (ring + 1) * u_res + u
                    
                    # Use reversed winding order for hemisphere body
                    triangles.extend([v0, v2, v1])
                    triangles.extend([v0, v3, v2])
            
            # Connect last ring to top pole (keep pole triangles as is)
            last_ring_start = current_ring_start + (hemisphere_rings - 1) * u_res
            for u in range(u_res):
                u1 = (u + 1) % u_res
                triangles.extend([top_pole_idx, last_ring_start + u1, last_ring_start + u])
            
            # Calculate normals (simplified - use normalized position for sphere-like surfaces)
            normals = []
            for vertex in vertices:
                length = math.sqrt(vertex[0]**2 + vertex[1]**2 + vertex[2]**2)
                if length > 0:
                    normals.append([vertex[0]/length, vertex[1]/length, vertex[2]/length])
                else:
                    normals.append([0.0, 1.0, 0.0])
            
            # UV mapping (simplified)
            uvs = []
            for vertex in vertices:
                u = 0.5 + math.atan2(vertex[2], vertex[0]) / (2 * math.pi)
                v = 0.5 + vertex[1] / height
                uvs.append([u, v])
            
            # Convert to numpy arrays
            import numpy as np
            mesh_data = {
                'prim_path': prim_path,
                'vertices': np.array(vertices, dtype=np.float32),
                'indices': np.array(triangles, dtype=np.uint32),
                'normals': np.array(normals, dtype=np.float32),
                'uvs': np.array(uvs, dtype=np.float32),
                'primvars': []  # Procedural geometry has no primvars
            }
            meshes.append(mesh_data)
        
        elif prim_type == 'Plane':
            plane = UsdGeom.Plane(prim)
            prim_path = str(prim.GetPath())
            
            # Get plane properties
            width = 2.0
            length = 2.0
            
            if plane.GetWidthAttr() and plane.GetWidthAttr().HasValue():
                width = float(plane.GetWidthAttr().Get())
            if plane.GetLengthAttr() and plane.GetLengthAttr().HasValue():
                length = float(plane.GetLengthAttr().Get())
            
            # Generate plane vertices (simple quad)
            half_width = width / 2.0
            half_length = length / 2.0
            
            vertices = [
                [-half_width, 0.0, -half_length],
                [half_width, 0.0, -half_length],
                [half_width, 0.0, half_length],
                [-half_width, 0.0, half_length]
            ]
            
            # Two triangles for the quad
            # Always use reversed winding order for procedural primitives
            triangles = [0, 2, 1, 0, 3, 2]
            
            # Normals (all pointing up)
            normals = [
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0]
            ]
            
            # UV coordinates
            uvs = [
                [0.0, 0.0],
                [1.0, 0.0],
                [1.0, 1.0],
                [0.0, 1.0]
            ]
            
            # Convert to numpy arrays
            import numpy as np
            mesh_data = {
                'prim_path': prim_path,
                'vertices': np.array(vertices, dtype=np.float32),
                'indices': np.array(triangles, dtype=np.uint32),
                'normals': np.array(normals, dtype=np.float32),
                'uvs': np.array(uvs, dtype=np.float32),
                'primvars': []  # Procedural geometry has no primvars
            }
            meshes.append(mesh_data)
    
    return {'meshes': meshes, 'up_axis': up_axis}", None, None)
                    .map_err(|e| format!("Failed to define extract function: {}", e))?;
                
                let locals = PyDict::new(py);
                locals.set_item("stage_path", file_path)
                    .map_err(|e| format!("Failed to set stage_path: {}", e))?;
                
                let result = py.eval(c"extract_all_meshes(stage_path)", None, Some(&locals))
                    .map_err(|e| format!("Failed to extract meshes: {}", e))?;
                
                // Convert Python result to Rust data
                let mut scene_data = USDSceneData {
                    stage_path: file_path.to_string(),
                    meshes: Vec::new(),
                    lights: Vec::new(),
                    materials: Vec::new(),
                    up_axis: "Z".to_string(), // Default to Z-up
                };
                
                // Extract the dictionary with meshes and up_axis
                if let Ok(result_dict) = result.extract::<HashMap<String, pyo3::PyObject>>() {
                    // Extract up axis
                    if let Some(up_axis_obj) = result_dict.get("up_axis") {
                        if let Ok(up_axis) = up_axis_obj.extract::<String>(py) {
                            scene_data.up_axis = up_axis;
                        }
                    }
                    
                    // Extract meshes
                    if let Some(meshes_obj) = result_dict.get("meshes") {
                        if let Ok(mesh_list) = meshes_obj.extract::<Vec<HashMap<String, pyo3::PyObject>>>(py) {
                    for mesh_dict in mesh_list {
                        if let (Ok(prim_path), Ok(vertices_array), Ok(indices_array), Ok(normals_array), Ok(uvs_array)) = (
                            mesh_dict.get("prim_path").unwrap().extract::<String>(py),
                            mesh_dict.get("vertices").unwrap().downcast_bound::<PyArray2<f32>>(py),
                            mesh_dict.get("indices").unwrap().downcast_bound::<PyArray1<u32>>(py),
                            mesh_dict.get("normals").unwrap().downcast_bound::<PyArray2<f32>>(py),
                            mesh_dict.get("uvs").unwrap().downcast_bound::<PyArray2<f32>>(py),
                        ) {
                            // Extract vertices efficiently from numpy array
                            let vertices_readonly = vertices_array.readonly();
                            let vertices_slice = vertices_readonly.as_slice().unwrap();
                            let vertices: Vec<Vec3> = vertices_slice.chunks_exact(3)
                                .map(|chunk| Vec3::new(chunk[0], chunk[1], chunk[2]))
                                .collect();
                            
                            // Extract indices efficiently from numpy array  
                            let indices_readonly = indices_array.readonly();
                            let indices: Vec<u32> = indices_readonly.as_slice().unwrap().to_vec();
                            
                            // Extract normals efficiently from numpy array
                            let normals_readonly = normals_array.readonly();
                            let normals_slice = normals_readonly.as_slice().unwrap();
                            let normals: Vec<Vec3> = normals_slice.chunks_exact(3)
                                .map(|chunk| Vec3::new(chunk[0], chunk[1], chunk[2]))
                                .collect();
                            
                            // Extract UVs efficiently from numpy array
                            let uvs_readonly = uvs_array.readonly();
                            let uvs_slice = uvs_readonly.as_slice().unwrap();
                            let uvs: Vec<Vec2> = uvs_slice.chunks_exact(2)
                                .map(|chunk| Vec2::new(chunk[0], chunk[1]))
                                .collect();
                            
                            // Extract vertex colors if available
                            let vertex_colors = if let Some(colors_obj) = mesh_dict.get("vertex_colors") {
                                if let Ok(colors_array) = colors_obj.downcast_bound::<PyArray2<f32>>(py) {
                                    let colors_readonly = colors_array.readonly();
                                    let colors_slice = colors_readonly.as_slice().unwrap();
                                    let colors: Vec<Vec3> = colors_slice.chunks_exact(3)
                                        .map(|chunk| Vec3::new(chunk[0], chunk[1], chunk[2]))
                                        .collect();
                                    println!("🎨 Found {} vertex colors for mesh {}", colors.len(), prim_path);
                                    Some(colors)
                                } else {
                                    println!("🎨 Vertex colors found but failed to extract for mesh {}", prim_path);
                                    None
                                }
                            } else {
                                None
                            };
                            
                            // Extract primvars if available
                            let mut primvars = Vec::new();
                            if let Some(primvars_obj) = mesh_dict.get("primvars") {
                                if let Ok(primvar_list) = primvars_obj.extract::<Vec<HashMap<String, pyo3::PyObject>>>(py) {
                                    for primvar_dict in primvar_list {
                                        if let (Ok(name), Ok(interpolation), Ok(data_type)) = (
                                            primvar_dict.get("name").unwrap().extract::<String>(py),
                                            primvar_dict.get("interpolation").unwrap().extract::<String>(py),
                                            primvar_dict.get("data_type").unwrap().extract::<String>(py),
                                        ) {
                                            let values = match data_type.as_str() {
                                                "float3" => {
                                                    if let Ok(values_list) = primvar_dict.get("values").unwrap().extract::<Vec<Vec<f32>>>(py) {
                                                        PrimvarValues::Float3(values_list.into_iter()
                                                            .map(|v| Vec3::new(v[0], v[1], v[2]))
                                                            .collect())
                                                    } else {
                                                        continue;
                                                    }
                                                },
                                                "float2" => {
                                                    if let Ok(values_list) = primvar_dict.get("values").unwrap().extract::<Vec<Vec<f32>>>(py) {
                                                        PrimvarValues::Float2(values_list.into_iter()
                                                            .map(|v| Vec2::new(v[0], v[1]))
                                                            .collect())
                                                    } else {
                                                        continue;
                                                    }
                                                },
                                                "float" => {
                                                    if let Ok(values_list) = primvar_dict.get("values").unwrap().extract::<Vec<f32>>(py) {
                                                        PrimvarValues::Float(values_list)
                                                    } else {
                                                        continue;
                                                    }
                                                },
                                                "int" => {
                                                    if let Ok(values_list) = primvar_dict.get("values").unwrap().extract::<Vec<i32>>(py) {
                                                        PrimvarValues::Int(values_list)
                                                    } else {
                                                        continue;
                                                    }
                                                },
                                                "string" => {
                                                    if let Ok(values_list) = primvar_dict.get("values").unwrap().extract::<Vec<String>>(py) {
                                                        PrimvarValues::String(values_list)
                                                    } else {
                                                        continue;
                                                    }
                                                },
                                                _ => continue,
                                            };
                                            
                                            let indices = if let Some(indices_obj) = primvar_dict.get("indices") {
                                                indices_obj.extract::<Vec<u32>>(py).ok()
                                            } else {
                                                None
                                            };
                                            
                                            primvars.push(USDPrimvar {
                                                name,
                                                interpolation,
                                                data_type,
                                                values,
                                                indices,
                                            });
                                        }
                                    }
                                }
                            }
                            
                            // Extract attributes if available
                            let mut attributes = Vec::new();
                            if let Some(attributes_obj) = mesh_dict.get("attributes") {
                                if let Ok(attr_list) = attributes_obj.extract::<Vec<HashMap<String, pyo3::PyObject>>>(py) {
                                    for attr_dict in attr_list {
                                        if let (Ok(name), Ok(value_type), Ok(is_custom)) = (
                                            attr_dict.get("name").unwrap().extract::<String>(py),
                                            attr_dict.get("value_type").unwrap().extract::<String>(py),
                                            attr_dict.get("is_custom").unwrap().extract::<bool>(py),
                                        ) {
                                            if let Some(value_obj) = attr_dict.get("value") {
                                                // Extract the attribute value based on its structure
                                                let value = Self::extract_attribute_value(value_obj, py);
                                                
                                                let metadata = if let Some(metadata_obj) = attr_dict.get("metadata") {
                                                    metadata_obj.extract::<std::collections::HashMap<String, String>>(py).unwrap_or_default()
                                                } else {
                                                    std::collections::HashMap::new()
                                                };
                                                
                                                attributes.push(USDAttribute {
                                                    name,
                                                    value_type,
                                                    value,
                                                    is_custom,
                                                    metadata,
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                            
                            let mesh_geom = USDMeshGeometry {
                                prim_path,
                                vertices,
                                indices,
                                normals,
                                uvs,
                                vertex_colors,
                                transform: Mat4::IDENTITY,
                                primvars,
                                attributes,
                            };
                            scene_data.meshes.push(mesh_geom);
                        }
                    }
                        }
                    }
                }
                
                if scene_data.meshes.is_empty() {
                    return Err("💥 FATAL: No mesh prims found in USD stage! Cannot render empty scene.".to_string());
                }
                
                // Store stage reference
                let identifier = format!("loaded_{}", self.stages.len());
                let stage_obj = USDStage {
                    path: file_path.to_string(),
                    identifier: identifier.clone(),
                };
                self.stages.insert(identifier, stage_obj);
                
                // Add default light
                let default_light = USDLightData {
                    prim_path: "/Kitchen_set/DefaultLight".to_string(),
                    light_type: "DistantLight".to_string(),
                    intensity: 1.0,
                    color: Vec3::new(1.0, 1.0, 0.9),
                    transform: Mat4::from_rotation_x(-30_f32.to_radians()),
                };
                scene_data.lights.push(default_light);
                
                println!("✅ USDEngine: Extracted {} meshes from USD stage in {:?}", 
                         scene_data.meshes.len(), start_time.elapsed());
                
                Ok(scene_data)
            })?;
            
            Ok(scene_data)
        }
        
        #[cfg(not(feature = "usd"))]
        {
            println!("Mock: Loading USD stage from {}", file_path);
            
            // Create mock scene data
            let mut scene_data = USDSceneData {
                stage_path: file_path.to_string(),
                meshes: Vec::new(),
                lights: Vec::new(),
                materials: Vec::new(),
                up_axis: "Z".to_string(), // Mock data uses Z-up
            };
            
            // Add mock geometry
            scene_data.meshes.push(USDMeshGeometry {
                prim_path: "/World/MockCube".to_string(),
                vertices: vec![
                    Vec3::new(-1.0, -1.0, -1.0),
                    Vec3::new(1.0, -1.0, -1.0),
                    Vec3::new(1.0, 1.0, -1.0),
                    Vec3::new(-1.0, 1.0, -1.0),
                ],
                indices: vec![0, 1, 2, 0, 2, 3],
                normals: vec![
                    Vec3::new(0.0, 0.0, -1.0),
                    Vec3::new(0.0, 0.0, -1.0),
                    Vec3::new(0.0, 0.0, -1.0),
                    Vec3::new(0.0, 0.0, -1.0),
                ],
                uvs: vec![
                    Vec2::new(0.0, 0.0),
                    Vec2::new(1.0, 0.0),
                    Vec2::new(1.0, 1.0),
                    Vec2::new(0.0, 1.0),
                ],
                vertex_colors: Some(vec![
                    Vec3::new(1.0, 0.0, 0.0),  // Red
                    Vec3::new(0.0, 1.0, 0.0),  // Green
                    Vec3::new(0.0, 0.0, 1.0),  // Blue
                    Vec3::new(1.0, 1.0, 0.0),  // Yellow
                ]),
                transform: Mat4::IDENTITY,
                primvars: vec![],  // Mock data has no primvars
                attributes: vec![], // Mock data has no attributes
            });
            
            Ok(scene_data)
        }
    }
    
    #[cfg(feature = "usd")]
    fn extract_meshes_recursive(
        &self,
        _py: Python,
        _stage: &PyAny,
        _prim: &PyAny,
        _usd_geom: &PyAny,
        scene_data: &mut USDSceneData,
    ) -> Result<(), String> {
        // For now, create a simple mock mesh since PyO3 USD integration is complex
        let prim_path = format!("/MockMesh_{}", scene_data.meshes.len());
        
        let mock_mesh = USDMeshGeometry {
            prim_path,
            vertices: vec![
                Vec3::new(-1.0, -1.0, 0.0),
                Vec3::new(1.0, -1.0, 0.0),
                Vec3::new(1.0, 1.0, 0.0),
                Vec3::new(-1.0, 1.0, 0.0),
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
            normals: vec![
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(0.0, 0.0, 1.0),
            ],
            uvs: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 1.0),
            ],
            vertex_colors: None,
            transform: Mat4::IDENTITY,
            primvars: vec![],
            attributes: vec![],
        };
        
        scene_data.meshes.push(mock_mesh);
        
        Ok(())
    }
    
    
    
    #[cfg(feature = "usd")]
    fn extract_lights_recursive(
        &self,
        _py: Python,
        _stage: &PyAny,
        _prim: &PyAny,
        _usd_lux: &PyAny,
        scene_data: &mut USDSceneData,
    ) -> Result<(), String> {
        // Add a simple mock light
        let mock_light = USDLightData {
            prim_path: "/MockLight".to_string(),
            light_type: "DistantLight".to_string(),
            intensity: 1.0,
            color: Vec3::new(1.0, 1.0, 0.9),
            transform: Mat4::from_rotation_x(-45_f32.to_radians()),
        };
        
        scene_data.lights.push(mock_light);
        
        Ok(())
    }
    
}