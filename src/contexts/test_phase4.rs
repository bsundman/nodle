//! Phase 4 testing - Context system with collections

#[cfg(test)]
mod tests {
    use super::super::registry::ContextRegistry;
    use super::super::base::BaseContext;
    use super::super::materialx::*;
    use crate::context::Context;
    use crate::nodes::factory::NodeFactory;
    use egui::Pos2;

    #[test]
    fn test_context_registry_creation() {
        let manager = ContextRegistry::create_context_manager();
        let contexts = manager.get_contexts();
        
        // Should have 2 contexts: Base, MaterialX
        assert_eq!(contexts.len(), 2);
        
        let context_names: Vec<&str> = contexts.iter()
            .map(|c| c.display_name())
            .collect();
            
        assert!(context_names.contains(&"Generic"));
        assert!(context_names.contains(&"MaterialX"));
    }
    
    #[test]
    fn test_base_context_node_creation() {
        let context = BaseContext::new();
        
        // Test enhanced nodes can be created through base context
        let add_node = context.create_context_node("Add", Pos2::new(0.0, 0.0));
        assert!(add_node.is_some());
        
        let node = add_node.unwrap();
        assert_eq!(node.title, "Add");
        assert_eq!(node.inputs.len(), 2);
        assert_eq!(node.outputs.len(), 1);
    }
    
    #[test]
    fn test_materialx_context_nodes() {
        let context = MaterialXContext::new();
        
        // Test MaterialX-specific enhanced nodes
        let noise_node = context.create_context_node("MaterialX_Noise", Pos2::new(0.0, 0.0));
        assert!(noise_node.is_some());
        
        let texture_node = context.create_context_node("MaterialX_Texture", Pos2::new(0.0, 0.0));
        assert!(texture_node.is_some());
        
        let mix_node = context.create_context_node("MaterialX_Mix", Pos2::new(0.0, 0.0));
        assert!(mix_node.is_some());
        
        let surface_node = context.create_context_node("MaterialX_StandardSurface", Pos2::new(0.0, 0.0));
        assert!(surface_node.is_some());
        
        // Test node properties
        let noise = noise_node.unwrap();
        assert_eq!(noise.title, "MaterialX_Noise");
        
        // Test that enhanced factory system provides rich metadata
        let metadata = MaterialXNoiseNode::metadata();
        assert_eq!(metadata.display_name, "Noise");
        assert_eq!(metadata.description, "Generates procedural noise patterns for MaterialX shading");
        assert_eq!(metadata.category.name(), "Texture");
    }
    
    #[test]
    fn test_materialx_context_compatibility() {
        let context = MaterialXContext::new();
        
        // Test generic node compatibility
        assert!(context.is_generic_node_compatible("Add"));
        assert!(context.is_generic_node_compatible("Multiply"));
        assert!(context.is_generic_node_compatible("Constant"));
        
        // Test incompatible nodes
        assert!(!context.is_generic_node_compatible("Print"));
        assert!(!context.is_generic_node_compatible("Debug"));
        assert!(!context.is_generic_node_compatible("Variable"));
    }
    
    #[test]
    fn test_context_menu_structure() {
        let context = MaterialXContext::new();
        let menu_structure = context.get_menu_structure();
        
        assert_eq!(menu_structure.len(), 1); // One main MaterialX category
        
        if let crate::context::ContextMenuItem::Category { name, items } = &menu_structure[0] {
            assert_eq!(name, "MaterialX");
            assert!(items.len() > 0); // Should have subcategories
        } else {
            panic!("Expected MaterialX category");
        }
    }
    
    #[test]
    fn test_enhanced_node_factory_integration() {
        // Test that MaterialX nodes work with the factory system
        let metadata = MaterialXStandardSurfaceNode::metadata();
        
        assert_eq!(metadata.node_type, "MaterialX_StandardSurface");
        assert_eq!(metadata.display_name, "Standard Surface");
        assert!(metadata.inputs.len() > 0); // Should have base_color, metallic, etc.
        assert_eq!(metadata.outputs.len(), 1); // Surface output
        
        // Test hierarchical categories
        assert_eq!(metadata.category.display_string(), "MaterialX > Shading");
    }
}