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
    
    
    /// Load a USD stage from file and extract scene data
    pub fn load_stage(&mut self, file_path: &str) -> Result<USDSceneData, String> {
        println!("🎬 🎬 🎬 USDEngine: REAL USD LOADING CALLED for {}", file_path);
        
        #[cfg(feature = "usd")]
        {
            let start_time = std::time::Instant::now();
            
            // Process all meshes in Python and return packed data
            let scene_data = Python::with_gil(|py| -> Result<USDSceneData, String> {
                println!("🎬 USDEngine: Opening USD stage with Python...");
                
                // Create Python script for batched processing
                let extract_script = r#"
def extract_all_meshes(stage_path):
    import numpy as np
    from pxr import Usd, UsdGeom
    
    # Open stage once
    stage = Usd.Stage.Open(stage_path)
    if not stage:
        return None
    
    meshes = []
    
    # Traverse once and collect all mesh data
    for prim in stage.Traverse():
        if prim.GetTypeName() == 'Mesh':
            mesh = UsdGeom.Mesh(prim)
            prim_path = str(prim.GetPath())
            
            # Get all attributes at once
            points_attr = mesh.GetPointsAttr()
            indices_attr = mesh.GetFaceVertexIndicesAttr() 
            counts_attr = mesh.GetFaceVertexCountsAttr()
            
            if not (points_attr and indices_attr and counts_attr):
                continue
                
            # Extract raw data (USD native types)
            points = points_attr.Get()  # Gf.Vec3fArray
            face_indices = indices_attr.Get()  # IntArray
            face_counts = counts_attr.Get()  # IntArray
            
            if not (points and face_indices and face_counts):
                continue
            
            # OPTIMIZED: Convert USD types to Python lists first (fast)
            # This avoids slow per-element USD type -> NumPy conversion
            vertices_list = [(float(v[0]), float(v[1]), float(v[2])) for v in points]
            indices_list = [int(i) for i in face_indices]
            counts_list = [int(c) for c in face_counts]
            
            # Only call NumPy once at the end (fast)
            vertices = np.array(vertices_list, dtype=np.float32)
            indices = np.array(indices_list, dtype=np.uint32)
            counts = np.array(counts_list, dtype=np.uint32)
            
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
            
            triangulated_indices = np.array(triangles, dtype=np.uint32)
            
            # Fast normal calculation
            normals = np.zeros_like(vertices)
            normal_counts = np.zeros(len(vertices), dtype=np.uint32)
            
            # Vectorized normal calculation
            for i in range(0, len(triangulated_indices), 3):
                if i + 2 < len(triangulated_indices):
                    i0, i1, i2 = triangulated_indices[i:i+3]
                    if i0 < len(vertices) and i1 < len(vertices) and i2 < len(vertices):
                        v0, v1, v2 = vertices[i0], vertices[i1], vertices[i2]
                        edge1 = v1 - v0
                        edge2 = v2 - v0
                        normal = np.cross(edge1, edge2)
                        normal_len = np.linalg.norm(normal)
                        if normal_len > 0:
                            normal = normal / normal_len
                            normals[i0] += normal
                            normals[i1] += normal
                            normals[i2] += normal
                            normal_counts[i0] += 1
                            normal_counts[i1] += 1
                            normal_counts[i2] += 1
            
            # Normalize accumulated normals
            for i in range(len(normals)):
                if normal_counts[i] > 0:
                    normals[i] = normals[i] / normal_counts[i]
                    norm_len = np.linalg.norm(normals[i])
                    if norm_len > 0:
                        normals[i] = normals[i] / norm_len
                else:
                    normals[i] = [0, 1, 0]  # Default up
            
            # Simple UV mapping
            uvs = np.zeros((len(vertices), 2), dtype=np.float32)
            uvs[:, 0] = (vertices[:, 0] + 1.0) * 0.5  # X -> U
            uvs[:, 1] = (vertices[:, 2] + 1.0) * 0.5  # Z -> V
            
            # Pack mesh data
            mesh_data = {
                'prim_path': prim_path,
                'vertices': vertices.tolist(),
                'indices': triangulated_indices.tolist(),
                'normals': normals.tolist(),
                'uvs': uvs.tolist()
            }
            
            meshes.append(mesh_data)
    
    return meshes
"#;
                
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
        if prim.GetTypeName() == 'Mesh':
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
                'uvs': uvs_array
            }
            
            # Add vertex colors if available
            if vertex_colors:
                vertex_colors_array = np.array(vertex_colors, dtype=np.float32)
                mesh_data['vertex_colors'] = vertex_colors_array
            
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
                                    Some(colors)
                                } else {
                                    None
                                }
                            } else {
                                None
                            };
                            
                            let mesh_geom = USDMeshGeometry {
                                prim_path,
                                vertices,
                                indices,
                                normals,
                                uvs,
                                vertex_colors,
                                transform: Mat4::IDENTITY,
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