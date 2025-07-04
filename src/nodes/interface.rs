//! Node interface panel system for parameter control

use egui::{Ui, DragValue, ComboBox, Color32};
use crate::nodes::NodeId;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Types of interface panels that nodes can specify
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelType {
    /// Parameter panels for node settings (default, positioned top-right)
    Parameter,
    /// Viewer panels for displaying output/results
    Viewer,
    /// Editor panels for complex editing interfaces
    Editor,
    /// Inspector panels for debugging/analysis
    Inspector,
    /// Viewport panels for 3D scene visualization and rendering
    Viewport,
}

/// Core data types that flow between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeData {
    /// Complete 3D scene with geometry, materials, lights
    Scene(SceneData),
    /// Geometric data (meshes, primitives)
    Geometry(GeometryData),
    /// Material and shading data
    Material(MaterialData),
    /// USD stage reference
    Stage(StageData),
    /// Lighting data
    Light(LightData),
    /// Image/texture data
    Image(ImageData),
    /// Generic value types
    Float(f32),
    Integer(i32),
    Vector3([f32; 3]),
    Color([f32; 4]),
    String(String),
    Boolean(bool),
    Any(String), // Generic reference/handle
    None, // Empty/null value
}

/// Scene hierarchy data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneData {
    pub geometry: Vec<GeometryData>,
    pub materials: Vec<MaterialData>,
    pub lights: Vec<LightData>,
    pub transforms: HashMap<String, [[f32; 4]; 4]>, // Transform matrices
}

impl Default for SceneData {
    fn default() -> Self {
        Self {
            geometry: Vec::new(),
            materials: Vec::new(),
            lights: Vec::new(),
            transforms: HashMap::new(),
        }
    }
}

/// Geometry data for 3D objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometryData {
    pub id: String,
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub material_id: Option<String>,
}

/// Material and shading data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialData {
    pub id: String,
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub normal_map: Option<String>,
    pub diffuse_map: Option<String>,
}

/// USD stage reference data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageData {
    pub identifier: String,
    pub file_path: Option<String>,
    pub prims: Vec<String>,
}

/// Lighting data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightData {
    pub id: String,
    pub light_type: LightType,
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LightType {
    Point,
    Directional { direction: [f32; 3] },
    Spot { direction: [f32; 3], cone_angle: f32 },
}

/// Image/texture data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    pub id: String,
    pub file_path: Option<String>,
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageFormat {
    RGB8,
    RGBA8,
    HDR,
}

/// Parameters that can be controlled in interface panels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterfaceParameter {
    Float { value: f32, min: f32, max: f32, step: f32 },
    Integer { value: i32, min: i32, max: i32 },
    Vector3 { value: [f32; 3] },
    Color { value: [f32; 4] },
    String { value: String },
    Boolean { value: bool },
    Enum { value: usize, options: Vec<String> },
    FilePath { value: String, filter: String },
}

