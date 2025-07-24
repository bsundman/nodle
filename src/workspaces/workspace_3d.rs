//! 3D workspace implementation for 3D graphics workflows

use crate::workspace::{Workspace, WorkspaceMenuItem};
use crate::nodes::factory::NodeRegistry;
use crate::nodes::three_d::*;
use crate::nodes::three_d::geometry::*;
use crate::nodes::three_d::transform::TranslateNode;
use crate::nodes::utility::{NullNode, TestNode};
use log::{debug, info, warn};

/// 3D workspace for 3D graphics, rendering, and modeling workflows
pub struct Workspace3D {
    node_registry: NodeRegistry,
}

impl Workspace3D {
    pub fn new() -> Self {
        info!("===== WORKSPACE3D BEING CREATED =====");
        let mut node_registry = NodeRegistry::new(); // Start with empty registry
        
        // Register utility nodes - available across workspaces
        node_registry.register::<NullNode>();
        node_registry.register::<TestNode>();
        
        // Register 3D transform nodes
        node_registry.register::<TranslateNode>();
        node_registry.register::<crate::nodes::three_d::transform::RotateNode>();
        node_registry.register::<crate::nodes::three_d::transform::ScaleNode>();
        
        // Register 3D geometry nodes
        node_registry.register::<CubeNodeFactory>();
        node_registry.register::<SphereNodeFactory>();
        node_registry.register::<CylinderNodeFactory>();
        node_registry.register::<ConeNodeFactory>();
        node_registry.register::<PlaneNodeFactory>();
        node_registry.register::<CapsuleNodeFactory>();
        
        // Register 3D lighting nodes
        node_registry.register::<crate::nodes::three_d::lighting::PointLightNode>();
        node_registry.register::<crate::nodes::three_d::lighting::DirectionalLightNode>();
        node_registry.register::<crate::nodes::three_d::lighting::SpotLightNode>();
        
        // Register 3D modify nodes
        node_registry.register::<crate::nodes::three_d::modify::ReverseNode>();
        
        // Register 3D UI nodes
        node_registry.register::<crate::nodes::three_d::ui::viewport::ViewportNode>();
        
        // Register USD data input nodes
        node_registry.register::<crate::nodes::data::usd_file_reader::UsdFileReaderNodeFactory>();
        
        // Register USD output nodes  
        node_registry.register::<crate::nodes::three_d::ui::scenegraph::ScenegraphNodeFactory>();
        node_registry.register::<crate::nodes::three_d::ui::attributes::AttributesNodeFactory>();
        
        // Register Console node for debugging
        node_registry.register::<crate::nodes::output::ConsoleNodeFactory>();
        
        // USD nodes now provided by USD plugin
        
        // Try to register plugin nodes using the global plugin manager
        if let Some(plugin_manager) = crate::workspace::get_global_plugin_manager() {
            if let Ok(manager) = plugin_manager.lock() {
                if let Err(e) = manager.register_plugin_nodes(&mut node_registry) {
                    warn!("Failed to register plugin nodes in Workspace3D: {}", e);
                } else {
                    let loaded_plugins = manager.get_loaded_plugins();
                    info!("Plugin nodes registered with Workspace3D: {} plugins", loaded_plugins.len());
                    
                    // Debug: Show what's in the registry now
                    debug!("Workspace3D registry categories after plugin registration:");
                    let menu_items = node_registry.generate_menu_structure(&["3D"]);
                    for item in &menu_items {
                        if let crate::workspace::WorkspaceMenuItem::Category { name, items } = item {
                            debug!("  Category '{}' has {} items", name, items.len());
                            for node_item in items {
                                match node_item {
                                    crate::workspace::WorkspaceMenuItem::Node { name, node_type } => {
                                        debug!("    {} ({})", name, node_type);
                                    }
                                    crate::workspace::WorkspaceMenuItem::Category { name: sub_name, items: sub_items } => {
                                        debug!("    Subcategory '{}' has {} items", sub_name, sub_items.len());
                                        for sub_node in sub_items {
                                            if let crate::workspace::WorkspaceMenuItem::Node { name, node_type } = sub_node {
                                                debug!("      {} ({})", name, node_type);
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Self {
            node_registry,
        }
    }
}

impl Workspace for Workspace3D {
    fn id(&self) -> &'static str {
        "3d"
    }
    
    fn display_name(&self) -> &'static str {
        "3D"
    }
    
    fn get_menu_structure(&self) -> Vec<WorkspaceMenuItem> {
        // Just return core nodes menu - plugin menu integration is handled at the workspace manager level
        self.node_registry.generate_menu_structure(&["3D"])
    }
    
    fn is_generic_node_compatible(&self, node_type: &str) -> bool {
        // Only allow output nodes in 3D workspace for debugging
        matches!(node_type, 
            "Print" | "Debug"
        )
    }
    
    fn create_workspace_node(&self, node_type: &str, position: egui::Pos2) -> Option<crate::nodes::Node> {
        // Try to create 3D-specific nodes using the registry first
        if let Some(node) = self.node_registry.create_node(node_type, position) {
            return Some(node);
        }
        
        // Fall back to generic node registry for whitelisted nodes
        if self.is_generic_node_compatible(node_type) {
            self.node_registry.create_node(node_type, position)
        } else {
            None
        }
    }
}

impl Default for Workspace3D {
    fn default() -> Self {
        Self::new()
    }
}