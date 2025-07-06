//! Enhanced node factory system with self-registration and rich metadata

use egui::{Color32, Pos2, Vec2};
use crate::nodes::{Node, NodeId, NodeGraph};
use crate::nodes::interface::PanelType;
use std::collections::{HashMap, BTreeMap};
use log::{debug, info, warn, error};

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

/// Panel positioning preferences
#[derive(Debug, Clone, PartialEq)]
pub enum PanelPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
    Custom(Vec2), // Custom offset from top-left
}

/// Stacking behavior for panels
#[derive(Debug, Clone, PartialEq)]
pub enum StackingMode {
    Floating,      // Individual windows
    VerticalStack, // Stacked vertically (parameter style)
    TabbedStack,   // Stacked with tabs (viewport style)
    Docked,        // Docked to window edges
}

/// Node execution behavior
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionMode {
    Realtime,     // Executes continuously
    OnDemand,     // Executes when inputs change
    Manual,       // Executes only when triggered
    Background,   // Executes in background thread
}

/// Processing cost hint for scheduling
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingCost {
    Minimal,      // < 1ms
    Low,          // 1-10ms
    Medium,       // 10-100ms
    High,         // 100ms-1s
    VeryHigh,     // > 1s
}

/// Rich metadata for nodes - the single source of truth for all node behavior
#[derive(Debug, Clone)]
pub struct NodeMetadata {
    // Core identity
    pub node_type: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub version: &'static str,
    
    // Visual appearance
    pub color: Color32,
    pub icon: &'static str,
    pub size_hint: Vec2,
    
    // Organization & categorization
    pub category: NodeCategory,
    pub workspace_compatibility: Vec<&'static str>,
    pub tags: Vec<&'static str>,
    
    // Interface behavior
    pub panel_type: crate::nodes::interface::PanelType,
    pub default_panel_position: PanelPosition,
    pub default_stacking_mode: StackingMode,
    pub resizable: bool,
    
    // Connectivity
    pub inputs: Vec<PortDefinition>,
    pub outputs: Vec<PortDefinition>,
    pub allow_multiple_connections: bool,
    
    // Execution behavior
    pub execution_mode: ExecutionMode,
    pub processing_cost: ProcessingCost,
    pub requires_gpu: bool,
    
    // Advanced properties
    pub is_workspace_node: bool,
    pub supports_preview: bool,
}

impl NodeMetadata {
    /// Create node metadata with sensible defaults
    pub fn new(
        node_type: &'static str,
        display_name: &'static str,
        category: NodeCategory,
        description: &'static str,
    ) -> Self {
        Self {
            // Core identity
            node_type,
            display_name,
            description,
            version: "1.0",
            
            // Visual appearance - defaults
            color: Color32::from_rgb(100, 100, 100),
            icon: "⚡",
            size_hint: Vec2::new(120.0, 80.0),
            
            // Organization & categorization
            category,
            workspace_compatibility: vec![], // Compatible with all workspaces
            tags: vec![],
            
            // Interface behavior - parameter panel by default
            panel_type: crate::nodes::interface::PanelType::Parameter,
            default_panel_position: PanelPosition::TopRight,
            default_stacking_mode: StackingMode::VerticalStack,
            resizable: true,
            
            // Connectivity - defaults
            inputs: vec![],
            outputs: vec![],
            allow_multiple_connections: true,
            
            // Execution behavior - sensible defaults
            execution_mode: ExecutionMode::OnDemand,
            processing_cost: ProcessingCost::Low,
            requires_gpu: false,
            
            // Advanced properties - defaults
            is_workspace_node: false,
            supports_preview: false,
        }
    }
    
    /// Create viewport node metadata with viewport-specific defaults
    pub fn viewport(
        node_type: &'static str,
        display_name: &'static str,
        category: NodeCategory,
        description: &'static str,
    ) -> Self {
        Self::new(node_type, display_name, category, description)
            .with_color(Color32::from_rgb(50, 150, 255))
            .with_icon("🖼️")
            .with_panel_type(crate::nodes::interface::PanelType::Viewport)
            .with_default_position(PanelPosition::TopLeft)
            .with_stacking_mode(StackingMode::TabbedStack)
            .with_execution_mode(ExecutionMode::Realtime)
            .with_gpu_requirement(true)
            .with_preview_support(true)
    }
    
