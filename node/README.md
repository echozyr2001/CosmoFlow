# Node Crate

Execution nodes for CosmoFlow workflows. This crate provides the core abstractions and implementations for building workflow nodes that can execute various types of computations.

## Features

This crate supports optional features for different storage backends:

### Storage Backend Features

- `storage-memory`: Enables in-memory storage backend support
- `storage-file`: Enables file-based storage backend support  
- `storage-full`: Enables all storage backends

### Usage

By default, no storage backends are enabled. You need to explicitly enable the features you want to use:

```toml
[dependencies]
node = { version = "0.1.0", features = ["storage-memory"] }
```

For multiple backends:

```toml
[dependencies]
node = { version = "0.1.0", features = ["storage-memory", "storage-file"] }
```

Or enable all backends:

```toml
[dependencies]
node = { version = "0.1.0", features = ["storage-full"] }
```

## Core Components

### ExecutionContext

Represents the execution context for a node, containing retry count, execution metadata, and other execution-related information.

### Node

A concrete implementation that wraps a NodeBackend and provides the complete execution lifecycle (prep → exec → post).

### NodeBackend

The core trait that defines how nodes execute. Implement this trait to create custom node types.

### NodeError

Error types for node operations, including execution errors, storage errors, validation errors, and preparation errors.

## Example

```rust
use node::{Node, NodeBackend, ExecutionContext};
use shared_store::{SharedStore, MemoryStorage};
use action::Action;
use async_trait::async_trait;

struct MyNode {
    message: String,
}

#[async_trait]
impl NodeBackend<MemoryStorage> for MyNode {
    type PrepResult = String;
    type ExecResult = String;
    type Error = node::NodeError;

    async fn prep(
        &mut self,
        store: &SharedStore<MemoryStorage>,
        context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        // Read data from store and prepare for execution
        Ok(format!("Prepared: {}", self.message))
    }

    async fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        // Perform the main computation
        Ok(format!("Executed: {}", prep_result))
    }

    async fn post(
        &mut self,
        store: &mut SharedStore<MemoryStorage>,
        prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        // Write results back to store
        store.set("result".to_string(), exec_result)?;
        Ok(Action::simple("next_step"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = MyNode {
        message: "Hello, World!".to_string(),
    };
    let mut node = Node::new(backend);
    let mut store = SharedStore::with_storage(MemoryStorage::new());
    
    let action = node.run(&mut store).await?;
    println!("Next action: {}", action);
    
    Ok(())
}
```

## Testing

The crate includes comprehensive tests covering all major functionality. To run tests:

```bash
cargo test --package node
```

Note: Tests automatically enable the `storage-memory` feature via dev-dependencies.