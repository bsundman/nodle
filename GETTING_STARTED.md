# Getting Started with Nodle

## Quick Start

### 1. Install Dependencies

**macOS/Linux:**
```bash
chmod +x install.sh
./install.sh
```

**Windows:**
```cmd
install.bat
```

This will:
- Check for Python 3.8+ 
- Install USD locally in `vendor/usd`
- Build Nodle with USD support

### 2. Run Nodle

```bash
cargo run --release --features usd
```

Or use the built executable:
- **macOS/Linux:** `./target/release/nodle`
- **Windows:** `target\release\nodle.exe`

## Manual Setup

If the install script doesn't work:

### Prerequisites

1. **Python 3.8-3.12**
   - macOS: `brew install python3`
   - Ubuntu: `sudo apt install python3 python3-pip`
   - Windows: Download from [python.org](https://python.org)

2. **Rust**
   - Install from [rustup.rs](https://rustup.rs)

### Install USD

```bash
python scripts/setup_usd.py
```

This creates a local USD installation in `vendor/usd/` that Nodle will use automatically.

### Build Nodle

```bash
cargo build --release --features usd
```

## Troubleshooting

### "USD Not Installed" Error

Run: `python scripts/setup_usd.py`

### Python Not Found

Make sure Python is in your PATH:
- Windows: Check "Add Python to PATH" during installation
- macOS/Linux: Python usually installs to PATH automatically

### Build Errors

1. Update Rust: `rustup update`
2. Clean build: `cargo clean && cargo build --release --features usd`

## Distribution

When distributing Nodle to end users:
1. Bundle the entire `vendor/usd` directory with your app
2. Users won't need Python or USD installed
3. Everything runs from the bundled installation

## Next Steps

1. Launch Nodle
2. Create a new workspace (File → New)
3. Add nodes from the menu
4. Connect nodes to create procedural 3D scenes
5. Use USD nodes for industry-standard 3D workflows

See [USD_STRATEGY.md](USD_STRATEGY.md) for technical details about our USD integration.