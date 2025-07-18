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
                'uvs': np.array(uvs, dtype=np.float32)
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
                'uvs': np.array(uvs, dtype=np.float32)
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
                'uvs': np.array(uvs, dtype=np.float32)
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
                'uvs': np.array(uvs, dtype=np.float32)
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
                'uvs': np.array(uvs, dtype=np.float32)
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
                'uvs': np.array(uvs, dtype=np.float32)
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