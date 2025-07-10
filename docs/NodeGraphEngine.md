# NodeGraphEngine Documentation

## Overview

The `NodeGraphEngine` is a comprehensive dataflow execution system that manages the evaluation and execution of node graphs in Nodle. It provides dirty state tracking, dependency resolution, topological sorting, and intelligent caching to ensure efficient and correct execution of connected nodes.

## Architecture

### Core Components

```rust
pub struct NodeGraphEngine {
    /// Current execution state for each node
    node_states: HashMap<NodeId, NodeState>,
    /// Cached output values for each node's output ports
    output_cache: HashMap<(NodeId, usize), NodeData>,
    /// Set of nodes that need re-evaluation
    dirty_nodes: HashSet<NodeId>,
    /// Execution order cache (invalidated when graph changes)
    execution_order_cache: Option<Vec<NodeId>>,
}
```

### Node States

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum NodeState {
    Clean,      // Node is up-to-date
    Dirty,      // Node needs re-evaluation
    Computing,  // Node is currently being processed
    Error,      // Node failed to execute
}
```

## Key Features

### 1. Dirty State Propagation

The engine automatically tracks which nodes need re-evaluation:

- **Upstream Changes**: When a node's parameters change, it's marked dirty
- **Connection Changes**: Adding/removing connections marks downstream nodes dirty
- **Cascade Propagation**: Dirty state propagates through the dependency chain
- **Smart Invalidation**: Only affected nodes are marked for re-execution

### 2. Topological Sorting

Ensures correct execution order by resolving dependencies:

- **Kahn's Algorithm**: Uses proven topological sort algorithm
- **Cycle Detection**: Prevents infinite loops in node graphs
- **Execution Order**: Guarantees upstream nodes execute before downstream
- **Caching**: Execution order is cached until graph structure changes

### 3. Output Caching

Optimizes performance by caching node outputs:

- **Port-Level Caching**: Each output port can have cached data
- **Automatic Invalidation**: Cache cleared when nodes become dirty
- **Memory Efficient**: Only caches outputs that are actually used
- **Type-Safe**: Uses `NodeData` enum for type safety

### 4. Event-Driven Execution

Responds to graph changes automatically:

- **Connection Events**: `on_connection_added()`, `on_connection_removed()`
- **Parameter Changes**: `on_node_parameter_changed()`
- **Manual Triggers**: `mark_dirty()`, `mark_all_dirty()`
- **Batch Operations**: Efficient bulk operations

## API Reference

### Core Methods

#### `new() -> Self`
Creates a new execution engine instance.

#### `execute_dirty_nodes(&mut self, graph: &NodeGraph) -> Result<(), String>`
Executes all dirty nodes in correct dependency order.

```rust
// Execute all dirty nodes
match engine.execute_dirty_nodes(&graph) {
    Ok(_) => println!("✅ All connections executed successfully"),
    Err(e) => println!("❌ Execution failed: {}", e),
}
```

#### `mark_dirty(&mut self, node_id: NodeId, graph: &NodeGraph)`
Marks a specific node as dirty and propagates to downstream nodes.

```rust
// Mark node 5 as dirty
engine.mark_dirty(5, &graph);
```

#### `get_execution_order(&mut self, graph: &NodeGraph) -> Result<Vec<NodeId>, String>`
Computes the topological execution order for all nodes.

### Event Handlers

#### `on_connection_added(&mut self, connection: &Connection, graph: &NodeGraph)`
Handles new connection creation.

#### `on_connection_removed(&mut self, connection: &Connection, graph: &NodeGraph)`
Handles connection deletion.

#### `on_node_parameter_changed(&mut self, node_id: NodeId, graph: &NodeGraph)`
Handles node parameter modifications.

### State Queries

#### `get_node_state(&self, node_id: NodeId) -> NodeState`
Returns the current execution state of a node.

#### `get_cached_output(&self, node_id: NodeId, port_idx: usize) -> Option<&NodeData>`
Retrieves cached output data for a specific port.

#### `get_stats(&self) -> ExecutionStats`
Returns execution statistics and performance metrics.

## Supported Node Types

The engine currently supports the following node types through its dispatch system:

### Data Nodes
- **USD File Reader**: Loads USD stage files
- **Constant**: Provides constant values
- **Variable**: Manages variable data

### Math Nodes
- **Add**: Addition operations
- **Subtract**: Subtraction operations
- **Multiply**: Multiplication operations
- **Divide**: Division operations

### Logic Nodes
- **And**: Boolean AND operations
- **Or**: Boolean OR operations
- **Not**: Boolean NOT operations

### Output Nodes
- **Print**: Console output
- **Debug**: Debug information output

### 3D Nodes
- **Viewport**: 3D viewport rendering
- **Transform**: Translation, rotation, scaling
- **Geometry**: Cube, sphere, plane generation
- **Lighting**: Point, directional, spot lights

### Fallback Handling
Unknown node types are gracefully handled with pass-through behavior.

## Integration

### Main Loop Integration

The engine integrates seamlessly with Nodle's main execution loop:

```rust
fn check_and_execute_connections(&mut self, _viewed_nodes: &HashMap<NodeId, Node>) {
    println!("🔗 MAIN LOOP: About to execute connections using new execution engine");
    
    // Get the current graph based on view context
    let graph = match self.navigation.current_view() {
        GraphView::Root => &self.graph,
        GraphView::WorkspaceNode(workspace_id) => {
            if let Some(workspace_node) = self.graph.nodes.get(workspace_id) {
                workspace_node.get_internal_graph().unwrap_or(&self.graph)
            } else {
                &self.graph
            }
        }
    };
    
    // Execute all dirty nodes using the new execution engine
    match self.execution_engine.execute_dirty_nodes(graph) {
        Ok(_) => println!("✅ All connections executed successfully"),
        Err(e) => println!("❌ Connection execution failed: {}", e),
    }
}
```

### Context Awareness

The engine works with both:
- **Root Graph**: Main application graph
- **Workspace Nodes**: Internal graphs within workspace nodes

## Performance Characteristics

### Time Complexity
- **Topological Sort**: O(V + E) where V = nodes, E = connections
- **Dirty Propagation**: O(D) where D = downstream nodes
- **Cache Lookup**: O(1) for output retrieval

### Space Complexity
- **Node States**: O(V) for all nodes
- **Output Cache**: O(P) where P = active output ports
- **Execution Order**: O(V) for cached order

### Optimizations
- **Lazy Evaluation**: Only executes dirty nodes
- **Smart Caching**: Avoids redundant computations
- **Batch Operations**: Efficient bulk updates
- **Early Termination**: Stops on first error

## Error Handling

The engine provides robust error handling:

### Error Types
- **Cycle Detection**: Prevents infinite loops
- **Node Execution Failures**: Graceful degradation
- **Missing Dependencies**: Clear error messages
- **Type Mismatches**: Safe fallback behavior

### Recovery Strategies
- **Partial Execution**: Successfully executed nodes remain valid
- **State Preservation**: Engine state remains consistent
- **Error Propagation**: Clear error reporting to caller
- **Cleanup**: Automatic cleanup of failed operations

## Debugging and Monitoring

### Execution Statistics

```rust
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    pub total_nodes: usize,
    pub clean_nodes: usize,
    pub dirty_nodes: usize,
    pub computing_nodes: usize,
    pub error_nodes: usize,
    pub cached_outputs: usize,
}
```

### Debug Output

The engine provides detailed logging:

```
🔄 Executing 3 dirty nodes
🔍 Execution order: [1, 2, 3]
🔄 Executing node 1 (Add)
➕ Executing Add node with 2 inputs
✅ Node 1 executed successfully
🔄 Executing node 2 (Multiply)
✖️ Executing Multiply node with 2 inputs
✅ Node 2 executed successfully
🔄 Executing node 3 (Viewport)
🖼️ Executing Viewport node with 1 inputs
✅ Node 3 executed successfully
✅ All dirty nodes executed successfully
```

## Best Practices

### For Engine Users

1. **Mark Dirty Appropriately**: Call `mark_dirty()` when node parameters change
2. **Handle Events**: Use event handlers for connection changes
3. **Check Results**: Always check return values from execution methods
4. **Monitor Performance**: Use `get_stats()` for performance monitoring

### For Node Implementers

1. **Stateless Operations**: Keep node execution functions pure when possible
2. **Error Handling**: Return appropriate errors for invalid inputs
3. **Type Safety**: Use `NodeData` enum correctly
4. **Performance**: Avoid expensive operations in node execution

## Future Enhancements

### Planned Features

1. **Parallel Execution**: Execute independent branches in parallel
2. **Incremental Updates**: More granular dirty state tracking
3. **Memory Management**: Better cache eviction strategies
4. **Plugin Integration**: Better integration with plugin system
5. **Profiling Tools**: Advanced performance profiling
6. **Serialization**: Save/load execution state

### Extension Points

1. **Custom Node Types**: Easy addition of new node execution logic
2. **Execution Strategies**: Pluggable execution algorithms
3. **Cache Policies**: Configurable caching strategies
4. **Event System**: Extended event handling capabilities

## Examples

### Basic Usage

```rust
// Create engine
let mut engine = NodeGraphEngine::new();

// Mark a node dirty
engine.mark_dirty(node_id, &graph);

// Execute dirty nodes
engine.execute_dirty_nodes(&graph)?;

// Check execution stats
let stats = engine.get_stats();
println!("Executed {} nodes", stats.total_nodes - stats.clean_nodes);
```

### Event Handling

```rust
// Handle connection addition
let connection = Connection::new(from_node, from_port, to_node, to_port);
engine.on_connection_added(&connection, &graph);

// Handle parameter change
engine.on_node_parameter_changed(node_id, &graph);

// Execute changes
engine.execute_dirty_nodes(&graph)?;
```

### Performance Monitoring

```rust
let stats = engine.get_stats();
println!("Performance Stats:");
println!("  Total nodes: {}", stats.total_nodes);
println!("  Clean nodes: {}", stats.clean_nodes);
println!("  Dirty nodes: {}", stats.dirty_nodes);
println!("  Cached outputs: {}", stats.cached_outputs);
```

## Conclusion

The `NodeGraphEngine` provides a robust, efficient, and extensible foundation for dataflow execution in Nodle. Its event-driven architecture, intelligent caching, and comprehensive error handling make it suitable for complex node graph applications while maintaining excellent performance characteristics.