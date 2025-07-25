//! USD File Reader Node
//!
//! Reads USD files from disk and provides stage and scene data for downstream processing.
//! This node serves as a data source for USD workflows, using the embedded USD-core library
//! to parse USD files and extract scene information.

pub mod parameters;
pub mod logic;
pub mod file_cache;
pub mod hooks;

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::{Node, NodeId, NodeFactory, NodeMetadata, NodeCategory};
use crate::nodes::factory::{DataType, PortDefinition, ProcessingCost};
use egui::{Color32, Ui};

/// USD File Reader Node Factory
#[derive(Default)]
pub struct UsdFileReaderNodeFactory;

impl NodeFactory for UsdFileReaderNodeFactory {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Data_UsdFileReader",
            "Read USD",
            NodeCategory::new(&["Data"]),
            "Reads USD files and extracts scene data for downstream processing"
        )
        .with_color(Color32::from_rgb(70, 130, 180)) // Steel blue for USD
        .with_icon("📁")
        .with_inputs(vec![
            // No inputs - this is a data source node
        ])
        .with_outputs(vec![
            PortDefinition::required("Scene", DataType::Any)
                .with_description("USD scene data with geometry, materials, and lights")
        ])
        .with_tags(vec!["usd", "file", "input", "3d", "scene", "geometry", "import"])
        .with_processing_cost(ProcessingCost::Medium)
        .with_workspace_compatibility(vec!["USD", "3D", "General"])
    }
    
    fn create(position: egui::Pos2) -> Node {
        let meta = Self::metadata();
        let mut node = Node::new(0, meta.display_name, position);
        node.set_type_id(meta.node_type);
        node.color = meta.color;
        
        // Add outputs
        for output in &meta.outputs {
            node.add_output(&output.name);
        }
        
        // Set panel type to Parameter so it shows the parameter interface
        node.set_panel_type(crate::nodes::interface::PanelType::Parameter);
        
        // Initialize default parameters
        node.parameters.insert("file_path".to_string(), NodeData::String(String::new()));
        node.parameters.insert("needs_reload".to_string(), NodeData::Boolean(false));
        node.parameters.insert("extract_geometry".to_string(), NodeData::Boolean(true));
        node.parameters.insert("extract_materials".to_string(), NodeData::Boolean(true));
        node.parameters.insert("extract_lights".to_string(), NodeData::Boolean(true));
        node.parameters.insert("extract_cameras".to_string(), NodeData::Boolean(false));
        
        // Update port positions
        node.update_port_positions();
        
        node
    }
}

/// USD File Reader Node implementation using Pattern A interface
pub struct UsdFileReaderNode;

impl UsdFileReaderNode {
    /// Build the parameter interface for the USD File Reader node
    pub fn build_interface(node: &mut Node, ui: &mut Ui) -> Vec<ParameterChange> {
        parameters::UsdFileReaderParameters::build_interface(node, ui)
    }
    
    /// Process the USD File Reader node's logic (called during graph execution)
    /// NOTE: This method is only used as fallback. The execution engine calls
    /// process_with_unified_cache directly for USD File Reader nodes.
    pub fn process_node(node: &Node, inputs: Vec<NodeData>) -> Vec<NodeData> {
        println!("📁 UsdFileReaderNode::process_node called for node '{}' (type_id: {})", node.title, node.type_id);
        println!("⚠️ Using fallback process_node - execution engine should call process_with_unified_cache");
        
        // Return error since this should not be called for USD File Reader
        vec![NodeData::None]
    }
}