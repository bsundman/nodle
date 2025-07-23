//! Attributes node logic implementation
//! 
//! High-performance attribute processing with caching, string interning,
//! and incremental updates for smooth UI performance.

use crate::nodes::interface::NodeData;
use crate::nodes::NodeId;
use crate::workspaces::three_d::usd::usd_engine::{USDMeshGeometry, USDSceneData, USDPrimvar, PrimvarValues};
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
    use std::sync::Once;
    use std::sync::MutexGuard;
    use std::sync::atomic::{AtomicUsize, Ordering};
    // Removed duplicate Instant import
    use tracing::{info, warn, error};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;
use glam::{Vec2, Vec3, Mat4};

// Adaptive throttling constants
const ADAPTIVE_CHECK_BASE: u64 = 30;
const ADAPTIVE_CHECK_MIN: u64 = 5;
const ADAPTIVE_CHECK_MAX: u64 = 120;

/// Global cache for attribute data with RwLock for performance
pub static ATTRIBUTES_INPUT_CACHE: Lazy<RwLock<HashMap<NodeId, (NodeData, std::sync::atomic::AtomicU64)>>> = 
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Global string interner for attribute labels
    /// Global performance metrics for attribute operations
    thread_local!(static ATTRIBUTE_METRICS: RefCell<AttributeMetrics> = RefCell::new(AttributeMetrics {
        cache_hit_rate: 0.0,
        avg_lookup_time: Duration::from_nanos(0),
        memory_usage: 0,
        cache_updates: 0,
        cache_misses: 0,
        cache_hits: 0,
        partial_updates: 0,
        last_updated: Instant::now(),
        extraction_time: Duration::from_nanos(0),
        mesh_count: 0,
        attribute_count: 0,
    }));
static STRING_INTERNER: Lazy<Arc<Mutex<StringInterner>>> = Lazy::new(|| {
    Arc::new(Mutex::new(StringInterner::default()))
});

    /// Performance metrics for attribute operations
    #[derive(Debug, Clone)]
    pub struct AttributeMetrics {
        pub cache_hit_rate: f64,
        pub avg_lookup_time: Duration,
        pub memory_usage: usize,
        pub cache_updates: u64,
        pub cache_misses: u64,
        pub cache_hits: u64,
        pub partial_updates: u64,
        pub last_updated: Instant,
        pub extraction_time: Duration, // New metric for tracking USD data extraction time
        pub mesh_count: usize,         // Track number of meshes processed
        pub attribute_count: usize,     // Total attributes extracted
    }
/// String interner for reducing memory allocations
#[derive(Default)]
struct StringInterner {
    strings: HashMap<String, Arc<str>>,
}

impl StringInterner {
    fn intern(&mut self, s: &str) -> Arc<str> {
        if let Some(interned) = self.strings.get(s) {
            interned.clone()
        } else {
            let interned: Arc<str> = s.into();
            self.strings.insert(s.to_string(), interned.clone());
            interned
        }
    }
}

/// USD attribute interpolation types
#[derive(Debug, Clone, PartialEq)]
pub enum InterpolationType {
    Constant,    // Detail attribute - one value per primitive
    Uniform,     // Primitive attribute - one value per face
    Vertex,      // Point attribute - one value per vertex
    FaceVarying, // Vertex attribute - one value per face-vertex
    Varying,     // NURBS varying - varies
}

impl std::fmt::Display for InterpolationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpolationType::Constant => write!(f, "constant"),
            InterpolationType::Uniform => write!(f, "uniform"),
            InterpolationType::Vertex => write!(f, "vertex"),
            InterpolationType::FaceVarying => write!(f, "faceVarying"),
            InterpolationType::Varying => write!(f, "varying"),
        }
    }
}

/// USD attribute value wrapper
#[derive(Debug, Clone)]
pub enum AttributeValue {
    // Scalar types
    Bool(bool),
    Int(i32),
    Float(f32),
    Double(f64),
    String(String),
    Token(String),
    
