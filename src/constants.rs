//! Application-wide constants and default values
//!
//! Centralized location for all hard-coded values to improve maintainability

/// Default menu bar height used throughout the application
pub const DEFAULT_MENU_BAR_HEIGHT: f32 = 34.0;

/// Panel sizing constants
pub mod panel {
    /// Default viewport panel size
    pub const DEFAULT_VIEWPORT_SIZE: [f32; 2] = [800.0, 600.0];
    
    /// Minimum viewport panel size
    pub const MIN_VIEWPORT_SIZE: [f32; 2] = [200.0, 200.0];
    
    /// Maximum viewport panel size
    pub const MAX_VIEWPORT_SIZE: [f32; 2] = [1600.0, 1200.0];
    
    /// Default parameter panel size
    pub const DEFAULT_PARAMETER_SIZE: [f32; 2] = [380.0, 500.0];
    
    /// Minimum parameter panel size
    pub const MIN_PARAMETER_SIZE: [f32; 2] = [300.0, 200.0];
    
    /// Maximum parameter panel size
    pub const MAX_PARAMETER_SIZE: [f32; 2] = [500.0, 800.0];
    
    /// Default parameter panel width for stacked mode
    pub const STACKED_PARAMETER_WIDTH: f32 = 400.0;
}

/// Menu system constants
pub mod menu {
    /// Timer delay before closing submenus (milliseconds)
    pub const SUBMENU_CLOSE_DELAY_MS: u128 = 800;
}

/// Node system constants  
pub mod node {
    /// Starting ID for temporary plugin nodes to avoid conflicts
    pub const TEMP_ID_START: usize = 1_000_000_000;
}

/// Camera manipulation sensitivity constants
pub mod camera {
    /// Default drag sensitivity for camera orbit/pan
    pub const DEFAULT_DRAG_SENSITIVITY: f32 = 0.01;
    
    /// Default scroll sensitivity for camera zoom
    pub const DEFAULT_SCROLL_SENSITIVITY: f32 = 0.01;
}

/// UI spacing and sizing constants
pub mod ui {
    /// Default margin for UI frames
    pub const DEFAULT_FRAME_MARGIN: f32 = 8.0;
    
    /// Default rounding for UI frames
    pub const DEFAULT_FRAME_ROUNDING: f32 = 4.0;
    
    /// Default spacing between UI elements
    pub const DEFAULT_ELEMENT_SPACING: f32 = 10.0;
    
    /// Minimum window size constraints
    pub const MIN_WINDOW_SIZE: [f32; 2] = [100.0, 100.0];
}

/// Performance and rendering constants
pub mod performance {
    /// Maximum output length before truncation
    pub const MAX_OUTPUT_LENGTH: usize = 30000;
    
    /// Default command timeout in milliseconds
    pub const DEFAULT_COMMAND_TIMEOUT_MS: u64 = 120000; // 2 minutes
    
    /// Maximum command timeout in milliseconds
    pub const MAX_COMMAND_TIMEOUT_MS: u64 = 600000; // 10 minutes
}