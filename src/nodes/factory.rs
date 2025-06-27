//! Enhanced node factory system with self-registration and rich metadata

use egui::{Color32, Pos2};
use crate::nodes::{Node, NodeId, NodeGraph};
use crate::NodeFactory as OldNodeFactory; // Import the old trait
use std::collections::HashMap;

/// Data types that can flow through ports
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataType {
    /// Floating point number
    Float,
    /// 3D vector (x, y, z)
    Vector3,
    /// RGB color value
    Color,
    /// Text string
    String,
    /// Boolean value
    Boolean,
    /// Any type (for generic ports)
    Any,
}

impl DataType {
    /// Check if this data type can connect to another
    pub fn can_connect_to(&self, other: &DataType) -> bool {
        self == other || *self == DataType::Any || *other == DataType::Any
    }
    
    /// Get a human-readable name for this data type
    pub fn name(&self) -> &'static str {
        match self {
            DataType::Float => "Float",
            DataType::Vector3 => "Vector3", 
            DataType::Color => "Color",
            DataType::String => "String",
            DataType::Boolean => "Boolean",
            DataType::Any => "Any",
        }
    }
    
    /// Get a color representing this data type
    pub fn color(&self) -> Color32 {
        match self {
            DataType::Float => Color32::from_rgb(100, 150, 255), // Blue
            DataType::Vector3 => Color32::from_rgb(255, 100, 100), // Red
            DataType::Color => Color32::from_rgb(255, 200, 100), // Orange
            DataType::String => Color32::from_rgb(100, 255, 100), // Green
            DataType::Boolean => Color32::from_rgb(255, 100, 255), // Magenta
            DataType::Any => Color32::from_rgb(150, 150, 150), // Gray
        }
    }
}

/// Hierarchical category system for organizing nodes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeCategory {
    path: Vec<String>,
}

impl NodeCategory {
    /// Create a new category from path components
    pub fn new(path: &[&str]) -> Self {
        Self {
            path: path.iter().map(|s| s.to_string()).collect(),
        }
    }
    
    /// Get the full path as a slice
    pub fn path(&self) -> &[String] {
        &self.path
    }
    
    /// Get the category name (last component)
    pub fn name(&self) -> &str {
        self.path.last().map(|s| s.as_str()).unwrap_or("")
    }
    
    /// Get the parent category
    pub fn parent(&self) -> Option<NodeCategory> {
        if self.path.len() > 1 {
            Some(NodeCategory {
                path: self.path[..self.path.len() - 1].to_vec(),
            })
        } else {
            None
        }
    }
    
    /// Check if this category is a child of another
    pub fn is_child_of(&self, other: &NodeCategory) -> bool {
        self.path.len() > other.path.len() && 
        self.path[..other.path.len()] == other.path
    }
    
    /// Get display string for UI
    pub fn display_string(&self) -> String {
        self.path.join(" > ")
    }
}

// Standard categories
impl NodeCategory {
    pub const MATH: NodeCategory = NodeCategory { path: vec![] }; // Will be filled properly
    pub const LOGIC: NodeCategory = NodeCategory { path: vec![] };
    pub const DATA: NodeCategory = NodeCategory { path: vec![] };
    pub const OUTPUT: NodeCategory = NodeCategory { path: vec![] };
    
    /// Get standard math category
    pub fn math() -> Self { Self::new(&["Math"]) }
    /// Get standard logic category  
    pub fn logic() -> Self { Self::new(&["Logic"]) }
    /// Get standard data category
    pub fn data() -> Self { Self::new(&["Data"]) }
    /// Get standard output category
    pub fn output() -> Self { Self::new(&["Output"]) }
    /// Get MaterialX shading category
    pub fn materialx_shading() -> Self { Self::new(&["MaterialX", "Shading"]) }
    /// Get MaterialX texture category
    pub fn materialx_texture() -> Self { Self::new(&["MaterialX", "Texture"]) }
}

/// Port definition for node creation
#[derive(Debug, Clone)]
pub struct PortDefinition {
    pub name: String,
    pub data_type: DataType,
    pub optional: bool,
    pub description: Option<String>,
}

impl PortDefinition {
    /// Create a required port
    pub fn required(name: &str, data_type: DataType) -> Self {
        Self {
            name: name.to_string(),
            data_type,
            optional: false,
            description: None,
        }
    }
    
    /// Create an optional port
    pub fn optional(name: &str, data_type: DataType) -> Self {
        Self {
            name: name.to_string(),
            data_type,
            optional: true,
            description: None,
        }
    }
    
    /// Add description to port
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
}

/// Rich metadata for nodes
#[derive(Debug, Clone)]
pub struct NodeMetadata {
    pub node_type: &'static str,
    pub display_name: &'static str,
    pub category: NodeCategory,
    pub description: &'static str,
    pub color: Color32,
    pub inputs: Vec<PortDefinition>,
    pub outputs: Vec<PortDefinition>,
}

