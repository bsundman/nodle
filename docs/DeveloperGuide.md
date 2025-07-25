# Nōdle - Node Editor Developer Guide

## Overview

Nōdle (pronounced like "noodle") is a high-performance node-based visual programming editor built in Rust using the egui/eframe framework. It implements a vertical flow design with GPU-accelerated rendering and an extensible plugin architecture supporting unlimited node types and specialized workspaces.

## Recent Major Updates

### Plugin SDK Modernization (2024)
- **Complete NodeData System**: Plugin SDK now matches main application's rich type system with Scene, Geometry, Material, USD, Light, and Image data types
- **Advanced Caching**: Multi-stage caching system with strategies for simple and complex plugins (like USD File Reader pattern)
- **Execution Hooks**: NodeExecutionHooks system for plugin lifecycle management and smart cache invalidation
- **Rich UI Components**: Comprehensive InterfaceParameter system with all modern UI elements
- **USD Integration**: Full USD data structures available to plugins for 3D scene processing
- **Performance Optimization**: Unified cache system exposed to plugins for high-performance operations

### Architecture Improvements
- **Manual Cook Button Fix**: Now correctly executes nodes in current workspace rather than root graph
- **Pattern Matching**: All enum variants properly handled across plugin interface
- **Type Safety**: Complete type conversion system between core and plugin data types

## 🌟 Node-Centric Architecture

Nōdle follows a pure node-centric philosophy where **"nodes are the star of the show"**. Everything in the system is driven by node metadata - from visual appearance to interface panel behavior. This approach ensures consistency, extensibility, and maintainability across the entire codebase.

## Project Structure

