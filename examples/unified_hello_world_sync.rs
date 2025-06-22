//! # Unified Hello World Example - Sync Version
//!
//! This example demonstrates the unified Node trait API in CosmoFlow using
//! synchronous execution for faster compilation and smaller binaries.
//! It showcases how to create a simple workflow using the simplified API
//! that eliminates the need for separate NodeBackend implementations.
//!
//! ## Key Features Demonstrated:
//! - Unified Node trait implementation (prep/exec/post phases) in sync mode
//! - Built-in MemoryStorage backend for simplicity
//! - Individual node execution without Flow complexity
//! - Data persistence and retrieval from shared storage
//! - Error handling with proper error propagation
//!
//! ## Performance Benefits:
//! - Faster compilation compared to async version
//! - Smaller binary size (no async runtime overhead)
//! - Perfect for CPU-intensive workflows
//! - Simpler debugging without async complexity
//!
//! ## Workflow Description:
//! 1. HelloNode generates a greeting message
//! 2. Message is stored in shared storage
//! 3. ResponseNode reads and responds to the greeting
//! 4. Both messages are displayed and stored
//!
//! Run with: `cargo run --bin unified_hello_world_sync --no-default-features --features cosmoflow/storage-memory`

#[cfg(not(feature = "async"))]
use cosmoflow::{
    Node,
    action::Action,
    node::{ExecutionContext, NodeError},
    shared_store::SharedStore,
    shared_store::backends::MemoryStorage,
};
#[cfg(not(feature = "async"))]
use std::time::Duration;

/// A simple greeting node that demonstrates the unified Node trait (sync version).
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
/// - Synchronous execution for better performance
#[cfg(not(feature = "async"))]
struct HelloNode {
    /// The message to include in the greeting
    message: String,
}

#[cfg(not(feature = "async"))]
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

#[cfg(not(feature = "async"))]
impl Node<MemoryStorage> for HelloNode {
    /// Preparation phase returns unit type (no data needed for execution)
    type PrepResult = ();
    /// Execution phase returns the generated greeting string
    type ExecResult = String;
    /// All errors are wrapped in NodeError for consistency
    type Error = NodeError;

    /// Returns the human-readable name of this node for logging and debugging.
    fn name(&self) -> &str {
        "HelloNode"
    }

