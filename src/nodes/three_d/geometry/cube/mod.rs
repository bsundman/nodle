//! USD-based Cube Geometry Node

pub mod parameters;
pub mod logic;

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory};
use crate::nodes::factory::{DataType, PortDefinition, ProcessingCost};
use egui::{Color32, Ui};

/// USD-based Cube Node Factory
#[derive(Default)]
pub struct CubeNodeFactory;

impl NodeFactory for CubeNodeFactory {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "3D_Cube",
            "Cube",
            NodeCategory::new(&["3D", "Geometry"]),
            "Creates a cube using USD procedural primitives with primitive/mesh toggle"
        )
        .with_color(Color32::from_rgb(100, 150, 200))
        .with_icon("🟫")
        .with_inputs(vec![
            // No inputs - this is a geometry source node
        ])
        .with_outputs(vec![
            PortDefinition::required("Scene", DataType::Any)
                .with_description("USD scene data with cube geometry")
        ])
        .with_tags(vec!["geometry", "primitive", "cube", "3d", "mesh", "usd"])
        .with_processing_cost(ProcessingCost::Low)
        .with_workspace_compatibility(vec!["3D", "USD", "MaterialX"])
    }
    
    fn create(position: egui::Pos2) -> Node {
        let meta = Self::metadata();
        let mut node = Node::new(0, meta.display_name, position);
        node.set_type_id(meta.node_type);
        node.color = meta.color;
        
        // No inputs for geometry source nodes
        
        // Add outputs
        for output in &meta.outputs {
            node.add_output(&output.name);
        }
        
        // Set panel type to Parameter
        node.set_panel_type(crate::nodes::interface::PanelType::Parameter);
        
        // Initialize default parameters
        node.parameters.insert("mode".to_string(), NodeData::String("primitive".to_string()));
        node.parameters.insert("needs_reload".to_string(), NodeData::Boolean(false));
        node.parameters.insert("size".to_string(), NodeData::Float(2.0));
        node.parameters.insert("size_x".to_string(), NodeData::Float(2.0));
        node.parameters.insert("size_y".to_string(), NodeData::Float(2.0));
        node.parameters.insert("size_z".to_string(), NodeData::Float(2.0));
        
        // Mesh subdivision parameters
        node.parameters.insert("subdivisions_x".to_string(), NodeData::Integer(1));
        node.parameters.insert("subdivisions_y".to_string(), NodeData::Integer(1));
        node.parameters.insert("subdivisions_z".to_string(), NodeData::Integer(1));
        node.parameters.insert("smooth_normals".to_string(), NodeData::Boolean(false));
        node.parameters.insert("generate_uvs".to_string(), NodeData::Boolean(true));
        
        // Update port positions
        node.update_port_positions();
        
        node
    }
}

/// USD-based Cube Node implementation
pub struct CubeNode;

impl CubeNode {
    /// Build the parameter interface for the Cube node
    pub fn build_interface(node: &mut Node, ui: &mut Ui) -> Vec<ParameterChange> {
        parameters::CubeParameters::build_interface(node, ui)
    }
    
    /// Process the Cube node's logic
    pub fn process_node(node: &Node, inputs: Vec<NodeData>) -> Vec<NodeData> {
        let mut logic = logic::CubeLogic::from_node(node);
        logic.process(inputs)
    }
}