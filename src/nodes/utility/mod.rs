//! Utility nodes module
//!
//! Contains general-purpose utility nodes that can be used across different workspaces
//! for organization, debugging, and workflow management.

pub mod null;
pub mod test;

// Re-export for convenience
pub use null::{NullLogic, NullNode};
pub use test::{TestLogic, TestNode};