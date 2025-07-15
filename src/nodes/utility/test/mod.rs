//! Test node module - comprehensive parameter testing

pub mod logic;
pub mod parameters;

pub use logic::TestLogic;
pub use parameters::TestNode;

use crate::nodes::NodeFactory;

impl NodeFactory for parameters::TestNode {
    fn metadata() -> crate::nodes::NodeMetadata {
        crate::nodes::NodeMetadata::new(
            "Test",
            "Test Node",
            crate::nodes::NodeCategory::new(&["Utility"]),
            "A comprehensive test node for parameter UI validation"
        )
        .with_color(egui::Color32::from_rgb(255, 100, 100))
        .with_icon("🧪")
        .with_inputs(vec![
            crate::nodes::PortDefinition::optional("Input", crate::nodes::DataType::Any)
                .with_description("Test input data"),
        ])
        .with_outputs(vec![
            crate::nodes::PortDefinition::optional("Output", crate::nodes::DataType::Any)
                .with_description("Test output data"),
        ])
        .with_panel_type(crate::nodes::interface::PanelType::Parameter)
        .with_tags(vec!["utility", "test", "debugging", "ui-test"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Low)
        .with_workspace_compatibility(vec!["3D", "General", "USD", "MaterialX"])
    }
    
    fn create(position: egui::Pos2) -> crate::nodes::Node {
        let meta = Self::metadata();
        let mut node = crate::nodes::Node::new(0, meta.display_name, position);
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
        
        // Set panel type from metadata
        node.set_panel_type(meta.panel_type);
        
        // Initialize default parameters
        node.parameters.insert("label".to_string(), crate::nodes::interface::NodeData::String("Test Node".to_string()));
        node.parameters.insert("enabled".to_string(), crate::nodes::interface::NodeData::Boolean(true));
        node.parameters.insert("description".to_string(), crate::nodes::interface::NodeData::String("Comprehensive parameter testing node".to_string()));
        
        // Numeric parameters
        node.parameters.insert("float_value".to_string(), crate::nodes::interface::NodeData::Float(1.0));
        node.parameters.insert("int_value".to_string(), crate::nodes::interface::NodeData::Integer(10));
        node.parameters.insert("slider_value".to_string(), crate::nodes::interface::NodeData::Float(0.5));
        node.parameters.insert("drag_value".to_string(), crate::nodes::interface::NodeData::Float(2.5));
        node.parameters.insert("angle_degrees".to_string(), crate::nodes::interface::NodeData::Float(45.0));
        
        // Boolean parameters
        node.parameters.insert("checkbox_1".to_string(), crate::nodes::interface::NodeData::Boolean(false));
        node.parameters.insert("checkbox_2".to_string(), crate::nodes::interface::NodeData::Boolean(true));
        node.parameters.insert("radio_option".to_string(), crate::nodes::interface::NodeData::Integer(0));
        
        // String parameters
        node.parameters.insert("text_input".to_string(), crate::nodes::interface::NodeData::String("Hello World".to_string()));
        node.parameters.insert("multiline_text".to_string(), crate::nodes::interface::NodeData::String("Line 1\nLine 2\nLine 3".to_string()));
        
        // Color parameters
        node.parameters.insert("color_rgb".to_string(), crate::nodes::interface::NodeData::Color([1.0, 0.5, 0.0, 1.0]));
        node.parameters.insert("color_rgba".to_string(), crate::nodes::interface::NodeData::Color([0.0, 0.5, 1.0, 0.8]));
        
        // Vector parameters
        node.parameters.insert("vector3".to_string(), crate::nodes::interface::NodeData::Vector3([1.0, 2.0, 3.0]));
        
        // Enum/combo parameters
        node.parameters.insert("operation_mode".to_string(), crate::nodes::interface::NodeData::String("passthrough".to_string()));
        node.parameters.insert("quality_setting".to_string(), crate::nodes::interface::NodeData::String("medium".to_string()));
        
        // Progress parameter
        node.parameters.insert("progress_value".to_string(), crate::nodes::interface::NodeData::Float(0.75));
        
        // Update port positions after adding ports
        node.update_port_positions();
        
        node
    }
}