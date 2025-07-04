# Nōdle - Node Editor Developer Guide

## Overview

Nōdle (pronounced like "noodle") is a high-performance node-based visual programming editor built in Rust using the egui/eframe framework. It implements a vertical flow design with GPU-accelerated rendering and an extensible architecture supporting unlimited node types and specialized contexts.

## 🌟 Node-Centric Architecture

Nōdle follows a pure node-centric philosophy where **"nodes are the star of the show"**. Everything in the system is driven by node metadata - from visual appearance to interface panel behavior. This approach ensures consistency, extensibility, and maintainability across the entire codebase.

## Project Structure

```
nodle/
├── src/
│   ├── main.rs                 # Application entry point
│   ├── editor/                 # Editor components
│   │   ├── mod.rs             # Main NodeEditor struct
│   │   ├── input.rs           # Input handling & event management
│   │   ├── viewport.rs        # Viewport management (pan/zoom)
│   │   ├── interaction.rs     # Node interaction (selection, dragging)
│   │   ├── menus.rs           # Context menu system
│   │   ├── rendering.rs       # Node rendering logic
│   │   ├── navigation.rs      # Navigation controls
│   │   ├── file_manager.rs    # File operations
│   │   ├── panels/            # Panel management
│   │   │   ├── mod.rs         # Panel manager
│   │   │   └── interface_panels.rs  # Interface panel system
│   │   ├── debug_tools.rs     # Debug utilities
│   │   ├── view_manager.rs    # View management
│   │   └── workspace_builder.rs # Workspace construction
│   ├── gpu/                    # GPU rendering system
│   │   ├── mod.rs             # GPU module exports
│   │   ├── renderer.rs        # Core GPU renderer
│   │   ├── shaders/           # WGSL shaders
│   │   │   ├── button.wgsl    # Button rendering
│   │   │   ├── connection.wgsl # Connection splines
│   │   │   ├── flag.wgsl      # Visibility flags
│   │   │   ├── node.wgsl      # Node rendering
│   │   │   └── port.wgsl      # Port rendering
│   │   └── pipelines/         # Render pipelines
│   ├── nodes/                  # Node system
│   │   ├── mod.rs             # Core node types
│   │   ├── factory.rs         # Node factory & registry
│   │   ├── graph.rs           # NodeGraph implementation
│   │   ├── node.rs            # Node struct & methods
│   │   ├── port.rs            # Port types & logic
│   │   ├── interface.rs       # Interface traits
│   │   ├── math/              # Math nodes
│   │   │   ├── add/           # Addition node
│   │   │   │   ├── mod.rs     # NodeFactory implementation
│   │   │   │   ├── logic.rs   # Computation logic
│   │   │   │   └── parameters.rs # UI parameters
│   │   │   └── [other math nodes...]
│   │   ├── logic/             # Logic nodes (AND, OR, NOT)
│   │   ├── data/              # Data nodes (Constant, Variable)
│   │   ├── output/            # Output nodes (Print, Debug)
│   │   ├── three_d/           # 3D nodes
│   │   │   ├── geometry/      # Geometry primitives
│   │   │   ├── transform/     # Transform operations
│   │   │   ├── lighting/      # Light sources
│   │   │   ├── output/        # 3D viewport
│   │   │   └── usd/           # USD integration
│   │   │       ├── stage/     # Stage management
│   │   │       ├── geometry/  # USD primitives
│   │   │       ├── transform/ # USD transforms
│   │   │       ├── lighting/  # USD lights
│   │   │       └── shading/   # USD materials
│   │   └── materialx/         # MaterialX nodes
│   ├── workspaces/            # Workspace implementations
│   │   ├── mod.rs             # Workspace registry
│   │   ├── workspace_general.rs # General workspace
│   │   ├── workspace_3d.rs    # 3D workspace
│   │   └── workspace_usd.rs   # USD workspace
│   ├── workspace.rs           # Workspace trait
│   ├── menu_hierarchy.rs      # Menu organization
│   └── startup_checks.rs      # Dependency verification
├── scripts/
│   └── setup_usd.py           # USD installation script
├── vendor/                     # Local dependencies
│   └── usd/                   # Local USD installation
├── Cargo.toml                 # Project manifest
├── install.sh                 # macOS/Linux installer
├── install.bat                # Windows installer
└── .gitignore                 # Git ignore rules
```

## Core Architecture

### Node Factory System

The NodeFactory trait is the cornerstone of Nōdle's extensibility:

```rust
pub trait NodeFactory: Send + Sync {
    /// Get comprehensive node metadata
    fn metadata() -> NodeMetadata where Self: Sized;
    
    /// Create a node instance (default implementation provided)
    fn create(position: Pos2) -> Node where Self: Sized { ... }
}
```

### Node Metadata

NodeMetadata drives everything about a node:

```rust
pub struct NodeMetadata {
    // Identity
    pub node_type: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    
    // Visual
    pub color: Color32,
    pub icon: &'static str,
    
    // Organization
    pub category: NodeCategory,
    pub workspace_compatibility: Vec<&'static str>,
    
    // Connectivity
    pub inputs: Vec<PortDefinition>,
    pub outputs: Vec<PortDefinition>,
    
    // Interface
    pub panel_type: PanelType,
    
    // Execution
    pub processing_cost: ProcessingCost,
}
```

### Modular Node Structure

Each node follows a consistent modular pattern:

```
nodes/category/node_name/
├── mod.rs          # NodeFactory implementation + metadata
├── logic.rs        # Core computation logic
└── parameters.rs   # Interface panel parameters
```

