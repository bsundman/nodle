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
        
        // CRITICAL FIX: First pass - auto-detect panel types and visibility for ALL visible nodes
        // This ensures all panel visibility is set before any rendering logic that depends on it
        for (&node_id, node) in viewed_nodes {
            if node.visible {
                self.auto_detect_panel_type(node_id, node);
            }
        }
        
        // Second pass - render panels now that all auto-detection is complete
        for (&node_id, node) in viewed_nodes {
            if node.visible {
                
                let panel_type = self.interface_panel_manager.get_panel_type(node_id);
                
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

    /// Auto-detect and set panel type based on node type
    fn auto_detect_panel_type(&mut self, node_id: NodeId, node: &Node) {
        // CRITICAL FIX: Always ensure panel visibility matches node visibility
        // This must happen on every call, not just the first time
        if node.visible {
            self.interface_panel_manager.set_panel_visibility(node_id, true);
        } else {
            self.interface_panel_manager.set_panel_visibility(node_id, false);
        }
        
        // Check if panel type has already been set
        if self.interface_panel_manager.has_panel_type_set(node_id) {
            return; // Already set, don't override panel type and stacking preferences
        }
        
        // Get panel type from node metadata (pure node-centric approach)
        // Use a shared registry instance to avoid creating it repeatedly
        let registry = crate::nodes::factory::NodeRegistry::default();
        let panel_type = if let Some(metadata) = registry.get_node_metadata(&node.title) {
            // Use the panel type defined in the node's metadata
            metadata.panel_type
        } else {
            // Ultimate fallback - should rarely be used in a pure "everything nodes" system
            PanelType::Parameter
        };
        
        // Set the detected panel type
        self.interface_panel_manager.set_panel_type(node_id, panel_type);
        
        // Set panel type-specific defaults if not already set
        if !self.interface_panel_manager.has_stacking_preference_set(node_id) {
            match panel_type {
                PanelType::Viewport => {
                    // Viewport panels should be unstacked by default (floating independently)
                    self.interface_panel_manager.set_panel_stacked(node_id, false);
                },
                PanelType::Parameter => {
                    // Parameter panels should be stacked by default
                    self.interface_panel_manager.set_panel_stacked(node_id, true);
                },
                PanelType::Viewer => {
                    // Viewer panels can be stacked
                    self.interface_panel_manager.set_panel_stacked(node_id, true);
                },
                PanelType::Editor => {
                    // Editor panels should float by default
                    self.interface_panel_manager.set_panel_stacked(node_id, false);
                },
                PanelType::Inspector => {
                    // Inspector panels can be stacked
                    self.interface_panel_manager.set_panel_stacked(node_id, true);
                },
            }
        }
    }
}

impl Default for PanelManager {
    fn default() -> Self {
        Self::new()
    }
}