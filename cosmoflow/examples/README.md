# CosmoFlow Examples

`examples/` is the feature showcase for CosmoFlow. Each program highlights one framework capability with minimal application code.

For complete applications built with CosmoFlow, use [`../../cookbook`](../../cookbook).

## Learning Path

1. `hello_world.rs`
   Basic sync `Node`, `FlowBuilder`, strongly typed state, and natural termination.

2. `custom_node.rs`
   Fallible custom nodes with domain validation and typed state.

3. `simple_loops.rs`
   Loops as ordinary state-machine routes.

4. `action_params.rs`
   `Action` params carried with a transition. Routing still uses only the action name.

5. `flow_macro.rs`
   Declarative construction with `cosmoflow::flow::flow!`.

6. `nested_flow.rs`
   A built `Flow<S>` used as a node inside another flow.

7. `shared_store.rs`
   `MemoryStorage` as an optional dynamic key-value state model.

8. `async_flow.rs`
   The async feature and async `Node` trait.

## Run Sync Examples

```bash
cargo run -p cosmoflow --example hello_world
cargo run -p cosmoflow --example custom_node
cargo run -p cosmoflow --example simple_loops
cargo run -p cosmoflow --example action_params
cargo run -p cosmoflow --example flow_macro
cargo run -p cosmoflow --example nested_flow
cargo run -p cosmoflow --example shared_store
```

## Run Async Example

```bash
cargo run -p cosmoflow --features async --example async_flow
```

Most examples are intentionally sync-only so the core ideas stay readable. They still compile under `--features async --examples`, but they print a pointer to `async_flow` instead of duplicating every node implementation.
