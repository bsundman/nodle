//! Phase 4 testing - Workspace system with collections

#[cfg(test)]
mod tests {
    use super::super::registry::WorkspaceRegistry;
    use super::super::base::BaseWorkspace;
    use super::super::materialx::*;
    use crate::workspace::Workspace;
    use crate::nodes::factory::NodeFactory;
    use egui::Pos2;

    #[test]
    fn test_workspace_registry_creation() {
        let manager = WorkspaceRegistry::create_workspace_manager();
        let workspaces = manager.get_workspaces();
        
        // Should have 3 workspaces: Base, 3D, MaterialX
        assert_eq!(workspaces.len(), 3);
        
        let workspace_names: Vec<&str> = workspaces.iter()
            .map(|w| w.display_name())
            .collect();
            
        assert!(workspace_names.contains(&"Generic"));
        assert!(workspace_names.contains(&"3D"));
        assert!(workspace_names.contains(&"MaterialX"));
    }
    
    #[test]
    fn test_base_workspace_node_creation() {
        let workspace = BaseWorkspace::new();
        
        // Test enhanced nodes can be created through base workspace
        let add_node = workspace.create_workspace_node("Add", Pos2::new(0.0, 0.0));
        assert!(add_node.is_some());
        
        let node = add_node.unwrap();
        assert_eq!(node.title, "Add");
        assert_eq!(node.inputs.len(), 2);
        assert_eq!(node.outputs.len(), 1);
    }
    
    #[test]
    fn test_materialx_workspace_nodes() {
        let workspace = MaterialXWorkspace::new();
        
        // Test MaterialX-specific enhanced nodes
        let noise_node = workspace.create_workspace_node("MaterialX_Noise", Pos2::new(0.0, 0.0));
        assert!(noise_node.is_some());
        
        let texture_node = workspace.create_workspace_node("MaterialX_Texture", Pos2::new(0.0, 0.0));
        assert!(texture_node.is_some());
        
        let mix_node = workspace.create_workspace_node("MaterialX_Mix", Pos2::new(0.0, 0.0));
        assert!(mix_node.is_some());
        
        let surface_node = workspace.create_workspace_node("MaterialX_StandardSurface", Pos2::new(0.0, 0.0));
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
    fn test_materialx_workspace_compatibility() {
        let workspace = MaterialXWorkspace::new();
        
        // Test generic node compatibility
        assert!(workspace.is_generic_node_compatible("Add"));
        assert!(workspace.is_generic_node_compatible("Multiply"));
        assert!(workspace.is_generic_node_compatible("Print"));
        assert!(workspace.is_generic_node_compatible("Debug"));
        
        // Test incompatible nodes (none for now, MaterialX is permissive)
        assert!(!workspace.is_generic_node_compatible("Variable")); // Example incompatible
    }
    
    #[test]
    fn test_workspace_menu_structure() {
        let workspace = MaterialXWorkspace::new();
        let menu_structure = workspace.get_menu_structure();
        
        assert_eq!(menu_structure.len(), 1); // One main MaterialX category
        
        if let crate::workspace::WorkspaceMenuItem::Category { name, items } = &menu_structure[0] {
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