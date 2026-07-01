//! Minimal workflow using the main CosmoFlow API.
//!
//! The example compiles in both sync mode and with the `async` feature enabled.

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

struct BuildGreeting;

#[cfg(not(feature = "async"))]
impl Node<MemoryStorage> for BuildGreeting {
    type Prep = String;
    type Output = String;
    type Error = ExampleError;

    fn prep(
        &mut self,
        state: &MemoryStorage,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        state
            .get("name")
            .map_err(|error| ExampleError::new(error.to_string()))?
            .ok_or_else(|| ExampleError::new("missing name"))
    }

    fn exec(
        &mut self,
        name: &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        Ok(format!("Hello, {name}!"))
    }

    fn post(
        &mut self,
        state: &mut MemoryStorage,
        _prep: Self::Prep,
        greeting: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        state
            .set("greeting".to_string(), greeting)
            .map_err(|error| ExampleError::new(error.to_string()))?;
        Ok(Action::new("complete"))
    }
}

#[cfg(feature = "async")]
#[async_trait]
impl Node<MemoryStorage> for BuildGreeting {
    type Prep = String;
    type Output = String;
    type Error = ExampleError;

    async fn prep(
        &mut self,
        state: &MemoryStorage,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        state
            .get("name")
            .map_err(|error| ExampleError::new(error.to_string()))?
            .ok_or_else(|| ExampleError::new("missing name"))
    }

    async fn exec(
        &mut self,
        name: &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        Ok(format!("Hello, {name}!"))
    }

    async fn post(
        &mut self,
        state: &mut MemoryStorage,
        _prep: Self::Prep,
        greeting: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        state
            .set("greeting".to_string(), greeting)
            .map_err(|error| ExampleError::new(error.to_string()))?;
        Ok(Action::new("complete"))
    }
}

#[cfg(not(feature = "async"))]
fn main() -> Result<(), Box<dyn Error>> {
    let mut state = MemoryStorage::new();
    state.set("name".to_string(), "CosmoFlow".to_string())?;

    let mut flow = FlowBuilder::new().node("greet", BuildGreeting).build()?;
    let execution = flow.run_recorded(&mut state)?;

    let greeting = state
        .get::<String>("greeting")?
        .ok_or_else(|| ExampleError::new("missing greeting"))?;

    println!("final action: {}", execution.final_action);
    println!("path: {:?}", execution.path);
    println!("{greeting}");
    Ok(())
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut state = MemoryStorage::new();
    state.set("name".to_string(), "CosmoFlow".to_string())?;

    let mut flow = FlowBuilder::new().node("greet", BuildGreeting).build()?;
    let execution = flow.run_recorded(&mut state).await?;

    let greeting = state
        .get::<String>("greeting")?
        .ok_or_else(|| ExampleError::new("missing greeting"))?;

    println!("final action: {}", execution.final_action);
    println!("path: {:?}", execution.path);
    println!("{greeting}");
    Ok(())
}