    // Vector types
    Float2(Vec2),
    Float3(Vec3),
    Color3f(Vec3),
    Normal3f(Vec3),
    Point3f(Vec3),
    Vector3f(Vec3),
    TexCoord2f(Vec2),
    
    // Matrix types
    Matrix4d(Mat4),
    
    // Array types
    BoolArray(Vec<bool>),
    IntArray(Vec<i32>),
    FloatArray(Vec<f32>),
    DoubleArray(Vec<f64>),
    StringArray(Vec<String>),
    TokenArray(Vec<String>),
    Float2Array(Vec<Vec2>),
    Float3Array(Vec<Vec3>),
    Color3fArray(Vec<Vec3>),
    Normal3fArray(Vec<Vec3>),
    Point3fArray(Vec<Vec3>),
    Vector3fArray(Vec<Vec3>),
    TexCoord2fArray(Vec<Vec2>),
}

impl AttributeValue {
    /// Get the display type name for this value
    pub fn type_name(&self) -> &'static str {
        match self {
            AttributeValue::Bool(_) => "bool",
            AttributeValue::Int(_) => "int",
            AttributeValue::Float(_) => "float",
            AttributeValue::Double(_) => "double",
            AttributeValue::String(_) => "string",
            AttributeValue::Token(_) => "token",
            AttributeValue::Float2(_) => "float2",
            AttributeValue::Float3(_) => "float3",
            AttributeValue::Color3f(_) => "color3f",
            AttributeValue::Normal3f(_) => "normal3f",
            AttributeValue::Point3f(_) => "point3f",
            AttributeValue::Vector3f(_) => "vector3f",
            AttributeValue::TexCoord2f(_) => "texCoord2f",
            AttributeValue::Matrix4d(_) => "matrix4d",
            AttributeValue::BoolArray(_) => "bool[]",
            AttributeValue::IntArray(_) => "int[]",
            AttributeValue::FloatArray(_) => "float[]",
            AttributeValue::DoubleArray(_) => "double[]",
            AttributeValue::StringArray(_) => "string[]",
            AttributeValue::TokenArray(_) => "token[]",
            AttributeValue::Float2Array(_) => "float2[]",
            AttributeValue::Float3Array(_) => "float3[]",
            AttributeValue::Color3fArray(_) => "color3f[]",
            AttributeValue::Normal3fArray(_) => "normal3f[]",
            AttributeValue::Point3fArray(_) => "point3f[]",
            AttributeValue::Vector3fArray(_) => "vector3f[]",
            AttributeValue::TexCoord2fArray(_) => "texCoord2f[]",
        }
    }
    
    /// Get the element count for array types
    pub fn element_count(&self) -> usize {
        match self {
            AttributeValue::BoolArray(arr) => arr.len(),
            AttributeValue::IntArray(arr) => arr.len(),
            AttributeValue::FloatArray(arr) => arr.len(),
            AttributeValue::DoubleArray(arr) => arr.len(),
            AttributeValue::StringArray(arr) => arr.len(),
            AttributeValue::TokenArray(arr) => arr.len(),
            AttributeValue::Float2Array(arr) => arr.len(),
            AttributeValue::Float3Array(arr) => arr.len(),
            AttributeValue::Color3fArray(arr) => arr.len(),
            AttributeValue::Normal3fArray(arr) => arr.len(),
            AttributeValue::Point3fArray(arr) => arr.len(),
            AttributeValue::Vector3fArray(arr) => arr.len(),
            AttributeValue::TexCoord2fArray(arr) => arr.len(),
            _ => 1, // Scalar values have count of 1
        }
    }
    
    /// Get a short preview of the value for display
    pub fn preview(&self) -> String {
        match self {
            AttributeValue::Bool(v) => v.to_string(),
            AttributeValue::Int(v) => v.to_string(),
            AttributeValue::Float(v) => format!("{:.3}", v),
            AttributeValue::Double(v) => format!("{:.3}", v),
            AttributeValue::String(v) => format!("\"{}\"", v),
            AttributeValue::Token(v) => v.clone(),
            AttributeValue::Float2(v) => format!("({:.3}, {:.3})", v.x, v.y),
            AttributeValue::Float3(v) => format!("({:.3}, {:.3}, {:.3})", v.x, v.y, v.z),
            AttributeValue::Color3f(v) => format!("({:.3}, {:.3}, {:.3})", v.x, v.y, v.z),
            AttributeValue::Normal3f(v) => format!("({:.3}, {:.3}, {:.3})", v.x, v.y, v.z),
            AttributeValue::Point3f(v) => format!("({:.3}, {:.3}, {:.3})", v.x, v.y, v.z),
            AttributeValue::Vector3f(v) => format!("({:.3}, {:.3}, {:.3})", v.x, v.y, v.z),
            AttributeValue::TexCoord2f(v) => format!("({:.3}, {:.3})", v.x, v.y),
            AttributeValue::Matrix4d(m) => format!("matrix4d [{}...]", m.col(0).x),
            // For arrays, show count and first element
            AttributeValue::BoolArray(arr) => {
                if arr.is_empty() { "[]".to_string() } 
                else { format!("[{}] ({})", arr.len(), arr[0]) }
            },
            AttributeValue::IntArray(arr) => {
                if arr.is_empty() { "[]".to_string() } 
                else { format!("[{}] ({})", arr.len(), arr[0]) }
            },
            AttributeValue::FloatArray(arr) => {
                if arr.is_empty() { "[]".to_string() } 
                else { format!("[{}] ({:.3})", arr.len(), arr[0]) }
            },
            AttributeValue::DoubleArray(arr) => {
                if arr.is_empty() { "[]".to_string() } 
                else { format!("[{}] ({:.3})", arr.len(), arr[0]) }
            },
            AttributeValue::StringArray(arr) => {
                if arr.is_empty() { "[]".to_string() } 
                else { format!("[{}] (\"{}\")", arr.len(), arr[0]) }
            },
            AttributeValue::TokenArray(arr) => {
                if arr.is_empty() { "[]".to_string() } 
                else { format!("[{}] ({})", arr.len(), arr[0]) }
            },
            AttributeValue::Float2Array(arr) => {
                if arr.is_empty() { "[]".to_string() } 
                else { format!("[{}] ({:.3}, {:.3})", arr.len(), arr[0].x, arr[0].y) }
            },
            AttributeValue::Float3Array(arr) => {
                if arr.is_empty() { "[]".to_string() } 
                else { format!("[{}] ({:.3}, {:.3}, {:.3})", arr.len(), arr[0].x, arr[0].y, arr[0].z) }
            },
            AttributeValue::Color3fArray(arr) => {
                if arr.is_empty() { "[]".to_string() } 
                else { format!("[{}] ({:.3}, {:.3}, {:.3})", arr.len(), arr[0].x, arr[0].y, arr[0].z) }
            },
            AttributeValue::Normal3fArray(arr) => {
                if arr.is_empty() { "[]".to_string() } 
                else { format!("[{}] ({:.3}, {:.3}, {:.3})", arr.len(), arr[0].x, arr[0].y, arr[0].z) }
            },
            AttributeValue::Point3fArray(arr) => {
                if arr.is_empty() { "[]".to_string() } 
                else { format!("[{}] ({:.3}, {:.3}, {:.3})", arr.len(), arr[0].x, arr[0].y, arr[0].z) }
            },
            AttributeValue::Vector3fArray(arr) => {
                if arr.is_empty() { "[]".to_string() } 
                else { format!("[{}] ({:.3}, {:.3}, {:.3})", arr.len(), arr[0].x, arr[0].y, arr[0].z) }
            },
            AttributeValue::TexCoord2fArray(arr) => {
                if arr.is_empty() { "[]".to_string() } 
                else { format!("[{}] ({:.3}, {:.3})", arr.len(), arr[0].x, arr[0].y) }
            },
        }
    }
}

