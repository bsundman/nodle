//! Test for the enhanced node registry system

#[cfg(test)]
mod tests {
    use super::super::factory::*;
    use super::super::math::AddNodeEnhanced;
    use egui::Pos2;

    #[test]
    fn test_enhanced_node_factory_system() {
        // Test the enhanced factory system
        let metadata = AddNodeEnhanced::metadata();
        
        assert_eq!(metadata.node_type, "Add");
        assert_eq!(metadata.display_name, "Add");
        assert_eq!(metadata.description, "Adds two numeric values together");
        
        // Test category system
        let math_category = NodeCategory::math();
        assert_eq!(math_category.name(), "Math");
        assert_eq!(metadata.category, math_category);
        
        // Test port definitions
        assert_eq!(metadata.inputs.len(), 2);
        assert_eq!(metadata.outputs.len(), 1);
        
        let input_a = &metadata.inputs[0];
        assert_eq!(input_a.name, "A");
        assert_eq!(input_a.data_type, DataType::Float);
        assert!(!input_a.optional);
        assert_eq!(input_a.description, Some("First input value".to_string()));
        
        let output = &metadata.outputs[0];
        assert_eq!(output.name, "Result");
        assert_eq!(output.data_type, DataType::Float);
    }
    
    #[test]
    fn test_node_creation_compatibility() {
        // Test that enhanced nodes can still be created using the old system
        let node = AddNodeEnhanced::create(Pos2::new(100.0, 100.0));
        
        assert_eq!(node.title, "Add");
        assert_eq!(node.inputs.len(), 2);
        assert_eq!(node.outputs.len(), 1);
        assert_eq!(node.inputs[0].name, "A");
        assert_eq!(node.inputs[1].name, "B");
        assert_eq!(node.outputs[0].name, "Result");
    }
    
    #[test]
    fn test_hierarchical_categories() {
        let math_cat = NodeCategory::math();
        let materialx_shading = NodeCategory::materialx_shading();
        let materialx_root = NodeCategory::new(&["MaterialX"]);
        
        // Test hierarchy
        assert!(materialx_shading.is_child_of(&materialx_root));
        assert!(!math_cat.is_child_of(&materialx_root));
        assert!(!materialx_root.is_child_of(&materialx_shading));
        
        // Test display strings
        assert_eq!(math_cat.display_string(), "Math");
        assert_eq!(materialx_shading.display_string(), "MaterialX > Shading");
        
        // Test parent relationships
        assert_eq!(materialx_shading.parent(), Some(materialx_root));
        assert_eq!(math_cat.parent(), None);
    }
    
    #[test]
    fn test_data_type_compatibility() {
        // Test basic compatibility
        assert!(DataType::Float.can_connect_to(&DataType::Float));
        assert!(DataType::Float.can_connect_to(&DataType::Any));
        assert!(DataType::Any.can_connect_to(&DataType::Float));
        
        // Test incompatibility
        assert!(!DataType::Float.can_connect_to(&DataType::String));
        assert!(!DataType::Vector3.can_connect_to(&DataType::Boolean));
        
        // Test Any type compatibility
        assert!(DataType::Any.can_connect_to(&DataType::Vector3));
        assert!(DataType::Color.can_connect_to(&DataType::Any));
    }
    
    #[test]
    fn test_port_definition_builder() {
        let port = PortDefinition::required("Input", DataType::Float)
            .with_description("A test input port");
            
        assert_eq!(port.name, "Input");
        assert_eq!(port.data_type, DataType::Float);
        assert!(!port.optional);
        assert_eq!(port.description, Some("A test input port".to_string()));
        
        let optional_port = PortDefinition::optional("Optional", DataType::String);
        assert!(optional_port.optional);
        assert_eq!(optional_port.description, None);
    }
    
    #[test]
    fn test_node_registry_legacy_compatibility() {
        let registry = NodeRegistry::new();
        
        // Test that legacy node creation still works
        let add_node = registry.create_node("Add", Pos2::new(0.0, 0.0));
        assert!(add_node.is_some());
        
        let node = add_node.unwrap();
        assert_eq!(node.title, "Add");
        assert_eq!(node.inputs.len(), 2);
        assert_eq!(node.outputs.len(), 1);
        
        // Test unknown node type
        let unknown = registry.create_node("Unknown", Pos2::new(0.0, 0.0));
        assert!(unknown.is_none());
    }
}