//! Startup checks for Nodle
//! Ensures all dependencies are properly installed

use std::process::Command;

#[cfg(feature = "usd")]
use crate::nodes::three_d::usd::local_usd;

/// Check if all required dependencies are installed
pub fn check_dependencies() -> Result<(), String> {
    println!("Checking Nodle dependencies...");
    
    // Check for local USD installation
    #[cfg(feature = "usd")]
    check_usd_installation()?;
    
    println!("✓ All dependencies verified");
    Ok(())
}

#[cfg(feature = "usd")]
fn check_usd_installation() -> Result<(), String> {
    println!("  Checking USD installation...");
    
    if !local_usd::is_usd_installed() {
        eprintln!("\n╭─────────────────────────────────────────────────────────╮");
        eprintln!("│ ⚠️  USD Not Installed                                    │");
        eprintln!("├─────────────────────────────────────────────────────────┤");
        eprintln!("│ Nodle requires a local USD installation to function.    │");
        eprintln!("│                                                         │");
        eprintln!("│ Please run the following command from the project root: │");
        eprintln!("│                                                         │");
        eprintln!("│   python scripts/setup_usd.py                           │");
        eprintln!("│                                                         │");
        eprintln!("│ This will download and install USD locally.            │");
        eprintln!("│ Installation takes about 2-5 minutes.                   │");
        eprintln!("╰─────────────────────────────────────────────────────────╯\n");
        
        return Err("USD not installed".to_string());
    }
    
    // Try to get version
    match local_usd::get_usd_version() {
        Ok(version) => {
            println!("    ✓ USD {} found at: {:?}", version, local_usd::get_usd_root());
        }
        Err(e) => {
            println!("    ⚠️  USD found but version check failed: {}", e);
            // This is a warning, not a fatal error
        }
    }
    
    Ok(())
}

/// Check if Python is available (for setup script)
pub fn check_python_available() -> bool {
    // Try python3 first, then python
    for cmd in &["python3", "python"] {
        if let Ok(output) = Command::new(cmd).arg("--version").output() {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("Found Python: {}", version.trim());
                return true;
            }
        }
    }
    false
}

/// Show helpful setup instructions if dependencies are missing
pub fn show_setup_help() {
    eprintln!("\n╭─────────────────────────────────────────────────────────╮");
    eprintln!("│ 📋 Nodle Setup Instructions                             │");
    eprintln!("├─────────────────────────────────────────────────────────┤");
    eprintln!("│ 1. Ensure Python 3.8+ is installed:                     │");
    eprintln!("│    - macOS: brew install python3                        │");
    eprintln!("│    - Ubuntu: sudo apt install python3 python3-pip       │");
    eprintln!("│    - Windows: Download from python.org                  │");
    eprintln!("│                                                         │");
    eprintln!("│ 2. Run USD setup from project root:                    │");
    eprintln!("│    python scripts/setup_usd.py                          │");
    eprintln!("│                                                         │");
    eprintln!("│ 3. Rebuild and run Nodle:                              │");
    eprintln!("│    cargo run --features usd                             │");
    eprintln!("│                                                         │");
    eprintln!("│ For more help, see vendor/USD_README.md                │");
    eprintln!("╰─────────────────────────────────────────────────────────╯\n");
}