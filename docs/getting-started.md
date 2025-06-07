# Getting Started with CosmoFlow

This guide will help you get up and running with CosmoFlow quickly.

## Installation

Add CosmoFlow to your `Cargo.toml`:

```toml
[dependencies]
cosmoflow = { version = "0.1.0", features = ["standard"] }
tokio = { version = "1.0", features = ["full"] }
```

## Your First Workflow

Here's a simple example to get you started:

```rust
use cosmoflow::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a shared store with memory backend
    let mut store = SharedStore::memory();

    // Create a flow builder
    let flow = Flow::builder("hello-world")
        .with_store(&mut store)
        .build()?;

    // Execute the workflow
    let context = ExecutionContext::new(3, Duration::from_secs(30));
    let result = flow.execute(&mut store).await?;
    
    println!("Workflow completed: {:?}", result);
    Ok(())
}
```

## Feature Selection

CosmoFlow offers different feature sets to optimize your build:

- **`minimal`**: Core functionality only (~50KB)
- **`basic`**: Core + memory storage (~100KB)
- **`standard`**: Memory storage + built-ins (~200KB) ⭐ *Recommended*
- **`full`**: All features enabled (~500KB)

See the [Features Guide](features.md) for detailed information.

## Next Steps

- Learn about the [Architecture](architecture.md)
- Explore the [API Reference](https://docs.rs/cosmoflow)
- Read about [Contributing](../CONTRIBUTING.md)

## Common Patterns

### Basic Node Creation

```rust
use cosmoflow::prelude::*;

struct MyNode;

#[async_trait::async_trait]
impl<S> NodeBackend<S> for MyNode 
where 
    S: StorageBackend + Send + Sync 
{
    async fn execute(
        &self, 
        _context: &ExecutionContext, 
        _store: &mut SharedStore<S>
    ) -> Result<Action, NodeError> {
        // Your node logic here
        Ok(Action::simple("next_node"))
    }
}
```

### Error Handling

```rust
use cosmoflow::{FlowError, Result};

match flow.execute(context).await {
    Ok(result) => println!("Success: {:?}", result),
    Err(FlowError::NodeError { node_id, source }) => {
        eprintln!("Node '{}' failed: {}", node_id, source);
    },
    Err(e) => eprintln!("Other error: {}", e),
}
```