//! Node types and core node functionality

use super::port::{Port, PortType};
use super::graph::NodeGraph;
use super::interface::{PanelType, NodeData};
use egui::{Color32, Pos2, Rect, Vec2};
use crate::theme;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a node
pub type NodeId = usize;

/// Port mapping for context nodes - maps external ports to internal node ports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    /// External port name
    pub external_port: String,
    /// Internal node ID
    pub internal_node_id: NodeId,
    /// Internal port name
    pub internal_port: String,
    /// Whether this is an input (true) or output (false) mapping
    pub is_input: bool,
}

/// Type of node - regular processing node or context/group node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    /// Regular processing node
    Regular,
    /// Workspace node that contains its own graph
    Workspace {
        /// The internal graph contained within this workspace node
        graph: NodeGraph,
        /// The workspace type (e.g., "3D", "MaterialX")
        workspace_type: String,
        /// Port mappings between external and internal ports
        port_mappings: Vec<PortMapping>,
    },
}

/// Core node structure representing a visual node in the graph
#[derive(Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub title: String,
    #[serde(with = "pos2_serde")]
    pub position: Pos2,
    #[serde(with = "vec2_serde")]
    pub size: Vec2,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
    #[serde(with = "color32_serde")]
    pub color: Color32,
    pub node_type: NodeType,
    /// Button states: [left_button_active, right_button_active]
    pub button_states: [bool; 2],
    /// Whether the node is visible (true) or hidden (false)
    pub visible: bool,
    /// The type of panel this node should display in (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub panel_type: Option<PanelType>,
    /// Node parameters for interface panels
    #[serde(default)]
    pub parameters: HashMap<String, NodeData>,
    /// Plugin node instance (if this is a plugin node)
    #[serde(skip)]
    pub plugin_node: Option<Box<dyn nodle_plugin_sdk::PluginNode>>,
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("position", &self.position)
            .field("size", &self.size)
            .field("inputs", &self.inputs)
            .field("outputs", &self.outputs)
            .field("color", &self.color)
            .field("node_type", &self.node_type)
            .field("button_states", &self.button_states)
            .field("visible", &self.visible)
            .field("panel_type", &self.panel_type)
            .field("parameters", &self.parameters)
            .field("plugin_node", &if self.plugin_node.is_some() { "Some(PluginNode)" } else { "None" })
            .finish()
    }
}

impl Clone for Node {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            title: self.title.clone(),
            position: self.position,
            size: self.size,
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
            color: self.color,
            node_type: self.node_type.clone(),
            button_states: self.button_states,
            visible: self.visible,
            panel_type: self.panel_type,
            parameters: self.parameters.clone(),
            plugin_node: None, // Plugin nodes cannot be cloned, so we set to None
        }
    }
}

impl Node {
    /// Creates a new node with the specified properties
    pub fn new(id: NodeId, title: impl Into<String>, position: Pos2) -> Self {
        let title_str = title.into();
        let new_node = Self {
            id,
            title: title_str.clone(),
            position,
            size: theme::dimensions().default_node_size,
            inputs: vec![],
            outputs: vec![],
            color: Color32::from_rgb(60, 60, 60),
            node_type: NodeType::Regular,
            button_states: [false, false],
            visible: true,
            panel_type: None, // Will be set by factory or with_panel_type()
            parameters: HashMap::new(),
            plugin_node: None, // Initialize plugin node as None
        };
        
        
        new_node
    }
    
    /// Creates a new workspace node
    pub fn new_workspace(id: NodeId, workspace_type: impl Into<String>, position: Pos2) -> Self {
        let workspace_type_str = workspace_type.into();
        let title = format!("{} Workspace", workspace_type_str);
        let new_node = Self {
            id,
            title: title.clone(),
            position,
            size: theme::dimensions().workspace_node_size,
            inputs: vec![],
            outputs: vec![],
            color: Color32::from_rgb(80, 100, 120), // Different color for workspace nodes
            node_type: NodeType::Workspace {
                graph: NodeGraph::new(),
                workspace_type: workspace_type_str,
                port_mappings: vec![],
            },
            button_states: [false, false],
            visible: true,
            panel_type: None, // Workspace nodes typically don't have panels
            parameters: HashMap::new(),
            plugin_node: None, // Initialize plugin node as None
        };
        
        
        new_node
    }

    /// Adds an input port to the node
    pub fn add_input(&mut self, name: impl Into<String>) -> &mut Self {
        let port_id = self.inputs.len();
        self.inputs.push(Port::new(port_id, name, PortType::Input));
        self
    }

