//! Reverse node module - comprehensive 3D reversal operations

pub mod logic;
pub mod parameters;

pub use parameters::ReverseNode;

use crate::nodes::NodeFactory;

impl NodeFactory for parameters::ReverseNode {
    fn metadata() -> crate::nodes::NodeMetadata {
        crate::nodes::NodeMetadata::new(
            "3D_Reverse",
            "Reverse",
            crate::nodes::NodeCategory::new(&["3D", "Modify"]),
            "Comprehensive reversal operations for USD geometry including normals, winding, mirroring, and UV flipping"
        )
        .with_color(egui::Color32::from_rgb(200, 120, 160)) // Purple-ish for modify operations
        .with_icon("🔄")
        .with_inputs(vec![
            crate::nodes::PortDefinition::required("Geometry", crate::nodes::DataType::Any)
                .with_description("USD scene data to apply reverse operations to"),
        ])
        .with_outputs(vec![
            crate::nodes::PortDefinition::required("Geometry", crate::nodes::DataType::Any)
                .with_description("Modified USD scene data with reverse operations applied"),
        ])
        .with_tags(vec!["3d", "modify", "reverse", "mirror", "flip", "normals", "winding", "uv", "interface", "pattern_a"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Medium)
        .with_workspace_compatibility(vec!["3D", "USD", "MaterialX"])
    }
    
    fn create(position: egui::Pos2) -> crate::nodes::Node {
        let meta = Self::metadata();
        let mut node = crate::nodes::Node::new(0, meta.display_name, position);
        node.color = meta.color;
        
        // Add inputs
        for input in &meta.inputs {
            node.add_input(&input.name);
        }
        
        // Add outputs  
        for output in &meta.outputs {
            node.add_output(&output.name);
        }
        
        // Set panel type to Parameter so it shows the parameter interface
        node.set_panel_type(crate::nodes::interface::PanelType::Parameter);
        
        // Initialize default parameters
        node.parameters.insert("reverse_normals".to_string(), crate::nodes::interface::NodeData::Boolean(false));
        node.parameters.insert("reverse_face_winding".to_string(), crate::nodes::interface::NodeData::Boolean(false));
        node.parameters.insert("reverse_point_order".to_string(), crate::nodes::interface::NodeData::Boolean(false));
        node.parameters.insert("mirror_axis".to_string(), crate::nodes::interface::NodeData::String("None".to_string()));
        node.parameters.insert("reverse_uvs_u".to_string(), crate::nodes::interface::NodeData::Boolean(false));
        node.parameters.insert("reverse_uvs_v".to_string(), crate::nodes::interface::NodeData::Boolean(false));
        node.parameters.insert("flip_vertex_colors".to_string(), crate::nodes::interface::NodeData::Boolean(false));
        node.parameters.insert("invert_transforms".to_string(), crate::nodes::interface::NodeData::Boolean(false));
        
        // Update port positions
        node.update_port_positions();
        
        node
    }
}