    /// Preparation phase: Validate inputs and prepare for execution.
    ///
    /// In this simple example, no preparation is needed, but this phase
    /// could be used to validate the message format, check permissions,
    /// or read configuration from storage.
    ///
    /// # Arguments
    /// * `_store` - Shared storage (unused in this example)
    /// * `context` - Execution context with retry info and metadata
    ///
    /// # Returns
    /// * `Ok(())` - Always succeeds in this example
    fn prep(
        &mut self,
        _store: &MemoryStorage,
        context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        println!(
            "🔄 [PREP] HelloNode (exec_id: {}) preparing message: {}",
            context.execution_id(),
            self.message
        );
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
    fn exec(
        &mut self,
        _prep_result: Self::PrepResult,
        context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        println!("⚡ [EXEC] HelloNode generating greeting...");

        // Simulate some synchronous work
        std::thread::sleep(std::time::Duration::from_millis(10));

        let greeting = format!(
            "🌟 Hello from CosmoFlow Unified (exec_id: {}): {}",
            context.execution_id(),
            self.message
        );

        println!("   Generated: {}", greeting);
        Ok(greeting)
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
    /// * `Ok(Action)` - The "next" action to continue to response node
    /// * `Err(NodeError)` - If storage operations fail
    fn post(
        &mut self,
        store: &mut MemoryStorage,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        println!("✅ [POST] HelloNode storing greeting: {}", exec_result);

        // Store the greeting in shared storage for use by the response node
        SharedStore::set(store, "greeting".to_string(), exec_result)
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        // Return action to continue to response node
        Ok(Action::simple("next"))
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

/// A response node that reads the greeting and generates a response (sync version)
#[cfg(not(feature = "async"))]
struct ResponseNode {
    responder_name: String,
}

#[cfg(not(feature = "async"))]
impl ResponseNode {
    fn new(responder_name: impl Into<String>) -> Self {
        Self {
            responder_name: responder_name.into(),
        }
    }
}

#[cfg(not(feature = "async"))]
impl Node<MemoryStorage> for ResponseNode {
    type PrepResult = String;
    type ExecResult = String;
    type Error = NodeError;

    fn name(&self) -> &str {
        "ResponseNode"
    }

    fn prep(
        &mut self,
        store: &MemoryStorage,
        context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        println!(
            "🔄 [PREP] ResponseNode (exec_id: {}) reading greeting...",
            context.execution_id()
        );

        // Read the greeting from storage
        let greeting: String = store
            .get("greeting")
            .map_err(|e| NodeError::StorageError(e.to_string()))?
            .ok_or_else(|| {
                NodeError::ValidationError("No greeting found in storage".to_string())
            })?;

        println!("   Retrieved: {}", greeting);
        Ok(greeting)
    }

    fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        println!(
            "⚡ [EXEC] ResponseNode generating response to: {}",
            prep_result
        );

        // Simulate some synchronous processing
        std::thread::sleep(std::time::Duration::from_millis(5));

        let response = format!(
            "🤝 Thank you for the greeting! Nice to meet you! - {}",
            self.responder_name
        );

        println!("   Generated: {}", response);
        Ok(response)
    }

    fn post(
        &mut self,
        store: &mut MemoryStorage,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        println!("✅ [POST] ResponseNode storing response: {}", exec_result);

        // Store the response
        store
            .set("response".to_string(), exec_result.clone())
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        Ok(Action::simple("complete"))
    }

    fn max_retries(&self) -> usize {
        1
    }

    fn retry_delay(&self) -> Duration {
        Duration::from_millis(50)
    }
}

/// Main function demonstrating the unified Node trait workflow in sync mode.
///
/// This example shows how to:
/// 1. Create nodes using the unified Node trait (sync version)
/// 2. Use built-in MemoryStorage backend
/// 3. Execute nodes individually in sequence
/// 4. Handle results and inspect shared storage
/// 5. Demonstrate the performance benefits of sync execution
///
/// The workflow consists of a HelloNode that generates a greeting,
/// stores it in shared storage, and a ResponseNode that reads the
/// greeting and generates a response.
#[cfg(all(feature = "sync", not(feature = "async")))]
#[cfg(not(feature = "async"))]
fn sync_main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 CosmoFlow Unified Node Trait (Sync Version)");
    println!("===============================================");
    println!("📦 Compiled without async features for optimal performance!\n");

    // Create shared storage with built-in MemoryStorage backend
    let mut store = MemoryStorage::new();

    // Create nodes using the unified Node trait
    let mut hello_node = HelloNode::new("CosmoFlow Unified Sync API!");
    let mut response_node = ResponseNode::new("CosmoFlow Assistant");

    println!("🔄 Executing unified workflow...");
    println!("-------------------------------\n");

    // Execute hello node
    println!("1️⃣ Executing HelloNode:");
    let hello_action = hello_node.run(&mut store)?;
    println!("   Action: {}\n", hello_action.name());

    // Execute response node if hello succeeded
    if hello_action.name() == "next" {
        println!("2️⃣ Executing ResponseNode:");
        let response_action = response_node.run(&mut store)?;
        println!("   Action: {}\n", response_action.name());
    }

    // Display final results
    println!("📊 Final Results:");
    println!("=================");

    if let Ok(Some(greeting)) = store.get::<String>("greeting") {
        println!("Greeting: {}", greeting);
    }

    if let Ok(Some(response)) = store.get::<String>("response") {
        println!("Response: {}", response);
    }

    // Display storage statistics
    println!("\n📈 Storage Statistics:");
    println!("=====================");
    println!("Total items: {}", store.len().unwrap_or(0));
    println!("Storage keys: {:?}", store.keys().unwrap_or_default());

    println!("\n🎯 Sync Version Benefits:");
    println!("========================");
    println!("• ⚡ Faster compilation (no async dependencies)");
    println!("• 📦 Smaller binary size (no tokio runtime)");
    println!("• 🎯 Perfect for CPU-intensive workflows");
    println!("• 🔧 Simpler debugging and profiling");
    println!("• 🚀 No async runtime overhead");
    println!("• 💡 Unified Node trait for cleaner code");

    println!("\n💡 Note: This example shows individual node execution");
    println!("   using the unified Node trait in synchronous mode.");
    println!("   Each node is executed manually in sequence for");
    println!("   maximum control and minimal dependencies.");

    Ok(())
}

fn main() {
    #[cfg(not(feature = "async"))]
    {
        if let Err(e) = sync_main() {
            eprintln!("Error running sync unified hello world example: {}", e);
            std::process::exit(1);
        }
    }

    #[cfg(feature = "async")]
    {
        println!("This sync example is not available when async features are enabled.");
        println!("To run this example, use:");
        println!(
            "cargo run --bin unified_hello_world_sync --no-default-features --features cosmoflow/storage-memory,sync"
        );
    }
}