    /// Adds an output port to the node
    pub fn add_output(&mut self, name: impl Into<String>) -> &mut Self {
        let port_id = self.outputs.len();
        self.outputs.push(Port::new(port_id, name, PortType::Output));
        self
    }

    /// Updates the positions of all ports based on the node's position and size
    pub fn update_port_positions(&mut self) {
        let port_spacing = theme::dimensions().port_spacing;

        // Input ports on TOP of node
        let input_start_x = if self.inputs.len() > 1 {
            (self.size.x - (self.inputs.len() - 1) as f32 * port_spacing) / 2.0
        } else {
            self.size.x / 2.0
        };

        for (i, input) in self.inputs.iter_mut().enumerate() {
            input.position =
                self.position + Vec2::new(input_start_x + i as f32 * port_spacing, 0.0);
        }

        // Output ports on BOTTOM of node
        let output_start_x = if self.outputs.len() > 1 {
            (self.size.x - (self.outputs.len() - 1) as f32 * port_spacing) / 2.0
        } else {
            self.size.x / 2.0
        };

        for (i, output) in self.outputs.iter_mut().enumerate() {
            output.position =
                self.position + Vec2::new(output_start_x + i as f32 * port_spacing, self.size.y);
        }
    }

    /// Returns the bounding rectangle of the node
    pub fn get_rect(&self) -> Rect {
        Rect::from_min_size(self.position, self.size)
    }

