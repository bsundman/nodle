# Nodle Plugin System Architecture

This document describes the current plugin system architecture that allows extending Nodle with dynamically loaded external nodes.

## Overview

Nodle's plugin system enables developers to create custom nodes as external dynamic libraries that can be loaded at runtime. This allows extending Nodle's functionality without modifying the core application.

### Key Features

- **Dynamic Loading**: Plugins are loaded from `.dll` (Windows), `.so` (Linux), or `.dylib` (macOS) files
- **Safe FFI**: Uses handle-based approach for safe foreign function interface
- **Viewport Integration**: Plugins can create viewport nodes with full 3D rendering capabilities
- **Hot Reload Support**: Plugins can be loaded and unloaded at runtime
- **Type Safety**: Strong typing with conversion layers between core and SDK types

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Core Binary   │    │  Plugin Manager │    │  External Plugin│
│                 │    │                 │    │   (.dylib/.so)  │
│ • Core Nodes    │◄──►│ • Dynamic Load  │◄──►│ • Custom Nodes  │
│ • Workspaces    │    │ • FFI Safety    │    │ • Node Factory  │
│ • UI System     │    │ • Type Convert  │    │ • Parameters    │
│ • Viewport      │    │ • Instance Mgmt │    │ • Processing    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Core Components

1. **Plugin Manager** (`src/plugins/mod.rs`)
   - Discovers and loads plugins from standard directories
   - Manages plugin lifecycle (load/unload)
   - Stores plugin node instances
   - Handles menu integration

2. **Plugin SDK** (`nodle-plugin-sdk` crate)
   - Provides interfaces for plugin development
   - Type definitions and traits
   - Viewport data structures
   - UI element definitions

3. **Plugin Interface** (`src/plugin_interface.rs`)
   - Type conversion layer between core and SDK
   - FFI-safe handle wrappers
   - Viewport data conversions

## Plugin Development Guide

### Prerequisites

- Rust toolchain (same version as Nodle core)
- Access to the `nodle-plugin-sdk` crate
- Nodle application for testing

### Getting Started

1. **Create a new Rust project**:
```bash
cargo new my-nodle-plugin --lib
cd my-nodle-plugin
```

2. **Configure `Cargo.toml`**:
```toml
[package]
name = "my-nodle-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
nodle-plugin-sdk = { path = "../nodle-plugin-sdk" }  # Or version from crates.io
egui = "0.24"
uuid = { version = "1.0", features = ["v4"] }  # For unique node IDs
```

3. **Implement the plugin interface**:

```rust
use nodle_plugin_sdk::*;

pub struct MyPlugin;

impl NodePlugin for MyPlugin {
    fn plugin_info(&self) -> PluginInfo {
        PluginInfo {
            name: "My Plugin".to_string(),
            version: "0.1.0".to_string(),
            author: "Your Name".to_string(),
            description: "A custom plugin for Nodle".to_string(),
            compatible_version: "0.1.0".to_string(),
        }
    }
    
    fn register_nodes(&self, registry: &mut dyn NodeRegistryTrait) {
        registry.register_node_factory(Box::new(MyNodeFactory));
    }
    
    fn get_menu_structure(&self) -> Vec<MenuStructure> {
        vec![
            MenuStructure::Category {
                name: "My Plugin".to_string(),
                items: vec![
                    MenuStructure::Node {
                        name: "My Custom Node".to_string(),
                        node_type: "MyPlugin.MyCustomNode".to_string(),
                        metadata: MyNodeFactory.metadata(),
                    }
                ],
            }
        ]
    }
}

// Export functions for dynamic loading
#[no_mangle]
pub extern "C" fn create_plugin() -> PluginHandle {
    PluginHandle::new(Box::new(MyPlugin))
}

#[no_mangle]
pub extern "C" fn destroy_plugin(plugin: PluginHandle) {
    // Handle is automatically cleaned up when dropped
    drop(plugin);
}
```

### Creating Custom Nodes

1. **Define a Node Factory**:

```rust
pub struct MyNodeFactory;

impl NodeFactory for MyNodeFactory {
    fn metadata(&self) -> NodeMetadata {
        NodeMetadata {
            name: "My Custom Node".to_string(),
            category: NodeCategory::new(&["Custom", "Processing"]),
            description: "A custom processing node".to_string(),
            color: Some([100, 150, 200, 255]),
            icon: Some("🔧"),
            panel_type: PanelType::Parameter,
            ..Default::default()
        }
    }
    
    fn create_node(&self, position: egui::Pos2) -> PluginNodeHandle {
        PluginNodeHandle::new(Box::new(MyCustomNode::new(position)))
    }
}
```

