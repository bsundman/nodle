//! Port types and functionality for node connections

use egui::Pos2;

/// Unique identifier for a port
pub type PortId = usize;

/// Type of port (input or output)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortType {
    Input,
    Output,
}

/// Represents a connection point on a node
#[derive(Debug, Clone)]
pub struct Port {
    pub id: PortId,
    pub name: String,
    pub port_type: PortType,
    pub position: Pos2,
}

impl Port {
    /// Creates a new port
    pub fn new(id: PortId, name: impl Into<String>, port_type: PortType) -> Self {
        Self {
            id,
            name: name.into(),
            port_type,
            position: Pos2::ZERO,
        }
    }

    /// Checks if this port is an input
    pub fn is_input(&self) -> bool {
        matches!(self.port_type, PortType::Input)
    }

    /// Checks if this port is an output
    pub fn is_output(&self) -> bool {
        matches!(self.port_type, PortType::Output)
    }
}