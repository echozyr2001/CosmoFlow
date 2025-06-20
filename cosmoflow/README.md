# CosmoFlow

A lightweight, type-safe workflow engine for Rust, optimized for LLM applications.

CosmoFlow provides a minimal yet powerful framework for building complex workflows
with clean abstractions and excellent performance. It is:

* **Lightweight**: Minimal dependencies with optional features
* **Type-Safe**: Full Rust type safety with async/await support  
* **LLM-Optimized**: Built-in patterns for AI/LLM workflows
* **Modular**: Enable only what you need through feature flags

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![Documentation][docs-badge]][docs-url]

[crates-badge]: https://img.shields.io/crates/v/cosmoflow.svg
[crates-url]: https://crates.io/crates/cosmoflow
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/echozyr2001/CosmoFlow/blob/main/LICENSE
[docs-badge]: https://docs.rs/cosmoflow/badge.svg
[docs-url]: https://docs.rs/cosmoflow

[Guides](./docs/getting-started.md) |
[API Docs](https://docs.rs/cosmoflow/latest/cosmoflow) |
[Examples](./examples/)

## Overview

**CosmoFlow** is a **next-generation workflow engine** that brings the elegant 
design philosophy of [PocketFlow](https://github.com/The-Pocket/PocketFlow) to 
the Rust ecosystem. Built from the ground up for **LLM applications**, **high-performance scenarios**, 
and **production reliability**. it provides a few major components:

* A lightweight, async-based workflow [scheduler].
* Pluggable storage system (memory, file, Redis).
* Asynchronous [node execution][nodes] with retry logic and error handling.

These components provide the runtime infrastructure necessary for building
complex workflow applications.

[nodes]: https://docs.rs/cosmoflow/latest/cosmoflow/node/index.html
[scheduler]: https://docs.rs/cosmoflow/latest/cosmoflow/flow/index.html

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
cosmoflow = { version = "0.3.0", features = ["storage-memory"] }
```

Create your first workflow:

```rust
use cosmoflow::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create storage
    let mut store = MemoryStorage::new();
    
    // Build workflow  
    let mut flow = FlowBuilder::new()
        .node("start", MyNode::new())
        .terminal_route("start", "complete")
        .build();
    
    // Execute
    let result = flow.execute(&mut store).await?;
    println!("Workflow completed: {:?}", result);
    Ok(())
}
```

To see a list of the available features flags that can be enabled, check our
[docs][feature-flag-docs].

[feature-flag-docs]: https://docs.rs/cosmoflow/#features

## Getting Help

First, see if the answer to your question can be found in the [Guides] or the
[API documentation]. If the answer is not there, there is an active community in
the [CosmoFlow Discord server][Chat]. We would be happy to try to answer your
question. You can also ask your question on [the discussions page][discussions].

[Guides]: ./docs/getting-started.md
[API documentation]: https://docs.rs/cosmoflow/latest/cosmoflow
[Chat]: https://discord.gg/cosmoflow
[discussions]: https://github.com/echozyr2001/CosmoFlow/discussions

## Core Modules

CosmoFlow provides a focused set of core modules:

* [`cosmoflow`]: Main integration and API crate for CosmoFlow workflows.
* [`cosmoflow::flow`]: Workflow orchestration engine for managing complex multi-node workflows.
* [`cosmoflow::node`]: Execution nodes system with async support and retry logic.
* [`cosmoflow::action`]: Control flow logic and condition evaluation.
* [`cosmoflow::shared_store`]: Thread-safe data communication layer between workflow components.

[`cosmoflow`]: https://docs.rs/cosmoflow/latest/cosmoflow
[`cosmoflow::flow`]: https://docs.rs/cosmoflow/latest/cosmoflow/flow/index.html
[`cosmoflow::node`]: https://docs.rs/cosmoflow/latest/cosmoflow/node/index.html
[`cosmoflow::action`]: https://docs.rs/cosmoflow/latest/cosmoflow/action/index.html
[`cosmoflow::shared_store`]: https://docs.rs/cosmoflow/latest/cosmoflow/shared_store/index.html

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/echozyr2001/CosmoFlow/blob/main/LICENSE

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in CosmoFlow by you, shall be licensed as MIT, without any additional
terms or conditions.
