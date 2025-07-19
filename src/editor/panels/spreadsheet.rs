//! Spreadsheet panel implementation
//! 
//! Handles spreadsheet-type interface panels for tabular data display

use egui::Context;
use crate::nodes::{Node, NodeId, InterfacePanelManager};
use crate::editor::panels::PanelAction;
use std::collections::HashMap;

/// Spreadsheet panel renderer
pub struct SpreadsheetPanel {
    /// Default spreadsheet panel size
    default_size: [f32; 2],
}

impl SpreadsheetPanel {
    pub fn new() -> Self {
        Self {
            default_size: [400.0, 300.0], // Default size for spreadsheet panels
        }
    }

    /// Render spreadsheet panels
    pub fn render(
        &mut self,
        ctx: &Context,
        node_id: NodeId,
        node: &Node,
        panel_manager: &mut InterfacePanelManager,
        menu_bar_height: f32,
        _viewed_nodes: &std::collections::HashMap<NodeId, Node>,
        graph: &mut crate::nodes::NodeGraph,
        execution_engine: &mut crate::nodes::NodeGraphEngine,
    ) -> PanelAction {
        // Check if panel is marked as visible
        if !panel_manager.is_panel_visible(node_id) {
            return PanelAction::None;
        }

        let panel_id = egui::Id::new(format!("spreadsheet_panel_{}", node_id));
        let mut panel_action = PanelAction::None;
        
        // Get panel open state reference
        let mut is_open = panel_manager.is_panel_open(node_id);
        
        // Create window title
        let title = format!("📊 {} - Spreadsheet", node.title);
        
        // Create window with size constraints like other panels
        let mut window = egui::Window::new(title)
            .id(panel_id)
            .open(&mut is_open)
            .default_size(self.default_size)
            .min_size([300.0, 200.0])
            .resizable(true)
            .collapsible(true)
            .constrain_to(egui::Rect::from_min_size(
                egui::Pos2::new(0.0, menu_bar_height),
                egui::Vec2::new(ctx.screen_rect().width(), ctx.screen_rect().height() - menu_bar_height)
            ));
        
        // Position spreadsheet panel to the right of the node (same as tree panel)
        let node_pos = node.position;
        window = window.default_pos(node_pos + egui::Vec2::new(200.0, 0.0));
        
        let _window_response = window.show(ctx, |ui| {
            // Render spreadsheet content based on node type
            match node.type_id.as_str() {
                "Attributes" => {
                    // Get the actual inputs from connections
                    let mut inputs = HashMap::new();
                    
                    // Find connections to this node's inputs
                    for (port_idx, input_port) in node.inputs.iter().enumerate() {
                        for connection in &graph.connections {
                            if connection.to_node == node_id && connection.to_port == port_idx {
                                // Get the output value from the source node
                                if let Some(source_value) = execution_engine.get_cached_output(connection.from_node, connection.from_port) {
                                    inputs.insert(input_port.name.clone(), source_value.clone());
                                }
                            }
                        }
                    }
                    
                    egui::ScrollArea::vertical()
                        .show(ui, |ui| {
                            crate::nodes::three_d::ui::attributes::parameters::render_attributes_parameters(
                                ui,
                                node_id,
                                &mut graph.nodes.get_mut(&node_id).unwrap().parameters,
                                &inputs
                            );
                        });
                }
                _ => {
                    ui.label("Spreadsheet view not implemented for this node type");
                }
            }
        });
        
        // Update panel open state
        panel_manager.set_panel_open(node_id, is_open);
        
        // Check if window was closed via X button
        if !is_open {
            panel_action = PanelAction::Close;
        }
        
        panel_action
    }
}