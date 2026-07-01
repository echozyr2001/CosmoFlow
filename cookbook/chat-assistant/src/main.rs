//! Chat loop example using the core state-machine API.

use async_trait::async_trait;
use cosmoflow::prelude::*;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct ChatError(String);

impl ChatError {
    fn new(error: impl fmt::Display) -> Self {
        Self(error.to_string())
    }
}

impl fmt::Display for ChatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for ChatError {}

struct InputNode;

#[async_trait]
impl Node<MemoryStorage> for InputNode {
    type Prep = ();
    type Output = ();
    type Error = ChatError;

    async fn prep(
        &mut self,
        _store: &MemoryStorage,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        Ok(())
    }

    async fn exec(
        &mut self,
        _prep: &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        println!("Getting user input...");
        Ok(())
    }

    async fn post(
        &mut self,
        store: &mut MemoryStorage,
        _prep: Self::Prep,
        _output: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        let message_count = store
            .get::<i32>("message_count")
            .map_err(ChatError::new)?
            .unwrap_or(0);
        store
            .set(
                "current_message".to_string(),
                format!("Message {}", message_count + 1),
            )
            .map_err(ChatError::new)?;
        store
            .set("message_count".to_string(), message_count + 1)
            .map_err(ChatError::new)?;

        if message_count >= 5 {
            Ok(Action::new("quit"))
        } else {
            Ok(Action::new("process"))
        }
    }
}

struct ProcessNode;

#[async_trait]
impl Node<MemoryStorage> for ProcessNode {
    type Prep = String;
    type Output = String;
    type Error = ChatError;

    async fn prep(
        &mut self,
        store: &MemoryStorage,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        store
            .get::<String>("current_message")
            .map_err(ChatError::new)?
            .ok_or_else(|| ChatError::new("missing current_message"))
    }

    async fn exec(
        &mut self,
        message: &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        println!("Processing: {message}");
        Ok(format!("Processed: {message}"))
    }

    async fn post(
        &mut self,
        store: &mut MemoryStorage,
        _message: Self::Prep,
        response: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        store
            .set("response".to_string(), response)
            .map_err(ChatError::new)?;
        Ok(Action::new("output"))
    }
}

struct OutputNode;

#[async_trait]
impl Node<MemoryStorage> for OutputNode {
    type Prep = String;
    type Output = ();
    type Error = ChatError;

    async fn prep(
        &mut self,
        store: &MemoryStorage,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        store
            .get::<String>("response")
            .map_err(ChatError::new)?
            .ok_or_else(|| ChatError::new("missing response"))
    }

    async fn exec(
        &mut self,
        response: &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        println!("Output: {response}");
        Ok(())
    }

    async fn post(
        &mut self,
        _store: &mut MemoryStorage,
        _response: Self::Prep,
        _output: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        Ok(Action::new("input"))
    }
}

struct QuitNode;

#[async_trait]
impl Node<MemoryStorage> for QuitNode {
    type Prep = ();
    type Output = ();
    type Error = ChatError;

    async fn prep(
        &mut self,
        _store: &MemoryStorage,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        Ok(())
    }

    async fn exec(
        &mut self,
        _prep: &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        println!("Chat session ended");
        Ok(())
    }

    async fn post(
        &mut self,
        _store: &mut MemoryStorage,
        _prep: Self::Prep,
        _output: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        Ok(Action::new("complete"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Chat Application Loop Pattern");

    let mut chat_flow = FlowBuilder::new()
        .node("input", InputNode)
        .node("process", ProcessNode)
        .node("output", OutputNode)
        .node("quit", QuitNode)
        .start("input")
        .route("input", "process", "process")
        .route("process", "output", "output")
        .route("output", "input", "input")
        .route("input", "quit", "quit")
        .build()?;

    let mut store = MemoryStorage::new();
    let execution = chat_flow.run_recorded(&mut store).await?;

    println!("Chat flow executed successfully");
    println!("Steps executed: {}", execution.steps);
    println!(
        "Messages processed: {}",
        store.get::<i32>("message_count")?.unwrap_or(0)
    );
    println!("Execution path: {:?}", execution.path);
    println!("Final action: {}", execution.final_action);

    Ok(())
}