```
nodle/
├── src/
│   ├── main.rs                 # Application entry point
│   ├── constants.rs            # Hard-coded constants (extracted from all modules)
│   ├── editor/                 # Editor components
│   │   ├── mod.rs             # Main NodeEditor struct (1658 lines - needs refactoring)
│   │   ├── input.rs           # Input handling & event management
│   │   ├── canvas.rs          # Canvas viewport management (pan/zoom)
│   │   ├── canvas_rendering.rs # Node/connection rendering logic
│   │   ├── interaction.rs     # Node interaction (selection, dragging)
│   │   ├── menus.rs           # Context menu system
│   │   ├── navigation.rs      # Navigation controls & workspace management
│   │   ├── file_manager.rs    # File operations
│   │   ├── panels/            # Panel management
│   │   │   ├── mod.rs         # Unified panel manager with NodePanelState
│   │   │   ├── parameter.rs   # Parameter panel rendering
│   │   │   └── viewport.rs    # 3D viewport panel rendering
│   │   ├── debug_tools.rs     # Debug utilities & performance overlay
│   │   └── workspace_builder.rs # Workspace construction helpers
│   ├── gpu/                    # GPU rendering system
│   │   ├── mod.rs             # GPU module exports (cleaned of unused re-exports)
│   │   ├── config.rs          # Graphics configuration
│   │   ├── canvas_instance.rs # Canvas instance data structures
│   │   ├── canvas_rendering.rs # Core GPU canvas renderer
│   │   ├── canvas_callback.rs # egui paint callback integration
│   │   ├── viewport_3d_rendering.rs # 3D viewport renderer
│   │   ├── viewport_3d_callback.rs # egui 3D viewport integration
│   │   └── shaders/           # WGSL shaders
│   │       ├── node.wgsl      # Node body rendering
│   │       ├── port.wgsl      # Port rendering
│   │       ├── button.wgsl    # Button rendering
│   │       ├── flag.wgsl      # Visibility flag rendering
│   │       ├── mesh3d.wgsl    # 3D mesh rendering
│   │       ├── grid3d.wgsl    # 3D grid rendering
│   │       ├── wireframe3d.wgsl # 3D wireframe rendering
│   │       └── axis_gizmo.wgsl # 3D axis gizmo rendering
│   ├── nodes/                  # Node system
│   │   ├── mod.rs             # Core node types (cleaned of unused re-exports)
│   │   ├── defaults.rs        # Default node values
│   │   ├── factory.rs         # Node factory & registry
│   │   ├── graph.rs           # NodeGraph implementation
│   │   ├── node.rs            # Node struct & methods
│   │   ├── port.rs            # Port types & logic
│   │   ├── interface.rs       # Interface traits (cleaned of unused macros)
│   │   ├── math_utils.rs      # Mathematical utilities
│   │   ├── math/              # Math nodes (modular structure)
│   │   │   ├── mod.rs         # Math module exports (cleaned)
│   │   │   ├── add/           # Addition node
│   │   │   │   ├── mod.rs     # NodeFactory implementation
│   │   │   │   ├── functions.rs # Helper functions
│   │   │   │   └── parameters.rs # UI parameters
│   │   │   ├── subtract/      # Subtraction node
│   │   │   ├── multiply/      # Multiplication node
│   │   │   └── divide/        # Division node
│   │   ├── logic/             # Logic nodes (AND, OR, NOT)
│   │   │   ├── mod.rs         # Logic module exports (cleaned)
│   │   │   ├── and/           # AND logic node
│   │   │   ├── or/            # OR logic node
│   │   │   └── not/           # NOT logic node
│   │   ├── data/              # Data nodes (Constant, Variable)
│   │   │   ├── mod.rs         # Data module exports (cleaned)
│   │   │   ├── constant/      # Constant value node
│   │   │   └── variable/      # Variable storage node
│   │   ├── output/            # Output nodes (Print, Debug)
│   │   │   ├── mod.rs         # Output module exports
│   │   │   ├── print/         # Print node (cleaned)
│   │   │   └── debug/         # Debug node (cleaned)
│   │   ├── utility/           # Utility nodes
│   │   │   ├── mod.rs         # Utility module exports
│   │   │   └── null/          # Null/passthrough node
│   │   ├── three_d/           # 3D nodes
│   │   │   ├── mod.rs         # 3D module exports
│   │   │   ├── geometry/      # Geometry primitives (cube, sphere, plane)
│   │   │   ├── transform/     # Transform operations (translate, rotate, scale)
│   │   │   ├── lighting/      # Light sources (point, directional, spot)
│   │   │   └── output/        # 3D viewport
│   │   │       └── viewport/  # Viewport node (cleaned of orphaned logic.rs)
│   │   └── materialx/         # MaterialX nodes
│   │       ├── mod.rs         # MaterialX module exports
│   │       ├── math.rs        # MaterialX math operations
│   │       ├── shading.rs     # MaterialX shading nodes
│   │       ├── textures.rs    # MaterialX texture nodes
│   │       └── utilities.rs   # MaterialX utility nodes (cleaned imports)
│   ├── workspaces/            # Workspace implementations
│   │   ├── mod.rs             # Workspace registry (cleaned)
│   │   ├── base.rs            # Base workspace functionality
│   │   ├── registry.rs        # Workspace registration
│   │   ├── workspace_2d.rs    # 2D workspace
│   │   ├── workspace_3d.rs    # 3D workspace (cleaned imports)
│   │   └── materialx.rs       # MaterialX workspace
│   ├── plugins/               # Plugin system
│   │   └── mod.rs             # Plugin loading and management
│   ├── workspace.rs           # Workspace trait definition
│   ├── menu_hierarchy.rs      # Centralized menu organization
│   ├── theme.rs               # UI theming
│   └── startup_checks.rs      # Dependency verification
├── plugins/                   # Plugin binaries (excluded from git)
├── vendor/                    # Third-party dependencies
│   └── python_runtime/        # Python runtime for USD
├── Cargo.toml                 # Project manifest
└── .gitignore                 # Git ignore rules (updated for plugin binaries)
```

## 🔧 Major Cleanup Completed

The codebase has undergone comprehensive cleanup:

### Removed Systems
- **Entire contexts system** (~533 lines) - was duplicate of workspaces
- **18+ unused import statements** and re-exports
- **3 unused macro definitions** (interface_float!, interface_vector3!, interface_enum!)
- **Unused utility functions** and orphaned test files
- **Install scripts** and setup scripts (now handled by plugins)

### Consolidated Systems
- **11 HashMaps → Single NodePanelState struct** in InterfacePanelManager
- **Hard-coded constants → constants.rs module**
- **debug println! → proper logging framework**
- **Standardized error handling patterns**

