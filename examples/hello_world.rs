//! Hello World Example - CosmoFlow Minimal Features
//!
//! This example demonstrates the simplest possible CosmoFlow workflow using only
//! core features. It implements a basic greeting workflow that shows:
//!
//! ## Workflow Behavior
//! - **Hello Node**: Displays a greeting message and stores it in shared storage
//! - **Response Node**: Reads the greeting from storage and responds to it
//! - **Simple Communication**: Data flows between nodes via the shared store
//!
//! ## Core Features Demonstrated
//! - **Custom Storage Backend**: Complete implementation of StorageBackend trait
//! - **Basic Node Implementation**: Simple prep, exec, and post phases
//! - **Minimal Dependencies**: Uses only CosmoFlow's core without built-ins
//! - **Data Communication**: Nodes sharing data via the SharedStore
//! - **Sequential Execution**: Linear workflow with simple routing
//!
//! ## Execution Flow
//! 1. Hello node generates and displays a greeting message
//! 2. Greeting is stored in the shared storage
//! 3. Response node retrieves the greeting from storage
//! 4. Response node generates and displays a response message
//! 5. Workflow completes successfully
//!
//! This is the perfect starting point for understanding CosmoFlow's core concepts.
//!
//! To run this example:
//! ```bash
//! cd examples && cargo run --bin hello_world --features minimal
//! ```

use async_trait::async_trait;
use cosmoflow::{
    action::Action,
    flow::{FlowBackend, FlowBuilder},
    node::{ExecutionContext, Node, NodeBackend, NodeError},
    shared_store::SharedStore,
    storage::StorageBackend,
};
use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;

/// A simple in-memory storage implementation
#[derive(Debug, Clone)]
pub struct SimpleStorage {
    data: HashMap<String, serde_json::Value>,
}

impl SimpleStorage {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl Default for SimpleStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageBackend for SimpleStorage {
    type Error = SimpleStorageError;

    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error> {
        match self.data.get(key) {
            Some(value) => {
                let deserialized = serde_json::from_value(value.clone())
                    .map_err(|e| SimpleStorageError::DeserializationError(e.to_string()))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    fn set<T: Serialize>(&mut self, key: String, value: T) -> Result<(), Self::Error> {
        let json_value = serde_json::to_value(value)
            .map_err(|e| SimpleStorageError::SerializationError(e.to_string()))?;
        self.data.insert(key, json_value);
        Ok(())
    }

    fn remove<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, Self::Error> {
        match self.data.remove(key) {
            Some(value) => {
                let deserialized = serde_json::from_value(value)
                    .map_err(|e| SimpleStorageError::DeserializationError(e.to_string()))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    fn contains_key(&self, key: &str) -> Result<bool, Self::Error> {
        Ok(self.data.contains_key(key))
    }

    fn keys(&self) -> Result<Vec<String>, Self::Error> {
        Ok(self.data.keys().cloned().collect())
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        self.data.clear();
        Ok(())
    }

    fn len(&self) -> Result<usize, Self::Error> {
        Ok(self.data.len())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SimpleStorageError {
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

/// A simple greeting node
struct HelloNodeBackend {
    message: String,
}

impl HelloNodeBackend {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[async_trait]
impl NodeBackend<SimpleStorage> for HelloNodeBackend {
    type PrepResult = String;
    type ExecResult = String;
    type Error = NodeError;

    async fn prep(
        &mut self,
        _store: &SharedStore<SimpleStorage>,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        Ok(self.message.clone())
    }

    async fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        let output = format!("🌟 {prep_result}");
        println!("{}", output);
        Ok(output)
    }

    async fn post(
        &mut self,
        store: &mut SharedStore<SimpleStorage>,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        // Store the greeting
        store
            .set("greeting".to_string(), exec_result)
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        Ok(Action::simple("next"))
    }

    fn name(&self) -> &str {
        "HelloNode"
    }
}

/// A simple response node
struct ResponseNodeBackend;

#[async_trait]
impl NodeBackend<SimpleStorage> for ResponseNodeBackend {
    type PrepResult = Option<String>;
    type ExecResult = String;
    type Error = NodeError;

    async fn prep(
        &mut self,
        store: &SharedStore<SimpleStorage>,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        match store.get("greeting") {
            Ok(Some(greeting)) => Ok(Some(greeting)),
            _ => Ok(None),
        }
    }

    async fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        let response = match prep_result {
            Some(greeting) => format!("✨ I received: '{greeting}'"),
            None => "⚠️ No greeting received".to_string(),
        };

        println!("{}", response);
        Ok(response)
    }

    async fn post(
        &mut self,
        store: &mut SharedStore<SimpleStorage>,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        // Store the response
        store
            .set("response".to_string(), exec_result)
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        Ok(Action::simple("complete"))
    }

    fn name(&self) -> &str {
        "ResponseNode"
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Hello World Example");
    println!("=================================");
    println!("This example uses only CosmoFlow's core features.");
    println!();

    // Create our custom storage
    let storage = SimpleStorage::new();
    let mut store = SharedStore::with_storage(storage);

    // Build a simple workflow
    let mut flow = FlowBuilder::new()
        .start_node("hello")
        .node(
            "hello",
            Node::new(HelloNodeBackend::new("Hello from CosmoFlow minimal!")),
        )
        .node("response", Node::new(ResponseNodeBackend))
        .route("hello", "next", "response")
        .terminal_action("complete")
        .build();

    println!("📋 Flow configuration:");
    println!("  Start node: {}", flow.config().start_node_id);
    println!("  Max steps: {}", flow.config().max_steps);
    println!("  Terminal actions: {:?}", flow.config().terminal_actions);
    println!();

    // Validate the flow
    if let Err(e) = flow.validate() {
        eprintln!("❌ Flow validation failed: {e}");
        return Err(e.into());
    }
    println!("✅ Flow validation passed");
    println!();

    // Execute the workflow
    println!("⚡ Executing workflow...");
    println!("------------------------");

    let start_time = std::time::Instant::now();
    let result = flow.execute(&mut store).await?;
    let duration = start_time.elapsed();

    println!();
    println!("🎯 Workflow completed!");
    println!("======================");
    println!("  Success: {}", result.success);
    println!("  Steps executed: {}", result.steps_executed);
    println!("  Final action: {}", result.final_action);
    println!("  Last node: {}", result.last_node_id);
    println!("  Execution path: {:?}", result.execution_path);
    println!("  Duration: {duration:?}");
    println!();

    // Show the final state of our custom storage
    println!("📊 Final storage state:");
    if let Ok(Some(greeting)) = store.get::<String>("greeting") {
        println!("  greeting: {greeting}");
    }
    if let Ok(Some(response)) = store.get::<String>("response") {
        println!("  response: {response}");
    }

    println!();
    println!("💡 Key minimal features demonstrated:");
    println!("  - Custom storage backend implementation");
    println!("  - Basic node backend implementation");
    println!("  - Core workflow execution without built-ins");
    println!("  - Manual storage interface usage");

    Ok(())
}
