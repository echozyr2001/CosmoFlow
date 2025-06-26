# Features Guide

This guide covers CosmoFlow's feature system and configuration options.

## 🎯 Feature Configurations

### Minimal Configuration
```toml
[dependencies]
cosmoflow = { version = "0.5.1", default-features = false, features = ["minimal"] }
```
- **Features**: Core engine only
- **Storage**: No storage backends enabled
- **Async**: No async support
- **Use case**: When you want to implement everything yourself or use as a core library

### Basic Configuration (Default)
```toml
[dependencies]
cosmoflow = { version = "0.5.1", features = ["basic"] }
# or simply
cosmoflow = "0.5.1"
```
- **Features**: Memory storage only
- **Storage**: In-memory storage backend
- **Async**: No async support
- **Use case**: Simple workflows that don't need persistence or async

### Standard Configuration (Recommended)
```toml
[dependencies]
cosmoflow = { version = "0.5.1", features = ["standard"] }
```
- **Features**: Memory storage + async support
- **Storage**: In-memory storage backend
- **Async**: Full async/await support
- **Use case**: Most applications that need async workflows

### Full Configuration
```toml
[dependencies]
cosmoflow = { version = "0.5.1", features = ["full"] }
```
- **Features**: All storage backends + async support
- **Storage**: Memory, file, and Redis storage
- **Async**: Full async/await support
- **Use case**: Applications that need all features and flexibility

## 🧩 Individual Features

### Storage Backend Features
- `storage-memory`: Enable in-memory storage (fast, non-persistent)
- `storage-file`: Enable file-based storage (persistent, disk-based)
- `storage-redis`: Enable Redis storage (distributed, persistent)
- `storage-full`: Enable all storage backends

### Core Features
- `async`: Enable async/await support with tokio integration

## 🛠️ Custom Combinations

You can mix and match features for your specific needs:

```toml
# Memory storage + async
cosmoflow = { version = "0.5.1", default-features = false, features = ["storage-memory", "async"] }

# File storage only
cosmoflow = { version = "0.5.1", default-features = false, features = ["storage-file"] }

# All storage backends with async
cosmoflow = { version = "0.5.1", default-features = false, features = ["storage-full", "async"] }

# Redis storage with async
cosmoflow = { version = "0.5.1", default-features = false, features = ["storage-redis", "async"] }
```

## 📊 Feature Comparison

| Configuration | Binary Size | Compile Time | Memory Storage | File Storage | Redis Storage | Async | Best For |
|---------------|-------------|--------------|----------------|--------------|---------------|-------|----------|
| minimal       | Smallest    | Fastest      | ❌             | ❌           | ❌            | ❌    | Core library usage |
| basic         | Small       | Fast         | ✅             | ❌           | ❌            | ❌    | Simple sync workflows |
| standard      | Medium      | Medium       | ✅             | ❌           | ❌            | ✅    | Most applications |
| full          | Largest     | Slowest      | ✅             | ✅           | ✅            | ✅    | Feature-rich apps |

## 🚀 Migration Guide

If you're upgrading from an earlier version:

1. **Keep current behavior**: Use `features = ["full"]` for maximum compatibility
2. **Optimize for your use case**: Choose `standard` for most async apps
3. **Minimize dependencies**: Use `basic` for simple sync workflows or `minimal` for core usage

The default configuration is now `basic` (memory storage only), which provides a good balance of functionality and minimal dependencies.