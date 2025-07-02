#!/usr/bin/env python3
"""
Setup script to download and install USD locally for Nodle
Ensures all users have the same compatible USD version
"""

import os
import sys
import subprocess
import platform
import shutil
from pathlib import Path

# Configuration
USD_VERSION = "24.08"  # Latest stable version
PYTHON_VERSION = f"{sys.version_info.major}.{sys.version_info.minor}"
NODLE_ROOT = Path(__file__).parent.parent
USD_INSTALL_DIR = NODLE_ROOT / "vendor" / "usd"
USD_PYTHON_DIR = USD_INSTALL_DIR / "lib" / "python"

def get_platform_info():
    """Get platform-specific information"""
    system = platform.system().lower()
    machine = platform.machine().lower()
    
    if system == "darwin":
        return "macos", machine
    elif system == "linux":
        return "linux", machine
    elif system == "windows":
        return "windows", machine
    else:
        raise RuntimeError(f"Unsupported platform: {system}")

def check_python_version():
    """Ensure Python version is compatible with USD"""
    if sys.version_info < (3, 8):
        print("ERROR: Python 3.8 or higher is required for USD")
        sys.exit(1)
    if sys.version_info >= (3, 13):
        print("WARNING: Python 3.13+ may not be fully supported by USD yet")

def create_directories():
    """Create necessary directories"""
    USD_INSTALL_DIR.mkdir(parents=True, exist_ok=True)
    (NODLE_ROOT / "vendor").mkdir(exist_ok=True)

def install_usd_from_pip():
    """Install USD using pip into local vendor directory"""
    print(f"Installing USD {USD_VERSION} to {USD_INSTALL_DIR}")
    
    # Create a virtual environment for USD
    venv_dir = USD_INSTALL_DIR / "venv"
    
    # Create virtual environment
    subprocess.run([
        sys.executable, "-m", "venv", str(venv_dir)
    ], check=True)
    
    # Get pip executable from venv
    if platform.system() == "Windows":
        pip_exe = venv_dir / "Scripts" / "pip.exe"
        python_exe = venv_dir / "Scripts" / "python.exe"
    else:
        pip_exe = venv_dir / "bin" / "pip"
        python_exe = venv_dir / "bin" / "python"
    
    # Upgrade pip
    subprocess.run([
        str(pip_exe), "install", "--upgrade", "pip"
    ], check=True)
    
    # Install usd-core
    subprocess.run([
        str(pip_exe), "install", f"usd-core=={USD_VERSION}"
    ], check=True)
    
    # Create activation script for development
    create_activation_script(venv_dir)
    
    return venv_dir

def build_usd_from_source():
    """Alternative: Build USD from source (more control but slower)"""
    print("Building USD from source...")
    
    # Clone OpenUSD repository
    repo_dir = NODLE_ROOT / "vendor" / "OpenUSD"
    if not repo_dir.exists():
        subprocess.run([
            "git", "clone", 
            "https://github.com/PixarAnimationStudios/OpenUSD.git",
            str(repo_dir)
        ], check=True)
    
    # Checkout specific version
    subprocess.run([
        "git", "-C", str(repo_dir), "checkout", f"v{USD_VERSION}"
    ], check=True)
    
    # Run build script
    build_script = repo_dir / "build_scripts" / "build_usd.py"
    subprocess.run([
        sys.executable, str(build_script),
        str(USD_INSTALL_DIR),
        "--python",
        "--no-tests",
        "--no-examples",
        "--no-tutorials",
        "--no-docs",
        "--no-imaging",  # Skip if you don't need Hydra
    ], check=True)

def create_activation_script(venv_dir):
    """Create script to activate USD environment"""
    activate_script = NODLE_ROOT / "activate_usd.sh"
    
    script_content = f"""#!/bin/bash
# Activate USD environment for Nodle development

export NODLE_USD_ROOT="{USD_INSTALL_DIR}"
export USD_INSTALL_ROOT="{USD_INSTALL_DIR}"
export PYTHONPATH="{venv_dir / 'lib' / f'python{PYTHON_VERSION}' / 'site-packages'}:$PYTHONPATH"
export PATH="{venv_dir / 'bin'}:$PATH"

echo "USD environment activated for Nodle"
echo "USD_INSTALL_ROOT: $USD_INSTALL_ROOT"
echo "Python: $(which python)"
"""
    
    with open(activate_script, 'w') as f:
        f.write(script_content)
    
    activate_script.chmod(0o755)
    
    # Windows version
    activate_bat = NODLE_ROOT / "activate_usd.bat"
    bat_content = f"""@echo off
rem Activate USD environment for Nodle development

set NODLE_USD_ROOT={USD_INSTALL_DIR}
set USD_INSTALL_ROOT={USD_INSTALL_DIR}
set PYTHONPATH={venv_dir}\\Lib\\site-packages;%PYTHONPATH%
set PATH={venv_dir}\\Scripts;%PATH%

echo USD environment activated for Nodle
echo USD_INSTALL_ROOT: %USD_INSTALL_ROOT%
echo Python: 
where python
"""
    
    with open(activate_bat, 'w') as f:
        f.write(bat_content)

def create_rust_config():
    """Create Rust configuration to use local USD"""
    config_content = f"""# Auto-generated USD configuration for Nodle
# This ensures Rust uses our local USD installation

[env]
NODLE_USD_ROOT = "{USD_INSTALL_DIR}"
USD_PYTHON_PATH = "{USD_INSTALL_DIR / 'venv' / 'bin' / 'python'}"
"""
    
    config_file = NODLE_ROOT / ".cargo" / "config.toml"
    config_file.parent.mkdir(exist_ok=True)
    
    with open(config_file, 'w') as f:
        f.write(config_content)

def verify_installation():
    """Verify USD installation works"""
    print("\nVerifying USD installation...")
    
    try:
        # Try importing USD
        test_script = f"""
import sys
sys.path.insert(0, '{USD_INSTALL_DIR / "venv" / "lib" / f"python{PYTHON_VERSION}" / "site-packages"}')
from pxr import Usd, UsdGeom
print("✓ Successfully imported USD modules")
print(f"  USD version: {{Usd.GetVersion()}}")
"""
        subprocess.run([sys.executable, "-c", test_script], check=True)
        print("✓ USD installation verified successfully")
        return True
    except subprocess.CalledProcessError:
        print("✗ USD installation verification failed")
        return False

def main():
    """Main setup function"""
    print(f"Setting up USD for Nodle")
    print(f"Platform: {platform.system()} {platform.machine()}")
    print(f"Python: {sys.version}")
    print("-" * 50)
    
    # Check prerequisites
    check_python_version()
    create_directories()
    
    # Choose installation method
    if "--build-from-source" in sys.argv:
        # Build from source (slower but more control)
        build_usd_from_source()
    else:
        # Use pip (faster and simpler)
        venv_dir = install_usd_from_pip()
    
    # Create Rust configuration
    create_rust_config()
    
    # Verify installation
    if verify_installation():
        print("\n" + "="*50)
        print("✓ USD setup completed successfully!")
        print(f"✓ USD installed to: {USD_INSTALL_DIR}")
        print("\nTo use USD in development:")
        if platform.system() == "Windows":
            print("  Run: activate_usd.bat")
        else:
            print("  Run: source activate_usd.sh")
        print("\nRust will automatically use this USD installation.")
    else:
        print("\n✗ USD setup failed. Please check the error messages above.")
        sys.exit(1)

if __name__ == "__main__":
    main()