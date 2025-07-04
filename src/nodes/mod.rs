//! Node system - Core data structures and generic node implementations

// Core node system modules
pub mod graph;
pub mod node;
pub mod port;
pub mod math_utils;
pub mod factory;

// Generic node implementations
pub mod math;
pub mod logic;
pub mod data;
pub mod output;

// Context-specific node implementations
pub mod materialx;

// 3D context node implementations
pub mod three_d;

// Re-export core types
pub use graph::{Connection, NodeGraph};
pub use node::{Node, NodeId, NodeType, PortMapping};
pub use port::PortId;

// Re-export factory types
pub use factory::{
    NodeFactory, NodeMetadata, NodeCategory,
    DataType, PortDefinition,
};