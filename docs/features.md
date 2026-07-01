# Features

CosmoFlow uses Cargo features to keep the core small and make storage and async
support opt-in.

## Core Features

### `async`

Enables the asynchronous node and flow APIs.

Use this when node phases need `.await`, external async I/O, or integration with
async runtimes. Async state types must satisfy `Send + Sync` because async
futures may be moved safely across executor threads.

### Storage Backends

Storage backends provide implementations for the optional `SharedStore` model.
They are useful when the workflow state is naturally dynamic key-value data.

- `storage-memory`: in-memory shared store backend.
- `storage-file`: file-backed shared store backend.
- `storage-redis`: Redis-backed shared store backend.
- `storage-full`: enables all storage backends.

These features do not change the core `Node<S>` or `Flow<S>` state model. A
plain Rust struct can still be used as state without any storage backend.

## Convenience Feature Sets

### `minimal`

Core engine only.

```toml
[dependencies]
cosmoflow = { version = "0.5.1", default-features = false, features = ["minimal"] }
```

Use this when application state is a custom type and no built-in storage backend
is needed.

### `basic`

Default feature set with memory storage.

```toml
[dependencies]
cosmoflow = "0.5.1"
```

Equivalent to:

```toml
[dependencies]
cosmoflow = { version = "0.5.1", features = ["basic"] }
```

### `standard`

Memory storage plus async support.

```toml
[dependencies]
cosmoflow = { version = "0.5.1", features = ["standard"] }
```

Use this when most workflow nodes need async execution and a simple shared store
backend is enough.

### `full`

All storage backends plus async support.

```toml
[dependencies]
cosmoflow = { version = "0.5.1", features = ["full"] }
```

Use this for applications that want every built-in backend available.

## Custom Combinations

```toml
# Async core without built-in storage
cosmoflow = { version = "0.5.1", default-features = false, features = ["async"] }

# File storage without async
cosmoflow = { version = "0.5.1", default-features = false, features = ["storage-file"] }

# Redis storage with async
cosmoflow = { version = "0.5.1", default-features = false, features = ["storage-redis", "async"] }

# All storage backends with async
cosmoflow = { version = "0.5.1", default-features = false, features = ["storage-full", "async"] }
```

## Comparison

| Feature set | Async | Memory | File | Redis | Best fit |
| --- | --- | --- | --- | --- | --- |
| `minimal` | No | No | No | No | Strong typed state and no built-in storage |
| `basic` | No | Yes | No | No | Simple synchronous flows |
| `standard` | Yes | Yes | No | No | Async flows with memory shared store |
| `full` | Yes | Yes | Yes | Yes | Applications using multiple storage backends |