    /// Create workspace node metadata with workspace-specific defaults
    pub fn workspace(
        node_type: &'static str,
        display_name: &'static str,
        category: NodeCategory,
        description: &'static str,
    ) -> Self {
        Self::new(node_type, display_name, category, description)
            .with_color(Color32::from_rgb(150, 100, 255))
            .with_icon("📂")
            .with_size_hint(Vec2::new(160.0, 100.0))
            .with_workspace_node(true)
            .with_panel_type(crate::nodes::interface::PanelType::Editor)
            .with_default_position(PanelPosition::Center)
            .with_stacking_mode(StackingMode::Floating)
    }
    
    /// Builder pattern methods for fluent configuration
    pub fn with_color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }
    
    pub fn with_icon(mut self, icon: &'static str) -> Self {
        self.icon = icon;
        self
    }
    
    pub fn with_size_hint(mut self, size: Vec2) -> Self {
        self.size_hint = size;
        self
    }
    
    pub fn with_panel_type(mut self, panel_type: crate::nodes::interface::PanelType) -> Self {
        self.panel_type = panel_type;
        self
    }
    
    pub fn with_default_position(mut self, position: PanelPosition) -> Self {
        self.default_panel_position = position;
        self
    }
    
    pub fn with_stacking_mode(mut self, mode: StackingMode) -> Self {
        self.default_stacking_mode = mode;
        self
    }
    
    pub fn with_execution_mode(mut self, mode: ExecutionMode) -> Self {
        self.execution_mode = mode;
        self
    }
    
    pub fn with_gpu_requirement(mut self, requires_gpu: bool) -> Self {
        self.requires_gpu = requires_gpu;
        self
    }
    
    pub fn with_preview_support(mut self, supports_preview: bool) -> Self {
        self.supports_preview = supports_preview;
        self
    }
    
    pub fn with_workspace_node(mut self, is_workspace_node: bool) -> Self {
        self.is_workspace_node = is_workspace_node;
        self
    }
    
    pub fn with_inputs(mut self, inputs: Vec<PortDefinition>) -> Self {
        self.inputs = inputs;
        self
    }
    
    pub fn with_outputs(mut self, outputs: Vec<PortDefinition>) -> Self {
        self.outputs = outputs;
        self
    }
    
    pub fn with_workspace_compatibility(mut self, workspaces: Vec<&'static str>) -> Self {
        self.workspace_compatibility = workspaces;
        self
    }
    
    pub fn with_tags(mut self, tags: Vec<&'static str>) -> Self {
        self.tags = tags;
        self
    }
    
    pub fn with_processing_cost(mut self, cost: ProcessingCost) -> Self {
        self.processing_cost = cost;
        self
    }
    
    pub fn with_version(mut self, version: &'static str) -> Self {
        self.version = version;
        self
    }
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
        
        // Set panel type from metadata
        node.set_panel_type(meta.panel_type);
        
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
    creators: BTreeMap<String, NodeCreator>,
    metadata_providers: BTreeMap<String, MetadataProvider>,
    categories: HashMap<NodeCategory, Vec<String>>,
    // Plugin support  
    plugin_factories: BTreeMap<String, Box<dyn nodle_plugin_sdk::NodeFactory>>,
    // Cached plugin metadata to avoid repeated calls
    plugin_metadata_cache: HashMap<String, nodle_plugin_sdk::NodeMetadata>,
}

