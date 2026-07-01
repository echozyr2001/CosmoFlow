# Getting Started

This guide introduces the CosmoFlow core model: typed state, nodes, actions,
and flows.

## Installation

```toml
[dependencies]
cosmoflow = { version = "0.5.1", features = ["basic"] }
```

Use the `async` feature when node execution should be asynchronous:

```toml
[dependencies]
cosmoflow = { version = "0.5.1", features = ["standard"] }
async-trait = "0.1"
```

## Your First Flow

A flow runs against a user-provided state value. That state can be a plain Rust
struct:

```rust
#[derive(Default)]
struct AppState {
    loaded_user: Option<String>,
    visits: Vec<String>,
}
```

A node implements `prep -> exec -> post`:

```rust
use cosmoflow::action::Action;
use cosmoflow::node::{Node, NodeContext};

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
        state.visits.push(ctx.node_id.as_str().to_string());
        state.loaded_user = Some(user);
        Ok(Action::new("score"))
    }
}
```

The returned `Action` is the state transition signal. Its name is used for
routing:

```rust
use cosmoflow::action::Action;
use cosmoflow::node::{Node, NodeContext};

struct ScoreUser;

impl Node<AppState> for ScoreUser {
    type Prep = String;
    type Output = u32;
    type Error = &'static str;

    fn prep(&mut self, state: &AppState, _ctx: &NodeContext) -> Result<Self::Prep, Self::Error> {
        state.loaded_user.clone().ok_or("missing loaded user")
    }

    fn exec(&mut self, user: &Self::Prep, _ctx: &NodeContext) -> Result<Self::Output, Self::Error> {
        Ok((user.len() as u32) * 10)
    }

    fn post(
        &mut self,
        state: &mut AppState,
        _user: Self::Prep,
        score: Self::Output,
        ctx: &NodeContext,
    ) -> Result<Action, Self::Error> {
        state.visits.push(ctx.node_id.as_str().to_string());
        Ok(Action::with_param("done", "score", serde_json::json!(score)))
    }
}
```

Build and run the flow:

```rust
use cosmoflow::flow::FlowBuilder;

let mut flow = FlowBuilder::new()
    .node("load", LoadUser)
    .node("score", ScoreUser)
    .route("load", "score", "score")
    .build()?;

let mut state = AppState::default();
let action = flow.run(&mut state)?;

assert_eq!(action.as_str(), "done");
assert_eq!(state.visits, vec!["load", "score"]);
```

The first registered node is the default start node. Use `.start(id)` only when
the start node should be different.

## Natural Termination

Flows do not need terminal routes. After a node returns an action, the flow
looks for a route from the current node using that action name. If no route
matches, execution stops and the action becomes the final action.

Action parameters do not participate in routing. They are carried data for the
caller or later nodes.

## Nested Flows

A built flow can be inserted as a node in another flow:

```rust
let child = FlowBuilder::new()
    .node("load", LoadUser)
    .node("score", ScoreUser)
    .route("load", "score", "score")
    .build()?;

let mut parent = FlowBuilder::new()
    .node("user_pipeline", child)
    .build()?;

let final_action = parent.run(&mut AppState::default())?;
```

The parent flow sees only the child flow's final action. The child's internal
path remains internal to the child flow.

## Optional Shared Store

Use a strong typed state struct when the workflow has a stable domain model.
Use `SharedStore` when dynamic key-value sharing, serialization, or a storage
backend is a better fit.

```rust
use cosmoflow::shared_store::SharedStore;
use cosmoflow::shared_store::backends::MemoryStorage;

let mut store = MemoryStorage::new();
store.set("user_id".to_string(), "ada".to_string())?;
let user_id: Option<String> = store.get("user_id")?;
```

`SharedStore` is a supported state model, not a requirement for `Node<S>` or
`Flow<S>`.

## Async Mode

With the `async` feature, the same model is available with async node phases and
async flow execution:

```rust
use async_trait::async_trait;
use cosmoflow::action::Action;
use cosmoflow::node::{Node, NodeContext};

struct AsyncNode;

#[async_trait]
impl Node<AppState> for AsyncNode {
    type Prep = ();
    type Output = ();
    type Error = std::convert::Infallible;

    async fn prep(&mut self, _state: &AppState, _ctx: &NodeContext) -> Result<Self::Prep, Self::Error> {
        Ok(())
    }

    async fn exec(&mut self, _prep: &Self::Prep, _ctx: &NodeContext) -> Result<Self::Output, Self::Error> {
        Ok(())
    }

    async fn post(
        &mut self,
        _state: &mut AppState,
        _prep: Self::Prep,
        _output: Self::Output,
        _ctx: &NodeContext,
    ) -> Result<Action, Self::Error> {
        Ok(Action::new("done"))
    }
}
```

Async state types must satisfy `Send + Sync` so generated futures can be moved
safely. This is not a shared-store requirement.
