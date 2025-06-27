//! Nōdle Application - Node-based visual programming editor

use eframe::egui;
use egui::{Color32, Pos2};
use crate::nodes::{NodeGraph, Node};

mod editor;
mod menu_hierarchy;
mod nodes;
mod workspaces;
mod workspace;
mod gpu;

use editor::NodeEditor;

/// Trait for creating standardized nodes
pub trait NodeFactory {
    /// Get the node type name
    fn node_type() -> &'static str where Self: Sized;
    
    /// Get the display name for the node
    fn display_name() -> &'static str where Self: Sized;
    
    /// Get the category for context menu organization
    fn category() -> NodeCategory where Self: Sized;
    
    /// Get the node color
    fn color() -> Color32 where Self: Sized;
    
    /// Create a new instance of this node at the given position
    fn create(position: Pos2) -> Node where Self: Sized;
    
    /// Add this node to the graph at the given position
    fn add_to_graph(graph: &mut NodeGraph, position: Pos2) -> crate::nodes::NodeId where Self: Sized {
        graph.add_node(Self::create(position))
    }
}

/// Categories for organizing nodes in the context menu
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeCategory {
    Math,
    Logic,
    Data,
    Output,
}

impl NodeCategory {
    pub fn name(&self) -> &'static str {
        match self {
            NodeCategory::Math => "Math",
            NodeCategory::Logic => "Logic", 
            NodeCategory::Data => "Data",
            NodeCategory::Output => "Output",
        }
    }
}

/// Registry of all available node types
pub struct NodeRegistry;

impl NodeRegistry {
    /// Create a node by type name (includes both generic and context-specific nodes)
    pub fn create_node(node_type: &str, position: Pos2) -> Option<Node> {
        // Try generic nodes first
        match node_type {
            "Add" => Some(nodes::math::AddNode::create(position)),
            "Subtract" => Some(nodes::math::SubtractNode::create(position)),
            "Multiply" => Some(nodes::math::MultiplyNode::create(position)),
            "Divide" => Some(nodes::math::DivideNode::create(position)),
            "AND" => Some(nodes::logic::AndNode::create(position)),
            "OR" => Some(nodes::logic::OrNode::create(position)),
            "NOT" => Some(nodes::logic::NotNode::create(position)),
            "Constant" => Some(nodes::data::ConstantNode::create(position)),
            "Variable" => Some(nodes::data::VariableNode::create(position)),
            "Print" => Some(nodes::output::PrintNode::create(position)),
            "Debug" => Some(nodes::output::DebugNode::create(position)),
            _ => None,
        }
    }
    
    /// Create a workspace-specific node
    pub fn create_workspace_node(workspace: &dyn workspace::Workspace, node_type: &str, position: Pos2) -> Option<Node> {
        workspace.create_workspace_node(node_type, position)
    }
}

/// Creates test nodes for demonstration using the modular system
pub fn create_test_nodes(graph: &mut NodeGraph) {
    // Create generic nodes for testing
    if let Some(node) = NodeRegistry::create_node("Add", Pos2::new(100.0, 100.0)) {
        graph.add_node(node);
    }
    if let Some(node) = NodeRegistry::create_node("Subtract", Pos2::new(100.0, 200.0)) {
        graph.add_node(node);
    }
    if let Some(node) = NodeRegistry::create_node("Multiply", Pos2::new(300.0, 100.0)) {
        graph.add_node(node);
    }
    if let Some(node) = NodeRegistry::create_node("Divide", Pos2::new(300.0, 200.0)) {
        graph.add_node(node);
    }
    
    if let Some(node) = NodeRegistry::create_node("AND", Pos2::new(500.0, 100.0)) {
        graph.add_node(node);
    }
    if let Some(node) = NodeRegistry::create_node("OR", Pos2::new(500.0, 200.0)) {
        graph.add_node(node);
    }
    if let Some(node) = NodeRegistry::create_node("NOT", Pos2::new(700.0, 150.0)) {
        graph.add_node(node);
    }
    
    if let Some(node) = NodeRegistry::create_node("Constant", Pos2::new(100.0, 350.0)) {
        graph.add_node(node);
    }
    if let Some(node) = NodeRegistry::create_node("Variable", Pos2::new(300.0, 350.0)) {
        graph.add_node(node);
    }
    
    if let Some(node) = NodeRegistry::create_node("Print", Pos2::new(500.0, 350.0)) {
        graph.add_node(node);
    }
    if let Some(node) = NodeRegistry::create_node("Debug", Pos2::new(700.0, 350.0)) {
        graph.add_node(node);
    }
}

/// Application entry point
fn main() -> Result<(), eframe::Error> {
    // Test USD functionality at startup
    // #[cfg(debug_assertions)]
    // {
    //     println!("Testing USD integration...");
    //     test_usd_nodes::test_usd_node_creation();
    //     test_usd_nodes::test_usd_execution();
    // }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_app_id("com.nodle.editor")
            .with_decorations(true)
            .with_title_shown(true),
        multisampling: 4, // Enable 4x multisampling antialiasing
        renderer: eframe::Renderer::Wgpu, // Use wgpu renderer for GPU acceleration
        wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
            supported_backends: wgpu::Backends::all(),
            device_descriptor: std::sync::Arc::new(|_adapter| wgpu::DeviceDescriptor {
                label: Some("Nōdle Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
            }),
            ..Default::default()
        },
        ..Default::default()
    };

    eframe::run_native(
        "Nōdle - Node Editor",
        options,
        Box::new(|cc| {
            // Set dark theme
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            cc.egui_ctx.set_theme(egui::Theme::Dark);
            
            Ok(Box::new(NodeEditor::new()))
        }),
    )
}