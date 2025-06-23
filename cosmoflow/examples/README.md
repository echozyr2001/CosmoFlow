# CosmoFlow Examples

This directory contains **simple examples** designed for learning CosmoFlow's core concepts. These examples prioritize clarity and fast compilation over advanced features.

## Learning Path

Start here to understand CosmoFlow fundamentals:

### 1. **hello_world.rs** - Your First Workflow
- Basic node creation and execution
- Data flow between nodes using SharedStore
- Understanding the core CosmoFlow concepts

### 2. **simple_loops.rs** - Control Flow Patterns  
- Loop constructs and iteration patterns
- Conditional execution and state management
- Building more complex workflows step by step

### 3. **custom_node.rs** - Advanced Node Implementation
- Creating custom nodes with complex logic
- Custom storage backends and data persistence
- Statistical analysis and data aggregation patterns

## Running Examples

All examples use **sync-only features** for minimal setup:

```bash
# Start with the basics
cargo run --example hello_world

# Learn control flow patterns
cargo run --example simple_loops

# Advanced node customization
cargo run --example custom_node
```

## Ready for Production?

Once you've mastered these basics, explore the **`../cookbook/`** directory for:

- **async-workflows/** - Advanced async patterns with FlowBuilder
- **chat-assistant/** - Production chat applications  
- **llm-request-handler/** - LLM integration patterns
- **unified-workflow/** - Complex workflow compositions
