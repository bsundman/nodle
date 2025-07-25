//! Node system - Core data structures and generic node implementations

// Core node system modules
pub mod graph;
pub mod node;
pub mod port;
pub mod math_utils;
pub mod factory;
pub mod interface;
pub mod defaults;
pub mod execution_engine;
pub mod hooks;
pub mod ownership;
pub mod cache;

// Generic node implementations
pub mod math;
pub mod logic;
pub mod data;
pub mod output;
pub mod utility;

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

// Re-export interface types
pub use interface::{
    InterfacePanelManager, PanelType,
};

// Re-export execution engine types
pub use execution_engine::{
    NodeGraphEngine, NodeState, ExecutionStats,
};