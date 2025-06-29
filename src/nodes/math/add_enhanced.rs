//! Enhanced addition node implementation using new factory system

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeFactory, NodeMetadata, NodeCategory, DataType, PortDefinition};

/// Addition node that takes two numeric inputs and produces their sum
#[derive(Default)]
pub struct AddNodeEnhanced;

impl NodeFactory for AddNodeEnhanced {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "Add",
            "Add",
            NodeCategory::math(),
            "Adds two numeric values together"
        )
        .with_color(Color32::from_rgb(45, 55, 65))
        .with_icon("➕")
        .with_inputs(vec![
            PortDefinition::required("A", DataType::Float)
                .with_description("First input value"),
            PortDefinition::required("B", DataType::Float)
                .with_description("Second input value"),
        ])
        .with_outputs(vec![
            PortDefinition::required("Result", DataType::Float)
                .with_description("Sum of A and B"),
        ])
        .with_tags(vec!["math", "arithmetic", "basic"])
        .with_processing_cost(crate::nodes::factory::ProcessingCost::Minimal)
    }
}

// Also implement the old trait for backward compatibility during transition
impl crate::NodeFactory for AddNodeEnhanced {
    fn node_type() -> &'static str {
        "Add"
    }
    
    fn display_name() -> &'static str {
        "Add"
    }
    
    fn category() -> crate::NodeCategory {
        crate::NodeCategory::Math
    }
    
    fn color() -> Color32 {
        Color32::from_rgb(45, 55, 65)
    }
    
    fn create(position: Pos2) -> Node {
        <Self as NodeFactory>::create(position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Pos2;

    #[test]
    fn test_enhanced_add_node_metadata() {
        let metadata = AddNodeEnhanced::metadata();
        assert_eq!(metadata.node_type, "Add");
        assert_eq!(metadata.display_name, "Add");
        assert_eq!(metadata.description, "Adds two numeric values together");
        assert_eq!(metadata.inputs.len(), 2);
        assert_eq!(metadata.outputs.len(), 1);
        
        // Test input metadata
        assert_eq!(metadata.inputs[0].name, "A");
        assert_eq!(metadata.inputs[0].data_type, DataType::Float);
        assert!(!metadata.inputs[0].optional);
        
        assert_eq!(metadata.inputs[1].name, "B");
        assert_eq!(metadata.inputs[1].data_type, DataType::Float);
        assert!(!metadata.inputs[1].optional);
        
        // Test output metadata
        assert_eq!(metadata.outputs[0].name, "Result");
        assert_eq!(metadata.outputs[0].data_type, DataType::Float);
    }

    #[test]
    fn test_enhanced_add_node_creation() {
        let node = AddNodeEnhanced::create(Pos2::new(100.0, 100.0));
        assert_eq!(node.title, "Add");
        assert_eq!(node.inputs.len(), 2);
        assert_eq!(node.outputs.len(), 1);
        assert_eq!(node.inputs[0].name, "A");
        assert_eq!(node.inputs[1].name, "B");
        assert_eq!(node.outputs[0].name, "Result");
    }
    
    #[test]
    fn test_data_type_compatibility() {
        assert!(DataType::Float.can_connect_to(&DataType::Float));
        assert!(DataType::Float.can_connect_to(&DataType::Any));
        assert!(DataType::Any.can_connect_to(&DataType::Float));
        assert!(!DataType::Float.can_connect_to(&DataType::String));
    }
    
    #[test]
    fn test_category_system() {
        let math_category = NodeCategory::math();
        assert_eq!(math_category.name(), "Math");
        assert_eq!(math_category.display_string(), "Math");
        
        let materialx_shading = NodeCategory::materialx_shading();
        assert_eq!(materialx_shading.name(), "Shading");
        assert_eq!(materialx_shading.display_string(), "MaterialX > Shading");
        
        // Test hierarchy
        assert!(materialx_shading.is_child_of(&NodeCategory::new(&["MaterialX"])));
        assert!(!math_category.is_child_of(&materialx_shading));
    }
}