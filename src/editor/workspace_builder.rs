//! Workspace and node creation orchestration system
//!
//! Handles high-level node creation logic including workspace detection,
//! node placement, workspace population, and view management integration.

use egui::{Pos2, Color32};
use crate::nodes::{NodeGraph, Node, NodeId};
use crate::workspace::WorkspaceManager;
use crate::editor::navigation::NavigationManager;

/// Orchestrates node and workspace creation with proper placement logic
pub struct WorkspaceBuilder;

impl WorkspaceBuilder {
    /// Create a new node or workspace at the specified position
    pub fn create_node(
        node_type: &str,
        position: Pos2,
        navigation: &NavigationManager,
        workspace_manager: &WorkspaceManager,
        graph: &mut NodeGraph,
    ) -> Option<NodeId> {
        // Check if this is a workspace node creation
        if Self::is_workspace_type(node_type) {
            Self::create_workspace_node(node_type, position, navigation, graph)
        } else {
            Self::create_regular_node(node_type, position, navigation, workspace_manager, graph)
        }
    }

    /// Check if a node type represents a workspace
    fn is_workspace_type(node_type: &str) -> bool {
        matches!(node_type, "WORKSPACE:2D" | "WORKSPACE:3D" | "WORKSPACE:MaterialX")
    }

    /// Create a workspace node with proper population
    fn create_workspace_node(
        node_type: &str,
        position: Pos2,
        navigation: &NavigationManager,
        graph: &mut NodeGraph,
    ) -> Option<NodeId> {
        let workspace_type = match node_type {
            "WORKSPACE:2D" => "2D",
            "WORKSPACE:3D" => "3D", 
            "WORKSPACE:MaterialX" => "MaterialX",
            _ => return None,
        };

        let mut workspace_node = Node::new_workspace(0, workspace_type, position);
        let node_id = workspace_node.id;
        
        // Workspace nodes have parameter panels for configuration
        workspace_node.set_panel_type(crate::nodes::interface::PanelType::Parameter);
        
        // Populate workspace with sample nodes
        Self::populate_workspace(&mut workspace_node, workspace_type);
        
        // Workspace nodes can only be created in the root graph
        if navigation.is_root_view() {
            graph.add_node(workspace_node);
            Some(node_id)
        } else {
            // Could potentially add to workspace internal graph, but for now restrict to root
            None
        }
    }