impl NodeRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            creators: BTreeMap::new(),
            metadata_providers: BTreeMap::new(),
            categories: HashMap::new(),
            plugin_factories: BTreeMap::new(),
            plugin_metadata_cache: HashMap::new(),
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
        // Try core nodes first
        if let Some(creator) = self.creators.get(node_type) {
            let mut node = creator(position);
            
            // Set the panel type from metadata
            if let Some(metadata_provider) = self.metadata_providers.get(node_type) {
                let metadata = metadata_provider();
                node.set_panel_type(metadata.panel_type);
            }
            
            return Some(node);
        }
        
        // Try plugin nodes - create adapter between PluginNode and core Node
        debug!("Looking for plugin factory for node type: {}", node_type);
        debug!("Available plugin factories: {:?}", self.plugin_factories.keys().collect::<Vec<_>>());
        if let Some(factory) = self.plugin_factories.get(node_type) {
            info!("Found plugin factory for: {}", node_type);
            debug!("Creating plugin node: {}", node_type);
            
            // Convert egui::Pos2 to nodle_plugin_sdk::Pos2
            let plugin_pos = nodle_plugin_sdk::Pos2::new(position.x, position.y);
            
            // Safety check: Verify the factory vtable is valid
            debug!("Verifying plugin factory safety...");
            let factory_ptr = factory.as_ref() as *const dyn nodle_plugin_sdk::NodeFactory;
            debug!("Factory pointer: {:p}", factory_ptr);
            
            // Use cached metadata instead of calling factory.metadata() again
            let metadata_test = match self.plugin_metadata_cache.get(node_type) {
                Some(cached_metadata) => {
                    debug!("Using cached metadata: {}", cached_metadata.display_name);
                    cached_metadata.clone()
                }
                None => {
                    error!("Missing cached metadata for plugin node type: {}", node_type);
                    return None;
                }
            };
            
            // Add error handling for plugin node creation
            let plugin_node = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                factory.create_node(plugin_pos)
            })) {
                Ok(node_handle) => {
                    // Safely convert the handle to a boxed trait object
                    let node = unsafe { node_handle.into_node() };
                    debug!("Created plugin node instance with ID: {}", node.id());
                    node
                }
                Err(_) => {
                    error!("Panic occurred while creating plugin node for type: {}", node_type);
                    return None;
                }
            };
            
            // Convert PluginNode to core Node (this is the adapter layer)
            // Plugin nodes use UUID strings, but core nodes need numeric IDs
            // Use a unique temporary ID - the actual ID will be assigned when added to the graph
            // Use very large numbers to ensure they don't conflict with real IDs
            static TEMP_ID_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(crate::constants::node::TEMP_ID_START);
            let node_id = TEMP_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            debug!("Plugin node ID: {}", plugin_node.id());
            debug!("Using temporary core node ID: {}", node_id);
            
            // Add error handling for core node creation
            let mut core_node = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                Node::new(node_id, node_type.to_string(), position)
            })) {
                Ok(node) => {
                    debug!("Created core node successfully");
                    node
                }
                Err(_) => {
                    error!("Panic occurred while creating core node");
                    return None;
                }
            };
            
            // Set the panel type from plugin metadata (use already tested metadata)
            debug!("Plugin metadata panel type: {:?}", metadata_test.panel_type);
            
            let panel_type = match metadata_test.panel_type {
                nodle_plugin_sdk::PanelType::Parameter => crate::nodes::interface::PanelType::Parameter,
                nodle_plugin_sdk::PanelType::Viewport => crate::nodes::interface::PanelType::Viewport,
                nodle_plugin_sdk::PanelType::Combined => crate::nodes::interface::PanelType::Parameter, // Fallback
            };
            debug!("Converted to core panel type: {:?}", panel_type);
            
            // Add error handling for panel type setting
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                core_node.set_panel_type(panel_type);
            })) {
                Ok(_) => {
                    debug!("Set panel type on core node: {:?}", core_node.get_panel_type());
                }
                Err(_) => {
                    error!("Panic occurred while setting panel type");
                    return None;
                }
            }
            
            // Check if plugin node supports viewport
            debug!("Plugin node supports viewport: {}", plugin_node.supports_viewport());
            
            // Add ports from plugin metadata to core node
            debug!("Adding ports from plugin metadata to core node");
            for input_def in &metadata_test.inputs {
                core_node.add_input(&input_def.name);
            }
            for output_def in &metadata_test.outputs {
                core_node.add_output(&output_def.name);
            }
            debug!("Added {} input ports and {} output ports", 
                   metadata_test.inputs.len(), metadata_test.outputs.len());
            
            // Update port positions after adding ports
            core_node.update_port_positions();
            debug!("Updated port positions for plugin node");
            
            // Handle storage based on panel type
            if panel_type == crate::nodes::interface::PanelType::Viewport {
                debug!("Node has viewport panel type - storing in global plugin manager");
                
                // For viewport nodes, store in global plugin manager for viewport rendering
                if let Some(plugin_manager) = crate::workspace::get_global_plugin_manager() {
                    if let Ok(mut manager) = plugin_manager.lock() {
                        debug!("Storing viewport plugin node {} in global manager", node_id);
                        manager.store_plugin_node_instance(node_id, plugin_node);
                    } else {
                        warn!("Failed to lock plugin manager for viewport node storage");
                    }
                } else {
                    warn!("No global plugin manager available for viewport node storage");
                }
                
                debug!("Viewport node stored in plugin manager only");
            } else {
                // For non-viewport nodes, store in core node for parameter rendering
                debug!("Storing plugin node instance in core node for parameter rendering");
                core_node.plugin_node = Some(plugin_node);
            }
            
            debug!("Plugin node creation process completed successfully");
            return Some(core_node);
        } else {
            warn!("No plugin factory found for node type: {}", node_type);
        }
        
        None
    }

    /// Get metadata for a node type without creating the node
    pub fn get_node_metadata(&self, node_type: &str) -> Option<NodeMetadata> {
        // Try core nodes first
        if let Some(metadata_provider) = self.metadata_providers.get(node_type) {
            return Some(metadata_provider());
        }
        
        // Try plugin nodes - convert from SDK metadata to core metadata  
        if let Some(_factory) = self.plugin_factories.get(node_type) {
            // Use cached metadata instead of calling factory.metadata()
            let plugin_meta = match self.plugin_metadata_cache.get(node_type) {
                Some(cached_metadata) => cached_metadata,
                None => {
                    error!("Missing cached metadata for plugin node type: {}", node_type);
                    return None;
                }
            };
            // Convert from plugin SDK metadata to core metadata
            return Some(NodeMetadata {
                node_type: plugin_meta.node_type.clone().leak(),
                display_name: plugin_meta.display_name.clone().leak(),
                description: plugin_meta.description.clone().leak(),
                version: plugin_meta.version.clone().leak(),
                color: egui::Color32::from_rgba_premultiplied(
                    plugin_meta.color.r(), 
                    plugin_meta.color.g(), 
                    plugin_meta.color.b(), 
                    plugin_meta.color.a()
                ),
                icon: plugin_meta.icon.clone().leak(),
                size_hint: egui::Vec2::new(plugin_meta.size_hint.x, plugin_meta.size_hint.y),
                category: NodeCategory::new(&plugin_meta.category.path().iter().map(|s| s.as_str()).collect::<Vec<_>>()),
                workspace_compatibility: plugin_meta.workspace_compatibility.iter().map(|s| s.clone().leak() as &str).collect(),
                tags: plugin_meta.tags.iter().map(|s| s.clone().leak() as &str).collect(),
                panel_type: match plugin_meta.panel_type {
                    nodle_plugin_sdk::PanelType::Parameter => crate::nodes::interface::PanelType::Parameter,
                    nodle_plugin_sdk::PanelType::Viewport => crate::nodes::interface::PanelType::Viewport,
                    nodle_plugin_sdk::PanelType::Combined => crate::nodes::interface::PanelType::Parameter, // Fallback to Parameter
                },
                default_panel_position: match plugin_meta.default_panel_position {
                    nodle_plugin_sdk::PanelPosition::TopLeft => PanelPosition::TopLeft,
                    nodle_plugin_sdk::PanelPosition::TopRight => PanelPosition::TopRight,
                    nodle_plugin_sdk::PanelPosition::BottomLeft => PanelPosition::BottomLeft,
                    nodle_plugin_sdk::PanelPosition::BottomRight => PanelPosition::BottomRight,
                    nodle_plugin_sdk::PanelPosition::Center => PanelPosition::Center,
                    nodle_plugin_sdk::PanelPosition::Custom(v) => PanelPosition::Custom(egui::Vec2::new(v.x, v.y)),
                },
                default_stacking_mode: match plugin_meta.default_stacking_mode {
                    nodle_plugin_sdk::StackingMode::Floating => StackingMode::Floating,
                    nodle_plugin_sdk::StackingMode::VerticalStack => StackingMode::VerticalStack,
                    nodle_plugin_sdk::StackingMode::TabbedStack => StackingMode::TabbedStack,
                    nodle_plugin_sdk::StackingMode::Docked => StackingMode::Docked,
                },
                resizable: plugin_meta.resizable,
                inputs: plugin_meta.inputs.iter().map(|p| PortDefinition {
                    name: p.name.clone(),
                    data_type: match p.data_type {
                        nodle_plugin_sdk::DataType::Float => DataType::Float,
                        nodle_plugin_sdk::DataType::Vector3 => DataType::Vector3,
                        nodle_plugin_sdk::DataType::Color => DataType::Color,
                        nodle_plugin_sdk::DataType::String => DataType::String,
                        nodle_plugin_sdk::DataType::Boolean => DataType::Boolean,
                        nodle_plugin_sdk::DataType::Any => DataType::Any,
                    },
                    optional: p.optional,
                    description: p.description.clone(),
                }).collect(),
                outputs: plugin_meta.outputs.iter().map(|p| PortDefinition {
                    name: p.name.clone(),
                    data_type: match p.data_type {
                        nodle_plugin_sdk::DataType::Float => DataType::Float,
                        nodle_plugin_sdk::DataType::Vector3 => DataType::Vector3,
                        nodle_plugin_sdk::DataType::Color => DataType::Color,
                        nodle_plugin_sdk::DataType::String => DataType::String,
                        nodle_plugin_sdk::DataType::Boolean => DataType::Boolean,
                        nodle_plugin_sdk::DataType::Any => DataType::Any,
                    },
                    optional: p.optional,
                    description: p.description.clone(),
                }).collect(),
                allow_multiple_connections: plugin_meta.allow_multiple_connections,
                execution_mode: match plugin_meta.execution_mode {
                    nodle_plugin_sdk::ExecutionMode::Realtime => ExecutionMode::Realtime,
                    nodle_plugin_sdk::ExecutionMode::OnDemand => ExecutionMode::OnDemand,
                    nodle_plugin_sdk::ExecutionMode::Manual => ExecutionMode::Manual,
                    nodle_plugin_sdk::ExecutionMode::Background => ExecutionMode::Background,
                },
                processing_cost: match plugin_meta.processing_cost {
                    nodle_plugin_sdk::ProcessingCost::Minimal => ProcessingCost::Minimal,
                    nodle_plugin_sdk::ProcessingCost::Low => ProcessingCost::Low,
                    nodle_plugin_sdk::ProcessingCost::Medium => ProcessingCost::Medium,
                    nodle_plugin_sdk::ProcessingCost::High => ProcessingCost::High,
                    nodle_plugin_sdk::ProcessingCost::VeryHigh => ProcessingCost::VeryHigh,
                },
                requires_gpu: plugin_meta.requires_gpu,
                is_workspace_node: plugin_meta.is_workspace_node,
                supports_preview: plugin_meta.supports_preview,
            });
        }
        
        None
    }

    /// Create a node by type name and return both node and metadata
    pub fn create_node_with_metadata(&self, node_type: &str, position: Pos2) -> Option<(Node, NodeMetadata)> {
        // Try core nodes first
        if let Some(metadata_provider) = self.metadata_providers.get(node_type) {
            let metadata = metadata_provider();
            if let Some(node) = self.create_node(node_type, position) {
                return Some((node, metadata));
            }
        }
        
        // Try plugin nodes
        if let Some(plugin_factory) = self.plugin_factories.get(node_type) {
            if let Some(metadata) = self.get_node_metadata(node_type) {
                // Create the plugin node instance
                if let Some(node) = self.create_node(node_type, position) {
                    return Some((node, metadata));
                }
            }
        }
        
        None
    }
    
    /// Convert plugin SDK metadata to core Nodle metadata
    fn convert_plugin_metadata_to_core(&self, plugin_meta: &nodle_plugin_sdk::NodeMetadata) -> NodeMetadata {
        NodeMetadata {
            // Core identity
            node_type: plugin_meta.node_type.clone().leak(),
            display_name: plugin_meta.display_name.clone().leak(),
            description: plugin_meta.description.clone().leak(),
            version: plugin_meta.version.clone().leak(),
            
            // Visual appearance
            color: plugin_meta.color,
            icon: plugin_meta.icon.clone().leak(),
            size_hint: plugin_meta.size_hint,
            
            // Organization & categorization
            category: NodeCategory::new(&plugin_meta.category.path().iter().map(|s| s.as_str()).collect::<Vec<_>>()),
            workspace_compatibility: plugin_meta.workspace_compatibility.iter().map(|s| s.clone().leak() as &str).collect(),
            tags: plugin_meta.tags.iter().map(|s| s.clone().leak() as &str).collect(),
            
            // Interface behavior
            panel_type: match plugin_meta.panel_type {
                nodle_plugin_sdk::PanelType::Parameter => PanelType::Parameter,
                nodle_plugin_sdk::PanelType::Viewport => PanelType::Viewport,
                nodle_plugin_sdk::PanelType::Combined => PanelType::Parameter, // Fallback to Parameter
            },
            default_panel_position: match plugin_meta.default_panel_position {
                nodle_plugin_sdk::PanelPosition::TopLeft => PanelPosition::TopLeft,
                nodle_plugin_sdk::PanelPosition::TopRight => PanelPosition::TopRight,
                nodle_plugin_sdk::PanelPosition::BottomLeft => PanelPosition::BottomLeft,
                nodle_plugin_sdk::PanelPosition::BottomRight => PanelPosition::BottomRight,
                nodle_plugin_sdk::PanelPosition::Center => PanelPosition::Center,
                nodle_plugin_sdk::PanelPosition::Custom(pos) => PanelPosition::Custom(pos),
            },
            default_stacking_mode: match plugin_meta.default_stacking_mode {
                nodle_plugin_sdk::StackingMode::Floating => StackingMode::Floating,
                nodle_plugin_sdk::StackingMode::VerticalStack => StackingMode::VerticalStack,
                nodle_plugin_sdk::StackingMode::TabbedStack => StackingMode::TabbedStack,
                nodle_plugin_sdk::StackingMode::Docked => StackingMode::Docked,
            },
            resizable: plugin_meta.resizable,
            
            // Connectivity
            inputs: plugin_meta.inputs.iter().map(|input| PortDefinition {
                name: input.name.clone(),
                data_type: self.convert_plugin_data_type(&input.data_type),
                optional: input.optional,
                description: input.description.clone(),
            }).collect(),
            outputs: plugin_meta.outputs.iter().map(|output| PortDefinition {
                name: output.name.clone(),
                data_type: self.convert_plugin_data_type(&output.data_type),
                optional: output.optional,
                description: output.description.clone(),
            }).collect(),
            allow_multiple_connections: plugin_meta.allow_multiple_connections,
            
            // Execution behavior
            execution_mode: match plugin_meta.execution_mode {
                nodle_plugin_sdk::ExecutionMode::Realtime => ExecutionMode::Realtime,
                nodle_plugin_sdk::ExecutionMode::OnDemand => ExecutionMode::OnDemand,
                nodle_plugin_sdk::ExecutionMode::Manual => ExecutionMode::Manual,
                nodle_plugin_sdk::ExecutionMode::Background => ExecutionMode::Background,
            },
            processing_cost: match plugin_meta.processing_cost {
                nodle_plugin_sdk::ProcessingCost::Minimal => ProcessingCost::Minimal,
                nodle_plugin_sdk::ProcessingCost::Low => ProcessingCost::Low,
                nodle_plugin_sdk::ProcessingCost::Medium => ProcessingCost::Medium,
                nodle_plugin_sdk::ProcessingCost::High => ProcessingCost::High,
                nodle_plugin_sdk::ProcessingCost::VeryHigh => ProcessingCost::VeryHigh,
            },
            requires_gpu: plugin_meta.requires_gpu,
            
            // Advanced properties
            is_workspace_node: plugin_meta.is_workspace_node,
            supports_preview: plugin_meta.supports_preview,
        }
    }
    
    /// Convert plugin SDK DataType to core DataType
    fn convert_plugin_data_type(&self, plugin_type: &nodle_plugin_sdk::DataType) -> DataType {
        match plugin_type {
            nodle_plugin_sdk::DataType::Float => DataType::Float,
            nodle_plugin_sdk::DataType::Vector3 => DataType::Vector3,
            nodle_plugin_sdk::DataType::Color => DataType::Color,
            nodle_plugin_sdk::DataType::String => DataType::String,
            nodle_plugin_sdk::DataType::Boolean => DataType::Boolean,
            nodle_plugin_sdk::DataType::Any => DataType::Any,
        }
    }
    
    /// Get all available node types
    pub fn node_types(&self) -> Vec<&str> {
        let mut types: Vec<&str> = self.creators.keys().map(|s| s.as_str()).collect();
        types.extend(self.plugin_factories.keys().map(|s| s.as_str()));
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
        // Try core nodes first
        if let Some(provider) = self.metadata_providers.get(node_type) {
            return Some(provider());
        }
        
        // Try plugin nodes
        self.get_node_metadata(node_type)
    }
    
    /// Generate menu structure from registered node categories
    pub fn generate_menu_structure(&self, workspace_filter: &[&str]) -> Vec<crate::workspace::WorkspaceMenuItem> {
        use crate::workspace::WorkspaceMenuItem;
        use std::collections::BTreeMap;
        
        // Group nodes by their category paths - use BTreeMap for deterministic ordering
        let mut category_groups: BTreeMap<Vec<String>, Vec<(String, String)>> = BTreeMap::new();
        
        // Process core nodes
        for (node_type, provider) in &self.metadata_providers {
            let metadata = provider();
            
            // NODE-CENTRIC: Only include nodes that declare compatibility with this workspace
            let is_compatible = if workspace_filter.is_empty() {
                // If no workspace filter, include all nodes
                true
            } else if metadata.workspace_compatibility.is_empty() {
                // If node declares no specific compatibility, include in all workspaces (legacy behavior)
                true
            } else {
                // Check if node explicitly declares compatibility with this workspace
                workspace_filter.iter().any(|workspace| 
                    metadata.workspace_compatibility.iter().any(|compat| compat == workspace)
                )
            };
            
            if is_compatible {
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
        
        // Process plugin nodes
        for (node_type, _factory) in &self.plugin_factories {
            // Use cached metadata instead of calling factory.metadata() repeatedly
            let metadata = match self.plugin_metadata_cache.get(node_type) {
                Some(cached_metadata) => cached_metadata,
                None => {
                    error!("Missing cached metadata for plugin node type: {}", node_type);
                    continue;
                }
            };
            
            // NODE-CENTRIC: Only include nodes that declare compatibility with this workspace
            let is_compatible = if workspace_filter.is_empty() {
                // If no workspace filter, include all nodes
                true
            } else if metadata.workspace_compatibility.is_empty() {
                // If node declares no specific compatibility, include in all workspaces (legacy behavior)
                true
            } else {
                // Check if node explicitly declares compatibility with this workspace
                workspace_filter.iter().any(|workspace| 
                    metadata.workspace_compatibility.iter().any(|compat| compat == workspace)
                )
            };
            
            if is_compatible {
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
                    .push((metadata.display_name.clone(), node_type.clone()));
            }
        }
        
        // Convert category groups to hierarchical menu items
        let mut menu_items = Vec::new();
        
        // Build hierarchical menu structure - use BTreeMap for deterministic ordering
        let mut hierarchy: BTreeMap<String, Vec<WorkspaceMenuItem>> = BTreeMap::new();
        
        for (category_path, mut nodes) in category_groups {
            if !nodes.is_empty() {
                // Sort nodes alphabetically by display name for consistent ordering
                nodes.sort_by(|a, b| a.0.cmp(&b.0));
                
                let mut node_items = Vec::new();
                for (display_name, node_type) in nodes {
                    node_items.push(WorkspaceMenuItem::Node {
                        name: display_name,
                        node_type,
                    });
                }
                
                // Note: node_items are already sorted due to nodes being sorted above
                
                if category_path.is_empty() {
                    // Root level items
                    for node_item in node_items {
                        menu_items.push(node_item);
                    }
                } else if category_path.len() == 1 {
                    // Top-level category
                    let category_name = &category_path[0];
                    hierarchy.entry(category_name.clone())
                        .or_insert_with(Vec::new)
                        .extend(node_items);
                } else {
                    // Multi-level category - create hierarchical structure
                    let top_level = &category_path[0];
                    let sub_path = &category_path[1..];
                    
                    // Create subcategory
                    let subcategory = WorkspaceMenuItem::Category {
                        name: sub_path.join(" > "),
                        items: node_items,
                    };
                    
                    hierarchy.entry(top_level.clone())
                        .or_insert_with(Vec::new)
                        .push(subcategory);
                }
            }
        }
        
        // Convert hierarchy to menu items - BTreeMap iteration is already sorted
        for (category_name, mut items) in hierarchy {
            // Sort items within each category for consistent ordering
            items.sort_by(|a, b| {
                match (a, b) {
                    (WorkspaceMenuItem::Node { name: name_a, .. }, WorkspaceMenuItem::Node { name: name_b, .. }) => name_a.cmp(name_b),
                    (WorkspaceMenuItem::Category { name: name_a, .. }, WorkspaceMenuItem::Category { name: name_b, .. }) => name_a.cmp(name_b),
                    (WorkspaceMenuItem::Node { .. }, WorkspaceMenuItem::Category { .. }) => std::cmp::Ordering::Less,
                    (WorkspaceMenuItem::Category { .. }, WorkspaceMenuItem::Node { .. }) => std::cmp::Ordering::Greater,
                    _ => std::cmp::Ordering::Equal,
                }
            });
            
            menu_items.push(WorkspaceMenuItem::Category {
                name: category_name,
                items,
            });
        }
        
        // Note: Categories are already sorted due to BTreeMap iteration order
        
        menu_items
    }
}

/// Plugin support for NodeRegistry
impl nodle_plugin_sdk::NodeRegistryTrait for NodeRegistry {
    /// Register a node factory from a plugin
    fn register_node_factory(&mut self, factory: Box<dyn nodle_plugin_sdk::NodeFactory>) -> Result<(), nodle_plugin_sdk::PluginError> {
        let metadata = factory.metadata();
        let node_type = metadata.node_type.clone();
        
        // Cache the metadata to avoid repeated calls
        self.plugin_metadata_cache.insert(node_type.clone(), metadata.clone());
        
        // Store the plugin factory
        self.plugin_factories.insert(node_type.clone(), factory);
        
        // Add to categories for menu generation
        self.categories
            .entry(NodeCategory::new(&metadata.category.path().iter().map(|s| s.as_str()).collect::<Vec<_>>()))
            .or_insert_with(Vec::new)
            .push(node_type);
            
        Ok(())
    }
    
    /// Get list of registered node types
    fn get_node_types(&self) -> Vec<String> {
        let mut types: Vec<String> = self.creators.keys().cloned().collect();
        types.extend(self.plugin_factories.keys().cloned());
        types.sort();
        types
    }
    
    /// Check if a node type is registered
    fn has_node_type(&self, node_type: &str) -> bool {
        self.creators.contains_key(node_type) || self.plugin_factories.contains_key(node_type)
    }
}

impl Default for NodeRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        
        // Register modular math nodes
        registry.register::<crate::nodes::math::add::AddNodeFactory>();
        registry.register::<crate::nodes::math::subtract::SubtractNodeFactory>();
        registry.register::<crate::nodes::math::multiply::MultiplyNodeFactory>();
        registry.register::<crate::nodes::math::divide::DivideNodeFactory>();
        
        // Register modular logic nodes
        registry.register::<crate::nodes::logic::and::AndNodeFactory>();
        registry.register::<crate::nodes::logic::or::OrNodeFactory>();
        registry.register::<crate::nodes::logic::not::NotNodeFactory>();
        
        // Register modular data nodes
        registry.register::<crate::nodes::data::constant::ConstantNodeFactory>();
        registry.register::<crate::nodes::data::variable::VariableNodeFactory>();
        
        // Register modular output nodes
        registry.register::<crate::nodes::output::PrintNodeFactory>();
        registry.register::<crate::nodes::output::DebugNodeFactory>();
        
        // Register 3D nodes and their interface versions
        registry.register::<crate::nodes::three_d::transform::TranslateNode>();
        registry.register::<crate::nodes::three_d::transform::RotateNode>();
        registry.register::<crate::nodes::three_d::transform::ScaleNode>();
        registry.register::<crate::nodes::three_d::geometry::CubeNode>();
        registry.register::<crate::nodes::three_d::geometry::SphereNode>();
        registry.register::<crate::nodes::three_d::geometry::PlaneNode>();
        registry.register::<crate::nodes::three_d::lighting::PointLightNode>();
        registry.register::<crate::nodes::three_d::lighting::DirectionalLightNode>();
        registry.register::<crate::nodes::three_d::lighting::SpotLightNode>();
        registry.register::<crate::nodes::three_d::output::ViewportNode>();
        
        // USD nodes now loaded via comprehensive USD plugin
        
        registry
    }
}