/// USD attribute definition
#[derive(Debug, Clone)]
pub struct USDAttribute {
    pub name: Arc<str>,
    pub full_name: Arc<str>, // e.g., "primvars:displayColor"
    pub value: AttributeValue,
    pub interpolation: Option<InterpolationType>,
    pub is_primvar: bool,
    pub is_indexed: bool,
    pub indices: Option<Vec<u32>>,
    // Pre-computed display strings
    pub type_display: Arc<str>,
    pub interpolation_display: Arc<str>,
    pub count_display: Arc<str>,
    pub preview_display: Arc<str>,
}

/// USD primitive with its attributes
#[derive(Debug, Clone)]
pub struct USDPrimitive {
    pub path: Arc<str>,
    pub prim_type: Arc<str>, // "Mesh", "Sphere", etc.
    pub attributes: Vec<USDAttribute>,
    // Performance optimizations
    pub attribute_count: usize,
    pub primvar_count: usize,
    pub geometry_count: usize,
}

/// Tracks changes for incremental updates
#[derive(Clone)]
struct AttributeChangeTracker {
    prim_hash: u64,
    attribute_hashes: Vec<u64>,
    last_update: Instant,
}

impl Default for AttributeChangeTracker {
    fn default() -> Self {
        Self {
            prim_hash: 0,
            attribute_hashes: Vec::new(),
            last_update: Instant::now(),
        }
    }
}


