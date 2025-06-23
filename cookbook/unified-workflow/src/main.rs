//! # Unified Hello World Example
//!
//! This example demonstrates the new unified Node trait API in CosmoFlow.
//! It showcases how to create a simple workflow using the simplified API
//! that eliminates the need for separate NodeBackend implementations.
//!
//! ## Key Features Demonstrated:
//! - Unified Node trait implementation (prep/exec/post phases)
//! - Custom storage backend implementation
//! - Simple workflow execution with terminal actions
//! - Data persistence and retrieval from shared storage
//! - Retry configuration and error handling
//!
//! ## Workflow Description:
//! 1. HelloNode generates a greeting message
//! 2. Message is stored in shared storage
//! 3. Workflow terminates with "complete" action
//!
//! Run with: `cargo run --bin unified_hello_world`

use async_trait::async_trait;
use cosmoflow::{
    action::Action,
    node::{ExecutionContext, NodeError},
    shared_store::{backends::MemoryStorage, SharedStore},
    FlowBackend, FlowBuilder, Node,
};
use std::time::Duration;

/// A simple greeting node that demonstrates the unified Node trait.
///
/// This node implements the three-phase execution model:
/// 1. **Prep**: Validates inputs and prepares for execution
/// 2. **Exec**: Generates the greeting message (core logic)
/// 3. **Post**: Stores the result and determines next action
///
/// ## Features Demonstrated:
/// - Custom retry configuration (2 retries with 100ms delay)
/// - Error handling with proper error propagation
/// - Data storage in shared storage
/// - Terminal action generation
struct HelloNode {
    /// The message to include in the greeting
    message: String,
}

impl HelloNode {
    /// Creates a new HelloNode with the specified message.
    ///
    /// # Arguments
    /// * `message` - The message to include in the greeting
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[async_trait]
impl Node<MemoryStorage> for HelloNode {
    /// Preparation phase returns unit type (no data needed for execution)
    type PrepResult = ();
    /// Execution phase returns the generated greeting string
    type ExecResult = String;
    /// All errors are wrapped in NodeError for consistency
    type Error = NodeError;

    /// Preparation phase: Validate inputs and prepare for execution.
    ///
    /// In this simple example, no preparation is needed, but this phase
    /// could be used to validate the message format, check permissions,
    /// or read configuration from storage.
    ///
    /// # Arguments
    /// * `_store` - Shared storage (unused in this example)
    /// * `_context` - Execution context with retry info and metadata
    ///
    /// # Returns
    /// * `Ok(())` - Always succeeds in this example
    async fn prep(
        &mut self,
        _store: &MemoryStorage,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        println!("Preparing HelloNode...");
        Ok(())
    }

    /// Execution phase: Generate the greeting message.
    ///
    /// This is the core logic of the node. It should be idempotent
    /// (safe to retry) and not have side effects. The actual greeting
    /// generation happens here.
    ///
    /// # Arguments
    /// * `_prep_result` - Result from prep phase (unused here)
    /// * `context` - Execution context with unique execution ID
    ///
    /// # Returns
    /// * `Ok(String)` - The generated greeting message
    async fn exec(
        &mut self,
        _prep_result: Self::PrepResult,
        context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        println!(
            "Executing HelloNode (execution_id: {})",
            context.execution_id()
        );
        Ok(format!("Hello: {}", self.message))
    }

    /// Post-processing phase: Store results and determine next action.
    ///
    /// This phase handles side effects like storing data, sending notifications,
    /// or updating external systems. It also determines what action should
    /// be taken next in the workflow.
    ///
    /// # Arguments
    /// * `store` - Mutable shared storage for writing results
    /// * `_prep_result` - Result from prep phase
    /// * `exec_result` - The greeting message from exec phase
    /// * `_context` - Execution context
    ///
    /// # Returns
    /// * `Ok(Action)` - The "complete" action to terminate the workflow
    /// * `Err(NodeError)` - If storage operations fail
    async fn post(
        &mut self,
        store: &mut MemoryStorage,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        println!("Post-processing HelloNode: {}", exec_result);

        // Store the greeting in shared storage for potential use by other nodes
        SharedStore::set(store, "greeting".to_string(), exec_result)
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        // Return terminal action to complete the workflow
        Ok(Action::simple("complete"))
    }

    /// Returns the human-readable name of this node for logging and debugging.
    fn name(&self) -> &str {
        "HelloNode"
    }

    /// Configure retry behavior: allow up to 2 retries for resilience.
    ///
    /// This is useful for handling transient failures like network issues
    /// or temporary resource unavailability.
    fn max_retries(&self) -> usize {
        2
    }

    /// Configure retry delay: wait 100ms between retry attempts.
    ///
    /// This prevents overwhelming external services and allows time
    /// for transient issues to resolve.
    fn retry_delay(&self) -> Duration {
        Duration::from_millis(100)
    }
}

/// Main function demonstrating the unified Node trait workflow.
///
/// This example shows how to:
/// 1. Create a custom storage backend
/// 2. Implement a node using the unified Node trait
/// 3. Build and execute a simple workflow
/// 4. Handle results and inspect shared storage
///
/// The workflow consists of a single HelloNode that generates a greeting,
/// stores it in shared storage, and terminates with a "complete" action.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 CosmoFlow Unified Node Trait Example");
    println!("========================================");

    // Create shared storage with MemoryStorage backend
    let mut store = MemoryStorage::new();

    // Build the workflow using the new unified API
    // - start_with() creates the starting node and sets it as the entry point
    // - terminal_route() defines explicit workflow termination
    let mut flow = FlowBuilder::new()
        .start_with("hello", HelloNode::new("CosmoFlow with Unified Node!"))
        .terminal_route("hello", "complete") // Explicit termination
        .build();

    println!("\n📋 Executing workflow...");
    println!("------------------");

    // Execute the workflow and capture results
    let result = flow.execute(&mut store).await?;

    // Display execution results
    println!("\n✅ Workflow execution completed!");
    println!("==================");
    println!("Success: {}", result.success);
    println!("Steps executed: {}", result.steps_executed);
    println!("Final action: {}", result.final_action.name());
    println!("Execution path: {:?}", result.execution_path);

    // Inspect shared storage contents
    println!("\n📊 Shared storage contents:");
    println!("------------------");

    // Check for the greeting we stored
    if let Ok(Some(greeting)) = SharedStore::get::<String>(&store, "greeting") {
        println!("greeting: {greeting}");
    }

    // Check for counter (won't exist in this example)
    if let Ok(Some(counter)) = SharedStore::get::<i32>(&store, "counter") {
        println!("counter: {counter}");
    }

    Ok(())
}
