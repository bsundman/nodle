//! Startup checks for Nodle
//! Ensures all dependencies are properly installed

use std::process::Command;

/// Check if all required dependencies are installed
pub fn check_dependencies() -> Result<(), String> {
    println!("Checking Nodle dependencies...");
    
    // USD functionality is now provided by plugins
    // No core dependencies to check
    
    println!("✓ All dependencies verified");
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

/// Show helpful setup instructions
pub fn show_setup_help() {
    eprintln!("\n╭─────────────────────────────────────────────────────────╮");
    eprintln!("│ 📋 Nodle Setup Instructions                             │");
    eprintln!("├─────────────────────────────────────────────────────────┤");
    eprintln!("│ Nodle is a modular node editor. Core functionality is   │");
    eprintln!("│ built-in, while advanced features like USD support are  │");
    eprintln!("│ provided through plugins.                               │");
    eprintln!("│                                                         │");
    eprintln!("│ Available plugins:                                      │");
    eprintln!("│ • USD Plugin - Universal Scene Description support      │");
    eprintln!("│ • MaterialX Plugin - Material authoring                │");
    eprintln!("│                                                         │");
    eprintln!("│ To install plugins, place .dylib/.dll/.so files in:    │");
    eprintln!("│ ~/.nodle/plugins/ or ./plugins/                        │");
    eprintln!("╰─────────────────────────────────────────────────────────╯\n");
}