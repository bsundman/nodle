# Nodle

A node-based visual programming editor built with Rust and egui.

This project is structured as a workspace containing:
- `nodle-core`: Core node graph library (can be used independently)
- `nodle-app`: Full visual editor application

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)

## Overview

Nodle is a custom node editor implementation featuring a vertical flow design where connections flow from top to bottom. It provides an intuitive interface for creating and connecting nodes in a visual programming environment.

## Features

- **Vertical Flow Design**: Input ports on top, output ports on bottom
- **Intuitive Controls**:
  - Single-click to create connections between ports
  - Multi-select nodes with box selection
  - Drag to move selected nodes
- **Navigation**:
  - Pan with middle mouse button
  - Zoom with mouse wheel (centered on cursor)
- **Context Menu**: Right-click to create new nodes
- **Keyboard Shortcuts**:
  - `Delete` - Remove selected nodes/connections
  - `ESC` - Cancel connection in progress
  - `Ctrl/Cmd + Click` - Multi-select nodes
- **Node Types**:
  - Math operations (Add, Subtract, Multiply, Divide)
  - Logic gates (AND, OR, NOT)
  - Data nodes (Constant, Variable)
  - Output nodes (Print, Debug)

## Quick Start

### Prerequisites

- Rust 1.70 or higher
- Cargo

### Building and Running

```bash
# Clone the repository
git clone https://github.com/bsundman/nodle.git
cd nodle

# Run the application
cargo run --package nodle

# Or use the shorter alias
cargo run -p nodle

# Build for release
cargo build --release -p nodle

# Test the core library
cargo test -p nodle-core
```

## Usage

1. **Creating Nodes**: Right-click on empty space to open the context menu and select a node type
2. **Connecting Nodes**: Click on an output port, then click on an input port to create a connection
3. **Moving Nodes**: Click and drag nodes to reposition them
4. **Selecting Multiple Nodes**: Click and drag on empty space to create a selection box
5. **Deleting Elements**: Select nodes or connections and press Delete

## Architecture

The editor is built using:
- **egui/eframe**: Immediate mode GUI framework
- **Custom rendering**: Bezier curves for smooth connections
- **Efficient data structures**: HashMap for nodes, Vec for connections

For detailed technical information, see [DEVELOPER_GUIDE.md](DEVELOPER_GUIDE.md).

## Using the Library

To use `nodle-core` in your own project, add it to your `Cargo.toml`:

```toml
[dependencies]
nodle-core = { git = "https://github.com/bsundman/nodle.git" }
# Or when published to crates.io:
# nodle-core = "0.1"
```

Example usage:

```rust
use nodle_core::{NodeGraph, Node, Connection};
use egui::Pos2;

let mut graph = NodeGraph::new();

// Create nodes
let mut node1 = Node::new(0, "Math", Pos2::new(100.0, 100.0));
node1.add_input("A").add_input("B").add_output("Result");

let mut node2 = Node::new(0, "Output", Pos2::new(300.0, 100.0));
node2.add_input("Value");

// Add to graph
let id1 = graph.add_node(node1);
let id2 = graph.add_node(node2);

// Connect them
let connection = Connection::new(id1, 0, id2, 0);
graph.add_connection(connection).unwrap();
```

## Development

See [DEVELOPER_GUIDE.md](DEVELOPER_GUIDE.md) for comprehensive documentation on:
- Architecture and core components
- Adding new node types
- Coordinate systems and transforms
- Input handling
- Debugging tips

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Author

Brian Sundman ([@bsundman](https://github.com/bsundman))