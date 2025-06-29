# Architecture Guide

This guide provides a comprehensive overview of CosmoFlow's architecture and component interactions.

## System Overview

CosmoFlow follows a modular, layered architecture where each component has clear responsibilities:

## Core Components

### Flows

The `cosmoflow::flow` module provides workflow orchestration and execution. A Flow represents a directed graph of nodes with routing rules.

**Key Features:**
- Async execution model
- Conditional routing
- Error handling and recovery
- Execution tracking

**Example:**
```rust
let flow = Flow::builder("my-workflow")
    .add_node("start", StartNode::new())
    .add_node("process", ProcessNode::new())
    .add_route("start", "process")
    .build()?;
```

### Nodes

The `cosmoflow::node` module defines the execution units of workflows. Each node implements the `NodeBackend` trait.

**Key Features:**
- Async execution
- Retry logic
- Context management
- Error propagation

**Example:**
```rust
#[async_trait]
impl NodeBackend for MyNode {
    async fn execute(&self, ctx: &ExecutionContext) -> Result<()> {
        // Your logic here
        Ok(())
    }
}
```

### Actions

The `cosmoflow::action` module handles flow control between nodes, supporting simple transitions and parameterized routing.

**Types of Actions:**
- **Simple**: Direct transition to named node (most common usage - 53.3%)
- **Parameterized**: Transition with additional data for complex routing logic

**Example:**
```rust
use cosmoflow::action::Action;
use serde_json::json;
use std::collections::HashMap;

// Simple action (most common)
let simple_action = Action::simple("next_node");

// Parameterized action for conditional routing
let mut params = HashMap::new();
params.insert("condition_key".to_string(), json!("user_authenticated"));
params.insert("condition_value".to_string(), json!(true));
params.insert("true_action".to_string(), json!("authenticated_flow"));
params.insert("false_action".to_string(), json!("login_flow"));
let conditional_action = Action::with_params("conditional", params);
```

### Shared Store

The `cosmoflow::shared_store` module provides type-safe data sharing between nodes with a simple `get()`/`set()` API.

**Key Features:**
- Type-safe operations
- Automatic serialization
- Storage backend agnostic
- Zero-copy where possible

**Example:**
```rust
// Store data
store.set("user_id", &user.id).await?;

// Retrieve data
let user_id: String = store.get("user_id").await?;
```

### Storage Backends

The `cosmoflow::storage` module provides pluggable storage implementations:

- **Memory**: Fast, non-persistent storage
- **File**: Persistent file-based storage
- **Custom**: Implement your own backends

## Data Flow

1. **Flow Initialization**: Create flow with nodes and routes
2. **Execution Start**: Begin with entry node
3. **Node Execution**: Execute current node with context
4. **Action Evaluation**: Determine next node based on action
5. **Data Sharing**: Nodes communicate via shared store
6. **Flow Completion**: Process completes or terminates

## Error Handling

CosmoFlow provides comprehensive error handling at multiple levels:

- **Node Level**: Individual node execution errors
- **Flow Level**: Workflow orchestration errors  
- **Storage Level**: Data persistence errors
- **Action Level**: Routing and condition errors

## Performance Considerations

- **Async/Await**: Full async support for concurrent operations
- **Zero-Copy**: Efficient data handling where possible
- **Memory Management**: Configurable storage backends
- **Type Safety**: Compile-time optimizations

## Extensibility

### Custom Nodes

Implement the `NodeBackend` trait:

```rust
struct CustomNode {
    config: MyConfig,
}

#[async_trait]
impl NodeBackend for CustomNode {
    async fn execute(&self, ctx: &ExecutionContext) -> Result<()> {
        // Custom logic
        Ok(())
    }
}
```

### Custom Storage

Implement the `Storage` trait:

```rust
struct CustomStorage;

#[async_trait]
impl Storage for CustomStorage {
    async fn get(&self, key: &str) -> Result<Option<String>> {
        // Custom get logic
    }
    
    async fn set(&self, key: &str, value: &str) -> Result<()> {
        // Custom set logic
    }
}
```

## Best Practices

1. **Keep Nodes Focused**: Each node should have a single responsibility
2. **Use Type Safety**: Leverage Rust's type system for correctness
3. **Handle Errors Gracefully**: Implement proper error handling
4. **Design for Reusability**: Create composable, reusable components
5. **Test Thoroughly**: Unit test nodes and integration test flows

## Next Steps

- Explore the [Features Guide](features.md) for configuration options
- Read the [API documentation](https://docs.rs/cosmoflow) for detailed reference
