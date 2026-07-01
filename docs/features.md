# Features

CosmoFlow keeps its default feature set small. The default crate contains the
core state-machine model: `Action`, `Node`, `Flow`, and the `SharedStore` trait.
Async execution and built-in shared-store backends are enabled explicitly.

## Default

```toml
[dependencies]
cosmoflow = "0.5.1"
```

This enables no optional runtime capability. It is the right starting point when
your flow state is a typed Rust struct.

## Async

### `async`

Enables the asynchronous node and flow APIs.

Use this when node phases need `.await`, external async I/O, or integration with
an async runtime:

```toml
[dependencies]
cosmoflow = { version = "0.5.1", features = ["async"] }
async-trait = "0.1"
```

Async state types must satisfy `Send + Sync` because async futures may be moved
across executor threads. This is a future-safety boundary, not a shared-store
requirement.

## Shared Store Backends

Storage backends implement the optional `SharedStore` key-value state model.
They do not change the core `Node<S>` or `Flow<S>` model; a plain Rust struct can
still be used as state without any storage backend.

### `storage-memory`

Enables `MemoryStorage`.

```toml
[dependencies]
cosmoflow = { version = "0.5.1", features = ["storage-memory"] }
```

Use this for tests, examples, and applications where in-memory dynamic key-value
state is enough.

### `storage-file`

Enables `FileStorage`.

```toml
[dependencies]
cosmoflow = { version = "0.5.1", features = ["storage-file"] }
```

Use this when a simple local JSON-backed shared store fits the application.

### `storage-redis`

Enables `RedisStorage` and the optional Redis dependency.

```toml
[dependencies]
cosmoflow = { version = "0.5.1", features = ["storage-redis"] }
```

Use this when shared state needs an external Redis backend.

### `storage-full`

Enables all built-in storage backends:

```toml
[dependencies]
cosmoflow = { version = "0.5.1", features = ["storage-full"] }
```

This is a convenience feature for applications that intentionally want every
built-in backend available.

## Common Combinations

```toml
# Async core without built-in storage
cosmoflow = { version = "0.5.1", features = ["async"] }

# Memory storage without async
cosmoflow = { version = "0.5.1", features = ["storage-memory"] }

# Async flow plus memory storage
cosmoflow = { version = "0.5.1", features = ["async", "storage-memory"] }

# Async flow plus all built-in storage backends
cosmoflow = { version = "0.5.1", features = ["async", "storage-full"] }
```

## Comparison

| Feature set | Async | Memory | File | Redis | Best fit |
| --- | --- | --- | --- | --- | --- |
| default | No | No | No | No | Typed state and minimal dependencies |
| `async` | Yes | No | No | No | Async nodes with typed state |
| `storage-memory` | No | Yes | No | No | Dynamic in-memory shared state |
| `storage-full` | No | Yes | Yes | Yes | All built-in shared-store backends |
| `async`, `storage-full` | Yes | Yes | Yes | Yes | Async apps that need all built-in backends |
