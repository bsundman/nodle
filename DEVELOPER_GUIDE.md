# Nodle - Node Editor Developer Guide

## Overview

Nodle is a custom node-based visual programming editor built in Rust using the egui/eframe framework. It implements a vertical flow design where connections flow from top to bottom, with input ports on the top of nodes and output ports on the bottom.

## Architecture

### Core Components

#### 1. Port Structure
```rust
#[derive(Debug, Clone)]
struct Port {
    id: usize,           // Unique identifier within the node
    name: String,        // Display name (e.g., "A", "B", "Result")
    is_input: bool,      // true for input ports, false for output ports
    position: Pos2,      // World coordinates of the port center
}
```

#### 2. Node Structure
```rust
#[derive(Debug, Clone)]  
struct Node {
    id: usize,                // Unique node identifier
    title: String,            // Display title
    position: Pos2,           // Top-left corner position
    size: Vec2,              // Width and height (width=150, height=30)
    inputs: Vec<Port>,        // Input ports (positioned on top edge)
    outputs: Vec<Port>,       // Output ports (positioned on bottom edge)
    color: Color32,          // Node background color
}
```

#### 3. Connection Structure
```rust
#[derive(Debug, Clone)]
struct Connection {
    from_node: usize,     // Source node ID
    from_port: usize,     // Source port index
    to_node: usize,       // Target node ID  
    to_port: usize,       // Target port index
}
```

#### 4. NodeEditor (Main State)
```rust
struct NodeEditor {
    nodes: HashMap<usize, Node>,           // All nodes by ID
    connections: Vec<Connection>,          // All connections
    next_node_id: usize,                  // ID counter for new nodes
    connecting_from: Option<(usize, usize, bool)>, // Active connection: (node_id, port_id, is_input)
    selected_nodes: HashSet<usize>,       // Currently selected nodes
    selected_connection: Option<usize>,   // Currently selected connection index
    context_menu_pos: Option<Pos2>,       // Right-click menu position
    pan_offset: Vec2,                     // Camera pan offset
    zoom: f32,                           // Camera zoom level (0.1 to 5.0)
    box_selection_start: Option<Pos2>,    // Box selection start point
    box_selection_end: Option<Pos2>,      // Box selection end point
    drag_offsets: HashMap<usize, Vec2>,   // Node drag offsets
}
```

## Key Systems

### 1. Coordinate System & Transforms

The editor uses two coordinate systems:
- **World Coordinates**: The logical space where nodes exist
- **Screen Coordinates**: The rendered space after pan/zoom transforms

```rust
// Transform world coordinates to screen coordinates
let transform_pos = |pos: Pos2| -> Pos2 {
    Pos2::new(
        pos.x * zoom + pan_offset.x,
        pos.y * zoom + pan_offset.y,
    )
};

// Transform screen coordinates to world coordinates  
let inverse_transform_pos = |pos: Pos2| -> Pos2 {
    Pos2::new(
        (pos.x - pan_offset.x) / zoom,
        (pos.y - pan_offset.y) / zoom,
    )
};
```

### 2. Port Positioning (Vertical Flow)

Ports are positioned automatically when nodes are updated:

```rust
fn update_port_positions(&mut self) {
    let port_spacing = 30.0;
    
    // Input ports on TOP edge (y = 0)
    let input_start_x = if self.inputs.len() > 1 {
        (self.size.x - (self.inputs.len() - 1) as f32 * port_spacing) / 2.0
    } else {
        self.size.x / 2.0  // Center single port
    };
    
    for (i, input) in self.inputs.iter_mut().enumerate() {
        input.position = self.position + Vec2::new(
            input_start_x + i as f32 * port_spacing, 
            0.0  // Top edge
        );
    }
    
    // Output ports on BOTTOM edge (y = node_height)
    let output_start_x = if self.outputs.len() > 1 {
        (self.size.x - (self.outputs.len() - 1) as f32 * port_spacing) / 2.0
    } else {
        self.size.x / 2.0  // Center single port
    };
    
    for (i, output) in self.outputs.iter_mut().enumerate() {
        output.position = self.position + Vec2::new(
            output_start_x + i as f32 * port_spacing,
            self.size.y  // Bottom edge
        );
    }
}
```

