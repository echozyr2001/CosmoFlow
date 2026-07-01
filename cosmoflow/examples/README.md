# CosmoFlow Examples

This directory contains small examples for the main CosmoFlow core API. They prioritize clarity over advanced integrations.

## Learning Path

Start here to understand CosmoFlow fundamentals:

### 1. **hello_world.rs** - Your First Workflow
- Node implementation with `prep -> exec -> post`
- Flow execution with natural termination
- `SharedStore` as an optional key-value state model

### 2. **simple_loops.rs** - Control Flow Patterns  
- Loops as ordinary state-machine routes
- Action names as routing identity
- Termination when an action has no matching route

### 3. **custom_node.rs** - Advanced Node Implementation
- Strongly typed workflow state
- Multi-node flow composition
- Keeping domain data in an ordinary Rust struct

## Running Examples

Run examples in sync mode:

```bash
# Start with the basics
cargo run --example hello_world

# Learn control flow patterns
cargo run --example simple_loops

# Advanced node customization
cargo run --example custom_node
```

Run the same examples with the async API:

```bash
cargo run --features async --example hello_world
cargo run --features async --example simple_loops
cargo run --features async --example custom_node
```

## Ready for Production?

Once you've mastered these basics, explore the **`../cookbook/`** directory for:

- **async-workflows/** - Async patterns with FlowBuilder
- **chat-assistant/** - Production chat applications  
- **llm-request-handler/** - LLM integration patterns
- **unified-workflow/** - Complex workflow compositions
