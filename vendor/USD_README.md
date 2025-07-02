# Local USD Installation for Nodle

This directory contains Nodle's local USD (Universal Scene Description) installation to ensure all users have a compatible version.

## Initial Setup

Run the setup script from the project root:

```bash
# Install USD locally (uses pip - fast)
python scripts/setup_usd.py

# Or build from source (slower but more control)
python scripts/setup_usd.py --build-from-source
```

## What This Does

1. **Creates a virtual environment** in `vendor/usd/venv`
2. **Installs USD 24.08** via pip (or builds from source)
3. **Configures Rust** to use this local installation
4. **Creates activation scripts** for development

## Directory Structure

```
vendor/
└── usd/
    ├── venv/           # Python virtual environment with USD
    │   ├── bin/        # (or Scripts/ on Windows)
    │   └── lib/        # USD Python packages
    └── (other files)
```

## For Development

Activate the USD environment:

```bash
# macOS/Linux
source activate_usd.sh

# Windows
activate_usd.bat
```

## For End Users

When distributing Nodle:
- The `vendor/usd` directory will be bundled with the application
- Users won't need Python or USD installed system-wide
- Everything runs from this local installation

## Troubleshooting

If USD import fails:
1. Ensure Python 3.8-3.12 is installed
2. Re-run `python scripts/setup_usd.py`
3. Check that `vendor/usd/venv` exists

## Version Management

Current USD version: **24.08**
Python requirement: **3.8 - 3.12**

To update USD version, modify `USD_VERSION` in `scripts/setup_usd.py`