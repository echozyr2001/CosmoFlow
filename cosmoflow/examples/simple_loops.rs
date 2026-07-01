//! A state-machine loop using routes.
//!
//! CosmoFlow core does not impose a step limit. A loop terminates when the
//! modeled state returns an action without a matching route.

use cosmoflow::action::Action;
use cosmoflow::flow::FlowBuilder;
use cosmoflow::node::{Node, NodeContext};
use cosmoflow::shared_store::SharedStore;
use cosmoflow::shared_store::backends::MemoryStorage;
use std::error::Error;
use std::fmt;

#[cfg(feature = "async")]
use async_trait::async_trait;

#[derive(Debug)]
struct ExampleError(String);

impl ExampleError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for ExampleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ExampleError {}

struct Counter {
    limit: i64,
}

#[cfg(not(feature = "async"))]
impl Node<MemoryStorage> for Counter {
    type Prep = i64;
    type Output = i64;
    type Error = ExampleError;

    fn prep(
        &mut self,
        state: &MemoryStorage,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        Ok(state
            .get("count")
            .map_err(|error| ExampleError::new(error.to_string()))?
            .unwrap_or(0))
    }

    fn exec(
        &mut self,
        count: &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        Ok(*count + 1)
    }

    fn post(
        &mut self,
        state: &mut MemoryStorage,
        _prep: Self::Prep,
        next_count: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        state
            .set("count".to_string(), next_count)
            .map_err(|error| ExampleError::new(error.to_string()))?;

        if next_count >= self.limit {
            Ok(Action::new("done"))
        } else {
            Ok(Action::new("again"))
        }
    }
}

#[cfg(feature = "async")]
#[async_trait]
impl Node<MemoryStorage> for Counter {
    type Prep = i64;
    type Output = i64;
    type Error = ExampleError;

    async fn prep(
        &mut self,
        state: &MemoryStorage,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        Ok(state
            .get("count")
            .map_err(|error| ExampleError::new(error.to_string()))?
            .unwrap_or(0))
    }

    async fn exec(
        &mut self,
        count: &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        Ok(*count + 1)
    }

    async fn post(
        &mut self,
        state: &mut MemoryStorage,
        _prep: Self::Prep,
        next_count: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        state
            .set("count".to_string(), next_count)
            .map_err(|error| ExampleError::new(error.to_string()))?;

        if next_count >= self.limit {
            Ok(Action::new("done"))
        } else {
            Ok(Action::new("again"))
        }
    }
}

#[cfg(not(feature = "async"))]
fn main() -> Result<(), Box<dyn Error>> {
    let mut state = MemoryStorage::new();
    let mut flow = FlowBuilder::new()
        .node("counter", Counter { limit: 3 })
        .route("counter", "again", "counter")
        .build()?;

    let execution = flow.run_recorded(&mut state)?;
    let count = state.get::<i64>("count")?.unwrap_or(0);

    println!("final action: {}", execution.final_action);
    println!("steps: {}", execution.steps);
    println!("path: {:?}", execution.path);
    println!("count: {count}");
    Ok(())
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut state = MemoryStorage::new();
    let mut flow = FlowBuilder::new()
        .node("counter", Counter { limit: 3 })
        .route("counter", "again", "counter")
        .build()?;

    let execution = flow.run_recorded(&mut state).await?;
    let count = state.get::<i64>("count")?.unwrap_or(0);

    println!("final action: {}", execution.final_action);
    println!("steps: {}", execution.steps);
    println!("path: {:?}", execution.path);
    println!("count: {count}");
    Ok(())
}
