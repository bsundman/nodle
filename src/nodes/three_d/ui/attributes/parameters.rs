//! Attributes parameter UI implementation
//! 
//! Provides a spreadsheet-style interface with tabs for different interpolation types,
//! virtual scrolling, filtering, and performance optimizations.

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::NodeId;
use crate::workspaces::three_d::usd::usd_engine::USDSceneData;
use egui::{self, Color32, Response, Ui, Vec2, Rect, Sense, TextEdit, ScrollArea, Stroke};
use egui_extras::{TableBuilder, Column};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use super::logic::*;

const ITEMS_PER_PAGE: usize = 100;
const MIN_ROW_HEIGHT: f32 = 24.0;
const HEADER_HEIGHT: f32 = 32.0;
const TAB_HEIGHT: f32 = 28.0;
const FILTER_HEIGHT: f32 = 32.0;

// Performance optimizations - virtual scrolling handles large datasets

/// Attribute display state for UI
pub struct AttributeDisplayState {
    pub current_tab: AttributeTab,
    pub filter_text: String,
    pub search_text: String,
    pub selected_prim: Option<String>,
    pub current_page: usize,
    pub items_per_page: usize,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    pub last_update: Instant,
    pub cached_primitives: Vec<USDPrimitive>,
    pub geometry_spreadsheet: GeometrySpreadsheet,
    pub virtual_scroll_offset: f32,
    pub visible_range: (usize, usize),
}