Example (Addition node):
```rust
// mod.rs
impl NodeFactory for AddNode {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new("Add", "Add", NodeCategory::math(), "Adds two values")
            .with_inputs(vec![
                PortDefinition::required("A", DataType::Float),
                PortDefinition::required("B", DataType::Float),
            ])
            .with_outputs(vec![
                PortDefinition::required("Result", DataType::Float),
            ])
    }
}
```

## GPU Rendering Pipeline

Nōdle uses wgpu for high-performance rendering:

1. **Batch Collection**: Nodes and connections are collected into instance buffers
2. **GPU Upload**: Instance data is uploaded to GPU buffers
3. **Shader Rendering**: WGSL shaders render nodes, ports, and connections
4. **egui Integration**: GPU content is composited with egui UI

### Shader System

- `node.wgsl`: Renders node bodies with borders and fills
- `port.wgsl`: Renders input/output ports
- `connection.wgsl`: Renders bezier curve connections
- `flag.wgsl`: Renders visibility toggle flags
- `button.wgsl`: Renders interactive buttons

## Interface Panel System

Nōdle features a comprehensive interface panel system with automatic visibility management and type-specific stacking behavior.

### Panel Types and Behavior

- **Parameter**: Node configuration panels that stack together by default (top-right area)
- **Viewport**: 3D/2D visualization panels that remain separate and floating
- **Editor**: Complex editing interfaces
- **Inspector**: Debug/analysis tools

### Automatic Panel Management

The panel system provides automatic visibility and stacking management:

**Panel Visibility**:
- Panels automatically appear when nodes are created
- Panel visibility is managed through the `node.visible` flag and panel manager state
- Reliable node ID detection ensures proper panel assignment

**Panel Stacking**:
- Parameter panels stack together by default for efficient screen space usage
- Viewport panels remain separate and floating to prevent interference
- Panel types are completely isolated - viewport and parameter panels never stack together

### Panel Creation Flow

When a node is created, the system:

1. **Node Factory**: Creates node with proper `panel_type` set via NodeMetadata
2. **ID Assignment**: NodeGraph assigns reliable, unique node IDs via `add_node()`
3. **Panel Setup**: Panel manager configures visibility and stacking based on panel type
4. **Auto-Display**: Panels automatically become visible without manual intervention

### Node Panel Assignment

Nodes specify their panel type through the NodeFactory metadata:

```rust
impl NodeFactory for ViewportNode {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new("USD_Viewport", "USD Viewport")
            .with_panel_type(PanelType::Viewport)
            // ...
    }
}

impl NodeFactory for TransformNode {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new("3D_Translate", "3D Translate")
            .with_panel_type(PanelType::Parameter)
            // ...
    }
}
```

### Panel System Architecture

**Separation of Concerns**:
- Viewport panels: Individual floating windows for 3D/2D visualization
- Parameter panels: Stacked interface for node configuration
- No cross-contamination between panel types

**Reliable Node Tracking**:
- Uses `NodeGraph::add_node()` return value for accurate node IDs
- Eliminates race conditions in node creation and panel assignment
- Ensures panel state is set on the correct node instance

## Workspace System

Workspaces provide context-specific node sets:

```rust
pub trait Workspace {
    fn name(&self) -> &str;
    fn create_workspace_node(&self, node_type: &str, position: Pos2) -> Option<Node>;
    fn get_menu_structure(&self) -> Vec<WorkspaceMenuItem>;
}
```

Built-in workspaces:
- **General**: All nodes available
- **3D**: 3D graphics and geometry
- **USD**: Universal Scene Description workflow
- **MaterialX**: Material authoring

## USD Integration

USD support is provided through PyO3 bindings:

1. **Local Installation**: `scripts/setup_usd.py` installs USD locally
2. **Node Categories**: Stage, Geometry, Transform, Lighting, Shading
3. **Data Flow**: USD operations through Python API, cached for rendering

See [USD_STRATEGY.md](USD_STRATEGY.md) for detailed USD integration documentation.

## Development Workflow

### Adding a New Node

1. Create directory structure:
   ```
   src/nodes/category/my_node/
   ├── mod.rs
   ├── logic.rs
   └── parameters.rs
   ```

2. Implement NodeFactory in `mod.rs`:
   ```rust
   impl NodeFactory for MyNode {
       fn metadata() -> NodeMetadata { ... }
   }
   ```

3. Add computation logic in `logic.rs`
4. Define parameters in `parameters.rs`
5. Register in workspace

### Testing

```bash
# Run all tests
cargo test

# Run with USD features
cargo test --features usd

# Run specific test
cargo test test_node_creation
```

### Performance Profiling

Enable debug overlay with `Ctrl+D` to see:
- FPS and frame time
- Node count
- GPU buffer usage
- Connection count

## Best Practices

1. **Node Design**:
   - Keep nodes focused on single operations
   - Use descriptive port names
   - Provide helpful descriptions

2. **Performance**:
   - Minimize per-frame allocations
   - Batch GPU operations
   - Cache computed values

3. **UI/UX**:
   - Follow established interaction patterns
   - Provide visual feedback
   - Support undo/redo operations

## Contributing

1. Fork the repository
2. Create a feature branch
3. Follow the modular node structure
4. Add tests for new functionality
5. Submit a pull request

## Resources

- [egui Documentation](https://docs.rs/egui)
- [wgpu Documentation](https://docs.rs/wgpu)
- [USD Documentation](https://openusd.org/docs)
- [WGSL Specification](https://www.w3.org/TR/WGSL/)