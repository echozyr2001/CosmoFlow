//! Single-node workflow example using the core Node trait.

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

struct HelloNode {
    message: String,
}

impl HelloNode {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[async_trait]
impl Node<MemoryStorage> for HelloNode {
    type Prep = ();
    type Output = String;
    type Error = DemoError;

    async fn prep(
        &mut self,
        _store: &MemoryStorage,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        println!("Preparing HelloNode");
        Ok(())
    }

    async fn exec(
        &mut self,
        _prep: &Self::Prep,
        context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        println!(
            "Executing HelloNode (execution_id: {})",
            context.execution_id
        );
        Ok(format!("Hello: {}", self.message))
    }

    async fn post(
        &mut self,
        store: &mut MemoryStorage,
        _prep: Self::Prep,
        greeting: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        println!("Post-processing HelloNode: {greeting}");
        store
            .set("greeting".to_string(), greeting)
            .map_err(DemoError::new)?;
        Ok(Action::new("complete"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("CosmoFlow Node Trait Example");

    let mut flow = FlowBuilder::new()
        .node("hello", HelloNode::new("CosmoFlow core API"))
        .build()?;
    let mut store = MemoryStorage::new();

    let execution = flow.run_recorded(&mut store).await?;

    println!("Workflow execution completed");
    println!("Steps executed: {}", execution.steps);
    println!("Final action: {}", execution.final_action);
    println!("Execution path: {:?}", execution.path);

    if let Some(greeting) = store.get::<String>("greeting")? {
        println!("greeting: {greeting}");
    }

    Ok(())
}
