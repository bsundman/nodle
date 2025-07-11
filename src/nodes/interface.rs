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
    /// Complete USD scene data with full geometry
    USDSceneData(crate::workspaces::three_d::usd::usd_engine::USDSceneData),
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
                    .range(*min..=*max)
                    .prefix(format!("{}: ", label)))
                    .changed()
            }
            InterfaceParameter::Integer { value, min, max } => {
                ui.add(DragValue::new(value)
                    .range(*min..=*max)
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

/// State for a single node's panel
#[derive(Default, Clone)]
pub struct NodePanelState {
    /// Whether the panel is visible
    pub visible: bool,
    /// Whether the panel is minimized (only title bar visible)
    pub minimized: bool,
    /// Whether the panel is open (for egui window state tracking)
    pub open: bool,
    /// Whether the panel is stacked (grouped together)
    pub stacked: bool,
    /// Whether the panel is pinned (stay on top and locked position)
    pub pinned: bool,
    /// Panel type for type-specific stacking
    pub panel_type: Option<PanelType>,
    /// Custom node name (overrides default node title)
    pub custom_name: Option<String>,
    /// Whether node should fit its name horizontally
    pub fit_name: bool,
    /// Panel position and size
    pub rect: Option<egui::Rect>,
    /// Original position before stacking
    pub original_position: Option<egui::Pos2>,
    /// Unstacked panel ID for avoiding conflicts
    pub unstacked_panel_id: Option<u64>,
    /// Cached parameter values
    pub parameter_cache: Vec<(&'static str, InterfaceParameter)>,
}

impl NodePanelState {
    /// Create a new panel state with sensible defaults
    pub fn new() -> Self {
        Self {
            visible: false,
            minimized: false,
            open: true,
            stacked: true, // Default to stacked as per current behavior
            pinned: false,
            panel_type: None,
            custom_name: None,
            fit_name: false,
            rect: None,
            original_position: None,
            unstacked_panel_id: None,
            parameter_cache: Vec::new(),
        }
    }

    /// Create a new panel state with specific panel type
    pub fn with_panel_type(panel_type: PanelType) -> Self {
        Self {
            panel_type: Some(panel_type),
            stacked: true, // All panels default to stacked
            ..Self::new()
        }
    }
}

/// Manager for all node interface panels - now simplified with consolidated state
#[derive(Default)]
pub struct InterfacePanelManager {
    /// All node panel states in a single consolidated structure
    node_states: HashMap<NodeId, NodePanelState>,
    /// Global stacking state per panel type
    stacking_initiators: HashMap<PanelType, NodeId>,
    stacking_order: HashMap<PanelType, Vec<NodeId>>,
    stack_positions: HashMap<PanelType, egui::Pos2>,
}

impl InterfacePanelManager {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Get or create panel state for a node
    fn get_or_create_state(&mut self, node_id: NodeId) -> &mut NodePanelState {
        self.node_states.entry(node_id).or_insert_with(NodePanelState::new)
    }
    
    /// Get panel state for a node (read-only)
    fn get_state(&self, node_id: NodeId) -> Option<&NodePanelState> {
        self.node_states.get(&node_id)
    }
    
    /// Toggle visibility of a node's interface panel
    pub fn toggle_panel_visibility(&mut self, node_id: NodeId) {
        let state = self.get_or_create_state(node_id);
        state.visible = !state.visible;
    }
    
    /// Set panel visibility
    pub fn set_panel_visibility(&mut self, node_id: NodeId, visible: bool) {
        self.get_or_create_state(node_id).visible = visible;
    }
    
    /// Check if a panel is visible
    pub fn is_panel_visible(&self, node_id: NodeId) -> bool {
        self.get_state(node_id).map(|s| s.visible).unwrap_or(false)
    }
    
    /// Set panel minimized state
    pub fn set_panel_minimized(&mut self, node_id: NodeId, minimized: bool) {
        self.get_or_create_state(node_id).minimized = minimized;
    }
    
    /// Check if a panel is minimized
    pub fn is_panel_minimized(&self, node_id: NodeId) -> bool {
        self.get_state(node_id).map(|s| s.minimized).unwrap_or(false)
    }
    
    /// Set panel open state
    pub fn set_panel_open(&mut self, node_id: NodeId, open: bool) {
        self.get_or_create_state(node_id).open = open;
    }
    
    /// Check if a panel is open (for egui window state)
    pub fn is_panel_open(&self, node_id: NodeId) -> bool {
        self.get_state(node_id).map(|s| s.open).unwrap_or(true)
    }
    
    /// Get mutable reference to panel open state
    pub fn get_panel_open_mut(&mut self, node_id: NodeId) -> &mut bool {
        &mut self.get_or_create_state(node_id).open
    }
    
    /// Set custom node name
    pub fn set_node_name(&mut self, node_id: NodeId, name: String) {
        self.get_or_create_state(node_id).custom_name = Some(name);
    }
    
    /// Get custom node name (returns None if using default)
    pub fn get_node_name(&self, node_id: NodeId) -> Option<&String> {
        self.get_state(node_id).and_then(|s| s.custom_name.as_ref())
    }
    
    /// Set fit name flag
    pub fn set_fit_name(&mut self, node_id: NodeId, fit: bool) {
        self.get_or_create_state(node_id).fit_name = fit;
    }
    
    /// Get fit name flag
    pub fn get_fit_name(&self, node_id: NodeId) -> bool {
        self.get_state(node_id).map(|s| s.fit_name).unwrap_or(false)
    }
    
    /// Toggle panel stacked state
    pub fn toggle_panel_stacked(&mut self, node_id: NodeId) {
        let state = self.get_or_create_state(node_id);
        state.stacked = !state.stacked;
        log::debug!("Node {} stacked state changed to {}", node_id, state.stacked);
    }
    
    /// Set panel stacked state
    pub fn set_panel_stacked(&mut self, node_id: NodeId, stacked: bool) {
        self.get_or_create_state(node_id).stacked = stacked;
    }
    
    /// Check if a panel is stacked
    pub fn is_panel_stacked(&self, node_id: NodeId) -> bool {
        // All panels (including viewport) default to stacked
        self.get_state(node_id).map(|s| s.stacked).unwrap_or(true)
    }
    
    /// Get the node that initiated stacking for a given panel type
    pub fn get_stacking_initiator(&self, panel_type: PanelType) -> Option<NodeId> {
        self.stacking_initiators.get(&panel_type).copied()
    }
    
    /// Check if there are any stacked panels of the given type
    pub fn has_stacked_panels_of_type(&self, panel_type: PanelType) -> bool {
        self.node_states.iter()
            .filter(|(_, state)| state.panel_type == Some(panel_type))
            .any(|(_, state)| state.stacked)
    }
    
    /// Clean up stacking initiator if no panels of that type are stacked anymore
    pub fn cleanup_stacking_initiator(&mut self, panel_type: PanelType) {
        // Count how many panels are currently stacked for this type
        let stacked_count = self.node_states.iter()
            .filter(|(_, state)| state.panel_type == Some(panel_type))
            .filter(|(_, state)| state.stacked)
            .count();
        
        // Only clean up if there are NO stacked panels left
        // If there's still 1 panel stacked, keep the stack state to maintain position
        if stacked_count == 0 {
            if self.stacking_initiators.remove(&panel_type).is_some() {
                log::debug!("Cleaned up stacking initiator for {:?} (no stacked panels remaining)", panel_type);
            }
            if self.stacking_order.remove(&panel_type).is_some() {
                log::debug!("Cleaned up stacking order for {:?} (no stacked panels remaining)", panel_type);
            }
            if self.stack_positions.remove(&panel_type).is_some() {
                log::debug!("Cleaned up stack position for {:?} (no stacked panels remaining)", panel_type);
            }
        } else {
            log::debug!("Keeping stack state for {:?} ({} stacked panels remaining)", panel_type, stacked_count);
        }
    }
    
    /// Toggle panel pinned state
    pub fn toggle_panel_pinned(&mut self, node_id: NodeId) {
        let state = self.get_or_create_state(node_id);
        state.pinned = !state.pinned;
    }
    
    /// Set panel pinned state
    pub fn set_panel_pinned(&mut self, node_id: NodeId, pinned: bool) {
        self.get_or_create_state(node_id).pinned = pinned;
    }
    
    /// Check if a panel is pinned
    pub fn is_panel_pinned(&self, node_id: NodeId) -> bool {
        self.get_state(node_id).map(|s| s.pinned).unwrap_or(false)
    }
    
    /// Set panel type
    pub fn set_panel_type(&mut self, node_id: NodeId, panel_type: PanelType) {
        let state = self.get_or_create_state(node_id);
        
        // Log panel type changes to help track contamination
        if let Some(old_type) = state.panel_type {
            if old_type != panel_type {
                log::warn!("Panel type changed for node {}: {:?} -> {:?}", 
                         node_id, old_type, panel_type);
            }
        } else {
            log::debug!("Setting panel type for node {} to {:?}", node_id, panel_type);
        }
        
        state.panel_type = Some(panel_type);
    }
    
    /// Get panel type (defaults to Parameter)
    pub fn get_panel_type(&self, node_id: NodeId) -> PanelType {
        self.get_state(node_id)
            .and_then(|s| s.panel_type)
            .unwrap_or(PanelType::Parameter)
    }
    
    /// Check if panel type has been explicitly set for a node
    pub fn has_panel_type_set(&self, node_id: NodeId) -> bool {
        self.get_state(node_id)
            .map(|s| s.panel_type.is_some())
            .unwrap_or(false)
    }
    
    /// Check if stacking preference has been explicitly set for a node
    pub fn has_stacking_preference_set(&self, node_id: NodeId) -> bool {
        self.node_states.contains_key(&node_id)
    }
    
    /// Store original position before stacking
    pub fn store_original_position(&mut self, node_id: NodeId, position: egui::Pos2) {
        let state = self.get_or_create_state(node_id);
        // Only store if not already stored (preserve the very first position)
        if state.original_position.is_none() {
            state.original_position = Some(position);
            log::debug!("Stored original position for node {} at ({:.1}, {:.1})", 
                node_id, position.x, position.y);
        }
    }
    
    /// Get original position for a node
    pub fn get_original_position(&self, node_id: NodeId) -> Option<egui::Pos2> {
        self.get_state(node_id).and_then(|s| s.original_position)
    }
    
    /// Clear original position when no longer needed
    pub fn clear_original_position(&mut self, node_id: NodeId) {
        if let Some(state) = self.node_states.get_mut(&node_id) {
            if state.original_position.take().is_some() {
                log::debug!("Cleared original position for node {}", node_id);
            }
        }
    }
    
    /// Clear all stored positions for a node (both original and unstacked IDs)
    pub fn clear_all_positions(&mut self, node_id: NodeId) {
        if let Some(state) = self.node_states.get_mut(&node_id) {
            if state.original_position.take().is_some() {
                log::debug!("Cleared original position for node {}", node_id);
            }
            if state.unstacked_panel_id.take().is_some() {
                log::debug!("Cleared unstacked panel ID for node {}", node_id);
            }
        }
    }
    
    /// Store stack position when first stack is created
    pub fn store_stack_position(&mut self, panel_type: PanelType, position: egui::Pos2) {
        // Only store if not already stored (preserve the original stack position)
        if !self.stack_positions.contains_key(&panel_type) {
            self.stack_positions.insert(panel_type, position);
            log::debug!("Stored stack position for {:?} at ({:.1}, {:.1})", 
                panel_type, position.x, position.y);
        }
    }
    
    /// Get stack position for a panel type
    pub fn get_stack_position(&self, panel_type: PanelType) -> Option<egui::Pos2> {
        self.stack_positions.get(&panel_type).copied()
    }
    
    /// Clear stack position when all panels are unstacked
    pub fn clear_stack_position(&mut self, panel_type: PanelType) {
        if self.stack_positions.remove(&panel_type).is_some() {
            log::debug!("Cleared stack position for {:?}", panel_type);
        }
    }
    
    /// Get unique window ID for unstacked panel (returns None if panel was never unstacked)
    pub fn get_unstacked_panel_id(&self, node_id: NodeId) -> Option<u64> {
        self.get_state(node_id).and_then(|s| s.unstacked_panel_id)
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
    
    /// Get all visible stacked panels of a specific type in simple node ID order
    pub fn get_stacked_panels_by_type(&self, panel_type: PanelType, viewed_nodes: &HashMap<NodeId, crate::nodes::Node>) -> Vec<NodeId> {
        // Get all currently visible and stacked nodes of the specified type
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
        
        // Simple sorting by node ID
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
                egui::Pos2::new(0.0, crate::constants::DEFAULT_MENU_BAR_HEIGHT),
                egui::Vec2::new(ctx.screen_rect().width(), ctx.screen_rect().height() - crate::constants::DEFAULT_MENU_BAR_HEIGHT)
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
                    self.get_or_create_state(node_id).parameter_cache = parameters;
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
        self.get_state(node_id).map(|s| &s.parameter_cache)
    }
    
    /// Clear cache for a node
    pub fn clear_cache(&mut self, node_id: NodeId) {
        if let Some(state) = self.node_states.get_mut(&node_id) {
            state.parameter_cache.clear();
        }
    }
}

// Helper macros removed - unused

/// Change notifications for parameter updates
#[derive(Debug, Clone)]
pub struct ParameterChange {
    pub parameter: String,
    pub value: NodeData,
}

// build_parameter_ui function removed - unused