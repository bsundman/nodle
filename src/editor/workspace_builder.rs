//! Workspace and node creation orchestration system
//!
//! Handles high-level node creation logic including workspace detection,
//! node placement, workspace population, and view management integration.

use egui::{Pos2, Color32};
use crate::nodes::{NodeGraph, Node};
use crate::workspace::WorkspaceManager;
use crate::editor::view_manager::ViewManager;

/// Orchestrates node and workspace creation with proper placement logic
pub struct WorkspaceBuilder;

impl WorkspaceBuilder {
    /// Create a new node or workspace at the specified position
    pub fn create_node(
        node_type: &str,
        position: Pos2,
        view_manager: &ViewManager,
        workspace_manager: &WorkspaceManager,
        graph: &mut NodeGraph,
    ) -> bool {
        // Check if this is a workspace node creation
        if Self::is_workspace_type(node_type) {
            Self::create_workspace_node(node_type, position, view_manager, graph)
        } else {
            Self::create_regular_node(node_type, position, view_manager, workspace_manager, graph)
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
        view_manager: &ViewManager,
        graph: &mut NodeGraph,
    ) -> bool {
        let workspace_type = match node_type {
            "WORKSPACE:2D" => "2D",
            "WORKSPACE:3D" => "3D", 
            "WORKSPACE:MaterialX" => "MaterialX",
            _ => return false,
        };

        let mut workspace_node = Node::new_workspace(0, workspace_type, position);
        
        // Workspace nodes have parameter panels for configuration
        workspace_node.set_panel_type(crate::nodes::interface::PanelType::Parameter);
        
        // Populate workspace with sample nodes
        Self::populate_workspace(&mut workspace_node, workspace_type);
        
        // Workspace nodes can only be created in the root graph
        if view_manager.is_root_view() {
            graph.add_node(workspace_node);
            true
        } else {
            // Could potentially add to workspace internal graph, but for now restrict to root
            false
        }
    }

    /// Create a regular node using the factory system
    fn create_regular_node(
        node_type: &str,
        position: Pos2,
        view_manager: &ViewManager,
        workspace_manager: &WorkspaceManager,
        graph: &mut NodeGraph,
    ) -> bool {
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
            Self::place_node_in_graph(node, view_manager, graph);
            println!("DEBUG: Node creation completed successfully");
            true
        } else {
            println!("DEBUG: Failed to create node");
            false
        }
    }

    /// Place a node in the appropriate graph based on the current view
    fn place_node_in_graph(
        node: Node,
        view_manager: &ViewManager,
        graph: &mut NodeGraph,
    ) {
        if view_manager.is_root_view() {
            graph.add_node(node);
        } else if let Some(workspace_node_id) = view_manager.get_workspace_node_id() {
            // Try to add to workspace internal graph
            if let Some(workspace_node) = graph.nodes.get_mut(&workspace_node_id) {
                if let Some(internal_graph) = workspace_node.get_internal_graph_mut() {
                    internal_graph.add_node(node);
                }
            }
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
        view_manager: &ViewManager,
        graph: &NodeGraph,
    ) -> bool {
        // Workspace nodes can only be created in root view
        if Self::is_workspace_type(node_type) {
            return view_manager.is_root_view();
        }
        
        // Regular nodes can be placed in any context
        // Could add workspace-specific compatibility checking here in the future
        true
    }

    /// Get appropriate position for a new node in the current context
    pub fn get_suggested_position(
        view_manager: &ViewManager,
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
        view_manager: &ViewManager,
        workspace_manager: &WorkspaceManager,
        graph: &NodeGraph,
    ) -> NodeCompatibility {
        // Check workspace restrictions
        if Self::is_workspace_type(node_type) && !view_manager.is_root_view() {
            return NodeCompatibility::Invalid("Workspace nodes can only be created in root view".to_string());
        }

        // Check if we're in a workspace context
        if let Some(workspace_type) = view_manager.get_workspace_type(graph) {
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