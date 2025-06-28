# Code Reduction Strategy for editor/mod.rs

## Current State
After completing 5 phases of modularization, editor/mod.rs has been reduced from 2450 to 1534 lines (916 lines extracted). However, there are still opportunities to reduce it by approximately 500 more lines through eliminating duplication and consolidating patterns.

## Identified Reduction Opportunities (~500 lines)

### 1. Graph Access Unification (~100 lines)
**Problem**: Repetitive match statements for accessing the correct graph based on current view.

**Current Pattern** (repeated 8+ times):
```rust
match self.view_manager.current_view() {
    GraphView::Root => {
        // operate on self.graph
    }
    GraphView::WorkspaceNode(node_id) => {
        if let Some(node) = self.graph.nodes.get(&node_id) {
            if let Some(internal_graph) = node.get_internal_graph() {
                // operate on internal_graph
            }
        }
    }
}
```

**Solution**: Create helper methods in `ViewManager`:
```rust
impl ViewManager {
    pub fn with_active_graph<T>(&self, root_graph: &NodeGraph, f: impl FnOnce(&NodeGraph) -> T) -> T;
    pub fn with_active_graph_mut<T>(&self, root_graph: &mut NodeGraph, f: impl FnOnce(&mut NodeGraph) -> T) -> T;
}
```

**Usage**:
```rust
// Instead of 10-line match statement:
self.view_manager.with_active_graph_mut(&mut self.graph, |graph| {
    self.interaction.update_drag(pos, graph);
});
```

### 2. Button Click Handler Extraction (~80 lines)
**Problem**: Large inline button click handling code repeated across different contexts.

**Current**: ~80 lines of inline button handling in `handle_mouse_input`

**Solution**: Extract to `InteractionManager`:
```rust
impl InteractionManager {
    pub fn handle_button_clicks(&mut self, mouse_pos: Pos2, view_manager: &ViewManager, graph: &mut NodeGraph) -> bool;
}
```

### 3. Connection Rendering Consolidation (~120 lines)
**Problem**: Similar connection rendering logic duplicated between CPU and GPU paths.

**Current**: Separate bezier calculation and rendering for each mode

**Solution**: Extract common logic to `MeshRenderer`:
```rust
impl MeshRenderer {
    pub fn prepare_connection_data(&self, connections: &[Connection], nodes: &HashMap<NodeId, Node>) -> Vec<ConnectionRenderData>;
    pub fn render_connections_cpu(&self, data: &[ConnectionRenderData], ui: &mut Ui);
    pub fn render_connections_gpu(&self, data: &[ConnectionRenderData]) -> Vec<InstanceData>;
}
```

### 4. View-Aware Operations (~150 lines)
**Problem**: Many operations need to be view-aware but duplicate the view-checking logic.

**Current**: Each operation manually checks current view and dispatches

**Solution**: Create operation dispatcher in `ViewManager`:
```rust
impl ViewManager {
    pub fn execute_node_operation<T>(&self, graph: &mut NodeGraph, op: impl NodeOperation<T>) -> T;
    pub fn execute_connection_operation<T>(&self, graph: &mut NodeGraph, op: impl ConnectionOperation<T>) -> T;
}

trait NodeOperation<T> {
    fn execute_on_graph(&self, graph: &mut NodeGraph) -> T;
}
```

### 5. Input State Processing (~50 lines)
**Problem**: Long chains of input state checking in update loop.

**Current**: Many separate `if` statements for different input combinations

**Solution**: Extract to `InputState`:
```rust
impl InputState {
    pub fn process_editor_inputs(&self, ui: &Ui) -> Vec<EditorAction>;
}

enum EditorAction {
    DeleteSelected,
    ToggleGpuRendering,
    ShowPerformanceInfo,
    AddBenchmarkNodes(usize),
    ClearGraph,
}
```

## Implementation Priority

1. **High Impact**: Graph Access Unification (most duplicated pattern)
2. **Medium Impact**: View-Aware Operations (architectural improvement)
3. **Medium Impact**: Connection Rendering (performance benefit)
4. **Low Impact**: Button Click Handler (code organization)
5. **Low Impact**: Input State Processing (minor cleanup)

## Expected Result
- **Current**: 1534 lines
- **Target**: ~1000 lines (534 line reduction)
- **Benefit**: Cleaner code, less duplication, easier maintenance

## Implementation Notes
- Each reduction should be implemented incrementally
- Test after each major change to ensure functionality is preserved
- Some reductions may require updates to the extracted modules
- Focus on maintaining the same external API for editor/mod.rs