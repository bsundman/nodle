# Plugin System Architecture

This document describes the current hybrid node system and provides comprehensive instructions for extending Nodle with plugin support for external nodes.

## Current Architecture (Hybrid Approach)

Nodle uses a **hybrid architecture** combining compile-time nodes with runtime extensibility:

### Core System Components

1. **Enhanced NodeFactory Pattern**: All nodes implement `NodeFactory` trait with rich metadata
2. **Dynamic Menu Generation**: Menus are generated automatically from node metadata
3. **Workspace-Specific Registration**: Each workspace registers only relevant nodes
4. **Node-Centric Metadata**: Nodes declare their own workspace compatibility, categories, and behavior

### Current Node Registration Flow

```rust
// Each workspace creates its own node registry
let mut node_registry = NodeRegistry::new();

// Register nodes specific to this workspace
node_registry.register::<TransformNode>();
node_registry.register::<GeometryNode>();
node_registry.register::<UtilityNode>();

// Menu is generated dynamically from registered nodes
let menu = node_registry.generate_menu_structure(&["3D"]);
```

## Phase 1: Current System (✅ Implemented)

### Features
- ✅ Self-describing nodes via `NodeFactory`
- ✅ Dynamic menu generation
- ✅ Workspace compatibility filtering
- ✅ Automatic parameter panel creation
- ✅ Node-centric architecture

### Limitations
- All nodes must be compiled into main binary
- Cannot add new nodes without recompilation
- No runtime discovery of external nodes

## Phase 2: Plugin System Implementation

### Overview

The plugin system will extend the current architecture to support dynamically loaded external nodes while maintaining the existing API.

### Architecture Design

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Core Binary   │    │  Plugin Manager │    │  External Plugin│
│                 │    │                 │    │                 │
│ • Core Nodes    │◄──►│ • Plugin Loading│◄──►│ • Custom Nodes  │
│ • Workspaces    │    │ • Registration  │    │ • Metadata      │
│ • UI System     │    │ • Lifecycle Mgmt│    │ • Logic         │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Plugin Interface Definition

```rust
// Plugin trait that external libraries must implement
pub trait NodePlugin: Send + Sync {
    /// Plugin metadata
    fn plugin_info(&self) -> PluginInfo;
    
    /// Register all nodes provided by this plugin
    fn register_nodes(&self, registry: &mut NodeRegistry);
    
    /// Called when plugin is loaded
    fn on_load(&self) -> Result<(), PluginError>;
    
    /// Called when plugin is unloaded
    fn on_unload(&self) -> Result<(), PluginError>;
}

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub compatible_version: String, // Nodle version compatibility
}
```

### Plugin Manager Implementation

```rust
pub struct PluginManager {
    loaded_plugins: HashMap<String, LoadedPlugin>,
    plugin_directories: Vec<PathBuf>,
    core_registry: NodeRegistry,
}

struct LoadedPlugin {
    library: Library,
    plugin: Box<dyn NodePlugin>,
    info: PluginInfo,
}

impl PluginManager {
    /// Scan directories for plugins and load them
    pub fn discover_and_load_plugins(&mut self) -> Result<(), PluginError>;
    
    /// Load a specific plugin from path
    pub fn load_plugin<P: AsRef<Path>>(&mut self, path: P) -> Result<(), PluginError>;
    
    /// Unload a plugin by name
    pub fn unload_plugin(&mut self, name: &str) -> Result<(), PluginError>;
    
    /// Get enhanced registry with both core and plugin nodes
    pub fn get_enhanced_registry(&self, workspace_id: &str) -> NodeRegistry;
}
```

## Plugin Development Guide

### Prerequisites

1. **Rust toolchain** (same version as main Nodle project)
2. **Nodle SDK** (contains plugin interfaces and common types)
3. **Plugin template** (starter project structure)

### Creating a Plugin

#### Step 1: Setup Plugin Project

```bash
# Clone the plugin template
git clone https://github.com/nodle/plugin-template my-plugin
cd my-plugin

# Update Cargo.toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
nodle-sdk = "0.1.0"
egui = "0.24"
```

