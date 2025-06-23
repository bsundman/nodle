# Nōdle - Node Editor Developer Guide

## Overview

Nōdle (pronounced like "noodle") is a custom node-based visual programming editor built in Rust using the egui/eframe framework. It implements a vertical flow design where connections flow from top to bottom, with input ports on the top of nodes and output ports on the bottom.

## Project Architecture

### Workspace Structure

Nōdle is organized as a Rust workspace with two main crates:

```
nodle/
├── nodle-core/          # Core library (reusable node graph functionality)
│   ├── src/
│   │   ├── lib.rs       # Public API exports
│   │   ├── graph.rs     # NodeGraph and Connection types
│   │   ├── node.rs      # Node type and implementation
│   │   ├── port.rs      # Port type and enums
│   │   └── math.rs      # Mathematical utilities (bezier curves, etc.)
│   └── Cargo.toml
├── nodle-app/           # GUI application
│   ├── src/
│   │   ├── main.rs      # Entry point + common traits (NodeFactory, etc.)
│   │   ├── editor/      # Main editor implementation
│   │   │   └── mod.rs   # NodeEditor struct and GUI logic
│   │   ├── math/        # Math node implementations
│   │   │   ├── mod.rs   # Re-exports
│   │   │   ├── add.rs   # Addition node
│   │   │   ├── subtract.rs
│   │   │   ├── multiply.rs
│   │   │   └── divide.rs
│   │   ├── logic/       # Logic node implementations
│   │   │   ├── mod.rs
│   │   │   ├── and.rs   # AND gate
│   │   │   ├── or.rs    # OR gate
│   │   │   └── not.rs   # NOT gate
│   │   ├── data/        # Data node implementations
│   │   │   ├── mod.rs
│   │   │   ├── constant.rs # Constant value node
│   │   │   └── variable.rs # Variable storage node
│   │   └── output/      # Output node implementations
│   │       ├── mod.rs
│   │       ├── print.rs # Print to console
│   │       └── debug.rs # Debug with passthrough
│   └── Cargo.toml
├── examples/
│   └── basic_graph.rs   # Usage example for nodle-core
├── Cargo.toml           # Workspace configuration
├── README.md
├── DEVELOPER_GUIDE.md   # This file
└── CLAUDE.md            # Development session memory (git excluded)
```

### Core Library (nodle-core)

The core library provides the fundamental data structures and algorithms for node graphs:

#### Port Structure
```rust
#[derive(Debug, Clone)]
pub struct Port {
    pub id: usize,           // Unique identifier within the node
    pub name: String,        // Display name (e.g., "A", "B", "Result")
    pub port_type: PortType, // Input or Output
    pub position: Pos2,      // World coordinates of the port center
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortType {
    Input,
    Output,
}
```

#### Node Structure
```rust
#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,             // Unique node identifier
    pub title: String,          // Display title
    pub position: Pos2,         // Top-left corner position
    pub size: Vec2,            // Width and height
    pub inputs: Vec<Port>,      // Input ports (positioned on top edge)
    pub outputs: Vec<Port>,     // Output ports (positioned on bottom edge)
    pub color: Color32,        // Node background color
}
```

#### Connection Structure
```rust
#[derive(Debug, Clone)]
pub struct Connection {
    pub from_node: NodeId,      // Source node ID
    pub from_port: usize,       // Source port index
    pub to_node: NodeId,        // Target node ID
    pub to_port: usize,         // Target port index
}
```

#### NodeGraph
```rust
pub struct NodeGraph {
    pub nodes: HashMap<NodeId, Node>,
    pub connections: Vec<Connection>,
    next_id: NodeId,
}
```

### GUI Application (nodle-app)

#### NodeEditor (Main GUI State)
```rust
pub struct NodeEditor {
    graph: NodeGraph,                              // The node graph from core
    dragging_node: Option<NodeId>,                 // Currently dragging node
    drag_offset: Vec2,                             // Drag offset for smooth dragging
    connecting_from: Option<(NodeId, PortId, bool)>, // Connection state (node, port, is_input)
    selected_nodes: HashSet<NodeId>,               // Multi-selection support
    selected_connection: Option<usize>,            // Selected connection index
    context_menu_pos: Option<Pos2>,               // Right-click menu position
    open_submenu: Option<String>,                  // Current submenu name
    submenu_pos: Option<Pos2>,                     // Submenu position
    pan_offset: Vec2,                              // Camera pan offset
    zoom: f32,                                     // Camera zoom level
    box_selection_start: Option<Pos2>,            // Box selection start
    box_selection_end: Option<Pos2>,              // Box selection end
    drag_offsets: HashMap<NodeId, Vec2>,          // Multi-node drag offsets
}
```