/// Extract attributes from USD mesh geometry with memory limits to prevent freezing
pub fn extract_attributes_from_mesh(mesh: &USDMeshGeometry) -> Vec<USDAttribute> {
    let mut attributes = Vec::new();
    
    // Extract ALL vertices and indices - no sampling limits
    
    // Get string interner lock
    if let Ok(mut interner) = STRING_INTERNER.lock() {
        // Geometric attributes (non-primvar)
        
        // Points - Use all vertices
        let sampled_vertices = mesh.vertices.clone();
        
        attributes.push(USDAttribute {
            name: interner.intern("points"),
            full_name: interner.intern("points"),
            value: AttributeValue::Point3fArray(sampled_vertices),
            interpolation: None,
            is_primvar: false,
            is_indexed: false,
            indices: None,
            type_display: interner.intern("point3f[]"),
            interpolation_display: interner.intern("N/A"),
            count_display: interner.intern(&mesh.vertices.len().to_string()),
            preview_display: interner.intern(&format!("[{}] vertex positions", mesh.vertices.len())),
        });
        
        // Face vertex indices - Use all indices
        let sampled_indices: Vec<i32> = mesh.indices.iter().map(|&i| i as i32).collect();
        
        attributes.push(USDAttribute {
            name: interner.intern("faceVertexIndices"),
            full_name: interner.intern("faceVertexIndices"),
            value: AttributeValue::IntArray(sampled_indices),
            interpolation: None,
            is_primvar: false,
            is_indexed: false,
            indices: None,
            type_display: interner.intern("int[]"),
            interpolation_display: interner.intern("N/A"),
            count_display: interner.intern(&mesh.indices.len().to_string()),
            preview_display: interner.intern(&format!("[{}] face connectivity", mesh.indices.len())),
        });
        
        // Extract primvars from USD data with their actual interpolation types
        for primvar in &mesh.primvars {
            let interpolation = match primvar.interpolation.as_str() {
                "vertex" => Some(InterpolationType::Vertex),
                "faceVarying" => Some(InterpolationType::FaceVarying),
                "uniform" => Some(InterpolationType::Uniform),
                "constant" => Some(InterpolationType::Constant),
                _ => None,
            };
            
            // Convert primvar values to attribute values with sampling
            let (value, count, preview) = match &primvar.values {
                PrimvarValues::Float3(values) => {
                    let count = values.len();
                    
                    // Use appropriate attribute type based on primvar name
                    let attr_value = if primvar.name.contains("normal") {
                        AttributeValue::Normal3fArray(values.clone())
                    } else if primvar.name.contains("color") || primvar.name == "displayColor" {
                        AttributeValue::Color3fArray(values.clone())
                    } else {
                        AttributeValue::Point3fArray(values.clone())
                    };
                    
                    (
                        attr_value,
                        count.to_string(),
                        format!("[{}] {}", count, primvar.name)
                    )
                },
                PrimvarValues::Float2(values) => {
                    let count = values.len();
                    (
                        AttributeValue::TexCoord2fArray(values.clone()),
                        count.to_string(),
                        format!("[{}] {}", count, primvar.name)
                    )
                },
                PrimvarValues::Float(values) => {
                    let count = values.len();
                    (
                        AttributeValue::FloatArray(values.clone()),
                        count.to_string(),
                        format!("[{}] {}", count, primvar.name)
                    )
                },
                PrimvarValues::Int(values) => {
                    let count = values.len();
                    (
                        AttributeValue::IntArray(values.clone()),
                        count.to_string(),
                        format!("[{}] {}", count, primvar.name)
                    )
                },
                PrimvarValues::String(values) => {
                    let preview = if values.is_empty() {
                        "empty".to_string()
                    } else if values.len() == 1 {
                        values[0].clone()
                    } else {
                        format!("[{}] {}", values.len(), primvar.name)
                    };
                    (
                        AttributeValue::String(values.join(", ")),
                        values.len().to_string(),
                        preview
                    )
                },
            };
            
            // Determine display type based on primvar data type
            let type_display = match &primvar.data_type.as_str() {
                &"float3" => {
                    if primvar.name.contains("normal") {
                        "normal3f[]"
                    } else if primvar.name.contains("color") {
                        "color3f[]"
                    } else {
                        "point3f[]"
                    }
                },
                &"float2" => "texCoord2f[]",
                &"float" => "float[]",
                &"int" => "int[]",
                &"string" => "string[]",
                _ => "unknown[]",
            };
            
            attributes.push(USDAttribute {
                name: interner.intern(&primvar.name),
                full_name: interner.intern(&format!("primvars:{}", primvar.name)),
                value,
                interpolation,
                is_primvar: true,
                is_indexed: primvar.indices.is_some(),
                indices: primvar.indices.clone(),
                type_display: interner.intern(type_display),
                interpolation_display: interner.intern(&primvar.interpolation),
                count_display: interner.intern(&count),
                preview_display: interner.intern(&preview),
            });
        }
        
        // Add legacy normals/uvs/colors if not already in primvars
        let has_normals_primvar = mesh.primvars.iter().any(|p| p.name.contains("normal"));
        let has_uv_primvar = mesh.primvars.iter().any(|p| p.name == "st" || p.name.contains("uv"));
        let has_color_primvar = mesh.primvars.iter().any(|p| p.name == "displayColor");
        
        if !has_normals_primvar && !mesh.normals.is_empty() {
            let sampled_normals = mesh.normals.clone();
            
            attributes.push(USDAttribute {
                name: interner.intern("normals"),
                full_name: interner.intern("normals"),
                value: AttributeValue::Normal3fArray(sampled_normals),
                interpolation: Some(InterpolationType::Vertex),
                is_primvar: false,
                is_indexed: false,
                indices: None,
                type_display: interner.intern("normal3f[]"),
                interpolation_display: interner.intern("vertex"),
                count_display: interner.intern(&mesh.normals.len().to_string()),
                preview_display: interner.intern(&format!("[{}] normals", mesh.normals.len())),
            });
        }
        
        if !has_uv_primvar && !mesh.uvs.is_empty() {
            let sampled_uvs = mesh.uvs.clone();
            
            attributes.push(USDAttribute {
                name: interner.intern("uvs"),
                full_name: interner.intern("uvs"),
                value: AttributeValue::TexCoord2fArray(sampled_uvs),
                interpolation: Some(InterpolationType::FaceVarying),
                is_primvar: false,
                is_indexed: false,
                indices: None,
                type_display: interner.intern("texCoord2f[]"),
                interpolation_display: interner.intern("faceVarying"),
                count_display: interner.intern(&mesh.uvs.len().to_string()),
                preview_display: interner.intern(&format!("[{}] UVs", mesh.uvs.len())),
            });
        }
        
        if !has_color_primvar && mesh.vertex_colors.is_some() {
            if let Some(ref colors) = mesh.vertex_colors {
                let sampled_colors = colors.clone();
                
                attributes.push(USDAttribute {
                    name: interner.intern("vertexColors"),
                    full_name: interner.intern("vertexColors"),
                    value: AttributeValue::Color3fArray(sampled_colors),
                    interpolation: Some(InterpolationType::Vertex),
                    is_primvar: false,
                    is_indexed: false,
                    indices: None,
                    type_display: interner.intern("color3f[]"),
                    interpolation_display: interner.intern("vertex"),
                    count_display: interner.intern(&colors.len().to_string()),
                    preview_display: interner.intern(&format!("[{}] vertex colors", colors.len())),
                });
            }
        }
        
        // Transform (as constant attribute)
        attributes.push(USDAttribute {
            name: interner.intern("transform"),
            full_name: interner.intern("transform"),
            value: AttributeValue::Matrix4d(mesh.transform),
            interpolation: Some(InterpolationType::Constant),
            is_primvar: false,
            is_indexed: false,
            indices: None,
            type_display: interner.intern("matrix4d"),
            interpolation_display: interner.intern("constant"),
            count_display: interner.intern("1"),
            preview_display: interner.intern("transformation matrix"),
        });
    }
    
    attributes
}