2. **Implement the Node Logic**:

```rust
use std::collections::HashMap;
use uuid::Uuid;

pub struct MyCustomNode {
    id: String,
    position: egui::Pos2,
    // Your node's state
    multiplier: f32,
}

impl MyCustomNode {
    pub fn new(position: egui::Pos2) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            position,
            multiplier: 1.0,
        }
    }
}

impl PluginNode for MyCustomNode {
    fn id(&self) -> String {
        self.id.clone()
    }
    
    fn position(&self) -> egui::Pos2 {
        self.position
    }
    
    fn set_position(&mut self, position: egui::Pos2) {
        self.position = position;
    }
    
    fn get_parameter(&self, name: &str) -> Option<NodeData> {
        match name {
            "multiplier" => Some(NodeData::Float(self.multiplier)),
            _ => None,
        }
    }
    
    fn set_parameter(&mut self, name: &str, value: NodeData) {
        match name {
            "multiplier" => {
                if let NodeData::Float(v) = value {
                    self.multiplier = v;
                }
            }
            _ => {}
        }
    }
    
    fn process(&mut self, inputs: &HashMap<String, NodeData>) -> HashMap<String, NodeData> {
        let mut outputs = HashMap::new();
        
        if let Some(NodeData::Float(input_value)) = inputs.get("input") {
            outputs.insert("output".to_string(), 
                NodeData::Float(input_value * self.multiplier));
        }
        
        outputs
    }
    
    fn get_parameter_ui(&self) -> ParameterUI {
        ParameterUI {
            elements: vec![
                UIElement::Heading("Parameters".to_string()),
                UIElement::Slider {
                    label: "Multiplier".to_string(),
                    value: self.multiplier,
                    min: 0.0,
                    max: 10.0,
                    parameter_name: "multiplier".to_string(),
                },
            ],
        }
    }
    
    fn handle_ui_action(&mut self, action: UIAction) -> Vec<ParameterChange> {
        match action {
            UIAction::ParameterChanged { parameter, value } => {
                self.set_parameter(&parameter, value.clone());
                vec![ParameterChange { parameter, value }]
            }
            _ => vec![],
        }
    }
}
```

### Creating Viewport Nodes

Viewport nodes can render 3D content:

```rust
impl PluginNode for MyViewportNode {
    fn supports_viewport(&self) -> bool {
        true
    }
    
    fn get_viewport_data(&self) -> Option<ViewportData> {
        Some(ViewportData {
            scene: SceneData {
                name: "My Scene".to_string(),
                meshes: vec![/* your mesh data */],
                materials: vec![/* your materials */],
                lights: vec![/* your lights */],
                camera: CameraData {
                    position: [5.0, 5.0, 5.0],
                    target: [0.0, 0.0, 0.0],
                    up: [0.0, 1.0, 0.0],
                    fov: 45.0,
                    near: 0.1,
                    far: 100.0,
                    aspect: 1.0,
                },
                bounding_box: ([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
            },
            settings: ViewportSettings {
                background_color: [0.2, 0.2, 0.2, 1.0],
                wireframe: false,
                lighting: true,
                show_grid: true,
                show_ground_plane: false,
                aa_samples: 4,
                shading_mode: ShadingMode::Smooth,
            },
            dimensions: (800, 600),
            scene_dirty: true,
            settings_dirty: false,
        })
    }
    
    fn handle_viewport_camera(&mut self, manipulation: CameraManipulation) {
        // Handle camera manipulation
        match manipulation {
            CameraManipulation::Orbit { delta_x, delta_y } => {
                // Update camera position
            }
            // ... other cases
        }
    }
}
```

## Building and Installing Plugins

1. **Build your plugin**:
```bash
cargo build --release
```

2. **Install the plugin**:
   - Copy the generated library to one of the plugin directories:
     - `~/.nodle/plugins/` (user plugins)
     - `./plugins/` (local plugins)
   
   Example:
   ```bash
   # macOS
   cp target/release/libmy_nodle_plugin.dylib ~/.nodle/plugins/
   
   # Linux
   cp target/release/libmy_nodle_plugin.so ~/.nodle/plugins/
   
   # Windows
   cp target/release/my_nodle_plugin.dll ~/.nodle/plugins/
   ```

3. **Verify installation**:
   - Start Nodle
   - Check the console for "Successfully loaded plugin: My Plugin"
   - Your nodes should appear in the context menu

## Plugin Loading Process

1. **Discovery**: On startup, Nodle scans plugin directories for compatible libraries
2. **Loading**: Each plugin's `create_plugin` function is called
3. **Validation**: Version compatibility is checked
4. **Registration**: Plugin nodes are registered with the node registry
5. **Menu Integration**: Plugin menu structures are merged with core menus