#### Step 2: Implement Plugin Interface

```rust
// src/lib.rs
use nodle_sdk::{NodePlugin, PluginInfo, NodeRegistry, NodeFactory, PluginError};

pub struct MyPlugin;

impl NodePlugin for MyPlugin {
    fn plugin_info(&self) -> PluginInfo {
        PluginInfo {
            name: "My Plugin".to_string(),
            version: "0.1.0".to_string(),
            author: "Your Name".to_string(),
            description: "Description of what this plugin does".to_string(),
            compatible_version: "0.1.0".to_string(),
        }
    }
    
    fn register_nodes(&self, registry: &mut NodeRegistry) {
        registry.register::<MyCustomNode>();
        registry.register::<AnotherNode>();
    }
    
    fn on_load(&self) -> Result<(), PluginError> {
        println!("MyPlugin loaded successfully");
        Ok(())
    }
    
    fn on_unload(&self) -> Result<(), PluginError> {
        println!("MyPlugin unloaded");
        Ok(())
    }
}

// Export the plugin creation function
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn NodePlugin {
    Box::into_raw(Box::new(MyPlugin))
}

#[no_mangle]
pub extern "C" fn destroy_plugin(plugin: *mut dyn NodePlugin) {
    unsafe {
        let _ = Box::from_raw(plugin);
    }
}
```

#### Step 3: Implement Custom Nodes

```rust
// src/nodes/my_custom_node.rs
use nodle_sdk::{NodeFactory, NodeMetadata, NodeCategory, Node, ParameterChange};
use egui::Pos2;

pub struct MyCustomNode;

impl NodeFactory for MyCustomNode {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new(
            "MyCustomNode",
            "My Custom Node",
            NodeCategory::new(&["Custom", "Utility"]),
            "A custom node that does something amazing"
        )
        .with_workspace_compatibility(vec!["3D", "MaterialX"])
        .with_color(egui::Color32::from_rgb(255, 100, 50))
        .with_icon("🔥")
        .with_inputs(vec![
            PortDefinition::required("Input", DataType::Float),
        ])
        .with_outputs(vec![
            PortDefinition::required("Output", DataType::Float),
        ])
    }
}

// Implement your node's parameter interface
impl MyCustomNode {
    pub fn build_interface(node: &mut Node, ui: &mut egui::Ui) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        
        // Add your custom UI here
        ui.label("My Custom Node Parameters");
        
        // Example parameter
        let mut value = node.get_parameter("custom_value")
            .and_then(|v| v.as_float())
            .unwrap_or(1.0);
            
        if ui.add(egui::Slider::new(&mut value, 0.0..=10.0).text("Custom Value")).changed() {
            changes.push(ParameterChange {
                parameter: "custom_value".to_string(),
                value: NodeData::Float(value),
            });
        }
        
        changes
    }
}
```

#### Step 4: Build and Deploy

```bash
# Build the plugin
cargo build --release

# The resulting .dll/.so/.dylib file is your plugin
# Copy to Nodle's plugins directory
cp target/release/libmy_plugin.dylib ~/.nodle/plugins/
```

### Plugin Directory Structure

```
~/.nodle/plugins/
├── my-plugin.dylib
├── another-plugin.dylib
└── experimental/
    └── test-plugin.dylib
```

### Plugin Metadata File (Optional)

```toml
# my-plugin.toml
[plugin]
name = "My Plugin"
version = "0.1.0"
author = "Your Name"
description = "Description"
library = "my-plugin.dylib"
compatible_versions = ["0.1.0", "0.1.1"]

[dependencies]
required_plugins = []
optional_plugins = ["utility-pack"]
```

## Implementation Roadmap

### Phase 2.1: Core Plugin Infrastructure

1. **Plugin Manager Implementation**
   - Dynamic library loading (`libloading` crate)
   - Plugin discovery and registration
   - Lifecycle management
   - Error handling and recovery

2. **Plugin Interface Design**
   - Define stable C ABI for plugin communication
   - Version compatibility checking
   - Plugin metadata validation

