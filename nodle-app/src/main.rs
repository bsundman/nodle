//! Nōdle Application - Node-based visual programming editor

use eframe::egui;
use egui::{Color32, Pos2};
use nodle_core::{graph::NodeGraph, node::Node};

mod editor;
mod math;
mod logic;
mod data;
mod output;

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
    fn add_to_graph(graph: &mut NodeGraph, position: Pos2) -> nodle_core::NodeId where Self: Sized {
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
    /// Create a node by type name
    pub fn create_node(node_type: &str, position: Pos2) -> Option<Node> {
        match node_type {
            "Add" => Some(math::AddNode::create(position)),
            "Subtract" => Some(math::SubtractNode::create(position)),
            "Multiply" => Some(math::MultiplyNode::create(position)),
            "Divide" => Some(math::DivideNode::create(position)),
            "AND" => Some(logic::AndNode::create(position)),
            "OR" => Some(logic::OrNode::create(position)),
            "NOT" => Some(logic::NotNode::create(position)),
            "Constant" => Some(data::ConstantNode::create(position)),
            "Variable" => Some(data::VariableNode::create(position)),
            "Print" => Some(output::PrintNode::create(position)),
            "Debug" => Some(output::DebugNode::create(position)),
            _ => None,
        }
    }
}

/// Creates test nodes for demonstration using the modular system
pub fn create_test_nodes(graph: &mut NodeGraph) {
    math::AddNode::add_to_graph(graph, Pos2::new(100.0, 100.0));
    math::SubtractNode::add_to_graph(graph, Pos2::new(100.0, 200.0));
    math::MultiplyNode::add_to_graph(graph, Pos2::new(300.0, 100.0));
    math::DivideNode::add_to_graph(graph, Pos2::new(300.0, 200.0));
    
    logic::AndNode::add_to_graph(graph, Pos2::new(500.0, 100.0));
    logic::OrNode::add_to_graph(graph, Pos2::new(500.0, 200.0));
    logic::NotNode::add_to_graph(graph, Pos2::new(700.0, 150.0));
    
    data::ConstantNode::add_to_graph(graph, Pos2::new(100.0, 350.0));
    data::VariableNode::add_to_graph(graph, Pos2::new(300.0, 350.0));
    
    output::PrintNode::add_to_graph(graph, Pos2::new(500.0, 350.0));
    output::DebugNode::add_to_graph(graph, Pos2::new(700.0, 350.0));
}

/// Application entry point
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_app_id("com.nodle.editor")
            .with_decorations(true)
            .with_title_shown(false),
        ..Default::default()
    };

    eframe::run_native(
        "Nōdle",
        options,
        Box::new(|cc| {
            // Set dark theme
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            cc.egui_ctx.set_theme(egui::Theme::Dark);
            Ok(Box::new(NodeEditor::new()))
        }),
    )
}