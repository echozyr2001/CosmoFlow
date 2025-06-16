# CosmoFlow

A runtime for writing reliable, asynchronous, and scalable workflow applications with
the Rust programming language. It is:

* **Fast**: CosmoFlow's zero-cost abstractions give you bare-metal
  performance for workflow orchestration.

* **Reliable**: CosmoFlow leverages Rust's ownership, type system, and
  concurrency model to reduce bugs and ensure thread safety.

* **Scalable**: CosmoFlow has a minimal footprint, and handles backpressure
  and cancellation naturally.

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
[API Docs](https://docs.rs/cosmoflow/latest/cosmoflow)

## Overview

**CosmoFlow** is a **next-generation workflow engine** that brings the elegant 
design philosophy of [PocketFlow](https://github.com/The-Pocket/PocketFlow) to 
the Rust ecosystem. Built from the ground up for **LLM applications**, **high-performance scenarios**, 
and **production reliability**. it provides a few major components:

* A multithreaded, async-based workflow [scheduler].
* A pluggable storage system backed by memory, files, or custom backends.
* Asynchronous [node execution][nodes] with retry logic and error handling.

These components provide the runtime infrastructure necessary for building
complex workflow applications.

[nodes]: https://docs.rs/cosmoflow/latest/cosmoflow/node/index.html
[scheduler]: https://docs.rs/cosmoflow/latest/cosmoflow/flow/index.html

## Example

A basic workflow with CosmoFlow.

Make sure you activated the full features of the cosmoflow crate on Cargo.toml:

```toml
[dependencies]
cosmoflow = { version = "0.3.0", features = ["full"] }
```

Then, on your main.rs:

```rust
use cosmoflow::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a shared store with memory backend
    let mut store = SharedStore::memory();
    
    // Create a flow builder
    let flow = Flow::builder("hello-workflow")
        .with_store(&mut store)
        .build()?;

    // Execute the workflow
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

## Module Ecosystem

CosmoFlow is built with a modular architecture. Each component is a module within the main
`cosmoflow` crate, allowing you to use only what you need through feature flags while 
maintaining full composability:

* [`cosmoflow`]: Main integration and API crate for CosmoFlow workflows.

* [`cosmoflow::flow`]: Workflow orchestration engine for managing complex multi-node workflows.

* [`cosmoflow::node`]: Execution nodes system with async support and retry logic.

* [`cosmoflow::action`]: Control flow logic and condition evaluation.

* [`cosmoflow::shared_store`]: Thread-safe data communication layer between workflow components.

* [`cosmoflow::storage`]: Pluggable storage backends (memory, file, and custom implementations).

* [`cosmoflow::builtin`]: Pre-built node components for common workflow operations.

[`cosmoflow`]: https://docs.rs/cosmoflow/latest/cosmoflow
[`cosmoflow::flow`]: https://docs.rs/cosmoflow/latest/cosmoflow/flow/index.html
[`cosmoflow::node`]: https://docs.rs/cosmoflow/latest/cosmoflow/node/index.html
[`cosmoflow::action`]: https://docs.rs/cosmoflow/latest/cosmoflow/action/index.html
[`cosmoflow::shared_store`]: https://docs.rs/cosmoflow/latest/cosmoflow/shared_store/index.html
[`cosmoflow::storage`]: https://docs.rs/cosmoflow/latest/cosmoflow/storage/index.html
[`cosmoflow::builtin`]: https://docs.rs/cosmoflow/latest/cosmoflow/builtin/index.html

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/echozyr2001/CosmoFlow/blob/main/LICENSE

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in CosmoFlow by you, shall be licensed as MIT, without any additional
terms or conditions.
