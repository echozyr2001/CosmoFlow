# CosmoFlow Examples

This directory contains comprehensive examples demonstrating CosmoFlow's capabilities from basic concepts to advanced patterns. These examples are designed to provide a progressive learning path for understanding CosmoFlow workflows.

## ✅ Available Examples

### 1. Hello World (`hello_world.rs`)

A simple greeting workflow that introduces fundamental CosmoFlow concepts with minimal code.

```bash
cd examples && cargo run --bin hello_world --features minimal
```

---

### 2. Custom Node (`custom_node.rs`)

An sophisticated iterative workflow demonstrating stateful nodes, data analysis, and report generation.

```bash
cd examples && cargo run --bin custom_node --features minimal
```

---

### 4. LLM Request (`llm_request.rs`)

A real-world example demonstrating LLM API integration with environment-based configuration and comprehensive error handling.

```bash
# Set environment variables first
export LLM_API_KEY="your_api_key"
export LLM_BASE_URL="https://api.openai.com/v1"
export LLM_MODEL="gpt-3.5-turbo"

# Run the example
cargo run --bin llm_request --features basic
```

**Features:**
- Real HTTP API calls to LLM providers (OpenAI, Anthropic, Local APIs)
- Environment variable configuration
- Lightweight HTTP client without heavy SDK dependencies
- Comprehensive error handling and retry logic
- Copyable patterns for production use

See `lightweight_llm_README.md` for detailed setup instructions.

---

### 6. Custom Node (Sync Version) (`custom_node_sync.rs`)

A synchronous version of the advanced iterative workflow demonstrating stateful nodes, data analysis, and report generation without async overhead.

```bash
cargo run --bin custom_node_sync --no-default-features --features cosmoflow/storage-memory
```

**Features:**
- Sync node implementation for faster compilation
- Advanced iterative counter workflow with statistical analysis
- Built-in MemoryStorage backend for simplicity
- Individual node execution without Flow complexity
- Perfect for CPU-intensive statistical computations

---

### 7. Unified Hello World (Sync Version) (`unified_hello_world_sync.rs`)

A synchronous version demonstrating the unified Node trait API without async complexity.

```bash
cargo run --bin unified_hello_world_sync --no-default-features --features cosmoflow/storage-memory
```

**Features:**
- Unified Node trait implementation in sync mode
- Built-in MemoryStorage backend
- Faster compilation and smaller binary size
- Perfect for CPU-intensive workflows
- Simpler debugging without async complexity

---

### 8. Flow Composition (Sync Version) (`flow_composition_sync.rs`)

A synchronous version demonstrating manual node orchestration and decision-based routing without the flow macro.

```bash
cargo run --bin flow_composition_sync --no-default-features --features cosmoflow/storage-memory
```

**Features:**
- Manual workflow orchestration without async
- Decision-based routing with explicit control flow
- Multiple execution paths with convergence
- Perfect for CPU-intensive decision workflows
- No flow macro dependency for maximum control

---

### 5. Flow Macro (`flow_macro.rs`)

Demonstrates the powerful `flow!` macro for declarative workflow construction with custom action routing.

```bash
cd examples && cargo run --bin flow_macro --features basic
```

**Syntax Example:**

```rust
let workflow = flow! {
    storage: SimpleStorage,
    start: "decision",
    nodes: {
        "decision" : DecisionNode,
        "success_path" : SuccessNode,
        "error_path" : ErrorNode,
        "final" : FinalNode,
    },
    routes: {
        "decision" - "default" => "success_path",
        "decision" - "error" => "error_path",
        "success_path" - "continue" => "final",
        "error_path" - "continue" => "final",
    }
};
```

## 🔧 Additional Examples

```bash
cargo run --bin unified_hello_world --features basic
cargo run --bin unified_counter --features basic  
```

## 🏗️ Feature Levels

CosmoFlow provides different feature tiers to match various use cases:

| Feature Level | Description | Storage | Built-ins | Use Case |
|---------------|-------------|---------|-----------|----------|
| **minimal** | Core workflow functionality only | Custom only | None | Learning, custom implementations |
| **basic** | Adds memory storage backend | Custom + Memory | None | Development, testing |
| **standard** | Adds built-in node types | Custom + Memory | Basic nodes | Common workflows |
| **full** | Complete feature set | All backends | All nodes | Production applications |

## 📋 Best Practices

### Node Implementation

1. **Three-Phase Pattern**:
   - `prep()`: Read and validate inputs, prepare resources
   - `exec()`: Perform core logic (keep stateless and idempotent when possible)
   - `post()`: Write outputs, update state, and determine next action

2. **Error Handling**:
   - Use appropriate error types for different failure modes
   - Implement retry logic for transient failures
   - Provide meaningful error messages and fallback behavior

3. **State Management**:
   - Use SharedStore for inter-node data communication
   - Keep node internal state minimal and recoverable
   - Consider persistence strategies for long-running workflows

### Flow Design

1. **Workflow Structure**:
   - Always validate flows before execution
   - Use meaningful and consistent node/action naming
   - Design and test all possible execution paths

2. **Performance Considerations**:
   - Set appropriate `max_steps` limits to prevent infinite loops
   - Simple max_steps protection for iterative workflows
   - Monitor execution metrics and optimize bottlenecks

3. **Maintainability**:
   - Document complex routing logic and business rules
   - Use consistent naming conventions across the project
   - Provide clear error messages and logging

## 🎯 Next Steps

After working through these examples, consider:

1. **Explore Built-in Features**: Try the `standard` and `full` feature sets
2. **Build Custom Workflows**: Combine patterns from multiple examples
3. **Production Deployment**: Implement proper error handling and monitoring
4. **Advanced Storage**: Explore Redis, file-based, or custom storage backends

## 📚 Additional Resources

- **Architecture**: `/docs/architecture.md` - System design and concepts
- **Features**: `/docs/features.md` - Complete feature documentation
- **Getting Started**: `/docs/getting-started.md` - Detailed setup guide
- **Built-in Nodes**: `/cosmoflow/src/builtin/` - Pre-built node implementations
- **Storage Backends**: `/cosmoflow/src/storage/` - Storage implementation examples

## 🤝 Contributing

Found an issue or want to improve an example? Check out the main project repository for contribution guidelines.