### Current Technical Debt
- **editor/mod.rs**: 1658 lines (target: ~1000 lines) - needs STRATEGY.md refactoring
- **Unused wildcard imports** in some modules
- **Deprecation warnings** for egui API usage

## Core Architecture

### Plugin System

Nōdle uses a comprehensive plugin architecture via the **nodle-plugin-sdk**:

```rust
// Plugin trait
pub trait NodePlugin: Send + Sync {
    fn plugin_info(&self) -> PluginInfo;
    fn register_nodes(&self, registry: &mut dyn NodeRegistryTrait);
    fn get_menu_structure(&self) -> Vec<MenuStructure>;
    fn on_load(&self) -> Result<(), PluginError>;
    fn on_unload(&self) -> Result<(), PluginError>;
}

// Node factory trait
pub trait NodeFactory: Send + Sync {
    fn metadata(&self) -> NodeMetadata;
    fn create_node(&self, position: egui::Pos2) -> Box<dyn PluginNode>;
}
```

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
    pub node_type: String,
    pub display_name: String,
    pub description: String,
    pub version: String,
    
    // Visual appearance
    pub color: Color32,
    pub icon: String,
    pub size_hint: Vec2,
    
    // Organization & categorization
    pub category: NodeCategory,
    pub workspace_compatibility: Vec<String>,
    pub tags: Vec<String>,
    
    // Interface behavior
    pub panel_type: PanelType,
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
impl NodeFactory for AddNodeFactory {
    fn metadata() -> NodeMetadata {
        NodeMetadata::new("Add", "Add", NodeCategory::math(), "Adds two values")
            .with_color(Color32::from_rgb(80, 160, 80))
            .with_icon("➕")
            .with_inputs(vec![
                PortDefinition::required("A", DataType::Float),
                PortDefinition::required("B", DataType::Float),
            ])
            .with_outputs(vec![
                PortDefinition::required("Result", DataType::Float),
            ])
            .with_processing_cost(ProcessingCost::Minimal)
    }
}
```

## Panel System Architecture

### NodePanelState Consolidation

The panel system uses a unified state structure:

```rust
pub struct NodePanelState {
    pub visible: bool,
    pub minimized: bool,
    pub open: bool,
    pub stacked: bool,
    pub pinned: bool,
    pub panel_type: Option<PanelType>,
    pub position: Option<Pos2>,
    pub size: Option<Vec2>,
    pub auto_managed: bool,
    pub viewport_data: Option<ViewportData>,
}
```

### Panel Types and Behavior

- **Parameter**: Node configuration panels that stack together by default (top-right area)
- **Viewport**: 3D/2D visualization panels that remain separate and floating
- **Combined**: Both parameter and viewport functionality

### Automatic Panel Management

**Panel Visibility**:
- Panels automatically appear when nodes are created
- Panel visibility is managed through the unified NodePanelState
- Reliable node ID detection ensures proper panel assignment

**Panel Stacking**:
- Parameter panels stack together by default for efficient screen space usage
- Viewport panels remain separate and floating to prevent interference
- Panel types are completely isolated - viewport and parameter panels never stack together

## GPU Rendering Pipeline

Nōdle uses wgpu for high-performance rendering:

1. **Instance Collection**: Nodes and connections are collected into instance buffers
2. **GPU Upload**: Instance data is uploaded to GPU buffers via canvas_instance.rs
3. **Shader Rendering**: WGSL shaders render nodes, ports, and connections
4. **egui Integration**: GPU content is composited with egui UI via canvas_callback.rs

### Shader System

- `node.wgsl`: Renders node bodies with borders and fills
- `port.wgsl`: Renders input/output ports with type-specific colors
- `button.wgsl`: Renders interactive buttons
- `flag.wgsl`: Renders visibility toggle flags
- `mesh3d.wgsl`: Renders 3D geometry in viewport
- `grid3d.wgsl`: Renders 3D grid overlay
- `wireframe3d.wgsl`: Renders wireframe mode
- `axis_gizmo.wgsl`: Renders 3D axis orientation gizmo

## Plugin SDK Features

The **nodle-plugin-sdk** provides comprehensive plugin development capabilities:

### Core Plugin Development
- **Plugin Interface**: Complete plugin lifecycle management
- **Node Factory Pattern**: Type-safe node creation
- **Rich Metadata System**: Comprehensive node behavior definition
- **Menu Integration**: Custom menu structures

### Data Types & Connectivity
- **Typed Port System**: Float, Vector3, Color, String, Boolean, Any
- **Type-safe Connections**: Automatic connection validation
- **Color-coded Ports**: Visual type identification

### 3D Viewport Support
- **Scene Data Interface**: Complete 3D scene representation
  - Mesh data (vertices, normals, UVs, indices)
  - Material data (PBR materials, textures)
  - Light data (directional, point, spot, area lights)
  - Camera data (position, target, FOV, clipping planes)
- **Viewport Settings**: Rendering modes, background, grid, lighting
- **Camera Manipulation**: Orbit, pan, zoom, reset operations
- **Real-time Updates**: Scene dirty flags for efficient rendering

### UI & Interface
- **Parameter Interface**: Custom node UIs via render_parameters()
- **Panel Management**: Flexible panel positioning and stacking
- **egui Integration**: Direct access to egui for UI rendering

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
- **2D**: 2D graphics and general purpose nodes
- **3D**: 3D graphics, geometry, transforms, and lighting
- **MaterialX**: Material authoring and shading workflows

## Plugin Integration (USD Example)

USD support is provided through a dedicated plugin system:

1. **Plugin Loading**: Dynamic loading of USD plugin binary
2. **Node Registration**: Comprehensive USD node types
3. **Python Integration**: USD operations through PyO3/Python API
4. **Data Caching**: USD data cached for real-time viewport rendering

### USD Plugin Nodes
- **Stage Management**: Create, load, save USD stages
- **Geometry**: USD primitives (sphere, cube, mesh, etc.)
- **Transform**: USD transform operations
- **Lighting**: USD light sources
- **Shading**: USD material and shader nodes
- **Viewport**: USD-specific 3D viewport with stage rendering

## Development Workflow

### Adding a New Core Node

1. Create directory structure:
   ```
   src/nodes/category/my_node/
   ├── mod.rs
   ├── logic.rs
   └── parameters.rs
   ```

2. Implement NodeFactory in `mod.rs`:
   ```rust
   impl NodeFactory for MyNodeFactory {
       fn metadata() -> NodeMetadata {
           NodeMetadata::new("MyNode", "My Node", NodeCategory::utility(), "Description")
               .with_color(Color32::from_rgb(100, 150, 200))
               .with_icon("🔧")
               .with_inputs(vec![...])
               .with_outputs(vec![...])
       }
   }
   ```

3. Add computation logic in `logic.rs`
4. Define parameters UI in `parameters.rs`
5. Register in factory.rs and appropriate workspace

### Creating a Plugin

1. Use the plugin template:
   ```rust
   use nodle_plugin_sdk::*;
   
   pub struct MyPlugin;
   
   impl NodePlugin for MyPlugin {
       fn plugin_info(&self) -> PluginInfo { ... }
       fn register_nodes(&self, registry: &mut dyn NodeRegistryTrait) { ... }
   }
   
   #[no_mangle]
   pub extern "C" fn get_plugin() -> *mut dyn NodePlugin {
       Box::into_raw(Box::new(MyPlugin))
   }
   ```

2. Build as dynamic library (cdylib)
3. Place in plugins/ directory
4. Plugin automatically loaded on startup

### Testing

```bash
# Run all tests
cargo test

