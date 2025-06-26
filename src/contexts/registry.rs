//! Context registry system for managing available contexts

use crate::context::{Context, ContextManager};
use crate::contexts::{
    base::BaseContext,
    subcontexts::materialx::MaterialXContext,
    context_3d::Context3D,
};

/// Registry for managing available contexts
pub struct ContextRegistry;

impl ContextRegistry {
    /// Create a new context manager with all available contexts registered
    pub fn create_context_manager() -> ContextManager {
        let mut manager = ContextManager::new();
        
        // Register all available contexts
        manager.register_context(Box::new(BaseContext::new()));
        manager.register_context(Box::new(Context3D::new()));
        manager.register_context(Box::new(MaterialXContext::new()));
        
        manager
    }
    
    /// Get list of all available context IDs and display names
    pub fn list_contexts() -> Vec<(&'static str, &'static str)> {
        vec![
            (BaseContext::new().id(), BaseContext::new().display_name()),
            (Context3D::new().id(), Context3D::new().display_name()),
            (MaterialXContext::new().id(), MaterialXContext::new().display_name()),
        ]
    }
}

/// Trait for auto-registering contexts (future enhancement)
pub trait ContextFactory: Send + Sync {
    /// Create a new instance of this context
    fn create() -> Box<dyn Context> where Self: Sized;
    
    /// Get the context metadata
    fn metadata() -> ContextMetadata where Self: Sized;
}

/// Metadata for context registration
pub struct ContextMetadata {
    pub id: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub category: ContextCategory,
}

/// Categories for organizing contexts
#[derive(Debug, Clone, PartialEq)]
pub enum ContextCategory {
    Generic,
    Shading,
    Simulation,
    GameDev,
    Custom(String),
}

impl ContextCategory {
    pub fn name(&self) -> &str {
        match self {
            ContextCategory::Generic => "Generic",
            ContextCategory::Shading => "Shading",
            ContextCategory::Simulation => "Simulation", 
            ContextCategory::GameDev => "Game Development",
            ContextCategory::Custom(name) => name,
        }
    }
}