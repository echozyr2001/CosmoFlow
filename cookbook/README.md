# CosmoFlow Cookbook

Each directory contains a complete, standalone project demonstrating how to solve specific problems in production environments.

## Projects

### **async-workflows** - Advanced Async Patterns
Complete async workflow implementation with:
- **FlowBuilder API** for complex workflow construction
- **Conditional routing** with custom actions (`"default"`, `"error"`, `"continue"`)
- **Decision nodes** and path convergence patterns
- **Custom storage backends** with JSON serialization
- **Error handling** in async contexts

**Use Case**: Building complex, multi-path workflows that require async I/O operations.

### **chat-assistant** - Conversational AI
Production chat assistant implementation featuring:
- Real-time conversation management
- Context preservation across interactions
- Integration with language models
- Session state management

**Use Case**: Building chatbots, virtual assistants, and conversational AI applications.

### **llm-request-handler** - LLM Integration
Efficient LLM request processing system with:
- Request batching and optimization
- Rate limiting and error handling
- Response parsing and validation
- Multiple provider support

**Use Case**: Integrating large language models into production applications.

### **unified-workflow** - Complex Compositions
Advanced workflow composition patterns:
- Mixed sync/async operations
- Multi-stage data processing
- Parallel execution patterns
- Resource management

**Use Case**: Complex data processing pipelines and orchestration systems.

## 🚀 Quick Start

Each project is fully standalone. Navigate to any directory and run:

```bash
# Choose your use case
cd async-workflows     # For complex async workflows
cd chat-assistant      # For conversational AI
cd llm-request-handler # For LLM integration  
cd unified-workflow    # For complex compositions

# Run immediately
cargo run
```

Or from the root workspace:

```bash
cargo run --bin async-workflows
cargo run --bin chat-assistant  
cargo run --bin llm-request-handler
cargo run --bin unified-workflow
```

## Learning Path

**New to CosmoFlow?** Start with the simple examples first:

1. **Begin with `../cosmoflow/examples/`** - Learn core concepts with sync examples
2. **Then explore cookbook projects** - Apply knowledge to production patterns
3. **Pick projects matching your use case** - Focus on relevant patterns

## Adding New Cookbook Projects

```bash
# 1. Create your project directory
mkdir cookbook/your-project-name
cd cookbook/your-project-name

# 2. Initialize as a Rust project
cargo init --name your-project-name

# 3. That's it! The workspace automatically discovers it
cargo run  # Works immediately!
```

**Requirements for cookbook projects:**
- ✅ **Production-ready** - Code quality suitable for real applications
- ✅ **Well-documented** - Clear README explaining the use case
- ✅ **Complete examples** - Full implementation, not just snippets
- ✅ **Real-world focus** - Solve actual problems developers face

## Contributing

When adding new cookbook examples:

1. Create a new directory with a descriptive name in the `cookbook/` folder
2. The directory will be **automatically discovered** by the workspace (no need to edit Cargo.toml files!)
3. Include a comprehensive README explaining the use case
4. Ensure the example is production-ready and well-documented

**Example**: To add a new cookbook project called `api-server`:
```bash
# Create the project directory
mkdir cookbook/api-server
cd cookbook/api-server

# Initialize with cargo
cargo init --name api-server

# The project is automatically included in the workspace!
# You can immediately run:
cargo run
```
