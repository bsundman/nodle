@echo off
REM Nodle Installation Script for Windows
REM Sets up all dependencies including local USD

echo ╭─────────────────────────────────────────────────────────╮
echo │               🎨 Nodle Installation                      │
echo │         Node-based 3D Visual Programming                 │
echo ╰─────────────────────────────────────────────────────────╯
echo.

REM Check for Python
echo 🔍 Checking for Python...
python --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ Python not found. Please install Python 3.8 or higher from python.org
    echo    Make sure to check "Add Python to PATH" during installation.
    pause
    exit /b 1
)

for /f "tokens=2" %%a in ('python --version 2^>^&1') do set PYTHON_VERSION=%%a
echo ✓ Found Python %PYTHON_VERSION%

REM Check Python version
for /f %%i in ('python -c "import sys; print(sys.version_info.major)"') do set PYTHON_MAJOR=%%i
for /f %%i in ('python -c "import sys; print(sys.version_info.minor)"') do set PYTHON_MINOR=%%i

if %PYTHON_MAJOR% lss 3 (
    echo ❌ Python 3.8 or higher is required. Found %PYTHON_VERSION%
    pause
    exit /b 1
)
if %PYTHON_MAJOR% equ 3 if %PYTHON_MINOR% lss 8 (
    echo ❌ Python 3.8 or higher is required. Found %PYTHON_VERSION%
    pause
    exit /b 1
)

REM Setup USD
echo.
echo 📦 Setting up USD (Universal Scene Description)...
echo    This may take 2-5 minutes on first install...
python scripts\setup_usd.py
if %errorlevel% neq 0 (
    echo ❌ USD setup failed
    pause
    exit /b 1
)

REM Check if cargo is installed
echo.
echo 🔍 Checking for Rust...
cargo --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ Rust not found. Please install from https://rustup.rs/
    pause
    exit /b 1
)
echo ✓ Found Rust

REM Build Nodle
echo.
echo 🔨 Building Nodle...
cargo build --release --features usd
if %errorlevel% neq 0 (
    echo ❌ Build failed
    pause
    exit /b 1
)

echo.
echo ╭─────────────────────────────────────────────────────────╮
echo │               ✅ Installation Complete!                  │
echo ├─────────────────────────────────────────────────────────┤
echo │ Run Nodle with:                                         │
echo │   cargo run --release --features usd                    │
echo │                                                         │
echo │ Or run the built executable directly:                   │
echo │   target\release\nodle.exe                              │
echo ╰─────────────────────────────────────────────────────────╯
echo.
pause