# Getting Started with CosmoFlow

This guide will help you get up and running with CosmoFlow quickly.

## Installation

Add CosmoFlow to your `Cargo.toml`:

```toml
[dependencies]
cosmoflow = { version = "0.5.1", features = ["standard"] }
tokio = { version = "1.0", features = ["full"] }
```

## Your First Workflow

Here's a simple example to get you started:

```rust
use cosmoflow::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create a memory-backed storage
    let mut store = MemoryStorage::new();
    
    // 2. Build your workflow
    let mut flow = Flow::builder()
        .start_node("greeting")
        .max_steps(10)
        .terminal_action("complete")
        .build();
    
    // 3. Add nodes to the workflow
    flow.add_node("greeting", log("Hello, CosmoFlow!"));
    flow.add_node("processing", log("Processing data..."));
    flow.add_node("farewell", log("Workflow complete!"));
    
    // 4. Define routing between nodes
    flow.add_route("greeting", Action::simple("next"), "processing");
    flow.add_route("processing", Action::simple("done"), "farewell");
    flow.add_route("farewell", Action::simple("complete"), "");
    
    // 5. Execute the workflow
    let result = flow.execute(&mut store).await?;
    
    // 6. Check results
    println!("Workflow Status: {}", if result.success { "SUCCESS" } else { "FAILED" });
    println!("Steps Executed: {}", result.steps_executed);
    println!("Execution Path: {:?}", result.execution_path);
    
    Ok(())
}
```

## Building Custom Nodes

Create powerful custom nodes with the three-phase execution model:

```rust
use cosmoflow::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct UserData {
    name: String,
    email: String,
}

struct UserValidationNode {
    name: String,
}

#[async_trait::async_trait]
impl<S: SharedStore> Node<S> for UserValidationNode {
    type PrepResult = UserData;
    type ExecResult = bool;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn name(&self) -> &str { &self.name }

    // PREP: Load and validate input data
    async fn prep(&mut self, store: &S, _ctx: &ExecutionContext) 
        -> Result<Self::PrepResult, Self::Error> {
        let user: UserData = store
            .get("user_data")?
            .ok_or("Missing user data")?;
        Ok(user)
    }

    // EXEC: Perform validation logic
    async fn exec(&mut self, user: Self::PrepResult, _ctx: &ExecutionContext) 
        -> Result<Self::ExecResult, Self::Error> {
        let is_valid = !user.name.is_empty() && user.email.contains("@");
        Ok(is_valid)
    }

    // POST: Store results and determine next action
    async fn post(&mut self, store: &mut S, _user: Self::PrepResult, 
                  is_valid: Self::ExecResult, _ctx: &ExecutionContext) 
        -> Result<Action, Self::Error> {
        store.set("validation_result".to_string(), is_valid)?;
        
        if is_valid {
            Ok(Action::simple("user_valid"))
        } else {
            Ok(Action::simple("user_invalid"))
        }
    }
}
```

## Robust Error Handling

CosmoFlow provides comprehensive error handling patterns:

```rust
use cosmoflow::flow::errors::FlowError;

match flow.execute(&mut store).await {
    Ok(result) if result.success => {
        println!("Workflow completed successfully!");
        println!("Executed {} steps in {:?}", 
                 result.steps_executed, result.execution_time);
    }
    
    Ok(result) => {
        println!("Workflow terminated early at: {}", result.last_node_id);
        println!("Final action: {:?}", result.final_action);
    }
    
    Err(FlowError::NodeNotFound(node_id)) => {
        eprintln!("Missing node: '{}'", node_id);
        // Handle missing node configuration
    }
    
    Err(FlowError::MaxStepsExceeded(limit)) => {
        eprintln!("Workflow exceeded {} steps - possible infinite loop", limit);
        // Handle runaway workflows
    }
    
    Err(FlowError::NodeError(msg)) => {
        eprintln!("Node execution failed: {}", msg);
        // Handle node-specific errors
    }
    
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
        // Handle other errors
    }
}
```

## Advanced Patterns

### Conditional Routing

```rust
use cosmoflow::action::Action;
use serde_json::json;
use std::collections::HashMap;

// For conditional routing, use parameterized actions
let mut params = HashMap::new();
params.insert("condition_key".to_string(), json!("user_score"));
params.insert("condition_value".to_string(), json!(80));
params.insert("true_action".to_string(), json!("high_score_path"));
params.insert("false_action".to_string(), json!("low_score_path"));

let conditional_action = Action::with_params("conditional", params);

// Note: For complex conditional logic (greater-than, less-than, etc.),
// use ConditionalNode with closure functions for more flexibility.
```

### Data Communication

```rust
// Store structured data
#[derive(Serialize, Deserialize)]
struct ProcessingConfig {
    threshold: f64,
    max_iterations: usize,
    algorithm: String,
}

let config = ProcessingConfig {
    threshold: 0.95,
    max_iterations: 100,
    algorithm: "advanced".to_string(),
};

store.set("config".to_string(), config)?;

// Retrieve with type safety
let config: ProcessingConfig = store
    .get("config")?
    .ok_or("Configuration not found")?;
```

## Next Steps

Now that you have the basics, explore these advanced topics:

### Architecture Deep Dive
- **[Architecture Guide](architecture.md)**: Understanding CosmoFlow's design
- **[Performance Tuning](features.md)**: Optimizing for your use case

### Advanced Features
- **[Built-in Nodes](https://docs.rs/cosmoflow/latest/cosmoflow/builtin/)**: Ready-to-use components
- **[Storage Backends](https://docs.rs/cosmoflow/latest/cosmoflow/storage/)**: Persistent data solutions
- **[LLM Integration](https://docs.rs/cosmoflow/latest/cosmoflow/builtin/llm/)**: AI-powered workflows

### Community Resources
- **[API Reference](https://docs.rs/cosmoflow)**: Complete API documentation
- **[Examples Repository](../examples/)**: Real-world implementations
- **[Contributing Guide](../CONTRIBUTING.md)**: Join the development

## You're Ready!

Congratulations! You now have everything you need to build powerful, type-safe workflows with CosmoFlow. Start with simple nodes and gradually build up to complex, production-ready applications.

Happy workflow building! 🚀