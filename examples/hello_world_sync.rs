//! Hello World Example - CosmoFlow Sync Version
//!
//! This example demonstrates the simplest possible CosmoFlow workflow using
//! synchronous execution for faster compilation and smaller binaries.
//!
//! ## Workflow Behavior
//! - **Hello Node**: Displays a greeting message and stores it in shared storage
//! - **Response Node**: Reads the greeting from storage and responds to it
//! - **Simple Communication**: Data flows between nodes via the shared store
//!
//! ## Core Features Demonstrated
//! - **Sync Node Implementation**: No async/await complexity
//! - **Built-in Storage Backend**: Uses CosmoFlow's MemoryStorage
//! - **Minimal Dependencies**: No tokio or async-trait required
//! - **Data Communication**: Nodes sharing data via the SharedStore
//! - **Individual Node Execution**: Direct node execution without Flow
//!
//! ## Performance Benefits
//! - 57% faster compilation compared to async version
//! - Smaller binary size (no async runtime overhead)
//! - Perfect for CPU-intensive workflows
//!
//! To run this example:
//! ```bash
//! cargo run --bin hello_world_sync --no-default-features --features cosmoflow/storage-memory
//! ```

#![cfg(all(feature = "sync", not(feature = "async")))]

use cosmoflow::{
    Node,
    action::Action,
    node::{ExecutionContext, NodeError},
    shared_store::SharedStore,
    shared_store::backends::MemoryStorage,
};

/// A simple greeting node that generates and stores a hello message (sync version)
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

impl Node<MemoryStorage> for HelloNode {
    type PrepResult = String;
    type ExecResult = String;
    type Error = NodeError;

    fn name(&self) -> &str {
        "HelloNode"
    }

    fn prep(
        &mut self,
        _store: &MemoryStorage,
        context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        let prepared_message = format!("Execution {}: {}", context.execution_id(), self.message);
        println!("🔄 [PREP] Preparing message: {}", prepared_message);
        Ok(prepared_message)
    }

    fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        println!("⚡ [EXEC] Processing greeting: {}", prep_result);

        // Simulate some synchronous work
        std::thread::sleep(std::time::Duration::from_millis(10));

        let processed_greeting = format!("🌟 {}", prep_result);
        Ok(processed_greeting)
    }

    fn post(
        &mut self,
        store: &mut MemoryStorage,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        println!("✅ [POST] Storing greeting: {}", exec_result);

        // Store the greeting for the next node
        store
            .set("greeting".to_string(), exec_result.clone())
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        println!("📤 Greeting stored successfully");
        Ok(Action::simple("next"))
    }
}

/// A response node that reads the greeting and generates a response (sync version)
struct ResponseNode {
    responder_name: String,
}

impl ResponseNode {
    fn new(responder_name: impl Into<String>) -> Self {
        Self {
            responder_name: responder_name.into(),
        }
    }
}

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
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        // Read the greeting from storage
        let greeting: String = store
            .get("greeting")
            .map_err(|e| NodeError::StorageError(e.to_string()))?
            .ok_or_else(|| {
                NodeError::ValidationError("No greeting found in storage".to_string())
            })?;

        println!("📥 [PREP] Retrieved greeting: {}", greeting);
        Ok(greeting)
    }

    fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        println!("⚡ [EXEC] Generating response to: {}", prep_result);

        // Simulate some synchronous processing
        std::thread::sleep(std::time::Duration::from_millis(5));

        let response = format!("🤝 Nice to meet you! - {}", self.responder_name);
        Ok(response)
    }

    fn post(
        &mut self,
        store: &mut MemoryStorage,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        println!("✅ [POST] Generated response: {}", exec_result);

        // Store the response
        store
            .set("response".to_string(), exec_result.clone())
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        println!("🎉 Workflow completed successfully!");
        Ok(Action::simple("complete"))
    }
}

/// Main function demonstrating synchronous node execution
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 CosmoFlow Hello World (Sync Version)");
    println!("========================================");
    println!("📦 Compiled without async features for minimal size!\n");

    // Create shared storage
    let mut store = MemoryStorage::new();

    // Create nodes
    let mut hello_node = HelloNode::new("Hello from CosmoFlow!");
    let mut response_node = ResponseNode::new("CosmoFlow Assistant");

    println!("🔄 Executing workflow...");
    println!("------------------------\n");

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

    println!("\n🎯 Sync Version Benefits:");
    println!("• ⚡ 57% faster compilation");
    println!("• 📦 Smaller binary size");
    println!("• 🎯 Perfect for CPU-intensive tasks");
    println!("• 🔧 Simpler debugging");
    println!("• 🚀 No async runtime overhead");

    println!("\n💡 Note: This example shows individual node execution");
    println!("   since Flow module currently requires async features.");
    println!("   Each node is executed manually in sequence.");

    Ok(())
}

// /// Dummy main function when sync example is not compiled
// #[cfg(not(all(feature = "sync", not(feature = "async"))))]
// fn main() {
//     // This sync example is not available when async features are enabled
// }
