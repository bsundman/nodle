//! Workspace registry system for managing available workspaces

use crate::workspace::{Workspace, WorkspaceManager};
use crate::workspaces::{
    base::BaseWorkspace,
    materialx::MaterialXWorkspace,
    workspace_3d::Workspace3D,
};

/// Registry for managing available workspaces
pub struct WorkspaceRegistry;

impl WorkspaceRegistry {
    /// Create a new workspace manager with all available workspaces registered
    pub fn create_workspace_manager() -> WorkspaceManager {
        let mut manager = WorkspaceManager::new();
        
        // Register all available workspaces
        manager.register_workspace(Box::new(BaseWorkspace::new()));
        manager.register_workspace(Box::new(Workspace3D::new()));
        manager.register_workspace(Box::new(MaterialXWorkspace::new()));
        
        manager
    }
    
    /// Get list of all available workspace IDs and display names
    pub fn list_workspaces() -> Vec<(&'static str, &'static str)> {
        vec![
            (BaseWorkspace::new().id(), BaseWorkspace::new().display_name()),
            (Workspace3D::new().id(), Workspace3D::new().display_name()),
            (MaterialXWorkspace::new().id(), MaterialXWorkspace::new().display_name()),
        ]
    }
}

/// Trait for auto-registering workspaces (future enhancement)
pub trait WorkspaceFactory: Send + Sync {
    /// Create a new instance of this workspace
    fn create() -> Box<dyn Workspace> where Self: Sized;
    
    /// Get the workspace metadata
    fn metadata() -> WorkspaceMetadata where Self: Sized;
}

/// Metadata for workspace registration
pub struct WorkspaceMetadata {
    pub id: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub category: WorkspaceCategory,
}

/// Categories for organizing workspaces
#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceCategory {
    Generic,
    Shading,
    Simulation,
    GameDev,
    Custom(String),
}

impl WorkspaceCategory {
    pub fn name(&self) -> &str {
        match self {
            WorkspaceCategory::Generic => "Generic",
            WorkspaceCategory::Shading => "Shading",
            WorkspaceCategory::Simulation => "Simulation", 
            WorkspaceCategory::GameDev => "Game Development",
            WorkspaceCategory::Custom(name) => name,
        }
    }
}