impl InterfaceParameter {
    /// Render the parameter in the UI and return if it changed
    pub fn render(&mut self, ui: &mut Ui, label: &str) -> bool {
        match self {
            InterfaceParameter::Float { value, min, max, step } => {
                ui.add(DragValue::new(value)
                    .speed(*step)
                    .clamp_range(*min..=*max)
                    .prefix(format!("{}: ", label)))
                    .changed()
            }
            InterfaceParameter::Integer { value, min, max } => {
                ui.add(DragValue::new(value)
                    .clamp_range(*min..=*max)
                    .prefix(format!("{}: ", label)))
                    .changed()
            }
            InterfaceParameter::Vector3 { value } => {
                ui.horizontal(|ui| {
                    ui.label(label);
                    let mut changed = false;
                    changed |= ui.add(DragValue::new(&mut value[0]).prefix("X:")).changed();
                    changed |= ui.add(DragValue::new(&mut value[1]).prefix("Y:")).changed();
                    changed |= ui.add(DragValue::new(&mut value[2]).prefix("Z:")).changed();
                    changed
                }).inner
            }
            InterfaceParameter::Color { value } => {
                ui.horizontal(|ui| {
                    ui.label(label);
                    let mut color = Color32::from_rgba_premultiplied(
                        (value[0] * 255.0) as u8,
                        (value[1] * 255.0) as u8,
                        (value[2] * 255.0) as u8,
                        (value[3] * 255.0) as u8,
                    );
                    let changed = ui.color_edit_button_srgba(&mut color).changed();
                    if changed {
                        let [r, g, b, a] = color.to_array();
                        value[0] = r as f32 / 255.0;
                        value[1] = g as f32 / 255.0;
                        value[2] = b as f32 / 255.0;
                        value[3] = a as f32 / 255.0;
                    }
                    changed
                }).inner
            }
            InterfaceParameter::String { value } => {
                ui.horizontal(|ui| {
                    ui.label(label);
                    ui.text_edit_singleline(value)
                }).inner.changed()
            }
            InterfaceParameter::Boolean { value } => {
                ui.checkbox(value, label).changed()
            }
            InterfaceParameter::Enum { value, options } => {
                let mut changed = false;
                ComboBox::from_label(label)
                    .selected_text(&options[*value])
                    .show_ui(ui, |ui| {
                        for (i, option) in options.iter().enumerate() {
                            if ui.selectable_value(value, i, option).changed() {
                                changed = true;
                            }
                        }
                    });
                changed
            }
            InterfaceParameter::FilePath { value, filter } => {
                ui.horizontal(|ui| {
                    ui.label(label);
                    let mut changed = ui.text_edit_singleline(value).changed();
                    if ui.button("📁 Browse").clicked() {
                        if let Ok(Some(path)) = Self::open_file_dialog(filter) {
                            *value = path;
                            changed = true;
                        }
                    }
                    changed
                }).inner
            }
        }
    }
    
    /// Get the current value as a generic type
    pub fn get_float(&self) -> Option<f32> {
        match self {
            InterfaceParameter::Float { value, .. } => Some(*value),
            _ => None,
        }
    }
    
    pub fn get_vector3(&self) -> Option<[f32; 3]> {
        match self {
            InterfaceParameter::Vector3 { value } => Some(*value),
            _ => None,
        }
    }
    
    pub fn get_string(&self) -> Option<&str> {
        match self {
            InterfaceParameter::String { value } => Some(value),
            InterfaceParameter::FilePath { value, .. } => Some(value),
            _ => None,
        }
    }
    
    pub fn get_bool(&self) -> Option<bool> {
        match self {
            InterfaceParameter::Boolean { value } => Some(*value),
            _ => None,
        }
    }
    
    pub fn get_int(&self) -> Option<i32> {
        match self {
            InterfaceParameter::Integer { value, .. } => Some(*value),
            _ => None,
        }
    }
    
    /// Open a file dialog with the specified filter
    fn open_file_dialog(filter: &str) -> Result<Option<String>, String> {
        use rfd::FileDialog;
        
        let mut dialog = FileDialog::new();
        
        // Parse filter string and add appropriate file extensions
        if filter.contains("USD") {
            dialog = dialog.add_filter("USD Files", &["usd", "usda", "usdc", "usdz"]);
        }
        
        // Add common filters
        dialog = dialog.add_filter("All Files", &["*"]);
        
        if let Some(path) = dialog.pick_file() {
            if let Some(path_str) = path.to_str() {
                Ok(Some(path_str.to_string()))
            } else {
                Err("Invalid file path encoding".to_string())
            }
        } else {
            Ok(None) // User cancelled dialog
        }
    }
}

/// Trait for nodes that have interface panels
pub trait NodeInterfacePanel: Send + Sync {
    /// Get the type of panel this node uses
    fn panel_type(&self) -> PanelType { PanelType::Parameter }
    
