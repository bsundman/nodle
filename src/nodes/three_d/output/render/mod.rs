//! USD Hydra Render Node

pub mod parameters;
pub mod logic;

use crate::nodes::interface::{NodeData, ParameterChange};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory};
use crate::nodes::factory::{DataType, PortDefinition, ProcessingCost};
use egui::{Color32, Ui};

/// USD Hydra Render Node Factory
#[derive(Default)]
pub struct RenderNodeFactory;

impl NodeFactory for RenderNodeFactory {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "3D_Render",
            "Render",
            NodeCategory::new(&["3D", "Output"]),
            "Renders USD scene data using Hydra render delegates via usdrecord"
        )
        .with_color(Color32::from_rgb(220, 80, 80))
        .with_icon("🎬")
        .with_inputs(vec![
            PortDefinition::required("Scene", DataType::Any)
                .with_description("USD scene data to render")
        ])
        .with_outputs(vec![
            // Output nodes typically don't have outputs, but we could add status output
            PortDefinition::optional("Status", DataType::String)
                .with_description("Render completion status")
        ])
        .with_tags(vec!["render", "hydra", "output", "usd", "image"])
        .with_processing_cost(ProcessingCost::High)
        .with_workspace_compatibility(vec!["3D", "USD"])
    }
    
    fn create(position: egui::Pos2) -> Node {
        let meta = Self::metadata();
        let mut node = Node::new(0, meta.display_name, position);
        node.set_type_id(meta.node_type);
        node.color = meta.color;
        
        // Add inputs
        for input in &meta.inputs {
            node.add_input(&input.name);
        }
        
        // Add outputs
        for output in &meta.outputs {
            node.add_output(&output.name);
        }
        
        // Set panel type to Parameter
        node.set_panel_type(crate::nodes::interface::PanelType::Parameter);
        
        // Initialize default parameters
        node.parameters.insert("renderer".to_string(), NodeData::String("Storm".to_string()));
        node.parameters.insert("output_path".to_string(), NodeData::String("render_output.png".to_string()));
        node.parameters.insert("temp_folder".to_string(), NodeData::String("/tmp/nodle_render".to_string()));
        node.parameters.insert("image_width".to_string(), NodeData::Integer(1920));
        // Note: image_height removed - usdrecord computes height from width and aspect ratio
        node.parameters.insert("camera_path".to_string(), NodeData::String("".to_string()));
        // Note: samples removed - not directly supported by usdrecord
        node.parameters.insert("complexity".to_string(), NodeData::String("high".to_string()));
        node.parameters.insert("color_correction".to_string(), NodeData::String("sRGB".to_string()));
        node.parameters.insert("available_renderers".to_string(), NodeData::String("Storm".to_string())); // Will be populated dynamically
        node.parameters.insert("last_render_status".to_string(), NodeData::String("Ready".to_string()));
        node.parameters.insert("trigger_render".to_string(), NodeData::Boolean(false)); // Only true when render button is clicked
        
        // Update port positions
        node.update_port_positions();
        
        node
    }
}

/// USD Hydra Render Node implementation
pub struct RenderNode;

impl RenderNode {
    /// Build the parameter interface for the Render node
    pub fn build_interface(node: &mut Node, ui: &mut Ui) -> Vec<ParameterChange> {
        parameters::RenderParameters::build_interface(node, ui)
    }
    
    /// Process the Render node's logic
    pub fn process_node(node: &Node, inputs: Vec<NodeData>) -> Vec<NodeData> {
        let mut logic = logic::RenderLogic::from_node(node);
        logic.process(inputs)
    }
}