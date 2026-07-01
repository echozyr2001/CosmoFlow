//! Multi-node counter workflow using action-name routing.

use async_trait::async_trait;
use cosmoflow::{
    action::Action,
    node::{Node, NodeContext},
    shared_store::{backends::MemoryStorage, SharedStore},
    FlowBuilder,
};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct DemoError(String);

impl DemoError {
    fn new(error: impl fmt::Display) -> Self {
        Self(error.to_string())
    }
}

impl fmt::Display for DemoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for DemoError {}

struct CounterNode {
    increment: i32,
}

impl CounterNode {
    fn new(increment: i32) -> Self {
        Self { increment }
    }
}

#[async_trait]
impl Node<MemoryStorage> for CounterNode {
    type Prep = i32;
    type Output = i32;
    type Error = DemoError;

    async fn prep(
        &mut self,
        store: &MemoryStorage,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        let current_count = store
            .get::<i32>("counter")
            .map_err(DemoError::new)?
            .unwrap_or(0);
        println!("Current counter value: {current_count}");
        Ok(current_count)
    }

    async fn exec(
        &mut self,
        current_count: &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        let new_count = *current_count + self.increment;
        println!(
            "Incrementing counter by {}: {} -> {}",
            self.increment, current_count, new_count
        );
        Ok(new_count)
    }

    async fn post(
        &mut self,
        store: &mut MemoryStorage,
        _current_count: Self::Prep,
        new_count: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        store
            .set("counter".to_string(), new_count)
            .map_err(DemoError::new)?;

        if new_count >= 10 {
            Ok(Action::new("complete"))
        } else {
            Ok(Action::new("continue"))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("CosmoFlow Counter Workflow Example");

    let mut flow = FlowBuilder::new()
        .node("counter1", CounterNode::new(3))
        .node("counter2", CounterNode::new(3))
        .node("counter3", CounterNode::new(5))
        .start("counter1")
        .route("counter1", "continue", "counter2")
        .route("counter2", "continue", "counter3")
        .build()?;
    let mut store = MemoryStorage::new();

    let execution = flow.run_recorded(&mut store).await?;

    println!("Workflow execution completed");
    println!("Steps executed: {}", execution.steps);
    println!("Final action: {}", execution.final_action);
    println!("Execution path: {:?}", execution.path);

    if let Some(counter) = store.get::<i32>("counter")? {
        println!("counter: {counter}");
    }

    Ok(())
}