    /// Create a regular node using the factory system
    fn create_regular_node(
        node_type: &str,
        position: Pos2,
        navigation: &NavigationManager,
        workspace_manager: &WorkspaceManager,
        graph: &mut NodeGraph,
    ) -> Option<NodeId> {
        // Map display names to internal names if needed
        let internal_node_type = match node_type {
            "Nōdle 2D Workspace" => "WORKSPACE:2D",
            "Nōdle 3D Workspace" => "WORKSPACE:3D", 
            "Nōdle MaterialX Workspace" => "WORKSPACE:MaterialX",
            _ => node_type, // Use original name for generic nodes
        };
        
        // Create the node using the factory system
        println!("DEBUG: Attempting to create node type: '{}'", internal_node_type);
        let new_node = if let Some(workspace) = workspace_manager.get_active_workspace() {
            println!("DEBUG: Using workspace to create node");
            let result = workspace.create_workspace_node(internal_node_type, position);
            if result.is_none() {
                println!("DEBUG: Workspace failed to create node '{}'", internal_node_type);
            }
            result
        } else {
            // Fall back to default registry for nodes outside workspaces
            println!("DEBUG: Using default registry to create node");
            let registry = crate::nodes::factory::NodeRegistry::default();
            let result = registry.create_node(internal_node_type, position);
            if result.is_none() {
                println!("DEBUG: Default registry failed to create node '{}'", internal_node_type);
            }
            result
        };
        
        // Add the node to the appropriate graph based on current view
        if let Some(node) = new_node {
            println!("DEBUG: Successfully created node, adding to graph");
            
            // Add error handling for adding node to graph
            let node_id = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                Self::place_node_in_graph(node, navigation, graph)
            })) {
                Ok(id) => {
                    println!("DEBUG: Node placement completed successfully");
                    id
                }
                Err(_) => {
                    println!("❌ Panic occurred while placing node in graph");
                    None
                }
            };
            
            println!("DEBUG: Node creation completed successfully");
            node_id
        } else {
            println!("DEBUG: Failed to create node");
            None
        }
    }

    /// Place a node in the appropriate graph based on the current view
    fn place_node_in_graph(
        node: Node,
        navigation: &NavigationManager,
        graph: &mut NodeGraph,
    ) -> Option<NodeId> {
        println!("🔧 WorkspaceBuilder: Placing node '{}' panel_type={:?}", node.title, node.get_panel_type());
        
        // Check if this is a viewport node that might need plugin instance updating
        let is_viewport_node = node.get_panel_type() == Some(crate::nodes::interface::PanelType::Viewport);
        let temp_node_id = node.id; // This should be 0 for plugin nodes
        
        let final_node_id = if navigation.is_root_view() {
            // add_node returns the actual assigned ID
            let node_id = graph.add_node(node);
            println!("🔧 WorkspaceBuilder: Added node {} to root graph", node_id);
            Some(node_id)
        } else if let Some(workspace_node_id) = navigation.get_workspace_node_id() {
            // Try to add to workspace internal graph
            if let Some(workspace_node) = graph.nodes.get_mut(&workspace_node_id) {
                if let Some(internal_graph) = workspace_node.get_internal_graph_mut() {
                    // add_node returns the actual assigned ID
                    let node_id = internal_graph.add_node(node);
                    println!("🔧 WorkspaceBuilder: Added node {} to workspace {} internal graph", node_id, workspace_node_id);
                    Some(node_id)
                } else {
                    println!("❌ WorkspaceBuilder: Workspace {} has no internal graph", workspace_node_id);
                    None
                }
            } else {
                println!("❌ WorkspaceBuilder: Workspace node {} not found", workspace_node_id);
                None
            }
        } else {
            println!("❌ WorkspaceBuilder: Not in root view and no workspace node ID");
            None
        };
        
        // Update plugin instance storage if this was a viewport plugin node
        if is_viewport_node && temp_node_id >= crate::constants::node::TEMP_ID_START {
            if let Some(new_node_id) = final_node_id {
                Self::update_plugin_instance_id(temp_node_id, new_node_id);
            }
        }
        
        final_node_id
    }
    
    /// Update plugin instance storage with the real node ID
    fn update_plugin_instance_id(temp_id: NodeId, real_id: NodeId) {
        println!("🔧 WorkspaceBuilder: Updating plugin instance ID from {} to {}", temp_id, real_id);
        
        if let Some(plugin_manager) = crate::workspace::get_global_plugin_manager() {
            if let Ok(mut manager) = plugin_manager.lock() {
                // Move the plugin instance from temp ID to real ID
                if let Some(plugin_instance) = manager.plugin_node_instances.remove(&temp_id) {
                    manager.plugin_node_instances.insert(real_id, plugin_instance);
                    println!("✅ WorkspaceBuilder: Successfully moved plugin instance from {} to {}", temp_id, real_id);
                } else {
                    println!("❌ WorkspaceBuilder: No plugin instance found for temp ID {}", temp_id);
                }
            } else {
                println!("❌ WorkspaceBuilder: Failed to lock plugin manager for ID update");
            }
        } else {
            println!("❌ WorkspaceBuilder: No global plugin manager available for ID update");
        }
    }

    /// Populate a workspace with appropriate sample nodes
    fn populate_workspace(workspace_node: &mut Node, workspace_type: &str) {
        match workspace_type {
            "2D" => Self::populate_2d_workspace(workspace_node),
            "3D" => Self::populate_3d_workspace(workspace_node),
            "MaterialX" => Self::populate_materialx_workspace(workspace_node),
            _ => {}, // Unknown workspace type, leave empty
        }
    }

    /// Populate a 2D workspace node with sample nodes for demonstration
    fn populate_2d_workspace(workspace_node: &mut Node) {
        if let Some(_internal_graph) = workspace_node.get_internal_graph_mut() {
            // 2D workspace starts empty - users can add nodes via context menu
        }
    }
    
    /// Populate a 3D workspace node with sample nodes for demonstration
    fn populate_3d_workspace(workspace_node: &mut Node) {
        if let Some(_internal_graph) = workspace_node.get_internal_graph_mut() {
            // 3D context starts empty - users can add nodes via context menu
        }
    }
    
    /// Populate a MaterialX context node with sample nodes for demonstration
    fn populate_materialx_workspace(workspace_node: &mut Node) {
        if let Some(internal_graph) = workspace_node.get_internal_graph_mut() {
            // Create sample MaterialX nodes
            let mut image_node = Node::new(1, "Image", Pos2::new(50.0, 100.0))
                .with_color(Color32::from_rgb(140, 180, 140));
            image_node.add_input("File");
            image_node.add_input("UV");
            image_node.add_output("Color");
            internal_graph.add_node(image_node);
            
            let mut multiply_node = Node::new(2, "Multiply", Pos2::new(250.0, 100.0))
                .with_color(Color32::from_rgb(140, 140, 180));
            multiply_node.add_input("Input1");
            multiply_node.add_input("Input2");
            multiply_node.add_output("Output");
            internal_graph.add_node(multiply_node);
            
            let mut surface_node = Node::new(3, "Standard Surface", Pos2::new(450.0, 100.0))
                .with_color(Color32::from_rgb(180, 140, 140));
            surface_node.add_input("Base Color");
            surface_node.add_input("Metallic");
            surface_node.add_input("Roughness");
            surface_node.add_output("BSDF");
            internal_graph.add_node(surface_node);
            
            // Create a connection from Image to Multiply
            let connection1 = crate::nodes::Connection::new(1, 0, 2, 0);
            let _ = internal_graph.add_connection(connection1);
            
            // Create a connection from Multiply to Standard Surface
            let connection2 = crate::nodes::Connection::new(2, 0, 3, 0);
            let _ = internal_graph.add_connection(connection2);
        }
    }

    /// Check if a node can be placed in the current view context
    pub fn can_place_node_in_context(
        node_type: &str,
        navigation: &NavigationManager,
        graph: &NodeGraph,
    ) -> bool {
        // Workspace nodes can only be created in root view
        if Self::is_workspace_type(node_type) {
            return navigation.is_root_view();
        }
        
        // Regular nodes can be placed in any context
        // Could add workspace-specific compatibility checking here in the future
        true
    }

    /// Get appropriate position for a new node in the current context
    pub fn get_suggested_position(
        navigation: &NavigationManager,
        graph: &NodeGraph,
        base_position: Pos2,
    ) -> Pos2 {
        // For now, just return the base position
        // Could implement smart positioning logic here (avoiding overlaps, etc.)
        base_position
    }

    /// Validate node type compatibility with current workspace context
    pub fn validate_node_compatibility(
        node_type: &str,
        navigation: &NavigationManager,
        workspace_manager: &WorkspaceManager,
        graph: &NodeGraph,
    ) -> NodeCompatibility {
        // Check workspace restrictions
        if Self::is_workspace_type(node_type) && !navigation.is_root_view() {
            return NodeCompatibility::Invalid("Workspace nodes can only be created in root view".to_string());
        }

        // Check if we're in a workspace context
        if let Some(workspace_type) = navigation.get_workspace_type(graph) {
            // Could add workspace-specific validation here
            // For now, allow all node types in all workspaces
            NodeCompatibility::Valid
        } else {
            // Root context - allow everything
            NodeCompatibility::Valid
        }
    }
}

/// Result of node compatibility validation
#[derive(Debug, Clone)]
pub enum NodeCompatibility {
    /// Node can be created
    Valid,
    /// Node cannot be created with reason
    Invalid(String),
    /// Node can be created but with warnings
    Warning(String),
}

impl NodeCompatibility {
    /// Check if the node can be created
    pub fn is_valid(&self) -> bool {
        matches!(self, NodeCompatibility::Valid | NodeCompatibility::Warning(_))
    }

    /// Get warning or error message if any
    pub fn message(&self) -> Option<&str> {
        match self {
            NodeCompatibility::Valid => None,
            NodeCompatibility::Invalid(msg) | NodeCompatibility::Warning(msg) => Some(msg),
        }
    }
}