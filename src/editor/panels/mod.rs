//! Interface panel management system for the node editor
//!
//! Handles rendering and managing interface panels for nodes, including
//! parameter editing, panel positioning, and state management.

mod parameter;
mod viewport;

pub use parameter::ParameterPanel;
pub use viewport::ViewportPanel;

use egui::{Ui, Context};
use crate::nodes::{
    NodeGraph, Node, NodeId, InterfacePanelManager, PanelType,
};
use crate::nodes::interface::NodeInterfacePanel;
use std::collections::HashMap;

// Import GraphView from the parent module
use crate::editor::GraphView;

/// Actions that can be performed on interface panels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelAction {
    None,
    Close,
    CloseAll, // Close all panels in a stacked window
    Minimize,
    Restore,
    ToggleStack,
    TogglePin,
}

/// Manages interface panels for the node editor
pub struct PanelManager {
    /// The core interface panel manager for state tracking
    interface_panel_manager: InterfacePanelManager,
    /// Current menu bar height for window constraints
    current_menu_bar_height: f32,
    /// Parameter panel renderer
    parameter_panel: ParameterPanel,
    /// Viewport panel renderer
    viewport_panel: ViewportPanel,
}

impl PanelManager {
    /// Create a new panel manager
    pub fn new() -> Self {
        Self {
            interface_panel_manager: InterfacePanelManager::new(),
            current_menu_bar_height: 0.0,
            parameter_panel: ParameterPanel::new(),
            viewport_panel: ViewportPanel::new(),
        }
    }

    /// Get a reference to the underlying interface panel manager
    pub fn interface_panel_manager(&self) -> &InterfacePanelManager {
        &self.interface_panel_manager
    }

    /// Get a mutable reference to the underlying interface panel manager
    pub fn interface_panel_manager_mut(&mut self) -> &mut InterfacePanelManager {
        &mut self.interface_panel_manager
    }

    /// Set the current menu bar height for window constraints
    pub fn set_menu_bar_height(&mut self, height: f32) {
        self.current_menu_bar_height = height;
    }

    /// Render all interface panels for the given nodes
    pub fn render_interface_panels(
        &mut self, 
        ui: &mut Ui, 
        viewed_nodes: &HashMap<NodeId, Node>, 
        menu_bar_height: f32,
        current_view: &GraphView,
        graph: &mut NodeGraph,
    ) {
        // Store menu bar height
        self.set_menu_bar_height(menu_bar_height);
        let ctx = ui.ctx();
        
        // Track which nodes to close and which actions to apply (to avoid borrowing issues)
        let mut nodes_to_close: Vec<NodeId> = Vec::new();
        let mut nodes_to_toggle_stack: Vec<NodeId> = Vec::new();
        let mut nodes_to_toggle_pin: Vec<NodeId> = Vec::new();
        
        // First pass - ensure panel manager knows about all visible nodes with their panel types
        for (&node_id, node) in viewed_nodes {
            if node.visible {
                // Use the node's own panel type if it has one
                if let Some(panel_type) = node.get_panel_type() {
                    // Only update if not already set to avoid overwriting user preferences
                    if !self.interface_panel_manager.has_panel_type_set(node_id) {
                        self.interface_panel_manager.set_panel_type(node_id, panel_type);
                    }
                }
            }
        }
        
        // Second pass - render panels based on node's panel type
        for (&node_id, node) in viewed_nodes {
            if node.visible {
                // Use the node's panel type directly
                let panel_type = node.get_panel_type().unwrap_or(PanelType::Parameter);
                
                // Delegate to the appropriate panel type renderer
                let panel_action = match panel_type {
                    PanelType::Viewport => {
                        self.viewport_panel.render(
                            ctx,
                            node_id,
                            node,
                            &mut self.interface_panel_manager,
                            menu_bar_height,
                            viewed_nodes,
                        )
                    },
                    _ => {
                        // All other types use parameter panel for now
                        self.parameter_panel.render(
                            ctx,
                            node_id,
                            node,
                            &mut self.interface_panel_manager,
                            menu_bar_height,
                            viewed_nodes,
                            graph,
                        )
                    }
                };
                
            // Handle panel actions
            match panel_action {
                PanelAction::Close => nodes_to_close.push(node_id),
                PanelAction::CloseAll => nodes_to_close.push(node_id),
                PanelAction::ToggleStack => nodes_to_toggle_stack.push(node_id),
                PanelAction::TogglePin => nodes_to_toggle_pin.push(node_id),
                PanelAction::None | PanelAction::Minimize | PanelAction::Restore => {
                    // egui handles minimize/restore automatically with collapsible(true)
                }
            }
        }
    }
        
        // Apply panel actions (after iteration to avoid borrowing conflicts)
        for node_id in nodes_to_close {
            self.close_node_panel(node_id, current_view, graph);
        }
        
        // Apply stack toggle actions
        for node_id in nodes_to_toggle_stack {
            self.interface_panel_manager.toggle_panel_stacked(node_id);
        }
        
        // Apply pin toggle actions  
        for node_id in nodes_to_toggle_pin {
            self.interface_panel_manager.toggle_panel_pinned(node_id);
        }
    }

    /// Close a node's interface panel and disable its visibility flag
    fn close_node_panel(
        &mut self, 
        node_id: NodeId, 
        current_view: &GraphView,
        graph: &mut NodeGraph,
    ) {
        // Find the node in the correct graph based on current view and set its visibility to false
        match current_view {
            GraphView::Root => {
                if let Some(node) = graph.nodes.get_mut(&node_id) {
                    node.visible = false;
                }
            }
            GraphView::WorkspaceNode(workspace_node_id) => {
                if let Some(workspace_node) = graph.nodes.get_mut(workspace_node_id) {
                    if let Some(internal_graph) = workspace_node.get_internal_graph_mut() {
                        if let Some(node) = internal_graph.nodes.get_mut(&node_id) {
                            node.visible = false;
                        }
                    }
                }
            }
        }
        
        // Clear all panel state
        self.interface_panel_manager.set_panel_visibility(node_id, false);
        self.interface_panel_manager.set_panel_minimized(node_id, false);
        self.interface_panel_manager.set_panel_open(node_id, true); // Reset for next time
    }

    /// Auto-load USD stage into a viewport node
    pub fn auto_load_usd_into_viewport(&mut self, viewport_node_id: NodeId, stage_id: &str) {
        self.viewport_panel.auto_load_usd_into_viewport(viewport_node_id, stage_id);
    }

    /// Get the file path from a LoadStage node's interface panel
    pub fn get_loadstage_file_path(&self, node_id: NodeId) -> Option<String> {
        // For USD LoadStage nodes, the file path is managed by the NodeInterfacePanel trait
        // The actual file path is stored in the node's interface panel implementation
        // Since we can't access the panel instance directly, we return None here
        // and let the connection system fall back to reading from node parameters
        None
    }

    // DEPRECATED: Auto-detection removed in favor of node self-assignment
    // Nodes now carry their own panel_type field and assign themselves
    // to the appropriate panel type when created
}

impl Default for PanelManager {
    fn default() -> Self {
        Self::new()
    }
}