    /// Sets the color of the node
    pub fn with_color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }
    
    /// Sets the panel type for this node
    pub fn with_panel_type(mut self, panel_type: PanelType) -> Self {
        self.panel_type = Some(panel_type);
        self
    }
    
    /// Gets the panel type for this node
    pub fn get_panel_type(&self) -> Option<PanelType> {
        self.panel_type
    }
    
    /// Sets the panel type for this node (mutable reference)
    pub fn set_panel_type(&mut self, panel_type: PanelType) {
        self.panel_type = Some(panel_type);
    }

    /// Sets the size of the node
    pub fn with_size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
    
    /// Check if this is a workspace node
    pub fn is_workspace(&self) -> bool {
        matches!(self.node_type, NodeType::Workspace { .. })
    }
    
    /// Get the internal graph if this is a workspace node
    pub fn get_internal_graph(&self) -> Option<&NodeGraph> {
        match &self.node_type {
            NodeType::Workspace { graph, .. } => Some(graph),
            NodeType::Regular => None,
        }
    }
    
    /// Get the mutable internal graph if this is a workspace node
    pub fn get_internal_graph_mut(&mut self) -> Option<&mut NodeGraph> {
        match &mut self.node_type {
            NodeType::Workspace { graph, .. } => Some(graph),
            NodeType::Regular => None,
        }
    }
    
    /// Get the workspace type if this is a workspace node
    pub fn get_workspace_type(&self) -> Option<&str> {
        match &self.node_type {
            NodeType::Workspace { workspace_type, .. } => Some(workspace_type),
            NodeType::Regular => None,
        }
    }
    
    /// Get port mappings if this is a workspace node
    pub fn get_port_mappings(&self) -> Option<&Vec<PortMapping>> {
        match &self.node_type {
            NodeType::Workspace { port_mappings, .. } => Some(port_mappings),
            NodeType::Regular => None,
        }
    }
    
    /// Get mutable port mappings if this is a workspace node
    pub fn get_port_mappings_mut(&mut self) -> Option<&mut Vec<PortMapping>> {
        match &mut self.node_type {
            NodeType::Workspace { port_mappings, .. } => Some(port_mappings),
            NodeType::Regular => None,
        }
    }
    
    /// Add a port mapping to this workspace node
    pub fn add_port_mapping(&mut self, mapping: PortMapping) -> Result<(), &'static str> {
        match &mut self.node_type {
            NodeType::Workspace { port_mappings, .. } => {
                port_mappings.push(mapping);
                Ok(())
            }
            NodeType::Regular => Err("Cannot add port mapping to regular node"),
        }
    }
    
    /// Add an external input port with mapping to internal node
    pub fn add_external_input(&mut self, 
        external_name: impl Into<String>, 
        internal_node_id: NodeId, 
        internal_port: impl Into<String>
    ) -> Result<(), &'static str> {
        let external_name = external_name.into();
        
        // Add the external port to this workspace node
        self.add_input(&external_name);
        
        // Create the port mapping
        let mapping = PortMapping {
            external_port: external_name,
            internal_node_id,
            internal_port: internal_port.into(),
            is_input: true,
        };
        
        self.add_port_mapping(mapping)
    }
    
    /// Add an external output port with mapping to internal node
    pub fn add_external_output(&mut self, 
        external_name: impl Into<String>, 
        internal_node_id: NodeId, 
        internal_port: impl Into<String>
    ) -> Result<(), &'static str> {
        let external_name = external_name.into();
        
        // Add the external port to this workspace node
        self.add_output(&external_name);
        
        // Create the port mapping
        let mapping = PortMapping {
            external_port: external_name,
            internal_node_id,
            internal_port: internal_port.into(),
            is_input: false,
        };
        
        self.add_port_mapping(mapping)
    }
    
    /// Get the internal node and port for an external port
    pub fn resolve_external_port(&self, external_port: &str, is_input: bool) -> Option<(NodeId, &str)> {
        if let Some(mappings) = self.get_port_mappings() {
            for mapping in mappings {
                if mapping.external_port == external_port && mapping.is_input == is_input {
                    return Some((mapping.internal_node_id, &mapping.internal_port));
                }
            }
        }
        None
    }
    
    /// Toggle the left button state
    pub fn toggle_left_button(&mut self) {
        self.button_states[0] = !self.button_states[0];
    }
    
    /// Toggle the right button state  
    pub fn toggle_right_button(&mut self) {
        self.button_states[1] = !self.button_states[1];
    }
    
    /// Toggle the visibility state
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }
    
    /// Check if a point is inside the left button area
    pub fn is_point_in_left_button(&self, point: Pos2) -> bool {
        let button_rect = Rect::from_two_pos(
            self.position,
            Pos2::new(self.position.x + 10.0, self.position.y + self.size.y)
        );
        button_rect.contains(point)
    }
    
    /// Check if a point is inside the right button area
    pub fn is_point_in_right_button(&self, point: Pos2) -> bool {
        let button_rect = Rect::from_two_pos(
            Pos2::new(self.position.x + 10.0, self.position.y),
            Pos2::new(self.position.x + 20.0, self.position.y + self.size.y)
        );
        button_rect.contains(point)
    }
    
    /// Get the visibility flag position (matches ports positioning style)
    pub fn get_flag_position(&self) -> Pos2 {
        let flag_margin_right = 15.0; // Equal distance from top edge (15.0)
        Pos2::new(
            self.position.x + self.size.x - flag_margin_right,
            self.position.y + 15.0
        )
    }
    
    /// Check if a point is inside the visibility flag area (right side of node)
    pub fn is_point_in_visibility_flag(&self, point: Pos2) -> bool {
        let flag_size = 12.0;
        let flag_center = self.get_flag_position();
        
        // Create a clickable area around the flag
        let flag_rect = Rect::from_center_size(flag_center, Vec2::new(flag_size, flag_size));
        flag_rect.contains(point)
    }
    
    /// Get the rectangle for the left button
    pub fn get_left_button_rect(&self) -> Rect {
        Rect::from_two_pos(
            self.position,
            Pos2::new(self.position.x + 10.0, self.position.y + self.size.y)
        )
    }
    
    /// Get the rectangle for the right button
    pub fn get_right_button_rect(&self) -> Rect {
        Rect::from_two_pos(
            Pos2::new(self.position.x + 10.0, self.position.y),
            Pos2::new(self.position.x + 20.0, self.position.y + self.size.y)
        )
    }
}

// Serde helper modules for egui types
mod pos2_serde {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(pos: &Pos2, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        [pos.x, pos.y].serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Pos2, D::Error>
    where
        D: Deserializer<'de>,
    {
        let [x, y] = <[f32; 2]>::deserialize(deserializer)?;
        Ok(Pos2::new(x, y))
    }
}

mod vec2_serde {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(vec: &Vec2, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        [vec.x, vec.y].serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec2, D::Error>
    where
        D: Deserializer<'de>,
    {
        let [x, y] = <[f32; 2]>::deserialize(deserializer)?;
        Ok(Vec2::new(x, y))
    }
}

mod color32_serde {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(color: &Color32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        [color.r(), color.g(), color.b(), color.a()].serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let [r, g, b, a] = <[u8; 4]>::deserialize(deserializer)?;
        Ok(Color32::from_rgba_unmultiplied(r, g, b, a))
    }
}