### 3. Connection System

#### Single-Click Connection Workflow:
1. **First Click**: Click on any port to start connection (sets `connecting_from`)
2. **Second Click**: Click on compatible port to complete connection
3. **ESC Key**: Cancel connection in progress

#### Connection Compatibility:
- **Output → Input**: Valid connection
- **Input → Output**: Valid connection (automatically reverses to output→input)
- **Same Node**: Invalid (prevented)
- **Same Port Type**: Invalid (input→input, output→output)

#### Connection Rendering:
Connections use cubic Bézier curves for smooth vertical flow:

```rust
// Calculate control points for vertical bezier curve
let vertical_distance = (to_pos.y - from_pos.y).abs();
let control_offset = if vertical_distance > 10.0 {
    vertical_distance * 0.4  // Proportional curve
} else {
    60.0 * zoom  // Minimum curve for short connections
};

let points = [
    from_pos,                                    // Start point
    from_pos + Vec2::new(0.0, control_offset),   // First control point (down)
    to_pos - Vec2::new(0.0, control_offset),     // Second control point (up from target)
    to_pos,                                      // End point
];
```

### 4. Input Handling

#### Mouse Controls:
- **Left Click**: Select nodes/ports, complete connections
- **Left Click + Drag**: Move selected nodes, box selection
- **Middle Mouse + Drag**: Pan camera
- **Right Click**: Context menu (on empty space)
- **Mouse Wheel**: Zoom (centered on cursor)

#### Keyboard Controls:
- **Delete**: Delete selected nodes/connections
- **ESC**: Cancel connection in progress
- **Ctrl/Cmd + Click**: Multi-select nodes

#### Event Processing Order:
1. **Right-Click Context Menu**: Handled first to prevent interference
2. **Pan/Zoom**: Middle mouse and wheel events
3. **Port Interactions**: Click/drag on ports for connections
4. **Node Selection/Dragging**: Click/drag on nodes
5. **Box Selection**: Click/drag on empty space
6. **Keyboard Input**: Delete and ESC keys

### 5. Node Types & Colors

The editor supports predefined node types with specific colors:

```rust
match node_type {
    // Math nodes - Green (80, 120, 80)
    "Add" | "Subtract" | "Multiply" | "Divide" => {
        node.add_input("A").add_input("B").add_output("Result");
        node.color = Color32::from_rgb(80, 120, 80);
    }
    
    // Logic nodes - Blue (80, 80, 120)  
    "AND" | "OR" => {
        node.add_input("A").add_input("B").add_output("Result");
        node.color = Color32::from_rgb(80, 80, 120);
    }
    "NOT" => {
        node.add_input("Input").add_output("Result");
        node.color = Color32::from_rgb(80, 80, 120);
    }
    
    // Data nodes - Purple (120, 80, 120)
    "Constant" => {
        node.add_output("Value");
        node.color = Color32::from_rgb(120, 80, 120);
    }
    "Variable" => {
        node.add_input("Set").add_output("Get");
        node.color = Color32::from_rgb(120, 80, 120);
    }
    
    // Output nodes - Red (120, 80, 80)
    "Print" => {
        node.add_input("Value");
        node.color = Color32::from_rgb(120, 80, 80);
    }
    "Debug" => {
        node.add_input("Value").add_output("Pass");
        node.color = Color32::from_rgb(120, 80, 80);
    }
}
```

## Rendering Pipeline

### 1. Coordinate Transformation
All world coordinates are transformed to screen coordinates using pan/zoom.

### 2. Connection Rendering
- Connections rendered first (behind nodes)
- Cubic Bézier curves for smooth vertical flow
- Selected connections highlighted in orange
- Connection preview shown in bright yellow during creation

### 3. Node Rendering
- Node background with rounded corners
- Border highlighting for selected nodes (orange)
- Title text centered horizontally, 15 pixels from top
- Port circles with different colors:
  - Input ports: Green (100, 150, 100)
  - Output ports: Red (150, 100, 100)