/// Extract primitives from USD scene data with STREAMING and LAZY loading
pub fn extract_primitives_from_scene(scene_data: &USDSceneData) -> Vec<USDPrimitive> {
    let mut primitives = Vec::new();
    
    // Process ALL meshes - no limits, virtual scrolling handles performance
    if let Ok(mut interner) = STRING_INTERNER.lock() {
        // Process all meshes in the scene
        let meshes_to_process: Vec<_> = scene_data.meshes.iter().collect();
        
        for (idx, mesh) in meshes_to_process.iter().enumerate() {
            // LAZY: Only extract essential attributes, not all data
            let attributes = extract_essential_attributes_only(mesh, &mut interner);
            
            let primvar_count = attributes.iter().filter(|attr| attr.is_primvar).count();
            let geometry_count = attributes.iter().filter(|attr| !attr.is_primvar).count();
            
            primitives.push(USDPrimitive {
                path: interner.intern(&mesh.prim_path),
                prim_type: interner.intern("Mesh"),
                attribute_count: attributes.len(),
                primvar_count,
                geometry_count,
                attributes,
            });
            
            // Process all meshes - no artificial limits
        }
    }
    
    primitives
}

/// Extract only essential attributes (LAZY loading) to prevent memory explosion
fn extract_essential_attributes_only(mesh: &USDMeshGeometry, interner: &mut StringInterner) -> Vec<USDAttribute> {
    let mut attributes = Vec::with_capacity(4); // Only essential attributes
    
    // ESSENTIAL 1: Mesh info (no data cloning)
    attributes.push(USDAttribute {
        name: interner.intern("mesh_info"),
        full_name: interner.intern("mesh_info"),
        value: AttributeValue::String(format!("{} vertices, {} faces", 
            mesh.vertices.len(), mesh.indices.len() / 3)),
        interpolation: Some(InterpolationType::Constant),
        is_primvar: false,
        is_indexed: false,
        indices: None,
        type_display: interner.intern("string"),
        interpolation_display: interner.intern("constant"),
        count_display: interner.intern("1"),
        preview_display: interner.intern(&format!("Mesh: {} verts, {} faces", 
            mesh.vertices.len(), mesh.indices.len() / 3)),
    });
    
    // ESSENTIAL 2: Bounding box (computed, not cloned)
    if !mesh.vertices.is_empty() {
        let (min_x, max_x) = mesh.vertices.iter().map(|v| v.x).fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), x| (min.min(x), max.max(x)));
        let (min_y, max_y) = mesh.vertices.iter().map(|v| v.y).fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), y| (min.min(y), max.max(y)));
        let (min_z, max_z) = mesh.vertices.iter().map(|v| v.z).fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), z| (min.min(z), max.max(z)));
        
        attributes.push(USDAttribute {
            name: interner.intern("extent"),
            full_name: interner.intern("extent"),
            value: AttributeValue::String(format!("({:.2}, {:.2}, {:.2}) to ({:.2}, {:.2}, {:.2})", 
                min_x, min_y, min_z, max_x, max_y, max_z)),
            interpolation: Some(InterpolationType::Constant),
            is_primvar: false,
            is_indexed: false,
            indices: None,
            type_display: interner.intern("string"),
            interpolation_display: interner.intern("constant"),
            count_display: interner.intern("1"),
            preview_display: interner.intern("bounding box"),
        });
    }
    
    // ESSENTIAL 3: Material info (if available)
    if mesh.vertex_colors.is_some() {
        attributes.push(USDAttribute {
            name: interner.intern("has_colors"),
            full_name: interner.intern("primvars:displayColor"),
            value: AttributeValue::Bool(true),
            interpolation: Some(InterpolationType::Vertex),
            is_primvar: true,
            is_indexed: false,
            indices: None,
            type_display: interner.intern("bool"),
            interpolation_display: interner.intern("vertex"),
            count_display: interner.intern("1"),
            preview_display: interner.intern("vertex colors available"),
        });
    }
    
    // ESSENTIAL 4: UV info (if available)
    if !mesh.uvs.is_empty() {
        attributes.push(USDAttribute {
            name: interner.intern("has_uvs"),
            full_name: interner.intern("primvars:st"),
            value: AttributeValue::Bool(true),
            interpolation: Some(InterpolationType::FaceVarying),
            is_primvar: true,
            is_indexed: false,
            indices: None,
            type_display: interner.intern("bool"),
            interpolation_display: interner.intern("faceVarying"),
            count_display: interner.intern("1"),
            preview_display: interner.intern("UV coordinates available"),
        });
    }
    
    attributes
}

