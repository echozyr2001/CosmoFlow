# Async Workflows

Advanced asynchronous workflow patterns using CosmoFlow with async/await.

## Examples

- **Flow Builder Example**: Demonstrates declarative workflow construction with custom action routing using conditional paths, error handling, and convergence patterns. Uses CosmoFlow's built-in MemoryStorage for optimal performance.

## Features Demonstrated

- **Async Node Implementation**: Full async/await support with tokio
- **Custom Action Routing**: Named actions like "default", "error", "continue" for explicit control flow
- **Decision-Based Routing**: Conditional workflow paths based on business logic
- **Path Convergence**: Multiple execution paths that merge at common endpoints
- **Built-in MemoryStorage**: High-performance in-memory storage with automatic JSON serialization
- **Error Handling**: Comprehensive error management in async contexts

## Running Examples

```bash
cd cookbook/async-workflows
cargo run  # Runs the advanced flow example
```

## Performance Characteristics

- **Async Runtime**: Uses tokio for async execution
- **Concurrent Execution**: Supports parallel node execution (when implemented)
- **Complex Workflows**: Handles multi-path, conditional workflows
- **Resource Efficiency**: Async/await for I/O-bound operations

This cookbook demonstrates production-ready async patterns for complex workflows that require conditional routing, error handling, and async I/O operations.