## Node Factory System

### NodeFactory Trait

Each node type implements a standardized interface:

```rust
pub trait NodeFactory {
    fn node_type() -> &'static str where Self: Sized;     // "Add", "Subtract", etc.
    fn display_name() -> &'static str where Self: Sized;  // Human-readable name
    fn category() -> NodeCategory where Self: Sized;      // Math, Logic, Data, Output
    fn color() -> Color32 where Self: Sized;             // Node color
    fn create(position: Pos2) -> Node where Self: Sized; // Create instance
    fn add_to_graph(graph: &mut NodeGraph, position: Pos2) -> NodeId where Self: Sized;
}
```

### Node Categories

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeCategory {
    Math,    // Mathematical operations
    Logic,   // Boolean logic gates
    Data,    // Constants and variables
    Output,  // Print and debug nodes
}
```

### Example Node Implementation

```rust
// src/math/add.rs
pub struct AddNode;

impl NodeFactory for AddNode {
    fn node_type() -> &'static str { "Add" }
    fn display_name() -> &'static str { "Add" }
    fn category() -> NodeCategory { NodeCategory::Math }
    fn color() -> Color32 { Color32::from_rgb(160, 170, 160) }
    
    fn create(position: Pos2) -> Node {
        let mut node = Node::new(0, Self::node_type(), position)
            .with_color(Self::color());
        node.add_input("A").add_input("B").add_output("Result");
        node
    }
}
```

### Adding New Node Types

1. Create new file (e.g., `src/math/power.rs`)
2. Implement `NodeFactory` trait
3. Add to category's `mod.rs`:
   ```rust
   mod power;
   pub use power::PowerNode;
   ```
4. Add to `NodeRegistry::create_node()` in `main.rs`:
   ```rust
   "Power" => Some(math::PowerNode::create(position)),
   ```

## Core Algorithms

### Port Positioning

Ports are automatically positioned when nodes are created or moved:

```rust
// Input ports: evenly distributed along top edge
fn update_input_positions(&mut self) {
    let count = self.inputs.len();
    for (i, port) in self.inputs.iter_mut().enumerate() {
        let x_offset = if count == 1 {
            self.size.x / 2.0  // Center single port
        } else {
            (i as f32 * self.size.x) / (count - 1) as f32  // Distribute multiple
        };
        port.position = self.position + Vec2::new(x_offset, 0.0);
    }
}

// Output ports: evenly distributed along bottom edge  
fn update_output_positions(&mut self) {
    let count = self.outputs.len();
    for (i, port) in self.outputs.iter_mut().enumerate() {
        let x_offset = if count == 1 {
            self.size.x / 2.0
        } else {
            (i as f32 * self.size.x) / (count - 1) as f32
        };
        port.position = self.position + Vec2::new(x_offset, self.size.y);
    }
}
```

### Connection Rendering

Connections are drawn as cubic Bézier curves:

```rust
fn draw_connection(painter: &Painter, from: Pos2, to: Pos2, zoom: f32) {
    let vertical_distance = (to.y - from.y).abs();
    let control_offset = if vertical_distance > 10.0 {
        vertical_distance * 0.4  // Proportional to distance
    } else {
        60.0 * zoom  // Minimum offset for short connections
    };
    
    let points = [
        from,                                    // Start point
        from + Vec2::new(0.0, control_offset),   // Control point 1
        to - Vec2::new(0.0, control_offset),     // Control point 2  
        to,                                      // End point
    ];
    
    painter.add(egui::Shape::CubicBezier(egui::epaint::CubicBezierShape {
        points,
        closed: false,
        fill: Color32::TRANSPARENT,
        stroke: Stroke::new(2.0 * zoom, Color32::from_rgb(150, 150, 150)),
    }));
}
```

### Coordinate Transforms

The editor uses two coordinate systems:

```rust
// World coordinates: logical space where nodes exist
// Screen coordinates: visual space after pan/zoom transforms

fn transform_pos(world_pos: Pos2, pan: Vec2, zoom: f32) -> Pos2 {
    Pos2::new(
        world_pos.x * zoom + pan.x,
        world_pos.y * zoom + pan.y
    )
}