    /// Get the parameters that should be shown in the interface panel
    fn get_parameters(&self) -> Vec<(&'static str, InterfaceParameter)>;
    
    /// Update the node with new parameter values
    fn set_parameters(&mut self, parameters: Vec<(&'static str, InterfaceParameter)>);
    
    /// Process the node with current parameters and input data
    fn process(&self, inputs: Vec<NodeData>) -> Vec<NodeData>;
    
    /// Get the display name for the interface panel
    fn panel_title(&self) -> String;
    
    /// Check if this node should show an interface panel
    fn has_interface_panel(&self) -> bool { true }
    
    /// Render custom UI elements (beyond standard parameters)
    fn render_custom_ui(&mut self, _ui: &mut Ui) -> bool { false }
}

/// Manager for all node interface panels
#[derive(Default)]
pub struct InterfacePanelManager {
    /// Which nodes have visible interface panels
    visible_panels: HashMap<NodeId, bool>,
    /// Which panels are minimized (only title bar visible)
    minimized_panels: HashMap<NodeId, bool>,
    /// Which panels are open (for egui window state tracking)
    open_panels: HashMap<NodeId, bool>,
    /// Custom node names (overrides default node title)
    node_names: HashMap<NodeId, String>,
    /// Whether nodes should fit their name horizontally
    fit_name_flags: HashMap<NodeId, bool>,
    /// Cached parameter values for each node
    parameter_cache: HashMap<NodeId, Vec<(&'static str, InterfaceParameter)>>,
    /// Panel positions and sizes
    panel_rects: HashMap<NodeId, egui::Rect>,
    /// Which panels are stacked (grouped together)
    stacked_panels: HashMap<NodeId, bool>,
    /// Which panels are pinned (stay on top and locked position)
    pinned_panels: HashMap<NodeId, bool>,
    /// Panel types for each node (for type-specific stacking)
    panel_types: HashMap<NodeId, PanelType>,
}

impl InterfacePanelManager {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Toggle visibility of a node's interface panel
    pub fn toggle_panel_visibility(&mut self, node_id: NodeId) {
        let current = self.visible_panels.get(&node_id).copied().unwrap_or(false);
        self.visible_panels.insert(node_id, !current);
    }
    
    /// Set panel visibility
    pub fn set_panel_visibility(&mut self, node_id: NodeId, visible: bool) {
        self.visible_panels.insert(node_id, visible);
    }
    
    /// Check if a panel is visible
    pub fn is_panel_visible(&self, node_id: NodeId) -> bool {
        self.visible_panels.get(&node_id).copied().unwrap_or(false)
    }
    
    /// Set panel minimized state
    pub fn set_panel_minimized(&mut self, node_id: NodeId, minimized: bool) {
        self.minimized_panels.insert(node_id, minimized);
    }
    
    /// Check if a panel is minimized
    pub fn is_panel_minimized(&self, node_id: NodeId) -> bool {
        self.minimized_panels.get(&node_id).copied().unwrap_or(false)
    }
    
    /// Set panel open state
    pub fn set_panel_open(&mut self, node_id: NodeId, open: bool) {
        self.open_panels.insert(node_id, open);
    }
    
    /// Check if a panel is open (for egui window state)
    pub fn is_panel_open(&self, node_id: NodeId) -> bool {
        self.open_panels.get(&node_id).copied().unwrap_or(true)
    }
    
    /// Get mutable reference to panel open state
    pub fn get_panel_open_mut(&mut self, node_id: NodeId) -> &mut bool {
        self.open_panels.entry(node_id).or_insert(true)
    }
    
    /// Set custom node name
    pub fn set_node_name(&mut self, node_id: NodeId, name: String) {
        self.node_names.insert(node_id, name);
    }
    
    /// Get custom node name (returns None if using default)
    pub fn get_node_name(&self, node_id: NodeId) -> Option<&String> {
        self.node_names.get(&node_id)
    }
    
    /// Set fit name flag
    pub fn set_fit_name(&mut self, node_id: NodeId, fit: bool) {
        self.fit_name_flags.insert(node_id, fit);
    }
    
    /// Get fit name flag
    pub fn get_fit_name(&self, node_id: NodeId) -> bool {
        self.fit_name_flags.get(&node_id).copied().unwrap_or(false)
    }
    
    /// Toggle panel stacked state
    pub fn toggle_panel_stacked(&mut self, node_id: NodeId) {
        let panel_type = self.get_panel_type(node_id);
        let default_stacked = if panel_type == PanelType::Viewport { false } else { true };
        let current = self.stacked_panels.get(&node_id).copied().unwrap_or(default_stacked);
        self.stacked_panels.insert(node_id, !current);
    }
    
    /// Set panel stacked state
    pub fn set_panel_stacked(&mut self, node_id: NodeId, stacked: bool) {
        self.stacked_panels.insert(node_id, stacked);
    }
    
    /// Check if a panel is stacked
    pub fn is_panel_stacked(&self, node_id: NodeId) -> bool {
        // Viewport panels should never be stacked by default
        let panel_type = self.get_panel_type(node_id);
        if panel_type == PanelType::Viewport {
            self.stacked_panels.get(&node_id).copied().unwrap_or(false) // Viewport panels default to unstacked
        } else {
            self.stacked_panels.get(&node_id).copied().unwrap_or(true) // Other panels default to stacked
        }
    }
    
    /// Toggle panel pinned state
    pub fn toggle_panel_pinned(&mut self, node_id: NodeId) {
        let current = self.pinned_panels.get(&node_id).copied().unwrap_or(false);
        self.pinned_panels.insert(node_id, !current);
    }
    
    /// Set panel pinned state
    pub fn set_panel_pinned(&mut self, node_id: NodeId, pinned: bool) {
        self.pinned_panels.insert(node_id, pinned);
    }
    
    /// Check if a panel is pinned
    pub fn is_panel_pinned(&self, node_id: NodeId) -> bool {
        self.pinned_panels.get(&node_id).copied().unwrap_or(false)
    }
    
    /// Set panel type
    pub fn set_panel_type(&mut self, node_id: NodeId, panel_type: PanelType) {
        // Debug: Log panel type changes to help track contamination
        if let Some(old_type) = self.panel_types.get(&node_id) {
            if *old_type != panel_type {
                eprintln!("WARNING: Panel type changed for node {}: {:?} -> {:?}", 
                         node_id, old_type, panel_type);
            }
        } else {
            eprintln!("DEBUG: Setting panel type for node {} to {:?}", node_id, panel_type);
        }
        
        self.panel_types.insert(node_id, panel_type);
    }
    
    /// Get panel type (defaults to Parameter)
    pub fn get_panel_type(&self, node_id: NodeId) -> PanelType {
        self.panel_types.get(&node_id).copied().unwrap_or(PanelType::Parameter)
    }
    
    /// Check if panel type has been explicitly set for a node
    pub fn has_panel_type_set(&self, node_id: NodeId) -> bool {
        self.panel_types.contains_key(&node_id)
    }
    
    /// Check if stacking preference has been explicitly set for a node
    pub fn has_stacking_preference_set(&self, node_id: NodeId) -> bool {
        self.stacked_panels.contains_key(&node_id)
    }
    
    // DEPRECATED: Panel type detection removed in favor of node self-assignment
    // Nodes now carry their own panel_type field and assign themselves
    // to the appropriate panel type when created
    
    /// Check if two panels can stack together based on their types
    pub fn can_stack_with(&self, node_id: NodeId, other_node_id: NodeId) -> bool {
        let self_type = self.get_panel_type(node_id);
        let other_type = self.get_panel_type(other_node_id);
        
        match (self_type, other_type) {
            // Viewport panels only stack with other viewport panels
            (PanelType::Viewport, PanelType::Viewport) => true,
            (PanelType::Viewport, _) | (_, PanelType::Viewport) => false,
            // All other types can stack together
            _ => true,
        }
    }
    
    /// Get all visible stacked panels of a specific type
    pub fn get_stacked_panels_by_type(&self, panel_type: PanelType, viewed_nodes: &HashMap<NodeId, crate::nodes::Node>) -> Vec<NodeId> {
        let mut panels: Vec<NodeId> = viewed_nodes.keys()
            .filter(|&&node_id| {
                let node = &viewed_nodes[&node_id];
                node.visible 
                && self.is_panel_visible(node_id)
                && self.is_panel_stacked(node_id) 
                && node.get_panel_type() == Some(panel_type)
            })
            .copied()
            .collect();
        
        // Sort by node ID to ensure consistent ordering
        // This prevents the "first" node from changing randomly
        panels.sort();
        panels
    }
    
    /// Render interface panel for a node
    pub fn render_panel<T: NodeInterfacePanel>(
        &mut self,
        ctx: &egui::Context,
        node_id: NodeId,
        node: &mut T,
        position: egui::Pos2,
    ) -> Option<egui::Response> {
        if !self.is_panel_visible(node_id) {
            return None;
        }
        
        let panel_id = egui::Id::new(format!("interface_panel_{}", node_id));
        
        // TODO: Move this to use NodeEditor.create_window() for consistency
        // For now, manually apply menu bar constraint since this module doesn't have access to NodeEditor
        Some(egui::Window::new(node.panel_title())
            .id(panel_id)
            .default_pos(position + egui::vec2(200.0, 0.0))
            .resizable(true)
            .collapsible(false)
            .constrain_to(egui::Rect::from_min_size(
                egui::Pos2::new(0.0, 34.0), // Hardcoded menu bar height for now
                egui::Vec2::new(ctx.screen_rect().width(), ctx.screen_rect().height() - 34.0)
            ))
            .show(ctx, |ui| {
                // Get current parameters
                let mut parameters = node.get_parameters();
                let mut changed = false;
                
                // Render standard parameters
                for (name, param) in &mut parameters {
                    if param.render(ui, name) {
                        changed = true;
                    }
                }
                
                ui.separator();
                
                // Render custom UI
                if node.render_custom_ui(ui) {
                    changed = true;
                }
                
                // Update node if parameters changed
                if changed {
                    node.set_parameters(parameters.clone());
                    self.parameter_cache.insert(node_id, parameters);
                }
                
                // Close button
                ui.separator();
                if ui.button("Close Panel").clicked() {
                    self.set_panel_visibility(node_id, false);
                }
            })?
            .response)
    }
    
    /// Get cached parameters for a node
    pub fn get_cached_parameters(&self, node_id: NodeId) -> Option<&Vec<(&'static str, InterfaceParameter)>> {
        self.parameter_cache.get(&node_id)
    }
    
    /// Clear cache for a node
    pub fn clear_cache(&mut self, node_id: NodeId) {
        self.parameter_cache.remove(&node_id);
    }
}

/// Helper macros for creating interface parameters
#[macro_export]
macro_rules! interface_float {
    ($value:expr) => {
        InterfaceParameter::Float { value: $value, min: 0.0, max: 100.0, step: 0.1 }
    };
    ($value:expr, $min:expr, $max:expr) => {
        InterfaceParameter::Float { value: $value, min: $min, max: $max, step: 0.1 }
    };
    ($value:expr, $min:expr, $max:expr, $step:expr) => {
        InterfaceParameter::Float { value: $value, min: $min, max: $max, step: $step }
    };
}

#[macro_export]
macro_rules! interface_vector3 {
    ($x:expr, $y:expr, $z:expr) => {
        InterfaceParameter::Vector3 { value: [$x, $y, $z] }
    };
}

#[macro_export]
macro_rules! interface_enum {
    ($value:expr, $($option:expr),*) => {
        InterfaceParameter::Enum { 
            value: $value, 
            options: vec![$(String::from($option)),*] 
        }
    };
}

/// Change notifications for parameter updates
#[derive(Debug, Clone)]
pub struct ParameterChange {
    pub parameter: String,
    pub value: NodeData,
}

/// Build parameter UI for a single parameter with custom renderer
pub fn build_parameter_ui<F>(
    ui: &mut Ui,
    param_name: &str,
    display_name: &str,
    current_value: NodeData,
    render_fn: F,
) -> Option<NodeData>
where
    F: FnOnce(&mut Ui, NodeData) -> Option<NodeData>,
{
    render_fn(ui, current_value)
}