- Port labels positioned above inputs, below outputs

### 4. UI Overlays
- Box selection rectangle (blue with transparency)
- Context menu (popup with node creation options)
- Connection preview during creation

## Performance Considerations

### Efficient Rendering
- Uses egui's retained mode GUI system
- Transform calculations cached during render loop
- Connection hit-testing uses line segment approximation
- Port interaction uses simple distance checks (< 10 pixels)

### Memory Management
- Nodes stored in HashMap for O(1) lookup by ID
- Connections stored in Vec for sequential processing
- Selected nodes stored in HashSet for O(1) membership testing

## Development Patterns

### Adding New Node Types

1. **Add to match statement** in `create_node()`:
```rust
"NewNodeType" => {
    node.add_input("Input1").add_output("Output1");
    node.color = Color32::from_rgb(r, g, b);
}
```

2. **Add to context menu** in the UI rendering section:
```rust
if ui.button("NewNodeType").clicked() {
    self.create_node("NewNodeType", menu_world_pos);
    self.context_menu_pos = None;
}
```

### Modifying Interaction Behavior

Input handling follows a clear priority order. To modify:

1. **High Priority**: Modify context menu handling
2. **Medium Priority**: Modify pan/zoom in main event loop
3. **Low Priority**: Modify node/port interactions

### Debugging Tips

1. **Print coordinates**: Add debug prints to understand world vs screen coords
2. **Visual debugging**: Draw debug rectangles/circles for hit-testing areas
3. **State inspection**: Print `connecting_from` state to debug connection issues
4. **Transform testing**: Verify coordinate transformations with known points

## Common Issues & Solutions

### Issue: Connections not appearing
**Solution**: Check that port positions are updated after node movement via `update_port_positions()`

### Issue: Incorrect click detection
**Solution**: Ensure using correct coordinate system (world vs screen) for hit-testing

### Issue: Pan/zoom feeling wrong
**Solution**: Verify `zoom_at_point()` correctly transforms mouse position to world coordinates before applying zoom

### Issue: Node selection not working
**Solution**: Check event processing order - ensure port clicks don't interfere with node selection

## Future Extension Points

### 1. Node Persistence
Add serialization/deserialization for saving/loading node graphs:
```rust
#[derive(Serialize, Deserialize)]
struct NodeGraph {
    nodes: HashMap<usize, Node>,
    connections: Vec<Connection>,
}
```

### 2. Node Execution Engine
Add a system to actually execute the node graph logic:
```rust
trait NodeExecutor {
    fn execute(&self, inputs: &[Value]) -> Vec<Value>;
}
```

### 3. Custom Node Types
Allow runtime definition of new node types through a plugin system.

### 4. Minimap
Add a minimap for large node graphs:
- Render entire graph at small scale
- Show current viewport bounds
- Allow click-to-navigate

### 5. Undo/Redo System
Implement command pattern for undoable operations:
```rust
trait Command {
    fn execute(&mut self, editor: &mut NodeEditor);
    fn undo(&mut self, editor: &mut NodeEditor);
}
```

## Dependencies

- **eframe 0.29**: Main GUI framework and application runner
- **egui 0.29**: Immediate mode GUI library
- **std::collections**: HashMap and HashSet for efficient data structures

## Build & Run

```bash
# Development build
cargo run

# Release build  
cargo build --release
cargo run --release
```

## Testing

The application includes test nodes created automatically in `add_test_nodes()`. These demonstrate:
- Different node types and colors
- Various port configurations
- Pre-made connections showing the connection system

## Code Style

- **Structs**: PascalCase (`NodeEditor`, `Connection`)
- **Functions**: snake_case (`create_node`, `update_port_positions`)  
- **Constants**: SCREAMING_SNAKE_CASE (not many used)
- **Variables**: snake_case (`connecting_from`, `selected_nodes`)
- **Comments**: Explain the "why" not the "what"
- **Error Handling**: Currently minimal, could be expanded

This guide covers the essential architecture and patterns needed to understand, maintain, and extend the Nodle node editor.