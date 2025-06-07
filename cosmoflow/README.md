# CosmoFlow

A Rust-native, lightweight, and extensible workflow engine for building complex, stateful, and event-driven applications.

## Key Features

* **Extensible and Composable:** Build workflows from reusable nodes.
* **Stateful and Persistent:** Automatically manage workflow state.
* **Async-First:** Built on top of Tokio for high-performance, non-blocking I/O.
* **Type-Safe:** Leverage Rust's type system to prevent entire classes of bugs.
* **Modular Architecture:** Use only what you need through feature flags.

## Modules

This crate contains the following modules:

- `cosmoflow::flow` - Workflow orchestration and execution
- `cosmoflow::node` - Node execution system and traits
- `cosmoflow::action` - Control flow logic and conditions
- `cosmoflow::shared_store` - Type-safe data communication
- `cosmoflow::storage` - Pluggable storage backends
- `cosmoflow::builtin` - Pre-built node components (optional)

## Getting Started

For a complete guide to getting started with CosmoFlow, please see our [documentation](https://docs.rs/cosmoflow).

## License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.
