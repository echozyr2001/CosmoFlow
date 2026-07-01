# CosmoFlow

CosmoFlow is a small Rust framework for building workflows as state machines.

The core design follows two principles:

- Every program can be modeled as a state machine.
- The core framework should stay small; policy such as retry, timeout, fallback,
  LLM calls, tools, memory, and tracing should be composed by users or added by
  optional extensions.

## Core Model

CosmoFlow has three core concepts:

- `Action`: a state transition signal. Its `name` is the routing identity, and
  `params` are optional data carried with the transition.
- `Node`: a unit of behavior implemented as `prep -> exec -> post`.
- `Flow`: a state-machine graph with build-time validation and a single
  sequential executor.

The runtime state is a generic `S`. It can be a strong typed application struct,
or it can be an official `SharedStore` implementation when dynamic key-value
sharing is the right model.

## Quick Start

```rust
use cosmoflow::action::Action;
use cosmoflow::flow::FlowBuilder;
use cosmoflow::node::{Node, NodeContext};

#[derive(Default)]
struct AppState {
    visits: Vec<String>,
}

struct LoadUser;

impl Node<AppState> for LoadUser {
    type Prep = ();
    type Output = String;
    type Error = std::convert::Infallible;

    fn prep(&mut self, _state: &AppState, _ctx: &NodeContext) -> Result<Self::Prep, Self::Error> {
        Ok(())
    }

    fn exec(&mut self, _prep: &Self::Prep, _ctx: &NodeContext) -> Result<Self::Output, Self::Error> {
        Ok("ada".to_string())
    }

    fn post(
        &mut self,
        state: &mut AppState,
        _prep: Self::Prep,
        user: Self::Output,
        ctx: &NodeContext,
    ) -> Result<Action, Self::Error> {
        state.visits.push(format!("{}:{user}", ctx.node_id.as_str()));
        Ok(Action::new("done"))
    }
}

let mut flow = FlowBuilder::new()
    .node("load", LoadUser)
    .build()?;

let mut state = AppState::default();
let action = flow.run(&mut state)?;

assert_eq!(action.as_str(), "done");
```

A flow starts at the first registered node unless `.start(id)` is provided.
After a node returns an action, routing uses only the action name. If the current
node has no route for that action, the flow terminates naturally.

## Optional Shared Store

`SharedStore` remains useful when nodes need dynamic key-value context or a
pluggable backend such as memory, file, or Redis storage. It is not required by
the core node or flow model; strong typed state is often simpler for application
logic.

See the repository docs for the full guides:

- [Getting Started](../docs/getting-started.md)
- [Architecture](../docs/architecture.md)
- [Features](../docs/features.md)

## License

CosmoFlow is licensed under the [MIT license](../LICENSE).
