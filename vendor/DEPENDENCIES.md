# Python Dependencies for Nōdle

## Current Python Runtime

**Python Version**: 3.9.19 (maintained for broader compatibility)  
**Installation Date**: Auto-updated by setup script  
**Location**: `./python-runtime/python/`

## Required Python Packages

### Core Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| `usd-core` | 25.5.1 | Universal Scene Description (USD) core libraries |
| `numpy` | 2.0.2 | Numerical computing library (required by USD) |
| `pip` | 25.1.1 | Package installer |
| `setuptools` | 80.9.0 | Package building tools |
| `wheel` | 0.45.1 | Package format |

### Package Details

#### usd-core (25.5.1)
- **Purpose**: Core USD libraries for 3D scene description
- **Size**: ~200MB
- **Platform**: Cross-platform (Windows, macOS, Linux)
- **Python Compatibility**: 3.9 - 3.12
- **USD Features**: Core USD functionality without optional plugins
- **Documentation**: https://openusd.org/

#### numpy (2.0.2)
- **Purpose**: Array computing, required by USD Python bindings
- **Size**: ~20MB
- **Platform**: Cross-platform with optimized builds
- **Python Compatibility**: 3.9+
- **Features**: Numerical arrays and mathematical functions

## Python Version Compatibility

### USD Python Support Matrix
- **Python 3.12**: ✅ Supported (usd-core 24.11+)
- **Python 3.11**: ✅ Supported (usd-core 23.5+)
- **Python 3.10**: ✅ Supported (usd-core 22.3+)
- **Python 3.9**: ✅ Supported (usd-core 21.2+)
- **Python 3.13**: ❌ Not yet supported by USD

### Why Python 3.9.19?
- **Broader Compatibility**: Works with wider range of systems and tools
- **Stability**: Mature version with extensive testing
- **USD Support**: Fully supported by latest USD versions
- **Long-term Support**: Python 3.9 is well-established and stable

## Installation Notes

### Embedded Python Runtime
This project uses an embedded Python runtime to ensure:
- **Consistency**: Same Python environment across all installations
- **Isolation**: No conflicts with system Python
- **Portability**: Works regardless of system Python version
- **USD Integration**: Guaranteed USD compatibility

### Platform-Specific Considerations

#### macOS
- **Architecture**: Universal binary (Intel + Apple Silicon)
- **Minimum OS**: macOS 10.15 (Catalina)
- **Library Path**: Uses `DYLD_LIBRARY_PATH` for runtime linking

#### Linux
- **Architecture**: x86_64
- **Minimum OS**: Recent distributions with glibc 2.17+
- **Library Path**: Uses `LD_LIBRARY_PATH` for runtime linking

#### Windows
- **Architecture**: x86_64
- **Minimum OS**: Windows 10
- **Library Path**: Uses `PATH` for runtime linking

## Build Configuration

### Cargo Configuration
The embedded Python is configured in `.cargo/config.toml`:
```toml
[env]
PYO3_PYTHON = "./vendor/python-runtime/python/bin/python3"
PYO3_PYTHON_VERSION = "3.9"
PYTHONPATH = "./vendor/python-runtime/python/lib/python3.9/site-packages"
```

### PyO3 Integration
- **Version**: 0.25 (supports Python 3.9)
- **Features**: `auto-initialize` for embedded Python
- **Binding**: Rust ↔ Python integration for USD calls

## Maintenance

### Update Schedule
- **Python**: Keep at 3.9.19 for stability and compatibility
- **USD**: Update to latest USD version with Python 3.9 support
- **Numpy**: Update to latest compatible version

### Update Process
1. Run `./vendor/setup_python.sh` to update Python runtime
2. Run `./vendor/test_python.sh` to verify installation
3. Update version numbers in this file
4. Test USD functionality after updates
5. Update `.cargo/config.toml` if needed

## Security Considerations

### Package Verification
- All packages installed from official PyPI repositories
- Package hashes verified during installation
- No custom or modified packages

### Isolation
- Embedded Python runtime is isolated from system Python
- No system-wide Python modifications
- Self-contained installation

## Troubleshooting

### Common Issues

#### "Python not found" errors
- **Cause**: Hardcoded paths in `.cargo/config.toml`
- **Solution**: Run `./vendor/setup_python.sh` to fix paths
- **Diagnosis**: Run `./vendor/test_python.sh` to identify issues

#### USD import errors
- **Cause**: Missing or incompatible USD installation
- **Solution**: Reinstall with `./vendor/setup_python.sh --clean`
- **Diagnosis**: Run `./vendor/test_python.sh -v` for detailed diagnostics

#### Performance issues
- **Cause**: Outdated Python or numpy version
- **Solution**: Update runtime with setup script

### Getting Help
- **USD Documentation**: https://openusd.org/
- **Python Documentation**: https://docs.python.org/3.9/
- **PyO3 Documentation**: https://pyo3.rs/

## File Structure

```
vendor/
├── DEPENDENCIES.md          # This file
├── setup_python.sh          # Python installation script
├── test_python.sh           # Python testing script
├── update_packages.sh       # Package update script
├── python-runtime/          # Embedded Python installation
│   └── python/
│       ├── bin/
│       │   └── python3      # Python executable
│       ├── lib/
│       │   ├── libpython3.9.dylib   # Python library
│       │   └── python3.9/   # Python standard library
│       └── include/         # Python headers
└── .gitignore              # Excludes python-runtime from git
```

## License Information

### Python License
- **License**: Python Software Foundation License
- **Version**: Compatible with commercial use
- **URL**: https://docs.python.org/3.9/license.html

### USD License
- **License**: Modified Apache 2.0 License
- **Version**: Compatible with commercial use
- **URL**: https://github.com/PixarAnimationStudios/OpenUSD/blob/dev/LICENSE.txt

### Numpy License
- **License**: BSD 3-Clause License
- **Version**: Compatible with commercial use
- **URL**: https://github.com/numpy/numpy/blob/main/LICENSE.txt