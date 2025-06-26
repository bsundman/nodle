//! Node types and core node functionality

use super::port::{Port, PortType};
use super::graph::NodeGraph;
use egui::{Color32, Pos2, Rect, Vec2};
use serde::{Deserialize, Serialize};

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
    /// Context node that contains its own graph
    Context {
        /// The internal graph contained within this context node
        graph: NodeGraph,
        /// The context type (e.g., "3D", "MaterialX")
        context_type: String,
        /// Port mappings between external and internal ports
        port_mappings: Vec<PortMapping>,
    },
}

/// Core node structure representing a visual node in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

impl Node {
    /// Creates a new node with the specified properties
    pub fn new(id: NodeId, title: impl Into<String>, position: Pos2) -> Self {
        Self {
            id,
            title: title.into(),
            position,
            size: Vec2::new(150.0, 30.0),
            inputs: vec![],
            outputs: vec![],
            color: Color32::from_rgb(60, 60, 60),
            node_type: NodeType::Regular,
            button_states: [false, false],
        }
    }
    
    /// Creates a new context node
    pub fn new_context(id: NodeId, context_type: impl Into<String>, position: Pos2) -> Self {
        let context_type_str = context_type.into();
        Self {
            id,
            title: format!("{} Context", context_type_str),
            position,
            size: Vec2::new(180.0, 50.0), // Slightly larger
            inputs: vec![],
            outputs: vec![],
            color: Color32::from_rgb(80, 100, 120), // Different color for context nodes
            node_type: NodeType::Context {
                graph: NodeGraph::new(),
                context_type: context_type_str,
                port_mappings: vec![],
            },
            button_states: [false, false],
        }
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
        let port_spacing = 30.0;

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

    /// Sets the size of the node
    pub fn with_size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
    
    /// Check if this is a context node
    pub fn is_context(&self) -> bool {
        matches!(self.node_type, NodeType::Context { .. })
    }
    
    /// Get the internal graph if this is a context node
    pub fn get_internal_graph(&self) -> Option<&NodeGraph> {
        match &self.node_type {
            NodeType::Context { graph, .. } => Some(graph),
            NodeType::Regular => None,
        }
    }
    
    /// Get the mutable internal graph if this is a context node
    pub fn get_internal_graph_mut(&mut self) -> Option<&mut NodeGraph> {
        match &mut self.node_type {
            NodeType::Context { graph, .. } => Some(graph),
            NodeType::Regular => None,
        }
    }
    
    /// Get the context type if this is a context node
    pub fn get_context_type(&self) -> Option<&str> {
        match &self.node_type {
            NodeType::Context { context_type, .. } => Some(context_type),
            NodeType::Regular => None,
        }
    }
    
    /// Get port mappings if this is a context node
    pub fn get_port_mappings(&self) -> Option<&Vec<PortMapping>> {
        match &self.node_type {
            NodeType::Context { port_mappings, .. } => Some(port_mappings),
            NodeType::Regular => None,
        }
    }
    
    /// Get mutable port mappings if this is a context node
    pub fn get_port_mappings_mut(&mut self) -> Option<&mut Vec<PortMapping>> {
        match &mut self.node_type {
            NodeType::Context { port_mappings, .. } => Some(port_mappings),
            NodeType::Regular => None,
        }
    }
    
    /// Add a port mapping to this context node
    pub fn add_port_mapping(&mut self, mapping: PortMapping) -> Result<(), &'static str> {
        match &mut self.node_type {
            NodeType::Context { port_mappings, .. } => {
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
        
        // Add the external port to this context node
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
        
        // Add the external port to this context node
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