3. **Enhanced Node Registry**
   - Merge core and plugin nodes
   - Conflict resolution (duplicate node types)
   - Runtime registration/deregistration

### Phase 2.2: Development Tools

1. **Plugin SDK Package**
   - Separate crate with plugin interfaces
   - Common types and utilities
   - Documentation and examples

2. **Plugin Template**
   - Starter project with build configuration
   - Example nodes and best practices
   - CI/CD templates for plugin authors

3. **Plugin Manager CLI**
   - Install/uninstall plugins
   - List available plugins
   - Validate plugin compatibility

### Phase 2.3: Advanced Features

1. **Hot Reload Support**
   - Reload plugins without restarting Nodle
   - Preserve scene state during reload
   - Development workflow optimization

2. **Plugin Dependencies**
   - Inter-plugin dependencies
   - Dependency resolution
   - Circular dependency detection

3. **Sandboxing and Security**
   - Plugin permission system
   - Resource usage limits
   - Safe plugin execution environment

## Migration Guide

### For Node Developers

**Current System (Phase 1):**
```rust
// Nodes are compiled into main binary
impl NodeFactory for MyNode { ... }
```

**Future Plugin System (Phase 2):**
```rust
// Nodes can be external plugins
impl NodeFactory for MyNode { ... }

// Plugin exports nodes
impl NodePlugin for MyPlugin {
    fn register_nodes(&self, registry: &mut NodeRegistry) {
        registry.register::<MyNode>();
    }
}
```

### For Workspace Developers

**Current System:**
```rust
// Workspaces register specific nodes
let mut registry = NodeRegistry::new();
registry.register::<MyNode>();
```

**Future Plugin System:**
```rust
// Workspaces get enhanced registry with plugin nodes
let registry = plugin_manager.get_enhanced_registry("3D");
// All compatible nodes (core + plugins) automatically available
```

## API Stability Guarantees

### Stable APIs (Breaking changes require major version bump)
- `NodeFactory` trait
- `NodeMetadata` structure
- Core plugin interface
- Workspace trait

### Unstable APIs (May change in minor versions)
- Internal plugin manager implementation
- Plugin discovery mechanisms
- Development tools CLI

## Security Considerations

### Plugin Validation
- Digital signature verification
- Source code scanning (for open-source plugins)
- Runtime behavior monitoring

### Resource Management
- Memory usage limits per plugin
- CPU time restrictions
- File system access controls

### User Consent
- Explicit plugin installation approval
- Permission requests for sensitive operations
- Plugin audit logging

## Performance Considerations

### Plugin Loading
- Lazy loading: Only load plugins when workspace is activated
- Parallel loading: Load multiple plugins concurrently
- Caching: Cache plugin metadata to speed up discovery

### Runtime Performance
- JIT compilation for hot paths
- Plugin-specific optimization hints
- Resource pooling across plugins

## Debugging and Diagnostics

### Plugin Debugging
- Debug symbols in plugin builds
- Plugin-specific log channels
- Performance profiling integration

### Error Reporting
- Structured error messages with plugin context
- Automatic crash reports with plugin information
- Plugin compatibility warnings

## Example Plugin Ideas

### Community Plugins
1. **Math Utilities Plugin**: Advanced mathematical operations
2. **File Format Plugin**: Support for additional file formats
3. **Rendering Plugin**: Custom rendering techniques
4. **Animation Plugin**: Animation and keyframing tools
5. **Physics Plugin**: Physics simulation nodes

### Development Examples
1. **Hello World Plugin**: Minimal example showing basic concepts
2. **Parameter Plugin**: Advanced parameter UI demonstrations
3. **Workspace Plugin**: Custom workspace implementation
4. **Integration Plugin**: External service integration examples

## Contributing to Plugin Ecosystem

### Plugin Registry
- Central repository for discovering plugins
- Automated testing and validation
- Community ratings and reviews

### Documentation
- Plugin development tutorials
- API reference documentation
- Best practices guide

### Community
- Plugin developer forums
- Code sharing and collaboration
- Plugin development contests

---

This plugin system design maintains backward compatibility with the current architecture while providing a clear path toward extensibility and community-driven development.