# CosmoFlow Documentation

CosmoFlow is a small Rust framework for modeling programs as state machines.
The core API is intentionally narrow: actions signal transitions, nodes perform
work, and flows connect nodes into validated graphs.

## Guides

- [Getting Started](getting-started.md): build a typed-state flow and run it.
- [Architecture](architecture.md): understand the core boundaries and design
  trade-offs.
- [Features](features.md): choose async and storage feature flags.
- [API Reference](https://docs.rs/cosmoflow): generated Rust API docs.

## Core Principles

- Every program can be modeled as a state machine.
- CosmoFlow core is a framework, not a policy runtime. Retry, fallback,
  timeout, agent/tool/LLM integration, memory, and tracing should be composed
  above the core or provided by optional extensions.

## Current API Note

These guides describe the intended main API after the current core model is
promoted from the v2 modules. Until that promotion is complete, the same model
is available under `cosmoflow::action::v2`, `cosmoflow::node::v2`, and
`cosmoflow::flow::v2`.
