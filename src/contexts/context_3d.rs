//! 3D context implementation for 3D graphics workflows

use crate::context::{Context, ContextMenuItem};
use crate::nodes::factory::{NodeRegistry, NodeFactory};
use crate::nodes::three_d::*;

/// 3D context for 3D graphics, rendering, and modeling workflows
pub struct Context3D {
    node_registry: NodeRegistry,
}

impl Context3D {
    pub fn new() -> Self {
        eprintln!("📦 ===== CONTEXT3D BEING CREATED =====");
        let mut node_registry = NodeRegistry::default();
        
        // Register 3D-specific nodes with interface panels
        node_registry.register::<crate::nodes::three_d::transform::TranslateNode>();
        node_registry.register::<crate::nodes::three_d::transform::RotateNode>();
        node_registry.register::<crate::nodes::three_d::transform::ScaleNode>();
        node_registry.register::<crate::nodes::three_d::geometry::CubeNode>();
        node_registry.register::<crate::nodes::three_d::geometry::SphereNode>();
        node_registry.register::<crate::nodes::three_d::geometry::PlaneNode>();
        node_registry.register::<crate::nodes::three_d::lighting::PointLightNode>();
        node_registry.register::<crate::nodes::three_d::lighting::DirectionalLightNode>();
        node_registry.register::<crate::nodes::three_d::lighting::SpotLightNode>();
        node_registry.register::<crate::nodes::three_d::output::viewport::ViewportNode>();
        
        // Register utility nodes (like Null node)
        node_registry.register::<crate::nodes::utility::null::parameters::NullNode>();
        
        // Try to register plugin nodes if any are available
        let mut plugin_manager = crate::plugins::PluginManager::new();
        if let Ok(loaded_plugins) = plugin_manager.discover_and_load_plugins() {
            if let Err(e) = plugin_manager.register_plugin_nodes(&mut node_registry) {
                println!("⚠️  Failed to register plugin nodes: {}", e);
            } else {
                println!("🔗 Plugin nodes registered with 3D context: {} plugins", loaded_plugins.len());
                
                // Debug: Show what's in the registry now
                println!("🔍 Registry categories after plugin registration:");
                let menu_items = node_registry.generate_menu_structure(&["3D"]);
                for item in &menu_items {
                    if let crate::workspace::WorkspaceMenuItem::Category { name, items } = item {
                        println!("  📁 Category '{}' has {} items", name, items.len());
                        for node_item in items {
                            if let crate::workspace::WorkspaceMenuItem::Node { name, node_type } = node_item {
                                println!("    📄 {} ({})", name, node_type);
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

impl Context for Context3D {
    fn id(&self) -> &'static str {
        "3d"
    }
    
    fn display_name(&self) -> &'static str {
        "3D"
    }
    
    fn get_menu_structure(&self) -> Vec<ContextMenuItem> {
        // Use dynamic menu generation from NodeRegistry to include plugin nodes
        let workspace_filter = vec!["3D"];
        let menu_items = self.node_registry.generate_menu_structure(&workspace_filter);
        
        // Convert WorkspaceMenuItem to ContextMenuItem
        menu_items.into_iter().map(|item| {
            match item {
                crate::workspace::WorkspaceMenuItem::Category { name, items } => {
                    ContextMenuItem::Category {
                        name,
                        items: items.into_iter().map(|node_item| {
                            match node_item {
                                crate::workspace::WorkspaceMenuItem::Node { name, node_type } => {
                                    ContextMenuItem::Node { name, node_type }
                                }
                                crate::workspace::WorkspaceMenuItem::Category { .. } => {
                                    // Nested categories not supported in ContextMenuItem, flatten
                                    ContextMenuItem::Node { name: "Unsupported".to_string(), node_type: "".to_string() }
                                }
                                crate::workspace::WorkspaceMenuItem::Workspace { name, .. } => {
                                    // Workspaces not supported in ContextMenuItem, convert to node
                                    ContextMenuItem::Node { name, node_type: "".to_string() }
                                }
                            }
                        }).collect(),
                    }
                }
                crate::workspace::WorkspaceMenuItem::Node { name, node_type } => {
                    ContextMenuItem::Node { name, node_type }
                }
                crate::workspace::WorkspaceMenuItem::Workspace { name, .. } => {
                    // Workspaces not supported at top level in ContextMenuItem
                    ContextMenuItem::Node { name, node_type: "".to_string() }
                }
            }
        }).collect()
    }
    
    fn is_generic_node_compatible(&self, node_type: &str) -> bool {
        // Only allow output nodes in 3D context for debugging
        matches!(node_type, 
            "Print" | "Debug"
        )
    }
    
    fn create_context_node(&self, node_type: &str, position: egui::Pos2) -> Option<crate::nodes::Node> {
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

impl Default for Context3D {
    fn default() -> Self {
        Self::new()
    }
}