/// Calculate hash for change detection with memory tracking
pub fn calculate_hash<T: std::hash::Hash>(data: &T) -> u64 {
    // Track memory usage before hashing
    // let start_memory = get_memory_usage(); // TODO: Implement memory tracking
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

/// Calculate hash for USD scene data to detect changes
pub fn calculate_usd_scene_hash(scene_data: &crate::workspaces::three_d::usd::usd_engine::USDSceneData) -> u64 {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    
    let mut hasher = DefaultHasher::new();
    
    // Hash key identifying features
    scene_data.stage_path.hash(&mut hasher);
    scene_data.meshes.len().hash(&mut hasher);
    scene_data.lights.len().hash(&mut hasher);
    scene_data.materials.len().hash(&mut hasher);
    scene_data.up_axis.hash(&mut hasher);
    
    // Hash all mesh data to detect content changes
    for (idx, mesh) in scene_data.meshes.iter().enumerate() { // Hash all meshes
        mesh.prim_path.hash(&mut hasher);
        mesh.vertices.len().hash(&mut hasher);
        mesh.indices.len().hash(&mut hasher);
        if !mesh.vertices.is_empty() {
            // Hash first and last vertex to detect geometry changes
            mesh.vertices[0].x.to_bits().hash(&mut hasher);
            mesh.vertices[0].y.to_bits().hash(&mut hasher);
            mesh.vertices[0].z.to_bits().hash(&mut hasher);
            let last_idx = mesh.vertices.len() - 1;
            mesh.vertices[last_idx].x.to_bits().hash(&mut hasher);
            mesh.vertices[last_idx].y.to_bits().hash(&mut hasher);
            mesh.vertices[last_idx].z.to_bits().hash(&mut hasher);
        }
    }
    
    hasher.finish()
}

/// Process attributes node with caching and performance optimizations
pub fn process_attributes_node(
    node_id: NodeId,
    inputs: &HashMap<String, NodeData>,
    _execution_cache: &mut HashMap<NodeId, NodeData>,
) -> HashMap<String, NodeData> {
    let mut outputs = HashMap::new();
    
    // Check for USD scene input
    if let Some(input_data) = inputs.get("USD Scene") {
        if let NodeData::USDSceneData(scene_data) = input_data {
            // Create a hash from the key identifying features of the USD scene data
            let input_hash = calculate_usd_scene_hash(scene_data);
            let mut should_process = true;
            
            // Check cache for existing processed data
            if let Ok(cache) = ATTRIBUTES_INPUT_CACHE.read() {
                if let Some((cached_data, cached_version)) = cache.get(&node_id) {
                    let cached_hash = cached_version.load(std::sync::atomic::Ordering::Relaxed);
                    if cached_hash == input_hash {
                        // Cache hit - no need to reprocess
                        should_process = false;
                        
                        // Update cache hit metrics
                        ATTRIBUTE_METRICS.with(|metrics| {
                            let mut metrics = metrics.borrow_mut();
                            metrics.cache_hits += 1;
                        });
                        
                        // Return cached result indication
                        outputs.insert("Primitives".to_string(), NodeData::String("Using cached attribute data".to_string()));
                    } else {
                        // Cache miss - input data has changed
                        ATTRIBUTE_METRICS.with(|metrics| {
                            let mut metrics = metrics.borrow_mut();
                            metrics.cache_misses += 1;
                        });
                    }
                } else {
                    // No cache entry - first time processing
                    ATTRIBUTE_METRICS.with(|metrics| {
                        let mut metrics = metrics.borrow_mut();
                        metrics.cache_misses += 1;
                    });
                }
            }
            
            if should_process {
                // Extract primitives with attributes and performance tracking
                let start_time = Instant::now();
                let primitives = extract_primitives_from_scene(scene_data);
                let extraction_time = start_time.elapsed();
                
                // Update metrics
                ATTRIBUTE_METRICS.with(|metrics| {
                    let mut metrics = metrics.borrow_mut();
                    metrics.extraction_time = extraction_time;
                    metrics.mesh_count = primitives.len();
                    metrics.attribute_count = primitives.iter().map(|p| p.attribute_count).sum();
                    metrics.cache_updates += 1;
                    metrics.last_updated = Instant::now();
                });
                
                // Cache the processed data
                if let Ok(mut cache) = ATTRIBUTES_INPUT_CACHE.write() {
                    // Store the primitives data in a format that can be retrieved by the UI panel
                    let cached_data = NodeData::String(format!("Processed {} primitives with {} total attributes", 
                        primitives.len(), 
                        primitives.iter().map(|p| p.attribute_count).sum::<usize>()));
                    
                    cache.insert(node_id, (cached_data.clone(), std::sync::atomic::AtomicU64::new(input_hash)));
                    
                    // Store processed data for UI panel
                    outputs.insert("Primitives".to_string(), cached_data);
                } else {
                    // Fallback if cache write fails
                    outputs.insert("Primitives".to_string(), NodeData::String(format!("Processed {} primitives", primitives.len())));
                }
            } else {
                // Using cached data - retrieve from cache
                if let Ok(cache) = ATTRIBUTES_INPUT_CACHE.read() {
                    if let Some((cached_data, _)) = cache.get(&node_id) {
                        outputs.insert("Primitives".to_string(), cached_data.clone());
                    }
                }
            }
        }
    }
    
    outputs
}