fn inverse_transform_pos(screen_pos: Pos2, pan: Vec2, zoom: f32) -> Pos2 {
    Pos2::new(
        (screen_pos.x - pan.x) / zoom,
        (screen_pos.y - pan.y) / zoom
    )
}
```

## User Interface

### Input Handling

#### Mouse Controls
- **Left Click**: Select nodes, click ports to connect
- **Left Drag**: Move selected nodes, box selection on empty space
- **Middle Drag**: Pan the camera
- **Right Click**: Context menu
- **Scroll Wheel**: Zoom (centered at mouse cursor)

#### Keyboard Controls
- **Delete**: Remove selected nodes and connections
- **Escape**: Cancel connection in progress, close menus
- **Ctrl/Cmd + Click**: Multi-select nodes

### Connection System

The connection system supports two interaction modes:

#### Click-to-Click Mode
1. Click an output port → connection preview appears
2. Click an input port → connection created
3. ESC to cancel

#### Drag Mode
1. Click and drag from any port
2. Release on compatible port → connection created
3. Release elsewhere → connection cancelled

### Context Menu System

The context menu uses a hierarchical structure:

#### Main Menu
- **Math** ▶
- **Logic** ▶  
- **Data** ▶
- **Output** ▶

#### Submenus
Each category opens a submenu with specific node types:
- **Math**: Add, Subtract, Multiply, Divide
- **Logic**: AND, OR, NOT
- **Data**: Constant, Variable
- **Output**: Print, Debug

#### Implementation Details
- Custom rendering using `allocate_exact_size()` for full-width hover areas
- No rounded corners (rectangular highlighting)
- Arrow indicators (▶) show submenu availability
- Smart positioning and closing behavior

## Visual Design

### Color Scheme (Dark Theme)

```rust
// Background
const BACKGROUND: Color32 = Color32::from_rgb(40, 40, 40);

// Node colors (light grey with category tints)
const MATH_COLOR: Color32 = Color32::from_rgb(160, 170, 160);    // Green tint
const LOGIC_COLOR: Color32 = Color32::from_rgb(160, 160, 170);   // Blue tint  
const DATA_COLOR: Color32 = Color32::from_rgb(170, 160, 170);    // Purple tint
const OUTPUT_COLOR: Color32 = Color32::from_rgb(170, 160, 160);  // Red tint

// UI elements
const SELECTED_BORDER: Color32 = Color32::from_rgb(255, 200, 100); // Orange
const CONNECTION_COLOR: Color32 = Color32::from_rgb(150, 150, 150); // Grey
const PREVIEW_COLOR: Color32 = Color32::from_rgb(255, 255, 100);    // Yellow
```

### Node Dimensions
- **Width**: 150 pixels
- **Height**: 30 pixels (compact design)
- **Port Radius**: 5 pixels
- **Port Spacing**: Evenly distributed along edges

## Building and Running

### Development
```bash
# Run the application
cargo run -p nodle

# Run with release optimizations
cargo run -p nodle --release

# Run core library tests
cargo test -p nodle-core

# Run example
cargo run --example basic_graph
```

### Project Setup
```bash
# Clone repository
git clone https://github.com/bsundman/nodle.git
cd nodle

# Build entire workspace
cargo build

# Check for issues
cargo check
cargo clippy
```

## Architecture Decisions

### Why Workspace Structure?
- **Separation of Concerns**: Core logic separate from GUI
- **Reusability**: Other applications can use `nodle-core`
- **Testing**: Core can be tested independently
- **Documentation**: Clear API boundaries

### Why Individual Node Files?
- **Maintainability**: Each node type is self-contained
- **Extensibility**: Easy to add new nodes without touching existing code
- **Testing**: Each node can have its own tests
- **Collaboration**: Multiple developers can work on different nodes

### Why Trait-Based Factory Pattern?
- **Consistency**: All nodes follow the same interface
- **Type Safety**: Compile-time guarantees
- **Flexibility**: Easy to extend with new functionality
- **Performance**: Zero-cost abstractions

### Why Custom Menu Rendering?
- **User Experience**: Full-width hover areas feel more natural
- **Visual Design**: No rounded corners for clean appearance
- **Control**: Complete control over interaction behavior
- **Performance**: Minimal allocations during rendering

## Future Enhancements

### Execution Engine
- Add ability to execute node graphs
- Type system for port compatibility
- Value propagation through connections
- Real-time execution visualization

### Serialization
- Save/load node graphs to files
- JSON or binary format
- Version compatibility
- Import/export functionality

### Plugin System
- Dynamic node loading
- Lua or WASM scripting
- Custom node categories
- Third-party node packages

### Advanced Features
- Undo/redo system
- Minimap for large graphs
- Node grouping/comments
- Performance profiling
- Connection routing algorithms

## Contributing

### Code Style
- Follow Rust conventions (rustfmt, clippy)
- Add documentation for public APIs
- Include tests for new functionality
- Keep commits focused and atomic

### Adding Features
1. Discuss design in issues first
2. Create feature branch
3. Implement with tests
4. Update documentation
5. Submit pull request

This guide covers the essential architecture and patterns needed to understand, maintain, and extend the Nōdle node editor.