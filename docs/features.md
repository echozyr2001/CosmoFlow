# Features Guide

This guide covers CosmoFlow's feature system and configuration options.

## 🎯 Feature Configurations

### Minimal Configuration
```toml
[dependencies]
cosmoflow = { version = "0.2.0", default-features = false, features = ["minimal"] }
```
- **Features**: None (truly minimal)
- **Storage**: No storage backends enabled
- **Built-ins**: No built-in nodes
- **Use case**: When you want to implement everything yourself or use as a core library

### Basic Configuration
```toml
[dependencies]
cosmoflow = { version = "0.2.0", default-features = false, features = ["basic"] }
```
- **Features**: Memory storage only
- **Storage**: In-memory storage backend
- **Built-ins**: No built-in nodes
- **Use case**: Simple workflows that don't need persistence

### Standard Configuration (Recommended)
```toml
[dependencies]
cosmoflow = { version = "0.2.0", default-features = false, features = ["standard"] }
```
- **Features**: Memory storage + built-in nodes
- **Storage**: In-memory storage backend
- **Built-ins**: All built-in node types
- **Use case**: Most applications that need quick setup and common functionality

### Full Configuration
```toml
[dependencies]
cosmoflow = { version = "0.2.0", default-features = false, features = ["full"] }
```
- **Features**: All storage backends + built-in nodes
- **Storage**: Memory and file-based storage
- **Built-ins**: All built-in node types
- **Use case**: Applications that need all features and flexibility

## 🧩 Individual Features

### Storage Backend Features
- `storage-memory`: Enable in-memory storage (fast, non-persistent)
- `storage-file`: Enable file-based storage (persistent, disk-based)
- `storage-full`: Enable all storage backends

### Functionality Features
- `builtin`: Enable built-in node types (SetValue, GetValue, Delay, Log)
- `builtin-full`: Enable all built-in functionality

## 🛠️ Custom Combinations

You can mix and match features for your specific needs:

```toml
# Memory storage + built-ins
cosmoflow = { version = "0.2.0", default-features = false, features = ["storage-memory", "builtin"] }

# File storage only
cosmoflow = { version = "0.2.0", default-features = false, features = ["storage-file"] }

# Both storage backends, no built-ins
cosmoflow = { version = "0.2.0", default-features = false, features = ["storage-full"] }
```

## 📊 Feature Comparison

| Configuration | Binary Size | Compile Time | Memory Storage | File Storage | Built-ins | Best For |
|---------------|-------------|--------------|----------------|--------------|-----------|----------|
| minimal       | Smallest    | Fastest      | ❌             | ❌           | ❌        | Core library usage |
| basic         | Small       | Fast         | ✅             | ❌           | ❌        | Simple workflows |
| standard      | Medium      | Medium       | ✅             | ❌           | ✅        | Most applications |
| full          | Largest     | Slowest      | ✅             | ✅           | ✅        | Feature-rich apps |

## 🚀 Migration Guide

If you're currently using CosmoFlow with all features, you can:

1. **Keep current behavior**: Use `features = ["full"]`
2. **Optimize for your use case**: Choose `standard` for most apps
3. **Minimize dependencies**: Use `basic` or `minimal` for specific needs

The default configuration is now empty (`default = []`), so you must explicitly choose features when adding CosmoFlow as a dependency.