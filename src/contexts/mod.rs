//! Context modules

pub mod base;
pub mod context_3d;
pub mod registry;
pub mod test_phase4;

// Subcontexts organized in subfolders
pub mod subcontexts {
    pub mod materialx;
}

pub use registry::ContextRegistry;