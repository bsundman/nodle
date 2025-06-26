//! Port types and functionality for node connections

use egui::Pos2;
use serde::{Deserialize, Serialize};

/// Unique identifier for a port
pub type PortId = usize;

/// Type of port (input or output)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortType {
    Input,
    Output,
}

/// Represents a connection point on a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    pub id: PortId,
    pub name: String,
    pub port_type: PortType,
    #[serde(with = "pos2_serde")]
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

// Serde helper module for Pos2
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