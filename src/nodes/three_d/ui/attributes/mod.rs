//! Attributes UI node implementation
//! 
//! Provides a spreadsheet-style interface for viewing and editing USD attributes
//! including primvars, geometric attributes, and custom properties.

use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition, NodeId};
use crate::nodes::interface::{PanelType, NodeData, ParameterChange};
use crate::nodes::factory::ProcessingCost;
use egui::{Color32, Pos2, Vec2};

pub mod parameters;
pub mod logic;

pub use parameters::*;
pub use logic::*;

/// Factory for creating Attributes nodes
pub struct AttributesNodeFactory;

impl NodeFactory for AttributesNodeFactory {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Attributes",
            "Attributes",
            NodeCategory::new(&["3D", "UI"]),
            "Spreadsheet-style viewer for USD attributes, primvars, and geometric properties",
        )
        .with_color(Color32::from_rgb(120, 80, 180)) // Purple color
        .with_icon("📊")
        .with_panel_type(PanelType::Spreadsheet)
        .with_inputs(vec![
            PortDefinition::required("USD Scene", DataType::Any)
                .with_description("USD scene data to visualize in spreadsheet view"),
        ])
        .with_outputs(vec![])
        .with_tags(vec!["3D", "USD", "Attributes", "Spreadsheet", "UI"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Minimal)
        .with_workspace_compatibility(vec!["USD", "3D", "General"])
    }
}