# Run with features
cargo test --features gpu

# Run specific test
cargo test test_node_creation

# Check for compilation issues
cargo check

# Run application
cargo run
```

### Performance Profiling

Enable debug overlay with `Ctrl+D` to see:
- FPS and frame time
- Node count and GPU buffer usage
- Connection count
- Memory usage
- Viewport rendering statistics

## Code Quality Standards

### Logging
- Use `log` crate with proper levels (error!, warn!, info!, debug!, trace!)
- No debug println! statements in production code
- Structured logging for complex operations

### Error Handling
- Use Result types for fallible operations
- Proper error propagation with context
- Avoid unwrap() in production code - use proper error handling

### Constants
- All magic numbers extracted to constants.rs
- Grouped by functionality (panel, viewport, canvas, etc.)
- Well-documented constant purposes

### Code Organization
- Follow modular structure consistently
- Single responsibility principle for modules
- Clear separation between logic, UI, and metadata

### Performance
- Minimize per-frame allocations
- Batch GPU operations efficiently
- Cache computed values when appropriate
- Use dirty flags for selective updates

## Architecture Patterns

### Centralized State Management
- NodePanelState consolidates all panel-related state
- Single source of truth for panel behavior
- Eliminates state synchronization issues

### Node-Centric Design
- Everything driven by NodeMetadata
- Consistent node behavior across the system
- Easy extensibility through factory pattern

### Plugin Architecture
- Clean separation between core and plugins
- Type-safe plugin interfaces
- Dynamic loading with version compatibility

### GPU-First Rendering
- Instanced rendering for performance
- Minimal CPU-GPU roundtrips
- Cached data with dirty flags

## Future Improvements

### Immediate (STRATEGY.md Implementation)
1. **Graph Access Unification** (~100 lines saved)
   - Eliminate repetitive `match current_view()` patterns
   - Create helper methods in ViewManager

2. **Button Click Handler Extraction** (~80 lines saved)
   - Extract inline button handling to InteractionManager

3. **Connection Rendering Consolidation** (~120 lines saved)
   - Unify CPU and GPU connection rendering logic

4. **View-Aware Operations** (~150 lines saved)
   - Create operation dispatcher in ViewManager

5. **Input State Processing** (~50 lines saved)
   - Extract input chains to InputState

### Long-term
- Advanced plugin capabilities (custom UI widgets, render passes)
- Multi-threaded node execution
- Advanced undo/redo system
- Collaborative editing features
- WebAssembly plugin support

## Best Practices

### Node Design
- Keep nodes focused on single operations
- Use descriptive port names and types
- Provide helpful descriptions and tooltips
- Follow consistent visual design (colors, icons)

### Plugin Development
- Implement comprehensive error handling
- Provide detailed plugin metadata
- Test plugin loading/unloading thoroughly
- Document plugin APIs clearly

### Performance
- Profile before optimizing
- Use dirty flags to avoid unnecessary updates
- Batch operations when possible
- Minimize allocations in hot paths

### UI/UX
- Follow established interaction patterns
- Provide immediate visual feedback
- Support keyboard shortcuts
- Maintain consistency across workspaces

## Contributing

1. Fork the repository
2. Create a feature branch
3. Follow the established code patterns and quality standards
4. Add tests for new functionality
5. Update documentation as needed
6. Submit a pull request with clear description

### Code Review Checklist
- [ ] Follows modular node structure
- [ ] Uses proper error handling
- [ ] No debug println! statements
- [ ] Constants extracted appropriately
- [ ] Comprehensive metadata provided
- [ ] Tests added for new functionality
- [ ] Documentation updated

## Resources

### Core Technologies
- [egui Documentation](https://docs.rs/egui) - Immediate mode GUI framework
- [wgpu Documentation](https://docs.rs/wgpu) - Safe Rust graphics API
- [WGSL Specification](https://www.w3.org/TR/WGSL/) - WebGPU Shading Language

### Plugin Development
- [nodle-plugin-sdk Documentation](../nodle-plugin-sdk/README.md)
- [libloading Documentation](https://docs.rs/libloading) - Dynamic library loading
- [serde Documentation](https://docs.rs/serde) - Serialization framework

### External Integrations
- [USD Documentation](https://openusd.org/docs) - Universal Scene Description
- [PyO3 Documentation](https://docs.rs/pyo3) - Python bindings for Rust
- [MaterialX Documentation](https://materialx.org/) - Material specification

### Development Tools
- [Rust Book](https://doc.rust-lang.org/book/) - Official Rust documentation
- [Cargo Book](https://doc.rust-lang.org/cargo/) - Cargo package manager
- [rustfmt](https://github.com/rust-lang/rustfmt) - Code formatting tool
- [clippy](https://github.com/rust-lang/rust-clippy) - Rust linter