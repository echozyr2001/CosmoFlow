# CosmoFlow Examples

This directory contains comprehensive examples demonstrating CosmoFlow's minimal feature set and core workflow capabilities. These examples show how to build workflows using only the core framework without relying on built-in dependencies.

## ✅ Working Examples

### 1. Hello World (`hello_world.rs`)
**Feature Required:** `minimal` (core only)

A simple greeting workflow that demonstrates the fundamental concepts of CosmoFlow with minimal code.

```bash
cd examples && cargo run --bin hello_world --features minimal
```

**Key Features Demonstrated:**
- Custom `StorageBackend` trait implementation
- Basic node backend implementation with prep, exec, and post phases
- Core workflow execution without any built-ins
- Data communication between nodes via SharedStore
- Sequential workflow with simple routing
- Type-safe data storage and retrieval

**Workflow Description:**
This example implements a basic greeting system:
1. Hello node generates and displays a greeting message
2. Greeting is stored in the shared storage
3. Response node retrieves the greeting from storage
4. Response node generates and displays a response message
5. Workflow completes successfully

Perfect for understanding CosmoFlow's core concepts and getting started.

### 2. Custom Node (`custom_node.rs`)
**Feature Required:** `minimal` (core only)

Demonstrates an advanced iterative workflow with custom stateful nodes, data analysis, and report generation.

```bash
cd examples && cargo run --bin custom_node --features minimal
```

**Key Features Demonstrated:**
- Stateful nodes that maintain internal state across executions
- Complex business logic with coordinator pattern
- Iterative workflows with cycle detection disabled
- Three-phase node execution (prep, exec, post)
- Data persistence and history tracking
- Statistical analysis and report generation
- Custom storage backend implementation

**Workflow Description:**
This example implements a sophisticated counter analysis system:
1. A coordinator node manages the workflow iteration logic
2. Two counter nodes (main: +5, secondary: +3) alternate execution
3. Counters persist their state and build execution history
4. After 6 total iterations, the workflow transitions to analysis
5. Statistics node calculates averages, growth rates, and other metrics
6. Report node generates a formatted analysis summary

## Overview

CosmoFlow provides several feature tiers to match different use cases:

- **minimal**: Core workflow functionality only
- **basic**: Includes memory storage backend  
- **standard**: Adds built-in node types
- **full**: Complete feature set with all storage backends and built-ins

## Running the Examples

Both examples use only the minimal feature set, demonstrating CosmoFlow's core capabilities:

```bash
cd examples

# Simple greeting workflow (perfect for beginners)
cargo run --bin hello_world --features minimal

# Advanced iterative workflow with data analysis
cargo run --bin custom_node --features minimal
```

## Best Practices

### Node Implementation

1. **Separation of Concerns**:
   - `prep()`: Read and validate inputs
   - `exec()`: Perform core logic (stateless, idempotent)
   - `post()`: Write outputs and determine next action

2. **Error Handling**:
   - Use appropriate error types for different phases
   - Implement retry logic for transient failures
   - Provide fallback behavior when possible

3. **State Management**:
   - Use SharedStore for data sharing between nodes
   - Keep node state minimal and recoverable
   - Consider persistence for long-running workflows

### Flow Design

1. **Validation**:
   - Always validate flows before execution
   - Use meaningful node and action names
   - Test all possible execution paths

2. **Performance**:
   - Set appropriate max_steps limits
   - Consider cycle detection overhead
   - Monitor execution metrics

3. **Maintainability**:
   - Document complex routing logic
   - Use consistent naming conventions
   - Provide clear error messages

## Next Steps

After exploring these examples, you might want to:

1. **Try Basic Features**: Use built-in storage backends and node types
2. **Explore Standard Features**: Leverage pre-built nodes for common tasks  
3. **Build Complex Workflows**: Combine multiple patterns for real applications
4. **Custom Storage**: Implement custom storage backends for your use case

For more advanced examples and documentation, see:
- `/docs/` - Architectural documentation
- `/cosmoflow/src/builtin/` - Built-in node implementations
- `/cosmoflow/src/storage/` - Storage backend examples