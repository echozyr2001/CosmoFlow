# Better LS

`better-ls` is a small directory listing application built with CosmoFlow. It keeps normal CLI behavior while organizing the program as a flow:

```text
InputNode -> LsNode -> OutputNode
```

## What It Demonstrates

- CLI argument parsing as the first node in a flow.
- Filesystem traversal and sorting inside a workflow node.
- Shared data transfer through `MemoryStorage`.
- Natural flow termination when `OutputNode` returns `complete` without a matching route.

## Why CosmoFlow Fits

The application has distinct phases with clear data handoff: parse input, scan the filesystem, then format output. CosmoFlow keeps those phases explicit without pretending the tool is a complex orchestration system. If this were only a direct `read_dir` wrapper, it would not belong in the cookbook.

## Run

From the workspace root:

```bash
cargo run -p better-ls -- .
cargo run -p better-ls -- -a .
cargo run -p better-ls -- -t .
cargo run -p better-ls -- -r .
cargo run -p better-ls -- --no-human-readable .
```

## Options

- `PATH`: directory to list, defaults to the current directory.
- `-a`, `--all`: show hidden files.
- `-t`, `--time`: sort by modification time.
- `-r`, `--reverse`: reverse sort order.
- `--no-human-readable`: show raw byte sizes.

## Flow Shape

The app uses `FlowBuilder` directly:

```rust
FlowBuilder::new()
    .node("input", InputNode::new())
    .node("ls", LsNode)
    .node("output", OutputNode)
    .route("input", "list", "ls")
    .route("ls", "output", "output")
    .build()?;
```

`complete` is not a special terminal action in CosmoFlow core. It terminates here because the final node has no route for that action.