/// Enhanced node factory trait with rich metadata
pub trait NodeFactory: Send + Sync {
    /// Get comprehensive node metadata
    fn metadata() -> NodeMetadata where Self: Sized;
    
    /// Create a node instance at the given position
    fn create(position: Pos2) -> Node where Self: Sized {
        let meta = Self::metadata();
        let mut node = Node::new(0, meta.node_type, position);
        node.color = meta.color;
        
        // Add inputs
        for input in &meta.inputs {
            node.add_input(&input.name);
        }
        
        // Add outputs  
        for output in &meta.outputs {
            node.add_output(&output.name);
        }
        
        // CRITICAL: Update port positions after adding ports
        node.update_port_positions();
        
        node
    }
    
    /// Add this node to a graph
    fn add_to_graph(graph: &mut NodeGraph, position: Pos2) -> NodeId where Self: Sized {
        graph.add_node(Self::create(position))
    }
}

/// Function pointer type for creating nodes
type NodeCreator = fn(Pos2) -> Node;
type MetadataProvider = fn() -> NodeMetadata;

/// Registry for managing node factories
pub struct NodeRegistry {
    creators: HashMap<String, NodeCreator>,
    metadata_providers: HashMap<String, MetadataProvider>,
    categories: HashMap<NodeCategory, Vec<String>>,
}

impl NodeRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            creators: HashMap::new(),
            metadata_providers: HashMap::new(),
            categories: HashMap::new(),
        }
    }
    
    /// Register a node factory
    pub fn register<T: NodeFactory + 'static>(&mut self) {
        let metadata = T::metadata();
        let node_type = metadata.node_type.to_string();
        
        // Register the creators
        self.creators.insert(node_type.clone(), T::create);
        self.metadata_providers.insert(node_type.clone(), T::metadata);
        
        // Register in category
        self.categories
            .entry(metadata.category.clone())
            .or_insert_with(Vec::new)
            .push(node_type);
    }
    
    /// Create a node by type name
    pub fn create_node(&self, node_type: &str, position: Pos2) -> Option<Node> {
        // Try dynamic factory first
        if let Some(creator) = self.creators.get(node_type) {
            return Some(creator(position));
        }
        
        // Fall back to legacy system for nodes not yet migrated
        self.create_node_legacy(node_type, position)
    }
    
    /// Legacy node creation (temporary during migration)
    fn create_node_legacy(&self, node_type: &str, position: Pos2) -> Option<Node> {
        match node_type {
            "Add" => Some(<crate::nodes::math::AddNode as OldNodeFactory>::create(position)),
            "Subtract" => Some(<crate::nodes::math::SubtractNode as OldNodeFactory>::create(position)),
            "Multiply" => Some(<crate::nodes::math::MultiplyNode as OldNodeFactory>::create(position)),
            "Divide" => Some(<crate::nodes::math::DivideNode as OldNodeFactory>::create(position)),
            "AND" => Some(<crate::nodes::logic::AndNode as OldNodeFactory>::create(position)),
            "OR" => Some(<crate::nodes::logic::OrNode as OldNodeFactory>::create(position)),
            "NOT" => Some(<crate::nodes::logic::NotNode as OldNodeFactory>::create(position)),
            "Constant" => Some(<crate::nodes::data::ConstantNode as OldNodeFactory>::create(position)),
            "Variable" => Some(<crate::nodes::data::VariableNode as OldNodeFactory>::create(position)),
            "Print" => Some(<crate::nodes::output::PrintNode as OldNodeFactory>::create(position)),
            "Debug" => Some(<crate::nodes::output::DebugNode as OldNodeFactory>::create(position)),
            _ => None,
        }
    }
    
    /// Get all available node types
    pub fn node_types(&self) -> Vec<&str> {
        // Include both dynamic and legacy node types
        let mut types: Vec<&str> = self.creators.keys().map(|s| s.as_str()).collect();
        
        // Add legacy types that aren't in dynamic registry
        let legacy_types = ["Add", "Subtract", "Multiply", "Divide", "AND", "OR", "NOT", "Constant", "Variable", "Print", "Debug"];
        for legacy_type in &legacy_types {
            if !self.creators.contains_key(*legacy_type) {
                types.push(legacy_type);
            }
        }
        
        types.sort();
        types
    }
    
    /// Get nodes in a specific category
    pub fn nodes_in_category(&self, category: &NodeCategory) -> Vec<&str> {
        self.categories
            .get(category)
            .map(|nodes| nodes.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }
    
    /// Get all categories
    pub fn categories(&self) -> Vec<&NodeCategory> {
        self.categories.keys().collect()
    }
    
    /// Get metadata for a node type
    pub fn get_metadata(&self, node_type: &str) -> Option<NodeMetadata> {
        self.metadata_providers.get(node_type).map(|provider| provider())
    }
    
    /// Generate menu structure from registered node categories
    pub fn generate_menu_structure(&self, workspace_filter: &[&str]) -> Vec<crate::workspace::WorkspaceMenuItem> {
        use crate::workspace::WorkspaceMenuItem;
        use std::collections::HashMap;
        
        // Group nodes by their category paths
        let mut category_groups: HashMap<Vec<String>, Vec<(String, String)>> = HashMap::new();
        
        for (node_type, provider) in &self.metadata_providers {
            let metadata = provider();
            
            // Only include nodes that match the workspace filter
            if workspace_filter.is_empty() || metadata.category.path().iter().any(|segment| workspace_filter.contains(&segment.as_str())) {
                let category_path = metadata.category.path();
                
                // Skip the first segment if it matches the workspace (e.g., "3D")
                let menu_path = if !workspace_filter.is_empty() && 
                                 !category_path.is_empty() && 
                                 workspace_filter.contains(&category_path[0].as_str()) {
                    if category_path.len() > 1 {
                        category_path[1..].to_vec()
                    } else {
                        vec!["General".to_string()]
                    }
                } else {
                    category_path.to_vec()
                };
                
                category_groups
                    .entry(menu_path)
                    .or_insert_with(Vec::new)
                    .push((metadata.display_name.to_string(), node_type.clone()));
            }
        }
        
        // Convert category groups to menu items
        let mut menu_items = Vec::new();
        for (category_path, nodes) in category_groups {
            if !nodes.is_empty() {
                let category_name = if category_path.is_empty() {
                    "General".to_string()
                } else {
                    category_path.last().unwrap_or(&"General".to_string()).clone()
                };
                
                let mut node_items = Vec::new();
                for (display_name, node_type) in nodes {
                    node_items.push(WorkspaceMenuItem::Node {
                        name: display_name,
                        node_type,
                    });
                }
                
                // Sort nodes alphabetically within each category
                node_items.sort_by(|a, b| {
                    if let (WorkspaceMenuItem::Node { name: name_a, .. }, WorkspaceMenuItem::Node { name: name_b, .. }) = (a, b) {
                        name_a.cmp(name_b)
                    } else {
                        std::cmp::Ordering::Equal
                    }
                });
                
                menu_items.push(WorkspaceMenuItem::Category {
                    name: category_name,
                    items: node_items,
                });
            }
        }
        
        // Sort categories alphabetically
        menu_items.sort_by(|a, b| {
            if let (WorkspaceMenuItem::Category { name: name_a, .. }, WorkspaceMenuItem::Category { name: name_b, .. }) = (a, b) {
                name_a.cmp(name_b)
            } else {
                std::cmp::Ordering::Equal
            }
        });
        
        menu_items
    }
}

