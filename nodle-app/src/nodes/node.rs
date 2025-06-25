//! Node types and core node functionality

use super::port::{Port, PortType};
use egui::{Color32, Pos2, Rect, Vec2};

/// Unique identifier for a node
pub type NodeId = usize;

/// Core node structure representing a visual node in the graph
#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub title: String,
    pub position: Pos2,
    pub size: Vec2,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
    pub color: Color32,
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
}