impl Default for AttributeDisplayState {
    fn default() -> Self {
        Self {
            current_tab: AttributeTab::default(),
            filter_text: String::new(),
            search_text: String::new(),
            selected_prim: None,
            current_page: 0,
            items_per_page: ITEMS_PER_PAGE,
            sort_column: SortColumn::default(),
            sort_ascending: true,
            last_update: Instant::now(),
            cached_primitives: Vec::new(),
            geometry_spreadsheet: GeometrySpreadsheet {
                columns: Vec::new(),
                rows: Vec::new(),
                element_type: "Points".to_string(),
            },
            virtual_scroll_offset: 0.0,
            visible_range: (0, 0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttributeTab {
    PointAttributes,     // USD "vertex" interpolation - one value per point
    FaceCornerAttributes,// USD "faceVarying" interpolation - one value per face corner
    FaceAttributes,      // USD "uniform" interpolation - one value per primitive/face
    GeometryAttributes,  // USD "constant" interpolation - one value for entire mesh
}

impl Default for AttributeTab {
    fn default() -> Self {
        AttributeTab::PointAttributes
    }
}

impl AttributeTab {
    pub fn display_name(&self) -> &'static str {
        match self {
            AttributeTab::PointAttributes => "Point Attributes",
            AttributeTab::FaceCornerAttributes => "Face-Corner Attributes", 
            AttributeTab::FaceAttributes => "Face Attributes",
            AttributeTab::GeometryAttributes => "Geometry Attributes",
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            AttributeTab::PointAttributes => "USD 'vertex' interpolation - one value per point",
            AttributeTab::FaceCornerAttributes => "USD 'faceVarying' interpolation - one value per face corner",
            AttributeTab::FaceAttributes => "USD 'uniform' interpolation - one value per primitive/face",
            AttributeTab::GeometryAttributes => "USD 'constant' interpolation - one value for entire mesh",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortColumn {
    Name,
    Type,
    Interpolation,
    Count,
    Primitive,
}

impl Default for SortColumn {
    fn default() -> Self {
        SortColumn::Name
    }
}

/// Geometry spreadsheet row - represents one geometric element (point, primitive, etc.)
#[derive(Debug, Clone)]
pub struct GeometryRow {
    pub element_index: usize,           // Point number, primitive number, etc.
    pub attribute_values: Vec<String>,  // Values for each attribute column
    pub row_height: f32,
}

/// Column definition for geometry spreadsheet
#[derive(Debug, Clone)]
pub struct AttributeColumn {
    pub name: String,
    pub type_name: String,
    pub width: f32,
}

/// Geometry spreadsheet data organized like Houdini
#[derive(Debug, Clone)]
pub struct GeometrySpreadsheet {
    pub columns: Vec<AttributeColumn>,
    pub rows: Vec<GeometryRow>,
    pub element_type: String, // "Points", "Primitives", etc.
}

/// Render attributes parameter panel
pub fn render_attributes_parameters(
    ui: &mut Ui,
    node_id: NodeId,
    parameters: &mut HashMap<String, NodeData>,
    _inputs: &HashMap<String, NodeData>,
) -> Vec<ParameterChange> {
    let mut changes = Vec::new();
    
    // Get or create display state
    static mut DISPLAY_STATES: Option<HashMap<NodeId, AttributeDisplayState>> = None;
    let display_states = unsafe {
        if DISPLAY_STATES.is_none() {
            DISPLAY_STATES = Some(HashMap::new());
        }
        DISPLAY_STATES.as_mut().unwrap()
    };
    
    let state = display_states.entry(node_id).or_insert_with(AttributeDisplayState::default);
    
    // OPTIMIZED: Simple cache invalidation - only update if cache is empty
    let mut scene_data_ref = None;
    if let Some(NodeData::USDSceneData(scene_data)) = _inputs.get("USD Scene") {
        scene_data_ref = Some(scene_data);
        
        // CRITICAL: Only update if cache is completely empty to prevent constant re-computation
        let needs_update = state.cached_primitives.is_empty();
        
        if needs_update {
            update_cached_primitives(state, scene_data);
        }
    }
    
    // Render the spreadsheet interface
    render_spreadsheet_interface(ui, state, parameters, &mut changes, scene_data_ref);
    
    changes
}

// REMOVED: calculate_scene_hash - was causing expensive computation every frame

/// Update cached primitives from scene data
fn update_cached_primitives(state: &mut AttributeDisplayState, scene_data: &USDSceneData) {
    state.cached_primitives = extract_primitives_from_scene(scene_data);
    state.last_update = Instant::now();
    
    // Rebuild geometry spreadsheet in Houdini style
    build_geometry_spreadsheet(state, scene_data);
}

/// Build geometry spreadsheet in USD style - rows are elements, columns are attributes
fn build_geometry_spreadsheet(state: &mut AttributeDisplayState, scene_data: &USDSceneData) {
    match state.current_tab {
        AttributeTab::PointAttributes => build_point_attributes_spreadsheet(state, scene_data),
        AttributeTab::FaceCornerAttributes => build_face_corner_attributes_spreadsheet(state, scene_data),
        AttributeTab::FaceAttributes => build_face_attributes_spreadsheet(state, scene_data),
        AttributeTab::GeometryAttributes => build_geometry_attributes_spreadsheet(state, scene_data),
    }
}

/// Build point attributes spreadsheet - USD "vertex" interpolation (one value per point)
fn build_point_attributes_spreadsheet(state: &mut AttributeDisplayState, scene_data: &USDSceneData) {
    let mut columns = Vec::new();
    let mut rows = Vec::new();
    
    // Always include point index column
    columns.push(AttributeColumn {
        name: "pt".to_string(),
        type_name: "int".to_string(),
        width: 60.0,
    });
    
    // Process ALL meshes to show complete scene data
    if !scene_data.meshes.is_empty() {
        // Take first mesh for column structure
        let first_mesh = &scene_data.meshes[0];
        // Add position columns (from points array)
        columns.push(AttributeColumn {
            name: "P.x".to_string(),
            type_name: "float".to_string(),
            width: 80.0,
        });
        columns.push(AttributeColumn {
            name: "P.y".to_string(),
            type_name: "float".to_string(),
            width: 80.0,
        });
        columns.push(AttributeColumn {
            name: "P.z".to_string(),
            type_name: "float".to_string(),
            width: 80.0,
        });
        
        // Check if we have normals as primvars or need to add them as fallback
        let has_normals_primvar = scene_data.meshes.iter().any(|m| 
            m.primvars.iter().any(|pv| pv.interpolation == "vertex" && (pv.name.contains("normal") || pv.name == "N"))
        );
        
        // Add normals columns if not in primvars but available in mesh data
        if !has_normals_primvar && scene_data.meshes.iter().any(|m| !m.normals.is_empty()) {
            columns.push(AttributeColumn {
                name: "N.x".to_string(),
                type_name: "float".to_string(),
                width: 80.0,
            });
            columns.push(AttributeColumn {
                name: "N.y".to_string(),
                type_name: "float".to_string(),
                width: 80.0,
            });
            columns.push(AttributeColumn {
                name: "N.z".to_string(),
                type_name: "float".to_string(),
                width: 80.0,
            });
        }
        
        // Add UV coordinates if available in mesh data
        let has_uv_primvar = scene_data.meshes.iter().any(|m| 
            m.primvars.iter().any(|pv| pv.interpolation == "vertex" && (pv.name.contains("uv") || pv.name == "st" || pv.name.contains("texCoord")))
        );
        
        if !has_uv_primvar && scene_data.meshes.iter().any(|m| !m.uvs.is_empty()) {
            columns.push(AttributeColumn {
                name: "uv.u".to_string(),
                type_name: "float".to_string(),
                width: 80.0,
            });
            columns.push(AttributeColumn {
                name: "uv.v".to_string(),
                type_name: "float".to_string(),
                width: 80.0,
            });
        }
        
        // Add vertex colors if available in mesh data
        let has_color_primvar = scene_data.meshes.iter().any(|m| 
            m.primvars.iter().any(|pv| pv.interpolation == "vertex" && (pv.name.contains("color") || pv.name == "Cd" || pv.name == "displayColor"))
        );
        
        if !has_color_primvar && scene_data.meshes.iter().any(|m| m.vertex_colors.is_some()) {
            columns.push(AttributeColumn {
                name: "Cd.r".to_string(),
                type_name: "float".to_string(),
                width: 80.0,
            });
            columns.push(AttributeColumn {
                name: "Cd.g".to_string(),
                type_name: "float".to_string(),
                width: 80.0,
            });
            columns.push(AttributeColumn {
                name: "Cd.b".to_string(),
                type_name: "float".to_string(),
                width: 80.0,
            });
        }
        
        // Collect ALL unique vertex primvars from ALL meshes (not just first mesh)
        let mut all_vertex_primvars = std::collections::HashMap::new();
        for (mesh_idx, mesh) in scene_data.meshes.iter().enumerate() {
            let mesh_vertex_primvars: Vec<_> = mesh.primvars.iter().filter(|pv| pv.interpolation == "vertex").collect();
            if mesh_idx < 5 && !mesh_vertex_primvars.is_empty() {  // Debug first 5 meshes
                println!("📊 Mesh[{}] {} has {} vertex primvars:", mesh_idx, mesh.prim_path, mesh_vertex_primvars.len());
                for pv in &mesh_vertex_primvars {
                    println!("    - {} ({})", pv.name, pv.data_type);
                }
            }
            for primvar in mesh_vertex_primvars {
                all_vertex_primvars.insert(primvar.name.clone(), primvar.clone());
            }
        }
        let vertex_primvars: Vec<_> = all_vertex_primvars.values().collect();
        
        // Debug: Summary of unique vertex primvars
        println!("📊 Total unique vertex primvars: {}", all_vertex_primvars.len());
        for (name, primvar) in &all_vertex_primvars {
            println!("    - {} ({})", name, primvar.data_type);
        }
        println!("📊 Point Attributes: Found {} unique vertex primvars across {} meshes", vertex_primvars.len(), scene_data.meshes.len());
        
        // Also debug total primvars per mesh
        if !scene_data.meshes.is_empty() {
            let total_primvars: usize = scene_data.meshes.iter().map(|m| m.primvars.len()).sum();
            println!("📊 Total primvars across all meshes: {}", total_primvars);
            
            // Count by interpolation type
            let mut interp_counts = std::collections::HashMap::new();
            for mesh in &scene_data.meshes {
                for primvar in &mesh.primvars {
                    *interp_counts.entry(primvar.interpolation.clone()).or_insert(0) += 1;
                }
            }
            println!("📊 Primvars by interpolation type:");
            for (interp, count) in &interp_counts {
                println!("    - {}: {}", interp, count);
            }
        }
            
        for primvar in &vertex_primvars {
            match &primvar.values {
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(_) => {
                    columns.push(AttributeColumn {
                        name: primvar.name.clone(),
                        type_name: "float".to_string(),
                        width: 80.0,
                    });
                }
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(_) => {
                    columns.push(AttributeColumn {
                        name: format!("{}.x", primvar.name),
                        type_name: "float".to_string(),
                        width: 80.0,
                    });
                    columns.push(AttributeColumn {
                        name: format!("{}.y", primvar.name),
                        type_name: "float".to_string(),
                        width: 80.0,
                    });
                }
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(_) => {
                    columns.push(AttributeColumn {
                        name: format!("{}.x", primvar.name),
                        type_name: "float".to_string(),
                        width: 80.0,
                    });
                    columns.push(AttributeColumn {
                        name: format!("{}.y", primvar.name),
                        type_name: "float".to_string(),
                        width: 80.0,
                    });
                    columns.push(AttributeColumn {
                        name: format!("{}.z", primvar.name),
                        type_name: "float".to_string(),
                        width: 80.0,
                    });
                }
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(_) => {
                    columns.push(AttributeColumn {
                        name: primvar.name.clone(),
                        type_name: "int".to_string(),
                        width: 80.0,
                    });
                }
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(_) => {
                    columns.push(AttributeColumn {
                        name: primvar.name.clone(),
                        type_name: "string".to_string(),
                        width: 120.0,
                    });
                }
            }
        }
        
        // Combine ALL vertices from ALL meshes for complete scene view
        let mut all_vertices = Vec::new();
        let mut all_normals = Vec::new(); // Collect normals if not in primvars
        let mut all_uvs = Vec::new(); // Collect UVs if not in primvars
        let mut all_vertex_colors = Vec::new(); // Collect vertex colors if not in primvars
        let mut all_vertex_primvars: Vec<Vec<f32>> = Vec::new();
        let mut all_vertex_primvars_float2: Vec<Vec<glam::Vec2>> = Vec::new();
        let mut all_vertex_primvars_float3: Vec<Vec<glam::Vec3>> = Vec::new();
        let mut all_vertex_primvars_int: Vec<Vec<i32>> = Vec::new();
        let mut all_vertex_primvars_string: Vec<Vec<String>> = Vec::new();
        
        // Initialize primvar storage based on first mesh structure
        for primvar in &vertex_primvars {
            match &primvar.values {
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(_) => all_vertex_primvars.push(Vec::new()),
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(_) => all_vertex_primvars_float2.push(Vec::new()),
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(_) => all_vertex_primvars_float3.push(Vec::new()),
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(_) => all_vertex_primvars_int.push(Vec::new()),
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(_) => all_vertex_primvars_string.push(Vec::new()),
            }
        }
        
        // Collect vertices from ALL meshes
        for mesh in &scene_data.meshes {
            // Add all vertices from this mesh
            all_vertices.extend(mesh.vertices.iter().cloned());
            
            // Add normals if not in primvars
            if !has_normals_primvar {
                if !mesh.normals.is_empty() {
                    all_normals.extend(mesh.normals.iter().cloned());
                } else {
                    // Fill with default normals
                    all_normals.extend(vec![glam::Vec3::new(0.0, 1.0, 0.0); mesh.vertices.len()]);
                }
            }
            
            // Add UVs if not in primvars
            if !has_uv_primvar {
                if !mesh.uvs.is_empty() {
                    all_uvs.extend(mesh.uvs.iter().cloned());
                } else {
                    // Fill with default UVs
                    all_uvs.extend(vec![glam::Vec2::new(0.0, 0.0); mesh.vertices.len()]);
                }
            }
            
            // Add vertex colors if not in primvars
            if !has_color_primvar {
                if let Some(ref vertex_colors) = mesh.vertex_colors {
                    if vertex_colors.len() >= mesh.vertices.len() {
                        // Use per-vertex colors
                        all_vertex_colors.extend(vertex_colors.iter().cloned());
                    } else if !vertex_colors.is_empty() {
                        // Use single color for all vertices (constant color)
                        let single_color = vertex_colors[0];
                        all_vertex_colors.extend(vec![single_color; mesh.vertices.len()]);
                    } else {
                        // Fill with default white
                        all_vertex_colors.extend(vec![glam::Vec3::new(1.0, 1.0, 1.0); mesh.vertices.len()]);
                    }
                } else {
                    // Fill with default white
                    all_vertex_colors.extend(vec![glam::Vec3::new(1.0, 1.0, 1.0); mesh.vertices.len()]);
                }
            }
            
            // Add primvar data from this mesh (matching by name and type)
            let mut float_idx = 0;
            let mut float2_idx = 0;
            let mut float3_idx = 0;
            let mut int_idx = 0;
            let mut string_idx = 0;
            
            for primvar in &vertex_primvars {
                // Find matching primvar in current mesh
                if let Some(mesh_primvar) = mesh.primvars.iter().find(|p| p.name == primvar.name && p.interpolation == "vertex") {
                    match (&primvar.values, &mesh_primvar.values) {
                        (crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(_), 
                         crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(vals)) => {
                            all_vertex_primvars[float_idx].extend(vals.iter().cloned());
                            float_idx += 1;
                        },
                        (crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(_), 
                         crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(vals)) => {
                            all_vertex_primvars_float2[float2_idx].extend(vals.iter().cloned());
                            float2_idx += 1;
                        },
                        (crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(_), 
                         crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(vals)) => {
                            all_vertex_primvars_float3[float3_idx].extend(vals.iter().cloned());
                            float3_idx += 1;
                        },
                        (crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(_), 
                         crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(vals)) => {
                            all_vertex_primvars_int[int_idx].extend(vals.iter().cloned());
                            int_idx += 1;
                        },
                        (crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(_), 
                         crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(vals)) => {
                            all_vertex_primvars_string[string_idx].extend(vals.iter().cloned());
                            string_idx += 1;
                        },
                        _ => {
                            // Type mismatch or missing primvar - fill with defaults
                            match &primvar.values {
                                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(_) => {
                                    all_vertex_primvars[float_idx].extend(vec![0.0; mesh.vertices.len()]);
                                    float_idx += 1;
                                },
                                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(_) => {
                                    all_vertex_primvars_float2[float2_idx].extend(vec![glam::Vec2::ZERO; mesh.vertices.len()]);
                                    float2_idx += 1;
                                },
                                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(_) => {
                                    all_vertex_primvars_float3[float3_idx].extend(vec![glam::Vec3::ZERO; mesh.vertices.len()]);
                                    float3_idx += 1;
                                },
                                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(_) => {
                                    all_vertex_primvars_int[int_idx].extend(vec![0; mesh.vertices.len()]);
                                    int_idx += 1;
                                },
                                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(_) => {
                                    all_vertex_primvars_string[string_idx].extend(vec![String::new(); mesh.vertices.len()]);
                                    string_idx += 1;
                                },
                            }
                        }
                    }
                } else {
                    // Primvar not found in this mesh - fill with defaults
                    match &primvar.values {
                        crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(_) => {
                            all_vertex_primvars[float_idx].extend(vec![0.0; mesh.vertices.len()]);
                            float_idx += 1;
                        },
                        crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(_) => {
                            all_vertex_primvars_float2[float2_idx].extend(vec![glam::Vec2::ZERO; mesh.vertices.len()]);
                            float2_idx += 1;
                        },
                        crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(_) => {
                            all_vertex_primvars_float3[float3_idx].extend(vec![glam::Vec3::ZERO; mesh.vertices.len()]);
                            float3_idx += 1;
                        },
                        crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(_) => {
                            all_vertex_primvars_int[int_idx].extend(vec![0; mesh.vertices.len()]);
                            int_idx += 1;
                        },
                        crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(_) => {
                            all_vertex_primvars_string[string_idx].extend(vec![String::new(); mesh.vertices.len()]);
                            string_idx += 1;
                        },
                    }
                }
            }
        }
        
        let total_points = all_vertices.len();
        
        // Create rows for ALL points from ALL meshes
        for i in 0..total_points {
            let mut values = Vec::new();
            
            // Point index
            values.push(i.to_string());
            
            // Position (from combined vertices array)
            if i < all_vertices.len() {
                let vertex = all_vertices[i];
                values.push(format!("{:.3}", vertex.x));
                values.push(format!("{:.3}", vertex.y));
                values.push(format!("{:.3}", vertex.z));
            } else {
                values.push("0.000".to_string());
                values.push("0.000".to_string());
                values.push("0.000".to_string());
            }
            
            // Add normals if not in primvars but available
            if !has_normals_primvar && i < all_normals.len() {
                let normal = all_normals[i];
                values.push(format!("{:.3}", normal.x));
                values.push(format!("{:.3}", normal.y));
                values.push(format!("{:.3}", normal.z));
            } else if !has_normals_primvar {
                values.push("0.000".to_string());
                values.push("1.000".to_string()); // Default up
                values.push("0.000".to_string());
            }
            
            // Add UVs if not in primvars but available
            if !has_uv_primvar && i < all_uvs.len() {
                let uv = all_uvs[i];
                values.push(format!("{:.3}", uv.x));
                values.push(format!("{:.3}", uv.y));
            } else if !has_uv_primvar {
                values.push("0.000".to_string());
                values.push("0.000".to_string());
            }
            
            // Add vertex colors if not in primvars but available
            if !has_color_primvar && i < all_vertex_colors.len() {
                let color = all_vertex_colors[i];
                values.push(format!("{:.3}", color.x));
                values.push(format!("{:.3}", color.y));
                values.push(format!("{:.3}", color.z));
            } else if !has_color_primvar {
                values.push("1.000".to_string()); // Default white
                values.push("1.000".to_string());
                values.push("1.000".to_string());
            }
            
            // Add values for vertex primvars (point attributes) from combined arrays
            let mut float_idx = 0;
            let mut float2_idx = 0;
            let mut float3_idx = 0;
            let mut int_idx = 0;
            let mut string_idx = 0;
            
            for primvar in &vertex_primvars {
                match &primvar.values {
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(_) => {
                        if float_idx < all_vertex_primvars.len() && i < all_vertex_primvars[float_idx].len() {
                            values.push(format!("{:.3}", all_vertex_primvars[float_idx][i]));
                        } else {
                            values.push("0.000".to_string());
                        }
                        float_idx += 1;
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(_) => {
                        if float2_idx < all_vertex_primvars_float2.len() && i < all_vertex_primvars_float2[float2_idx].len() {
                            let val = all_vertex_primvars_float2[float2_idx][i];
                            values.push(format!("{:.3}", val.x));
                            values.push(format!("{:.3}", val.y));
                        } else {
                            values.push("0.000".to_string());
                            values.push("0.000".to_string());
                        }
                        float2_idx += 1;
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(_) => {
                        if float3_idx < all_vertex_primvars_float3.len() && i < all_vertex_primvars_float3[float3_idx].len() {
                            let val = all_vertex_primvars_float3[float3_idx][i];
                            values.push(format!("{:.3}", val.x));
                            values.push(format!("{:.3}", val.y));
                            values.push(format!("{:.3}", val.z));
                        } else {
                            values.push("0.000".to_string());
                            values.push("0.000".to_string());
                            values.push("0.000".to_string());
                        }
                        float3_idx += 1;
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(_) => {
                        if int_idx < all_vertex_primvars_int.len() && i < all_vertex_primvars_int[int_idx].len() {
                            values.push(all_vertex_primvars_int[int_idx][i].to_string());
                        } else {
                            values.push("0".to_string());
                        }
                        int_idx += 1;
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(_) => {
                        if string_idx < all_vertex_primvars_string.len() && i < all_vertex_primvars_string[string_idx].len() {
                            values.push(all_vertex_primvars_string[string_idx][i].clone());
                        } else {
                            values.push("".to_string());
                        }
                        string_idx += 1;
                    }
                }
            }
            
            rows.push(GeometryRow {
                element_index: i,
                attribute_values: values,
                row_height: MIN_ROW_HEIGHT,
            });
        }
    }
    
    state.geometry_spreadsheet = GeometrySpreadsheet {
        columns,
        rows,
        element_type: "Point Attributes".to_string(),
    };
}

/// Build face attributes spreadsheet - USD "uniform" interpolation (one value per primitive/face)
fn build_face_attributes_spreadsheet(state: &mut AttributeDisplayState, scene_data: &USDSceneData) {
    let mut columns = Vec::new();
    let mut rows = Vec::new();
    
    // Face index column
    columns.push(AttributeColumn {
        name: "face".to_string(),
        type_name: "int".to_string(),
        width: 60.0,
    });
    
    // Add vertex count column (always 3 for triangles)
    columns.push(AttributeColumn {
        name: "vertices".to_string(),
        type_name: "int".to_string(),
        width: 80.0,
    });
    
    // Process ALL meshes to show complete scene data
    if !scene_data.meshes.is_empty() {
        // Take first mesh for column structure
        let first_mesh = &scene_data.meshes[0];
        // Collect ALL unique uniform primvars from ALL meshes (not just first mesh)
        let mut all_uniform_primvars = std::collections::HashMap::new();
        for (mesh_idx, mesh) in scene_data.meshes.iter().enumerate() {
            let mesh_uniform_primvars: Vec<_> = mesh.primvars.iter().filter(|pv| pv.interpolation == "uniform").collect();
            if mesh_idx < 5 && !mesh_uniform_primvars.is_empty() {  // Debug first 5 meshes
                println!("📊 Mesh[{}] {} has {} uniform primvars:", mesh_idx, mesh.prim_path, mesh_uniform_primvars.len());
                for pv in &mesh_uniform_primvars {
                    println!("    - {} ({})", pv.name, pv.data_type);
                }
            }
            for primvar in mesh_uniform_primvars {
                all_uniform_primvars.insert(primvar.name.clone(), primvar.clone());
            }
        }
        let uniform_primvars: Vec<_> = all_uniform_primvars.values().collect();
        println!("📊 Face Attributes: Found {} unique uniform primvars across {} meshes", uniform_primvars.len(), scene_data.meshes.len());
            
        for primvar in &uniform_primvars {
            match &primvar.values {
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(_) => {
                    columns.push(AttributeColumn {
                        name: primvar.name.clone(),
                        type_name: "float".to_string(),
                        width: 80.0,
                    });
                }
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(_) => {
                    columns.push(AttributeColumn {
                        name: format!("{}.x", primvar.name),
                        type_name: "float".to_string(),
                        width: 80.0,
                    });
                    columns.push(AttributeColumn {
                        name: format!("{}.y", primvar.name),
                        type_name: "float".to_string(),
                        width: 80.0,
                    });
                }
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(_) => {
                    columns.push(AttributeColumn {
                        name: format!("{}.x", primvar.name),
                        type_name: "float".to_string(),
                        width: 80.0,
                    });
                    columns.push(AttributeColumn {
                        name: format!("{}.y", primvar.name),
                        type_name: "float".to_string(),
                        width: 80.0,
                    });
                    columns.push(AttributeColumn {
                        name: format!("{}.z", primvar.name),
                        type_name: "float".to_string(),
                        width: 80.0,
                    });
                }
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(_) => {
                    columns.push(AttributeColumn {
                        name: primvar.name.clone(),
                        type_name: "int".to_string(),
                        width: 80.0,
                    });
                }
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(_) => {
                    columns.push(AttributeColumn {
                        name: primvar.name.clone(),
                        type_name: "string".to_string(),
                        width: 120.0,
                    });
                }
            }
        }
        
        // Combine ALL face primvars from ALL meshes for complete scene view
        let mut all_face_primvars: Vec<Vec<f32>> = Vec::new();
        let mut all_face_primvars_float2: Vec<Vec<glam::Vec2>> = Vec::new();
        let mut all_face_primvars_float3: Vec<Vec<glam::Vec3>> = Vec::new();
        let mut all_face_primvars_int: Vec<Vec<i32>> = Vec::new();
        let mut all_face_primvars_string: Vec<Vec<String>> = Vec::new();
        
        // Initialize primvar storage based on first mesh structure
        for primvar in &uniform_primvars {
            match &primvar.values {
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(_) => all_face_primvars.push(Vec::new()),
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(_) => all_face_primvars_float2.push(Vec::new()),
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(_) => all_face_primvars_float3.push(Vec::new()),
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(_) => all_face_primvars_int.push(Vec::new()),
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(_) => all_face_primvars_string.push(Vec::new()),
            }
        }
        
        let mut total_faces = 0;
        
        // Collect face primvars from ALL meshes
        for mesh in &scene_data.meshes {
            let mesh_faces = mesh.indices.len() / 3;
            total_faces += mesh_faces;
            
            // Add primvar data from this mesh (matching by name and type)
            let mut float_idx = 0;
            let mut float2_idx = 0;
            let mut float3_idx = 0;
            let mut int_idx = 0;
            let mut string_idx = 0;
            
            for primvar in &uniform_primvars {
                // Find matching primvar in current mesh
                if let Some(mesh_primvar) = mesh.primvars.iter().find(|p| p.name == primvar.name && p.interpolation == "uniform") {
                    match (&primvar.values, &mesh_primvar.values) {
                        (crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(_), 
                         crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(vals)) => {
                            all_face_primvars[float_idx].extend(vals.iter().cloned());
                            float_idx += 1;
                        },
                        (crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(_), 
                         crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(vals)) => {
                            all_face_primvars_float2[float2_idx].extend(vals.iter().cloned());
                            float2_idx += 1;
                        },
                        (crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(_), 
                         crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(vals)) => {
                            all_face_primvars_float3[float3_idx].extend(vals.iter().cloned());
                            float3_idx += 1;
                        },
                        (crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(_), 
                         crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(vals)) => {
                            all_face_primvars_int[int_idx].extend(vals.iter().cloned());
                            int_idx += 1;
                        },
                        (crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(_), 
                         crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(vals)) => {
                            all_face_primvars_string[string_idx].extend(vals.iter().cloned());
                            string_idx += 1;
                        },
                        _ => {
                            // Type mismatch or missing primvar - fill with defaults
                            match &primvar.values {
                                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(_) => {
                                    all_face_primvars[float_idx].extend(vec![0.0; mesh_faces]);
                                    float_idx += 1;
                                },
                                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(_) => {
                                    all_face_primvars_float2[float2_idx].extend(vec![glam::Vec2::ZERO; mesh_faces]);
                                    float2_idx += 1;
                                },
                                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(_) => {
                                    all_face_primvars_float3[float3_idx].extend(vec![glam::Vec3::ZERO; mesh_faces]);
                                    float3_idx += 1;
                                },
                                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(_) => {
                                    all_face_primvars_int[int_idx].extend(vec![0; mesh_faces]);
                                    int_idx += 1;
                                },
                                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(_) => {
                                    all_face_primvars_string[string_idx].extend(vec![String::new(); mesh_faces]);
                                    string_idx += 1;
                                },
                            }
                        }
                    }
                } else {
                    // Primvar not found in this mesh - fill with defaults
                    match &primvar.values {
                        crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(_) => {
                            all_face_primvars[float_idx].extend(vec![0.0; mesh_faces]);
                            float_idx += 1;
                        },
                        crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(_) => {
                            all_face_primvars_float2[float2_idx].extend(vec![glam::Vec2::ZERO; mesh_faces]);
                            float2_idx += 1;
                        },
                        crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(_) => {
                            all_face_primvars_float3[float3_idx].extend(vec![glam::Vec3::ZERO; mesh_faces]);
                            float3_idx += 1;
                        },
                        crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(_) => {
                            all_face_primvars_int[int_idx].extend(vec![0; mesh_faces]);
                            int_idx += 1;
                        },
                        crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(_) => {
                            all_face_primvars_string[string_idx].extend(vec![String::new(); mesh_faces]);
                            string_idx += 1;
                        },
                    }
                }
            }
        }
        
        // Create rows for ALL faces from ALL meshes
        for face_idx in 0..total_faces {
            let mut values = Vec::new();
            
            // Face index
            values.push(face_idx.to_string());
            
            // Vertex count (always 3 for triangles)
            values.push("3".to_string());
            
            // Add values for uniform primvars (face attributes) from combined arrays
            let mut float_idx = 0;
            let mut float2_idx = 0;
            let mut float3_idx = 0;
            let mut int_idx = 0;
            let mut string_idx = 0;
            
            for primvar in &uniform_primvars {
                match &primvar.values {
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(_) => {
                        if float_idx < all_face_primvars.len() && face_idx < all_face_primvars[float_idx].len() {
                            values.push(format!("{:.3}", all_face_primvars[float_idx][face_idx]));
                        } else {
                            values.push("0.000".to_string());
                        }
                        float_idx += 1;
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(_) => {
                        if float2_idx < all_face_primvars_float2.len() && face_idx < all_face_primvars_float2[float2_idx].len() {
                            let val = all_face_primvars_float2[float2_idx][face_idx];
                            values.push(format!("{:.3}", val.x));
                            values.push(format!("{:.3}", val.y));
                        } else {
                            values.push("0.000".to_string());
                            values.push("0.000".to_string());
                        }
                        float2_idx += 1;
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(_) => {
                        if float3_idx < all_face_primvars_float3.len() && face_idx < all_face_primvars_float3[float3_idx].len() {
                            let val = all_face_primvars_float3[float3_idx][face_idx];
                            values.push(format!("{:.3}", val.x));
                            values.push(format!("{:.3}", val.y));
                            values.push(format!("{:.3}", val.z));
                        } else {
                            values.push("0.000".to_string());
                            values.push("0.000".to_string());
                            values.push("0.000".to_string());
                        }
                        float3_idx += 1;
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(_) => {
                        if int_idx < all_face_primvars_int.len() && face_idx < all_face_primvars_int[int_idx].len() {
                            values.push(all_face_primvars_int[int_idx][face_idx].to_string());
                        } else {
                            values.push("0".to_string());
                        }
                        int_idx += 1;
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(_) => {
                        if string_idx < all_face_primvars_string.len() && face_idx < all_face_primvars_string[string_idx].len() {
                            values.push(all_face_primvars_string[string_idx][face_idx].clone());
                        } else {
                            values.push("".to_string());
                        }
                        string_idx += 1;
                    }
                }
            }
            
            rows.push(GeometryRow {
                element_index: face_idx,
                attribute_values: values,
                row_height: MIN_ROW_HEIGHT,
            });
        }
    }
    
    state.geometry_spreadsheet = GeometrySpreadsheet {
        columns,
        rows,
        element_type: "Face Attributes".to_string(),
    };
}

/// Build face-corner attributes spreadsheet - USD "faceVarying" interpolation (one value per face corner)
fn build_face_corner_attributes_spreadsheet(state: &mut AttributeDisplayState, scene_data: &USDSceneData) {
    let mut columns = Vec::new();
    let mut rows = Vec::new();
    
    // Corner index column
    columns.push(AttributeColumn {
        name: "corner".to_string(),
        type_name: "int".to_string(),
        width: 60.0,
    });
    
    // Point index column (shows which point this corner refers to)
    columns.push(AttributeColumn {
        name: "ptnum".to_string(),
        type_name: "int".to_string(),
        width: 60.0,
    });
    
    // Process ALL meshes to show complete scene data
    if !scene_data.meshes.is_empty() {
        // Take first mesh for column structure
        let first_mesh = &scene_data.meshes[0];
        // Collect ALL unique faceVarying primvars from ALL meshes (not just first mesh)
        let mut all_face_varying_primvars = std::collections::HashMap::new();
        for (mesh_idx, mesh) in scene_data.meshes.iter().enumerate() {
            let mesh_facevarying_primvars: Vec<_> = mesh.primvars.iter().filter(|pv| pv.interpolation == "faceVarying").collect();
            if mesh_idx < 5 && !mesh_facevarying_primvars.is_empty() {  // Debug first 5 meshes
                println!("📊 Mesh[{}] {} has {} faceVarying primvars:", mesh_idx, mesh.prim_path, mesh_facevarying_primvars.len());
                for pv in &mesh_facevarying_primvars {
                    println!("    - {} ({})", pv.name, pv.data_type);
                }
            }
            for primvar in mesh_facevarying_primvars {
                all_face_varying_primvars.insert(primvar.name.clone(), primvar.clone());
            }
        }
        let face_varying_primvars: Vec<_> = all_face_varying_primvars.values().collect();
        println!("📊 Corner Attributes: Found {} unique faceVarying primvars across {} meshes", face_varying_primvars.len(), scene_data.meshes.len());
            
        for primvar in &face_varying_primvars {
            match &primvar.values {
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(_) => {
                    columns.push(AttributeColumn {
                        name: primvar.name.clone(),
                        type_name: "float".to_string(),
                        width: 80.0,
                    });
                }
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(_) => {
                    columns.push(AttributeColumn {
                        name: format!("{}.x", primvar.name),
                        type_name: "float".to_string(),
                        width: 80.0,
                    });
                    columns.push(AttributeColumn {
                        name: format!("{}.y", primvar.name),
                        type_name: "float".to_string(),
                        width: 80.0,
                    });
                }
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(_) => {
                    columns.push(AttributeColumn {
                        name: format!("{}.x", primvar.name),
                        type_name: "float".to_string(),
                        width: 80.0,
                    });
                    columns.push(AttributeColumn {
                        name: format!("{}.y", primvar.name),
                        type_name: "float".to_string(),
                        width: 80.0,
                    });
                    columns.push(AttributeColumn {
                        name: format!("{}.z", primvar.name),
                        type_name: "float".to_string(),
                        width: 80.0,
                    });
                }
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(_) => {
                    columns.push(AttributeColumn {
                        name: primvar.name.clone(),
                        type_name: "int".to_string(),
                        width: 80.0,
                    });
                }
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(_) => {
                    columns.push(AttributeColumn {
                        name: primvar.name.clone(),
                        type_name: "string".to_string(),
                        width: 120.0,
                    });
                }
            }
        }
        
        // Fallback: If no faceVarying primvars, show legacy UVs as example
        if face_varying_primvars.is_empty() && !first_mesh.uvs.is_empty() {
            columns.push(AttributeColumn {
                name: "uv.x".to_string(),
                type_name: "float".to_string(),
                width: 80.0,
            });
            columns.push(AttributeColumn {
                name: "uv.y".to_string(),
                type_name: "float".to_string(),
                width: 80.0,
            });
        }
        
        // Combine ALL corner indices and primvars from ALL meshes for complete scene view
        let mut all_indices = Vec::new();
        let mut all_uvs = Vec::new();
        let mut all_corner_primvars: Vec<Vec<f32>> = Vec::new();
        let mut all_corner_primvars_float2: Vec<Vec<glam::Vec2>> = Vec::new();
        let mut all_corner_primvars_float3: Vec<Vec<glam::Vec3>> = Vec::new();
        let mut all_corner_primvars_int: Vec<Vec<i32>> = Vec::new();
        let mut all_corner_primvars_string: Vec<Vec<String>> = Vec::new();
        
        // Initialize primvar storage based on first mesh structure
        for primvar in &face_varying_primvars {
            match &primvar.values {
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(_) => all_corner_primvars.push(Vec::new()),
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(_) => all_corner_primvars_float2.push(Vec::new()),
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(_) => all_corner_primvars_float3.push(Vec::new()),
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(_) => all_corner_primvars_int.push(Vec::new()),
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(_) => all_corner_primvars_string.push(Vec::new()),
            }
        }
        
        let mut vertex_offset = 0; // Track vertex offset for proper point indexing
        
        // Collect corner data from ALL meshes
        for mesh in &scene_data.meshes {
            // Add indices from this mesh (adjusted by vertex offset)
            let adjusted_indices: Vec<u32> = mesh.indices.iter().map(|&idx| idx + vertex_offset).collect();
            all_indices.extend(adjusted_indices);
            
            // Add UVs from this mesh
            all_uvs.extend(mesh.uvs.iter().cloned());
            
            // Update vertex offset for next mesh
            vertex_offset += mesh.vertices.len() as u32;
            
            // Add primvar data from this mesh (matching by name and type)
            let mut float_idx = 0;
            let mut float2_idx = 0;
            let mut float3_idx = 0;
            let mut int_idx = 0;
            let mut string_idx = 0;
            
            for primvar in &face_varying_primvars {
                // Find matching primvar in current mesh
                if let Some(mesh_primvar) = mesh.primvars.iter().find(|p| p.name == primvar.name && p.interpolation == "faceVarying") {
                    match (&primvar.values, &mesh_primvar.values) {
                        (crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(_), 
                         crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(vals)) => {
                            all_corner_primvars[float_idx].extend(vals.iter().cloned());
                            float_idx += 1;
                        },
                        (crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(_), 
                         crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(vals)) => {
                            all_corner_primvars_float2[float2_idx].extend(vals.iter().cloned());
                            float2_idx += 1;
                        },
                        (crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(_), 
                         crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(vals)) => {
                            all_corner_primvars_float3[float3_idx].extend(vals.iter().cloned());
                            float3_idx += 1;
                        },
                        (crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(_), 
                         crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(vals)) => {
                            all_corner_primvars_int[int_idx].extend(vals.iter().cloned());
                            int_idx += 1;
                        },
                        (crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(_), 
                         crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(vals)) => {
                            all_corner_primvars_string[string_idx].extend(vals.iter().cloned());
                            string_idx += 1;
                        },
                        _ => {
                            // Type mismatch or missing primvar - fill with defaults
                            match &primvar.values {
                                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(_) => {
                                    all_corner_primvars[float_idx].extend(vec![0.0; mesh.indices.len()]);
                                    float_idx += 1;
                                },
                                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(_) => {
                                    all_corner_primvars_float2[float2_idx].extend(vec![glam::Vec2::ZERO; mesh.indices.len()]);
                                    float2_idx += 1;
                                },
                                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(_) => {
                                    all_corner_primvars_float3[float3_idx].extend(vec![glam::Vec3::ZERO; mesh.indices.len()]);
                                    float3_idx += 1;
                                },
                                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(_) => {
                                    all_corner_primvars_int[int_idx].extend(vec![0; mesh.indices.len()]);
                                    int_idx += 1;
                                },
                                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(_) => {
                                    all_corner_primvars_string[string_idx].extend(vec![String::new(); mesh.indices.len()]);
                                    string_idx += 1;
                                },
                            }
                        }
                    }
                } else {
                    // Primvar not found in this mesh - fill with defaults
                    match &primvar.values {
                        crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(_) => {
                            all_corner_primvars[float_idx].extend(vec![0.0; mesh.indices.len()]);
                            float_idx += 1;
                        },
                        crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(_) => {
                            all_corner_primvars_float2[float2_idx].extend(vec![glam::Vec2::ZERO; mesh.indices.len()]);
                            float2_idx += 1;
                        },
                        crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(_) => {
                            all_corner_primvars_float3[float3_idx].extend(vec![glam::Vec3::ZERO; mesh.indices.len()]);
                            float3_idx += 1;
                        },
                        crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(_) => {
                            all_corner_primvars_int[int_idx].extend(vec![0; mesh.indices.len()]);
                            int_idx += 1;
                        },
                        crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(_) => {
                            all_corner_primvars_string[string_idx].extend(vec![String::new(); mesh.indices.len()]);
                            string_idx += 1;
                        },
                    }
                }
            }
        }
        
        let total_corners = all_indices.len();
        
        // Create rows for ALL corners from ALL meshes
        for corner_idx in 0..total_corners {
            let mut values = Vec::new();
            
            // Corner index
            values.push(corner_idx.to_string());
            
            // Point index (from combined faceVertexIndices)
            if corner_idx < all_indices.len() {
                let point_idx = all_indices[corner_idx];
                values.push(point_idx.to_string());
            } else {
                values.push("0".to_string());
            }
            
            // Add values for faceVarying primvars (corner attributes) from combined arrays
            let mut float_idx = 0;
            let mut float2_idx = 0;
            let mut float3_idx = 0;
            let mut int_idx = 0;
            let mut string_idx = 0;
            
            for primvar in &face_varying_primvars {
                match &primvar.values {
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(_) => {
                        if float_idx < all_corner_primvars.len() && corner_idx < all_corner_primvars[float_idx].len() {
                            values.push(format!("{:.3}", all_corner_primvars[float_idx][corner_idx]));
                        } else {
                            values.push("0.000".to_string());
                        }
                        float_idx += 1;
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(_) => {
                        if float2_idx < all_corner_primvars_float2.len() && corner_idx < all_corner_primvars_float2[float2_idx].len() {
                            let val = all_corner_primvars_float2[float2_idx][corner_idx];
                            values.push(format!("{:.3}", val.x));
                            values.push(format!("{:.3}", val.y));
                        } else {
                            values.push("0.000".to_string());
                            values.push("0.000".to_string());
                        }
                        float2_idx += 1;
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(_) => {
                        if float3_idx < all_corner_primvars_float3.len() && corner_idx < all_corner_primvars_float3[float3_idx].len() {
                            let val = all_corner_primvars_float3[float3_idx][corner_idx];
                            values.push(format!("{:.3}", val.x));
                            values.push(format!("{:.3}", val.y));
                            values.push(format!("{:.3}", val.z));
                        } else {
                            values.push("0.000".to_string());
                            values.push("0.000".to_string());
                            values.push("0.000".to_string());
                        }
                        float3_idx += 1;
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(_) => {
                        if int_idx < all_corner_primvars_int.len() && corner_idx < all_corner_primvars_int[int_idx].len() {
                            values.push(all_corner_primvars_int[int_idx][corner_idx].to_string());
                        } else {
                            values.push("0".to_string());
                        }
                        int_idx += 1;
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(_) => {
                        if string_idx < all_corner_primvars_string.len() && corner_idx < all_corner_primvars_string[string_idx].len() {
                            values.push(all_corner_primvars_string[string_idx][corner_idx].clone());
                        } else {
                            values.push("".to_string());
                        }
                        string_idx += 1;
                    }
                }
            }
            
            // Fallback: Legacy UVs if no faceVarying primvars
            if face_varying_primvars.is_empty() && corner_idx < all_uvs.len() {
                let uv = all_uvs[corner_idx];
                values.push(format!("{:.3}", uv.x));
                values.push(format!("{:.3}", uv.y));
            } else if face_varying_primvars.is_empty() {
                values.push("0.000".to_string());
                values.push("0.000".to_string());
            }
            
            rows.push(GeometryRow {
                element_index: corner_idx,
                attribute_values: values,
                row_height: MIN_ROW_HEIGHT,
            });
        }
    }
    
    state.geometry_spreadsheet = GeometrySpreadsheet {
        columns,
        rows,
        element_type: "Face-Corner Attributes".to_string(),
    };
}

/// Build geometry attributes spreadsheet - USD "constant" interpolation (one value per entire mesh)
fn build_geometry_attributes_spreadsheet(state: &mut AttributeDisplayState, scene_data: &USDSceneData) {
    let mut columns = Vec::new();
    let mut rows = Vec::new();
    
    columns.push(AttributeColumn {
        name: "Attribute".to_string(),
        type_name: "string".to_string(),
        width: 150.0,
    });
    columns.push(AttributeColumn {
        name: "Value".to_string(),
        type_name: "string".to_string(),
        width: 200.0,
    });
    
    let mut row_index = 0;
    
    // Add ALL USD attributes from ALL meshes (not just primvars)
    if !scene_data.meshes.is_empty() {
        // First add constant primvars
        let first_mesh = &scene_data.meshes[0];
        let constant_primvars: Vec<_> = first_mesh.primvars.iter()
            .filter(|pv| pv.interpolation == "constant")
            .collect();
            
        for primvar in &constant_primvars {
            let value_str = match &primvar.values {
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(vals) => {
                    if let Some(val) = vals.first() {
                        format!("{:.3}", val)
                    } else {
                        "0.000".to_string()
                    }
                }
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(vals) => {
                    if let Some(val) = vals.first() {
                        format!("({:.3}, {:.3})", val.x, val.y)
                    } else {
                        "(0.000, 0.000)".to_string()
                    }
                }
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(vals) => {
                    if let Some(val) = vals.first() {
                        format!("({:.3}, {:.3}, {:.3})", val.x, val.y, val.z)
                    } else {
                        "(0.000, 0.000, 0.000)".to_string()
                    }
                }
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(vals) => {
                    if let Some(val) = vals.first() {
                        val.to_string()
                    } else {
                        "0".to_string()
                    }
                }
                crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(vals) => {
                    if let Some(val) = vals.first() {
                        format!("\"{}\"", val)
                    } else {
                        "\"\"".to_string()
                    }
                }
            };
            
            rows.push(GeometryRow {
                element_index: row_index,
                attribute_values: vec![primvar.name.clone(), value_str],
                row_height: MIN_ROW_HEIGHT,
            });
            row_index += 1;
        }
        
        // Add separator row
        rows.push(GeometryRow {
            element_index: row_index,
            attribute_values: vec!["--- Primvars ---".to_string(), "".to_string()],
            row_height: MIN_ROW_HEIGHT,
        });
        row_index += 1;
        
        // Add ALL USD attributes from ALL meshes
        for (mesh_idx, mesh) in scene_data.meshes.iter().enumerate() {
            if !mesh.attributes.is_empty() {
                // Add mesh header
                rows.push(GeometryRow {
                    element_index: row_index,
                    attribute_values: vec![format!("Mesh[{}]: {}", mesh_idx, mesh.prim_path), "".to_string()],
                    row_height: MIN_ROW_HEIGHT,
                });
                row_index += 1;
                
                // Add all attributes for this mesh
                for attr in &mesh.attributes {
                    let value_str = match &attr.value {
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::Bool(v) => v.to_string(),
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::Int(v) => v.to_string(),
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::Float(v) => format!("{:.3}", v),
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::Double(v) => format!("{:.6}", v),
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::String(v) => format!("\"{}\"", v),
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::Token(v) => v.clone(),
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::Asset(v) => format!("@{}@", v),
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::Float2(v) => format!("({:.3}, {:.3})", v.x, v.y),
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::Float3(v) => format!("({:.3}, {:.3}, {:.3})", v.x, v.y, v.z),
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::Color3f(v) => format!("color({:.3}, {:.3}, {:.3})", v.x, v.y, v.z),
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::Normal3f(v) => format!("normal({:.3}, {:.3}, {:.3})", v.x, v.y, v.z),
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::Point3f(v) => format!("point({:.3}, {:.3}, {:.3})", v.x, v.y, v.z),
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::Vector3f(v) => format!("vector({:.3}, {:.3}, {:.3})", v.x, v.y, v.z),
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::TexCoord2f(v) => format!("uv({:.3}, {:.3})", v.x, v.y),
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::Matrix4d(m) => {
                            format!("matrix4d[{:.2},{:.2},{:.2},{:.2}...]", m.x_axis.x, m.x_axis.y, m.x_axis.z, m.x_axis.w)
                        },
                        // Array types - show count and first element
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::BoolArray(arr) => {
                            if arr.is_empty() {
                                format!("bool[0]")
                            } else {
                                format!("bool[{}] ({}...)", arr.len(), arr[0])
                            }
                        },
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::IntArray(arr) => {
                            if arr.is_empty() {
                                format!("int[0]")
                            } else {
                                format!("int[{}] ({}...)", arr.len(), arr[0])
                            }
                        },
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::FloatArray(arr) => {
                            if arr.is_empty() {
                                format!("float[0]")
                            } else {
                                format!("float[{}] ({:.3}...)", arr.len(), arr[0])
                            }
                        },
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::DoubleArray(arr) => {
                            if arr.is_empty() {
                                format!("double[0]")
                            } else {
                                format!("double[{}] ({:.6}...)", arr.len(), arr[0])
                            }
                        },
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::StringArray(arr) => {
                            if arr.is_empty() {
                                format!("string[0]")
                            } else {
                                format!("string[{}] (\"{}\"...)", arr.len(), arr[0])
                            }
                        },
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::TokenArray(arr) => {
                            if arr.is_empty() {
                                format!("token[0]")
                            } else {
                                format!("token[{}] ({}...)", arr.len(), arr[0])
                            }
                        },
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::AssetArray(arr) => {
                            if arr.is_empty() {
                                format!("asset[0]")
                            } else {
                                format!("asset[{}] (@{}@...)", arr.len(), arr[0])
                            }
                        },
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::Float2Array(arr) => {
                            if arr.is_empty() {
                                format!("float2[0]")
                            } else {
                                format!("float2[{}] ({:.3},{:.3}...)", arr.len(), arr[0].x, arr[0].y)
                            }
                        },
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::Float3Array(arr) => {
                            if arr.is_empty() {
                                format!("float3[0]")
                            } else {
                                format!("float3[{}] ({:.3},{:.3},{:.3}...)", arr.len(), arr[0].x, arr[0].y, arr[0].z)
                            }
                        },
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::Color3fArray(arr) => {
                            if arr.is_empty() {
                                format!("color3f[0]")
                            } else {
                                format!("color3f[{}] ({:.3},{:.3},{:.3}...)", arr.len(), arr[0].x, arr[0].y, arr[0].z)
                            }
                        },
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::Normal3fArray(arr) => {
                            if arr.is_empty() {
                                format!("normal3f[0]")
                            } else {
                                format!("normal3f[{}] ({:.3},{:.3},{:.3}...)", arr.len(), arr[0].x, arr[0].y, arr[0].z)
                            }
                        },
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::Point3fArray(arr) => {
                            if arr.is_empty() {
                                format!("point3f[0]")
                            } else {
                                format!("point3f[{}] ({:.3},{:.3},{:.3}...)", arr.len(), arr[0].x, arr[0].y, arr[0].z)
                            }
                        },
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::Vector3fArray(arr) => {
                            if arr.is_empty() {
                                format!("vector3f[0]")
                            } else {
                                format!("vector3f[{}] ({:.3},{:.3},{:.3}...)", arr.len(), arr[0].x, arr[0].y, arr[0].z)
                            }
                        },
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::TexCoord2fArray(arr) => {
                            if arr.is_empty() {
                                format!("texCoord2f[0]")
                            } else {
                                format!("texCoord2f[{}] ({:.3},{:.3}...)", arr.len(), arr[0].x, arr[0].y)
                            }
                        },
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::Matrix4dArray(arr) => {
                            format!("matrix4d[{}]", arr.len())
                        },
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::Relationship(paths) => {
                            if paths.is_empty() {
                                format!("relationship[0]")
                            } else {
                                format!("relationship[{}] ({}...)", paths.len(), paths[0])
                            }
                        },
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::TimeSamples(samples) => {
                            format!("timeSamples[{}]", samples.len())
                        },
                        crate::workspaces::three_d::usd::usd_engine::AttributeValue::Unknown(s) => {
                            format!("unknown: {}", s)
                        },
                    };
                    
                    let attr_name = if attr.is_custom {
                        format!("{} [custom]", attr.name)
                    } else {
                        attr.name.clone()
                    };
                    
                    rows.push(GeometryRow {
                        element_index: row_index,
                        attribute_values: vec![attr_name, value_str],
                        row_height: MIN_ROW_HEIGHT,
                    });
                    row_index += 1;
                }
            }
        }
        
        // Add separator row
        rows.push(GeometryRow {
            element_index: row_index,
            attribute_values: vec!["--- Summary ---".to_string(), "".to_string()],
            row_height: MIN_ROW_HEIGHT,
        });
        row_index += 1;
    }
    
    // Add summary information about the geometry
    let detail_info = vec![
        ("Mesh Count", scene_data.meshes.len().to_string()),
        ("Total Vertices", scene_data.meshes.iter().map(|m| m.vertices.len()).sum::<usize>().to_string()),
        ("Total Faces", scene_data.meshes.iter().map(|m| m.indices.len() / 3).sum::<usize>().to_string()),
        ("Light Count", scene_data.lights.len().to_string()),
        ("Material Count", scene_data.materials.len().to_string()),
    ];
    
    for (name, value) in detail_info.iter() {
        rows.push(GeometryRow {
            element_index: row_index,
            attribute_values: vec![name.to_string(), value.to_string()],
            row_height: MIN_ROW_HEIGHT,
        });
        row_index += 1;
    }
    
    state.geometry_spreadsheet = GeometrySpreadsheet {
        columns,
        rows,
        element_type: "Geometry Attributes".to_string(),
    };
}

// NOTE: Sorting is now handled in build_geometry_spreadsheet functions
// This function is no longer needed with the Houdini-style geometry spreadsheet

/// Render the main spreadsheet interface
fn render_spreadsheet_interface(
    ui: &mut Ui,
    state: &mut AttributeDisplayState,
    parameters: &mut HashMap<String, NodeData>,
    changes: &mut Vec<ParameterChange>,
    scene_data: Option<&USDSceneData>,
) {
    // Use standard vertical layout - let content determine natural size
    ui.vertical(|ui| {
        // Render tabs
        render_attribute_tabs(ui, state, changes, scene_data);
        
        ui.add_space(2.0);
        
        // Render filter controls
        render_filter_controls(ui, state, parameters, changes);
        
        ui.add_space(2.0);
        
        // Render spreadsheet content without fixed height
        render_spreadsheet_content(ui, state, changes);
        
        ui.add_space(2.0);
        
        // Render status bar
        render_status_bar(ui, state);
    });
}

/// Render attribute tabs
fn render_attribute_tabs(
    ui: &mut Ui,
    state: &mut AttributeDisplayState,
    changes: &mut Vec<ParameterChange>,
    scene_data: Option<&USDSceneData>,
) {
    let tabs = [
        AttributeTab::PointAttributes,
        AttributeTab::FaceCornerAttributes,
        AttributeTab::FaceAttributes,
        AttributeTab::GeometryAttributes,
    ];
    
    ui.horizontal(|ui| {
        for tab in &tabs {
            let is_selected = state.current_tab == *tab;
            let button_color = if is_selected {
                Color32::from_rgb(100, 100, 140)
            } else {
                Color32::from_rgb(60, 60, 80)
            };
            
            ui.visuals_mut().widgets.inactive.bg_fill = button_color;
            ui.visuals_mut().widgets.hovered.bg_fill = Color32::from_rgb(80, 80, 120);
            
            if ui.button(tab.display_name()).on_hover_text(tab.description()).clicked() {
                if state.current_tab != *tab {
                    state.current_tab = tab.clone();
                    
                    // Rebuild spreadsheet for new tab with scene data
                    if let Some(scene_data) = scene_data {
                        build_geometry_spreadsheet(state, scene_data);
                    } else {
                        // If no scene data, just update the element type
                        state.geometry_spreadsheet.element_type = tab.display_name().to_string();
                    }
                    
                    changes.push(ParameterChange {
                        parameter: "current_tab".to_string(),
                        value: NodeData::String(format!("{:?}", tab)),
                    });
                }
            }
        }
    });
}

/// Render filter controls
fn render_filter_controls(
    ui: &mut Ui,
    state: &mut AttributeDisplayState,
    parameters: &mut HashMap<String, NodeData>,
    changes: &mut Vec<ParameterChange>,
) {
    ui.horizontal(|ui| {
        ui.label("Filter:");
        
        let mut filter_text = state.filter_text.clone();
        let response = ui.add(TextEdit::singleline(&mut filter_text).hint_text("Search attributes..."));
        
        if response.changed() {
            state.filter_text = filter_text;
            // TODO: Implement filtering for geometry spreadsheet if needed
            changes.push(ParameterChange {
                parameter: "filter_text".to_string(),
                value: NodeData::String(state.filter_text.clone()),
            });
        }
        
        ui.separator();
        
        // Primitive selector
        ui.label("Primitive:");
        let current_prim = state.selected_prim.as_deref().unwrap_or("All");
        egui::ComboBox::from_id_salt("prim_selector")
            .selected_text(current_prim)
            .show_ui(ui, |ui| {
                if ui.selectable_value(&mut state.selected_prim, None, "All").clicked() {
                    // TODO: Implement primitive filtering for geometry spreadsheet if needed
                }
                
                let primitives = state.cached_primitives.clone();
                for primitive in &primitives {
                    let path = primitive.path.as_ref();
                    let mut selected = Some(path.to_string());
                    if ui.selectable_value(&mut selected, Some(path.to_string()), path).clicked() {
                        state.selected_prim = selected;
                        // TODO: Rebuild spreadsheet with filtered primitive data
                        changes.push(ParameterChange {
                            parameter: "selected_prim".to_string(),
                            value: NodeData::String(path.to_string()),
                        });
                    }
                }
            });
    });
}

/// Render geometry spreadsheet content (Houdini style) using TableBuilder
fn render_spreadsheet_content(
    ui: &mut Ui,
    state: &mut AttributeDisplayState,
    changes: &mut Vec<ParameterChange>,
) {
    let total_items = state.geometry_spreadsheet.rows.len();
    if total_items == 0 {
        ui.centered_and_justified(|ui| {
            ui.label("No geometry data to display");
        });
        return;
    }
    
    let columns = &state.geometry_spreadsheet.columns;
    
    // Create TableBuilder with proper spreadsheet styling and optimizations
    let mut table_builder = TableBuilder::new(ui)
        .striped(true)  // Alternating row colors like Excel
        .resizable(true)  // Allow column resizing
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .max_scroll_height(600.0)  // Larger viewport for better UX
        .stick_to_bottom(false)  // Don't auto-scroll to bottom
        .auto_shrink([false, true]);  // Don't shrink width, allow height shrinking
    
    // Add columns with appropriate widths
    for column in columns {
        table_builder = table_builder.column(Column::initial(column.width).at_least(60.0).resizable(true));
    }
    
    // Build the table with header and body
    table_builder
        .header(HEADER_HEIGHT, |mut header| {
            for column in columns {
                header.col(|ui| {
                    ui.strong(&column.name);
                    ui.label(format!("({})", column.type_name));
                });
            }
        })
        .body(|mut body| {
            let row_height = MIN_ROW_HEIGHT;
            
            // Use body.rows() for efficient virtual scrolling of many rows
            body.rows(row_height, total_items, |mut row| {
                let row_index = row.index();
                if let Some(geometry_row) = state.geometry_spreadsheet.rows.get(row_index) {
                    // Pre-cache attribute values to avoid repeated indexing
                    let attribute_values = &geometry_row.attribute_values;
                    
                    // Render each column in this row with optimized rendering
                    for (col_index, _column) in columns.iter().enumerate() {
                        row.col(|ui| {
                            if let Some(value) = attribute_values.get(col_index) {
                                // Color code the first column (element index) like Excel row numbers
                                if col_index == 0 {
                                    ui.colored_label(Color32::from_rgb(100, 100, 140), value);
                                } else {
                                    // Use optimized label rendering for data cells
                                    ui.label(value);
                                }
                            }
                        });
                    }
                }
            });
        });
}

// NOTE: Header and row rendering functions removed - now handled by TableBuilder

/// Render status bar
fn render_status_bar(ui: &mut Ui, state: &AttributeDisplayState) {
    ui.horizontal(|ui| {
        // Show geometry spreadsheet information
        let total_rows = state.geometry_spreadsheet.rows.len();
        let total_columns = state.geometry_spreadsheet.columns.len();
        
        ui.label(format!("Showing {} {} with {} attributes", 
            total_rows, 
            state.geometry_spreadsheet.element_type.to_lowercase(),
            total_columns.saturating_sub(1) // Subtract 1 for element index column
        ));
        
        // Show performance info for large datasets
        if total_rows > 10000 {
            ui.separator();
            ui.colored_label(
                egui::Color32::LIGHT_YELLOW,
                format!("Large dataset ({} elements) - using virtual scrolling for performance", total_rows)
            );
        }
        
        if let Some(ref selected_prim) = state.selected_prim {
            ui.separator();
            ui.label(format!("Primitive: {}", selected_prim));
        }
        
        ui.separator();
        ui.label(format!("Tab: {}", state.current_tab.display_name()));
    });
}