impl Default for NodeRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        
        // Register enhanced math nodes
        registry.register::<crate::nodes::math::AddNodeEnhanced>();
        registry.register::<crate::nodes::math::SubtractNodeEnhanced>();
        registry.register::<crate::nodes::math::MultiplyNodeEnhanced>();
        registry.register::<crate::nodes::math::DivideNodeEnhanced>();
        
        // Register enhanced logic nodes
        registry.register::<crate::nodes::logic::AndNodeEnhanced>();
        registry.register::<crate::nodes::logic::OrNodeEnhanced>();
        registry.register::<crate::nodes::logic::NotNodeEnhanced>();
        
        // Register enhanced data nodes
        registry.register::<crate::nodes::data::ConstantNodeEnhanced>();
        registry.register::<crate::nodes::data::VariableNodeEnhanced>();
        
        // Register enhanced output nodes
        registry.register::<crate::nodes::output::PrintNodeEnhanced>();
        registry.register::<crate::nodes::output::DebugNodeEnhanced>();
        
        // Register 3D nodes
        registry.register::<crate::nodes::three_d::TranslateNode3D>();
        registry.register::<crate::nodes::three_d::RotateNode3D>();
        registry.register::<crate::nodes::three_d::ScaleNode3D>();
        registry.register::<crate::nodes::three_d::CubeNode3D>();
        registry.register::<crate::nodes::three_d::SphereNode3D>();
        registry.register::<crate::nodes::three_d::PlaneNode3D>();
        registry.register::<crate::nodes::three_d::PointLightNode3D>();
        registry.register::<crate::nodes::three_d::DirectionalLightNode3D>();
        registry.register::<crate::nodes::three_d::SpotLightNode3D>();
        registry.register::<crate::nodes::three_d::ViewportNode3D>();
        
        registry
    }
}