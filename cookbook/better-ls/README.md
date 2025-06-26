# CosmoFlow Better-LS

An enhanced directory listing tool built with the **CosmoFlow** framework, showcasing CosmoFlow's core features and capabilities.

## 🌟 CosmoFlow Features Showcase

### 1. **Declarative Workflow Definition**
Using CosmoFlow's `flow!` macro to define clear workflows declaratively:

```rust
let mut flow = flow! {
    storage: MemoryStorage,
    start: "input",
    nodes: {
        "input": InputNode::new(),
        "ls": LsNode,
        "output": OutputNode,
    },
    routes: {
        "input" - "list" => "ls",
        "ls" - "output" => "output",
    },
    terminals: {
        "output" - "complete",
    }
};
```

### 2. **Type-Safe Data Flow**
Each node has strongly-typed preparation, execution, and post-processing phases:

```rust
impl Node<MemoryStorage> for LsNode {
    type PrepResult = Args;           // Type-safe preparation result
    type ExecResult = Vec<FileEntry>; // Type-safe execution result
    type Error = NodeError;           // Unified error handling
    
    fn prep(&mut self, store: &MemoryStorage, _context: &ExecutionContext) 
        -> Result<Self::PrepResult, Self::Error> { ... }
        
    fn exec(&mut self, prep_result: Self::PrepResult, _context: &ExecutionContext) 
        -> Result<Self::ExecResult, Self::Error> { ... }
        
    fn post(&mut self, store: &mut MemoryStorage, _prep_result: Self::PrepResult, 
           exec_result: Self::ExecResult, _context: &ExecutionContext) 
        -> Result<Action, Self::Error> { ... }
}
```

### 3. **Shared Storage Mechanism**
Safe data transfer between nodes through built-in shared storage:

- `InputNode` parses command-line arguments and stores them in `"args"`
- `LsNode` retrieves arguments, processes directory, and stores file list in `"file_listing"`
- `OutputNode` retrieves data, formats output, and displays beautiful tables

### 4. **Built-in Validation Mechanism**
CosmoFlow validates workflow integrity before execution:

```rust
if let Err(e) = flow.validate() {
    eprintln!("❌ Flow validation failed: {e}");
    return Err(e.into());
}
```

### 5. **Flexible Routing System**
Define routes between nodes using concise syntax:

```rust
routes: {
    "input" - "list" => "ls",        // InputNode connects to LsNode via "list" action
    "ls" - "output" => "output",     // LsNode connects to OutputNode via "output" action
}
```

### 6. **Built-in Validation and Error Handling**
CosmoFlow provides comprehensive validation and error handling mechanisms:

```rust
// Built-in validation mechanism
if let Err(e) = flow.validate() {
    eprintln!("❌ Flow validation failed: {e}");
    return Err(e.into());
}

// Unified error handling
if let Err(e) = flow.execute(&mut storage) {
    eprintln!("❌ Execution failed: {e}");
    return Err(e.into());
}
```

## 🚀 Usage Examples

```bash
# Basic usage
cargo run

# Show hidden files
cargo run -- -a

# Sort by modification time
cargo run -- -t

# Reverse sort order
cargo run -- -r

# Specify directory
cargo run -- /path/to/directory

# Combine options
cargo run -- -at /path/to/directory
```

## 📊 Output Features

The program displays beautiful table output with the following characteristics:

1. **Beautiful Table Format**: Generated with tabled crate featuring rounded borders
2. **Rich File Information**: Type, name, size, permissions, modification time, creation time
3. **Color Output**: Different file types distinguished by different colors
4. **Smart Sorting**: Support for sorting by name or time, with reverse sorting
5. **Human-readable Sizes**: Automatic conversion to KB, MB, GB units

### Output Example

```
╭──────┬─────────────┬───────┬─────────────┬──────────────────┬──────────────────╮
│ Type │ Name        │  Size │ Permissions │ Modified         │ Created          │
├──────┼─────────────┼───────┼─────────────┼──────────────────┼──────────────────┤
│ DIR  │ src         │ <DIR> │ rwxr-xr-x   │ 2025-06-26 06:36 │ 2025-06-26 04:38 │
│ FILE │ Cargo.toml  │ 251 B │ rw-r--r--   │ 2025-06-26 06:32 │ 2025-06-26 04:38 │
│ FILE │ README.md   │ 4.2K  │ rw-r--r--   │ 2025-06-26 07:15 │ 2025-06-26 05:20 │
╰──────┴─────────────┴───────┴─────────────┴──────────────────┴──────────────────╯
```

## 🎯 CosmoFlow Advantages

- **🔒 Type Safety**: Compile-time assurance of data flow correctness
- **🔧 Modularity**: Each node is an independent, reusable component
- **📊 Observability**: Built-in execution tracking and context management
- **🚀 Performance**: Zero-cost abstractions with minimal runtime overhead
- **🔄 Extensibility**: Easy addition of new nodes and routes
- **🛡️ Error Handling**: Unified error handling mechanism
- **💾 Data Sharing**: Flexible shared storage backends

## 🏗️ Architecture Highlights

This demo project shows how to build a practical command-line tool using CosmoFlow while maintaining code clarity and maintainability. The workflow contains three core nodes:

- **InputNode**: Command-line argument parsing and path validation
- **LsNode**: File system scanning and metadata collection
- **OutputNode**: Data formatting and beautiful display

### Workflow Execution Process

```
InputNode → LsNode → OutputNode
    ↓         ↓         ↓
Parse Args  Scan Dir  Format Output
Validate    Collect   Display Table
Path        Metadata
```

This concise three-node design embodies CosmoFlow's core philosophy: **Simple, Clear, Efficient**. Each node has a single responsibility, clear data flow, easy to understand and maintain.

## 💡 CosmoFlow Design Advantages

Through this practical case, we can see CosmoFlow's design advantages:

1. **Declarative Definition**: Workflow structure is clear at a glance
2. **Type Safety**: Compile-time checks ensure correct data types
3. **Separation of Concerns**: Each node focuses on a single responsibility
4. **Data Sharing**: Safe data transfer through shared storage
5. **Error Handling**: Unified error handling mechanism
6. **Easy Extension**: Can easily add new nodes or modify processes

This allows complex business logic to be decomposed into simple, reusable components, greatly improving code maintainability and testability.
