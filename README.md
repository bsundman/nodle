# Nōdle

A node-based visual programming editor built with Rust and egui.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)

## Overview

Nōdle is a high-performance node editor implementation featuring a vertical flow design where connections flow from top to bottom. It provides an intuitive interface for creating and connecting nodes in a visual programming environment with GPU-accelerated rendering.

## Features

- **Vertical Flow Design**: Input ports on top, output ports on bottom
- **GPU-Accelerated Rendering**: High-performance node and connection rendering using wgpu
- **Rich Node System**: Extensible node factory with categories for Math, Logic, Data, 3D, USD, and more
- **Intuitive Controls**:
  - Single-click to create connections between ports
  - Multi-select nodes with box selection
  - Drag to move selected nodes
  - Freehand connection drawing with C key
  - Cut connections with X key
- **Navigation**:
  - Pan with middle mouse button or space+drag
  - Zoom with mouse wheel (centered on cursor)
  - Frame all nodes with F key
- **Context Menu**: Right-click to create new nodes
- **Keyboard Shortcuts**:
  - `Delete` - Remove selected nodes/connections
  - `ESC` - Cancel connection in progress
  - `Ctrl/Cmd + Click` - Multi-select nodes
  - `C` - Freehand connection drawing mode
  - `X` - Connection cutting mode
  - `F` - Frame all nodes
- **USD Integration**: Comprehensive Universal Scene Description nodes for 3D workflows
- **Workspace System**: Context-specific workspaces (General, 3D, USD, MaterialX)
- **Interface Panels**: Parameter panels for node configuration with real-time updates

## Getting Started

See [GETTING_STARTED.md](GETTING_STARTED.md) for detailed setup instructions.

### Quick Start

1. **Install Dependencies**:
   ```bash
   # macOS/Linux
   ./install.sh
   
   # Windows
   install.bat
   ```

2. **Run Nodle**:
   ```bash
   cargo run --release --features usd
   ```

## Architecture

Nōdle uses a node-centric architecture where nodes drive everything through their metadata:

- **Node Factory System**: Self-registering nodes with rich metadata
- **Modular Node Structure**: Each node type has `mod.rs`, `logic.rs`, and `parameters.rs`
- **GPU Rendering Pipeline**: Efficient batch rendering of nodes and connections
- **Interface Panel System**: Automatic UI generation from node parameters

See [DEVELOPER_GUIDE.md](DEVELOPER_GUIDE.md) for detailed technical documentation.

## Node Categories

- **Math**: Add, Subtract, Multiply, Divide
- **Logic**: AND, OR, NOT, Compare
- **Data**: Constant, Variable, Convert
- **3D Geometry**: Cube, Sphere, Plane, Transform nodes
- **USD**: Complete USD pipeline nodes for industry-standard 3D workflows
- **Lighting**: Point, Directional, Spot lights
- **Output**: Print, Debug, Viewport

## Building from Source

```bash
# Clone the repository
git clone https://github.com/bsundman/nodle.git
cd nodle

# Build and run
cargo build --release
cargo run --release
```

### Features

- `usd` - Enable USD (Universal Scene Description) support

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [egui](https://github.com/emilk/egui) - an immediate mode GUI library for Rust
- GPU rendering powered by [wgpu](https://github.com/gfx-rs/wgpu) - cross-platform graphics API
- USD integration via [PyO3](https://github.com/PyO3/pyo3) - Rust bindings for Python