### Plugin Directories

Nodle searches for plugins in these locations (in order):
1. `~/.nodle/plugins/` - User-specific plugins
2. `./plugins/` - Local plugins (relative to Nodle executable)

### Console Output

During startup, you'll see:
```
🔌 Initializing global plugin system...
📦 No plugins found in standard directories  # If no plugins
✅ Successfully loaded plugin: My Plugin v0.1.0  # For each loaded plugin
❌ Failed to load plugin /path/to/plugin.dylib: <error>  # For failures
```

## Type Conversion

The plugin system handles automatic conversion between core and SDK types:

- **NodeData**: Parameter values (floats, vectors, strings, etc.)
- **ViewportData**: Complete viewport rendering information
- **UI Elements**: Parameter interface descriptions
- **Camera/Material/Light data**: 3D scene components

### NodeData Types

```rust
pub enum NodeData {
    Float(f32),
    Vector3([f32; 3]),
    Color([f32; 3]),
    String(String),
    Boolean(bool),
}
```

### Port Definition

When defining node ports in metadata:

```rust
fn metadata(&self) -> NodeMetadata {
    NodeMetadata {
        // ... other fields ...
        inputs: vec![
            PortDefinition {
                name: "input".to_string(),
                data_type: DataType::Float,
                required: true,
            },
        ],
        outputs: vec![
            PortDefinition {
                name: "output".to_string(),
                data_type: DataType::Float,
                required: true,
            },
        ],
        ..Default::default()
    }
}
```

## Best Practices

1. **Error Handling**: Implement robust error handling in your plugins
2. **Performance**: Process nodes efficiently to avoid blocking the UI
3. **Memory Management**: Clean up resources properly
4. **Version Compatibility**: Always specify correct compatible_version
5. **Unique IDs**: Use unique node type identifiers (e.g., "PluginName.NodeName")

## Debugging Plugins

1. **Console Output**: Use `println!` for debug messages
2. **Check Loading**: Verify plugin appears in console on startup
3. **Node Creation**: Confirm nodes appear in context menu
4. **Parameter UI**: Test all parameter controls
5. **Processing**: Verify node processing with test data

## Security Considerations

- Plugins have full access to the Rust runtime
- Only load plugins from trusted sources
- Plugins run in the same process as Nodle
- No sandboxing is currently implemented

## API Stability

The plugin API is designed for stability:
- Core traits (`NodePlugin`, `PluginNode`) are stable
- Type conversion layers handle version differences
- Breaking changes will increment major version

## Complete Example Plugin

Here's a complete, working example of a simple math plugin:

```rust
// src/lib.rs
use nodle_plugin_sdk::*;
use std::collections::HashMap;
use uuid::Uuid;

/// The main plugin struct
pub struct MathPlugin;

impl NodePlugin for MathPlugin {
    fn plugin_info(&self) -> PluginInfo {
        PluginInfo {
            name: "Math Plugin".to_string(),
            version: "0.1.0".to_string(),
            author: "Nodle Team".to_string(),
            description: "Basic math operations".to_string(),
            compatible_version: "0.1.0".to_string(),
        }
    }
    
    fn register_nodes(&self, registry: &mut dyn NodeRegistryTrait) {
        registry.register_node_factory(Box::new(AddNodeFactory));
        registry.register_node_factory(Box::new(MultiplyNodeFactory));
    }
    
    fn get_menu_structure(&self) -> Vec<MenuStructure> {
        vec![
            MenuStructure::Category {
                name: "Math".to_string(),
                items: vec![
                    MenuStructure::Node {
                        name: "Add".to_string(),
                        node_type: "MathPlugin.Add".to_string(),
                        metadata: AddNodeFactory.metadata(),
                    },
                    MenuStructure::Node {
                        name: "Multiply".to_string(),
                        node_type: "MathPlugin.Multiply".to_string(),
                        metadata: MultiplyNodeFactory.metadata(),
                    },
                ],
            }
        ]
    }
}

/// Factory for Add nodes
struct AddNodeFactory;

impl NodeFactory for AddNodeFactory {
    fn metadata(&self) -> NodeMetadata {
        NodeMetadata {
            name: "Add".to_string(),
            category: NodeCategory::new(&["Math", "Basic"]),
            description: "Adds two numbers".to_string(),
            color: Some([150, 200, 150, 255]),
            icon: Some("➕"),
            panel_type: PanelType::Parameter,
            inputs: vec![
                PortDefinition {
                    name: "a".to_string(),
                    data_type: DataType::Float,
                    required: true,
                },
                PortDefinition {
                    name: "b".to_string(),
                    data_type: DataType::Float,
                    required: true,
                },
            ],
            outputs: vec![
                PortDefinition {
                    name: "sum".to_string(),
                    data_type: DataType::Float,
                    required: true,
                },
            ],
            ..Default::default()
        }
    }
    
    fn create_node(&self, position: egui::Pos2) -> PluginNodeHandle {
        PluginNodeHandle::new(Box::new(AddNode::new(position)))
    }
}

/// The Add node implementation
struct AddNode {
    id: String,
    position: egui::Pos2,
}

impl AddNode {
    fn new(position: egui::Pos2) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            position,
        }
    }
}

impl PluginNode for AddNode {
    fn id(&self) -> String {
        self.id.clone()
    }
    
    fn position(&self) -> egui::Pos2 {
        self.position
    }
    
    fn set_position(&mut self, position: egui::Pos2) {
        self.position = position;
    }
    
    fn process(&mut self, inputs: &HashMap<String, NodeData>) -> HashMap<String, NodeData> {
        let mut outputs = HashMap::new();
        
        let a = inputs.get("a")
            .and_then(|v| v.as_float())
            .unwrap_or(0.0);
            
        let b = inputs.get("b")
            .and_then(|v| v.as_float())
            .unwrap_or(0.0);
        
        outputs.insert("sum".to_string(), NodeData::Float(a + b));
        outputs
    }
    
    fn get_parameter_ui(&self) -> ParameterUI {
        ParameterUI {
            elements: vec![
                UIElement::Label("Connect inputs to add numbers".to_string()),
            ],
        }
    }
    
    fn handle_ui_action(&mut self, _action: UIAction) -> Vec<ParameterChange> {
        vec![] // No parameters to change
    }
    
    fn get_parameter(&self, _name: &str) -> Option<NodeData> {
        None // No parameters
    }
    
    fn set_parameter(&mut self, _name: &str, _value: NodeData) {
        // No parameters to set
    }
}

// Plugin entry points
#[no_mangle]
pub extern "C" fn create_plugin() -> PluginHandle {
    PluginHandle::new(Box::new(MathPlugin))
}

#[no_mangle]
pub extern "C" fn destroy_plugin(plugin: PluginHandle) {
    drop(plugin);
}
```

## Additional Resources

- **Plugin SDK Documentation**: Full API reference for all SDK types and traits
- **Example Plugins**: 
  - [Basic Math Plugin](https://github.com/nodle/plugin-examples/tree/main/math-plugin)
  - [USD Viewport Plugin](https://github.com/nodle/plugin-examples/tree/main/usd-plugin)
  - [Image Processing Plugin](https://github.com/nodle/plugin-examples/tree/main/image-plugin)
- **Plugin Template**: [Starter template with build configuration](https://github.com/nodle/plugin-template)

## Troubleshooting

**Plugin not loading?**
- Check file extension matches platform (.dylib/.so/.dll)
- Verify `create_plugin` and `destroy_plugin` are exported
- Check version compatibility
- Look for error messages in console

**Nodes not appearing?**
- Verify `register_nodes` is implemented
- Check node metadata is valid
- Ensure menu structure is properly defined

**Crashes on load?**
- Check for panic in plugin initialization
- Verify all dependencies are available
- Test plugin in isolation first

## Frequently Asked Questions

**Q: Can plugins access the file system?**
A: Yes, plugins run in the same process as Nodle and have full Rust capabilities, including file I/O.

**Q: How do I debug my plugin?**
A: Use `println!` statements which will appear in Nodle's console. You can also use a debugger by attaching to the Nodle process.

**Q: Can I use external crates in my plugin?**
A: Yes, you can use any Rust crate. Just add them to your plugin's `Cargo.toml` dependencies.

**Q: How do I handle errors in my plugin?**
A: Return appropriate `NodeData` values or use `println!` to log errors. Panics in plugins will crash Nodle, so handle errors gracefully.

**Q: Can multiple plugins define nodes with the same name?**
A: Yes, but use unique node type identifiers (e.g., "MyPlugin.NodeName") to avoid conflicts.

**Q: How do I update my plugin?**
A: Simply replace the plugin file and restart Nodle. Future versions may support hot reloading.

**Q: Can plugins communicate with each other?**
A: Not directly. Plugins communicate through the node graph by connecting their nodes.

**Q: What happens if my plugin crashes?**
A: Since plugins run in-process, a plugin crash will crash Nodle. Always test thoroughly.

---

For more information and examples, visit the [Nodle Plugin SDK repository](https://github.com/nodle/nodle-plugin-sdk).