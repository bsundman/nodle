//! Interface panel management system for the node editor
//!
//! Handles rendering and managing interface panels for nodes, including
//! parameter editing, panel positioning, and state management.

mod parameter;
mod viewport;

pub use parameter::ParameterPanel;
pub use viewport::ViewportPanel;

use egui::Ui;
use crate::nodes::{
    NodeGraph, Node, NodeId, InterfacePanelManager, PanelType,
};
use std::collections::HashMap;
use log::debug;

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
        execution_engine: &mut crate::nodes::NodeGraphEngine,
    ) {
        // Store menu bar height
        self.set_menu_bar_height(menu_bar_height);
        let ctx = ui.ctx();
        
        // Track which nodes to close and which actions to apply (to avoid borrowing issues)
        let mut nodes_to_close: Vec<NodeId> = Vec::new();
        let mut nodes_to_toggle_stack: Vec<NodeId> = Vec::new();
        let mut nodes_to_toggle_pin: Vec<NodeId> = Vec::new();
        
        // Debug: Check if we have any viewport nodes
        let viewport_count = viewed_nodes.iter()
            .filter(|(_, node)| node.get_panel_type() == Some(PanelType::Viewport))
            .count();
        if viewport_count > 0 {
            debug!("PanelManager: Found {} viewport nodes in viewed_nodes", viewport_count);
            for (&node_id, node) in viewed_nodes {
                if node.get_panel_type() == Some(PanelType::Viewport) {
                    debug!("  - Viewport node {} '{}' visible={}", node_id, node.title, node.visible);
                }
            }
        }
        
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
                
                // Debug logging for viewport nodes
                if panel_type == PanelType::Viewport {
                    debug!("PanelManager: Processing viewport node {} with visible={}", node_id, node.visible);
                    debug!("PanelManager: Node title: {}", node.title);
                    debug!("PanelManager: Panel visibility: {}", self.interface_panel_manager.is_panel_visible(node_id));
                    debug!("PanelManager: Panel open: {}", self.interface_panel_manager.is_panel_open(node_id));
                }
                
                // Delegate to the appropriate panel type renderer
                let panel_action = match panel_type {
                    PanelType::Viewport => {
                        debug!("PanelManager: Rendering viewport panel for node {}", node_id);
                        let result = self.viewport_panel.render(
                            ctx,
                            node_id,
                            node,
                            &mut self.interface_panel_manager,
                            menu_bar_height,
                            viewed_nodes,
                            graph,
                            execution_engine,
                        );
                        debug!("PanelManager: Viewport panel render completed for node {}, result: {:?}", node_id, result);
                        result
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
                            execution_engine,
                        )
                    }
                };
                
                debug!("PanelManager: About to handle panel action: {:?}", panel_action);
                
            // Handle panel actions
            match panel_action {
                PanelAction::Close => {
                    debug!("PanelManager: Closing node {}", node_id);
                    nodes_to_close.push(node_id);
                },
                PanelAction::CloseAll => {
                    debug!("PanelManager: CloseAll for node {}", node_id);
                    nodes_to_close.push(node_id);
                },
                PanelAction::ToggleStack => {
                    debug!("PanelManager: ToggleStack for node {}", node_id);
                    nodes_to_toggle_stack.push(node_id);
                },
                PanelAction::TogglePin => {
                    debug!("PanelManager: TogglePin for node {}", node_id);
                    nodes_to_toggle_pin.push(node_id);
                },
                PanelAction::None | PanelAction::Minimize | PanelAction::Restore => {
                    debug!("PanelManager: No action needed for node {}", node_id);
                    // egui handles minimize/restore automatically with collapsible(true)
                }
            }
            debug!("PanelManager: Completed processing node {}", node_id);
            }
        }
        
        
        // Apply panel actions (after iteration to avoid borrowing conflicts)
        for node_id in nodes_to_close {
            debug!("PanelManager: Applying close action for node {}", node_id);
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
        debug!("close_node_panel: Closing panel for node {}", node_id);
        
        // The key insight: the 'graph' parameter is already the correct graph to work with
        // - For Root view: it's the main graph
        // - For WorkspaceNode view: it's the workspace's internal graph
        // So we should just directly update the node in the provided graph
        
        if let Some(node) = graph.nodes.get_mut(&node_id) {
            debug!("close_node_panel: Setting node {} '{}' visibility to false", node_id, node.title);
            node.visible = false;
        } else {
            debug!("close_node_panel: Node {} not found in graph (available nodes: {:?})", 
                node_id, graph.nodes.keys().collect::<Vec<_>>());
            
            // If we're in a workspace view and the node wasn't found, it might be a logic error
            // Log additional debug info
            match current_view {
                GraphView::Root => {
                    debug!("close_node_panel: In root view, node should have been in main graph");
                }
                GraphView::WorkspaceNode(workspace_node_id) => {
                    debug!("close_node_panel: In workspace {} view, node should have been in internal graph", workspace_node_id);
                }
            }
        }
        
        // Clear panel state properly
        debug!("close_node_panel: Clearing panel state for node {}", node_id);
        self.interface_panel_manager.set_panel_visibility(node_id, false);
        self.interface_panel_manager.set_panel_open(node_id, false); // Set to false, not true!
        self.interface_panel_manager.set_panel_minimized(node_id, false);
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