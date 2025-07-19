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

// CRITICAL: Limit attributes to prevent UI freezing
const MAX_ATTRIBUTES_TO_DISPLAY: usize = 500; // Absolute maximum for UI performance
const MAX_PRIMITIVES_TO_PROCESS: usize = 100; // Limit number of primitives processed

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
    
    // Take first mesh for sampling
    if let Some(first_mesh) = scene_data.meshes.first() {
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
        
        // Add columns for USD primvars with "vertex" interpolation (point attributes)
        let vertex_primvars: Vec<_> = first_mesh.primvars.iter()
            .filter(|pv| pv.interpolation == "vertex")
            .collect();
            
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
        
        // Limit points for performance
        const MAX_POINTS_TO_SHOW: usize = 1000;
        let num_points = first_mesh.vertices.len().min(MAX_POINTS_TO_SHOW);
        
        // Create rows for each point
        for i in 0..num_points {
            let mut values = Vec::new();
            
            // Point index
            values.push(i.to_string());
            
            // Position (from points array)
            if i < first_mesh.vertices.len() {
                let vertex = first_mesh.vertices[i];
                values.push(format!("{:.3}", vertex.x));
                values.push(format!("{:.3}", vertex.y));
                values.push(format!("{:.3}", vertex.z));
            } else {
                values.push("0.000".to_string());
                values.push("0.000".to_string());
                values.push("0.000".to_string());
            }
            
            // Add values for vertex primvars (point attributes)
            for primvar in &vertex_primvars {
                match &primvar.values {
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(vals) => {
                        if i < vals.len() {
                            values.push(format!("{:.3}", vals[i]));
                        } else {
                            values.push("0.000".to_string());
                        }
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(vals) => {
                        if i < vals.len() {
                            values.push(format!("{:.3}", vals[i].x));
                            values.push(format!("{:.3}", vals[i].y));
                        } else {
                            values.push("0.000".to_string());
                            values.push("0.000".to_string());
                        }
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(vals) => {
                        if i < vals.len() {
                            values.push(format!("{:.3}", vals[i].x));
                            values.push(format!("{:.3}", vals[i].y));
                            values.push(format!("{:.3}", vals[i].z));
                        } else {
                            values.push("0.000".to_string());
                            values.push("0.000".to_string());
                            values.push("0.000".to_string());
                        }
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(vals) => {
                        if i < vals.len() {
                            values.push(vals[i].to_string());
                        } else {
                            values.push("0".to_string());
                        }
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(vals) => {
                        if i < vals.len() {
                            values.push(vals[i].clone());
                        } else {
                            values.push("".to_string());
                        }
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
    
    // Take first mesh for sampling
    if let Some(first_mesh) = scene_data.meshes.first() {
        // Add columns for USD primvars with "uniform" interpolation (face attributes)
        let uniform_primvars: Vec<_> = first_mesh.primvars.iter()
            .filter(|pv| pv.interpolation == "uniform")
            .collect();
            
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
        
        const MAX_FACES_TO_SHOW: usize = 1000;
        let num_faces = (first_mesh.indices.len() / 3).min(MAX_FACES_TO_SHOW);
        
        // Create rows for each face (triangle)
        for face_idx in 0..num_faces {
            let mut values = Vec::new();
            
            // Face index
            values.push(face_idx.to_string());
            
            // Vertex count (always 3 for triangles)
            values.push("3".to_string());
            
            // Add values for uniform primvars (face attributes)
            for primvar in &uniform_primvars {
                match &primvar.values {
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(vals) => {
                        if face_idx < vals.len() {
                            values.push(format!("{:.3}", vals[face_idx]));
                        } else {
                            values.push("0.000".to_string());
                        }
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(vals) => {
                        if face_idx < vals.len() {
                            values.push(format!("{:.3}", vals[face_idx].x));
                            values.push(format!("{:.3}", vals[face_idx].y));
                        } else {
                            values.push("0.000".to_string());
                            values.push("0.000".to_string());
                        }
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(vals) => {
                        if face_idx < vals.len() {
                            values.push(format!("{:.3}", vals[face_idx].x));
                            values.push(format!("{:.3}", vals[face_idx].y));
                            values.push(format!("{:.3}", vals[face_idx].z));
                        } else {
                            values.push("0.000".to_string());
                            values.push("0.000".to_string());
                            values.push("0.000".to_string());
                        }
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(vals) => {
                        if face_idx < vals.len() {
                            values.push(vals[face_idx].to_string());
                        } else {
                            values.push("0".to_string());
                        }
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(vals) => {
                        if face_idx < vals.len() {
                            values.push(vals[face_idx].clone());
                        } else {
                            values.push("".to_string());
                        }
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
    
    // Take first mesh for sampling
    if let Some(first_mesh) = scene_data.meshes.first() {
        // Add columns for USD primvars with "faceVarying" interpolation (corner attributes)
        let face_varying_primvars: Vec<_> = first_mesh.primvars.iter()
            .filter(|pv| pv.interpolation == "faceVarying")
            .collect();
            
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
        
        // Limit corners for performance
        const MAX_CORNERS_TO_SHOW: usize = 3000; // 1000 faces * 3 corners
        let num_corners = first_mesh.indices.len().min(MAX_CORNERS_TO_SHOW);
        
        // Create rows for each corner (face vertex index)
        for corner_idx in 0..num_corners {
            let mut values = Vec::new();
            
            // Corner index
            values.push(corner_idx.to_string());
            
            // Point index (from faceVertexIndices)
            if corner_idx < first_mesh.indices.len() {
                let point_idx = first_mesh.indices[corner_idx];
                values.push(point_idx.to_string());
            } else {
                values.push("0".to_string());
            }
            
            // Add values for faceVarying primvars (corner attributes)
            for primvar in &face_varying_primvars {
                match &primvar.values {
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float(vals) => {
                        if corner_idx < vals.len() {
                            values.push(format!("{:.3}", vals[corner_idx]));
                        } else {
                            values.push("0.000".to_string());
                        }
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float2(vals) => {
                        if corner_idx < vals.len() {
                            values.push(format!("{:.3}", vals[corner_idx].x));
                            values.push(format!("{:.3}", vals[corner_idx].y));
                        } else {
                            values.push("0.000".to_string());
                            values.push("0.000".to_string());
                        }
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Float3(vals) => {
                        if corner_idx < vals.len() {
                            values.push(format!("{:.3}", vals[corner_idx].x));
                            values.push(format!("{:.3}", vals[corner_idx].y));
                            values.push(format!("{:.3}", vals[corner_idx].z));
                        } else {
                            values.push("0.000".to_string());
                            values.push("0.000".to_string());
                            values.push("0.000".to_string());
                        }
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::Int(vals) => {
                        if corner_idx < vals.len() {
                            values.push(vals[corner_idx].to_string());
                        } else {
                            values.push("0".to_string());
                        }
                    }
                    crate::workspaces::three_d::usd::usd_engine::PrimvarValues::String(vals) => {
                        if corner_idx < vals.len() {
                            values.push(vals[corner_idx].clone());
                        } else {
                            values.push("".to_string());
                        }
                    }
                }
            }
            
            // Fallback: Legacy UVs if no faceVarying primvars
            if face_varying_primvars.is_empty() && !first_mesh.uvs.is_empty() {
                if corner_idx < first_mesh.uvs.len() {
                    let uv = first_mesh.uvs[corner_idx];
                    values.push(format!("{:.3}", uv.x));
                    values.push(format!("{:.3}", uv.y));
                } else {
                    values.push("0.000".to_string());
                    values.push("0.000".to_string());
                }
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
    
    // Add USD constant primvars from the first mesh
    if let Some(first_mesh) = scene_data.meshes.first() {
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
    
    // Create TableBuilder with proper spreadsheet styling
    let mut table_builder = TableBuilder::new(ui)
        .striped(true)  // Alternating row colors like Excel
        .resizable(true)  // Allow column resizing
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .max_scroll_height(400.0);  // Cap height and add scrolling
    
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
            
            // Use body.rows() for efficient rendering of many rows
            body.rows(row_height, total_items, |mut row| {
                let row_index = row.index();
                if let Some(geometry_row) = state.geometry_spreadsheet.rows.get(row_index) {
                    // Render each column in this row
                    for (col_index, _column) in columns.iter().enumerate() {
                        row.col(|ui| {
                            let empty_string = String::new();
                            let value = geometry_row.attribute_values.get(col_index).unwrap_or(&empty_string);
                            
                            // Color code the first column (element index) like Excel row numbers
                            if col_index == 0 {
                                ui.colored_label(Color32::from_rgb(100, 100, 140), value);
                            } else {
                                ui.label(value);
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
        
        // Show sampling warning if data was limited
        const MAX_ELEMENTS_TO_SHOW: usize = 1000;
        if total_rows >= MAX_ELEMENTS_TO_SHOW {
            ui.separator();
            ui.colored_label(
                egui::Color32::LIGHT_RED,
                format!("Limited to {} elements for performance", MAX_ELEMENTS_TO_SHOW)
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