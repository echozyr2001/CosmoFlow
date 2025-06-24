//! # Unified Counter Example
//!
//! This example demonstrates a more complex workflow using the unified Node trait
//! with multiple nodes, state management, and conditional routing. It showcases
//! how to build iterative workflows that process data through multiple stages.
//!
//! ## Key Features Demonstrated:
//! - Multiple nodes in a single workflow
//! - State persistence and retrieval between nodes
//! - Conditional action generation based on computed values
//! - Sequential node execution with data flow
//! - Custom storage backend with error handling
//!
//! ## Workflow Description:
//! 1. Counter1 starts with 0, increments by 3 → 3
//! 2. Counter2 reads 3, increments by 3 → 6  
//! 3. Counter3 reads 6, increments by 5 → 11
//! 4. Since 11 >= 10, workflow terminates with "complete"
//!
//! Run with: `cargo run --bin unified_counter`

use async_trait::async_trait;
use cosmoflow::{
    action::Action,
    node::{ExecutionContext, NodeError},
    shared_store::{backends::MemoryStorage, SharedStore},
    FlowBackend, FlowBuilder, Node,
};

/// A counter node that demonstrates stateful computation and conditional routing.
///
/// This node implements a counter that:
/// 1. **Prep**: Reads the current counter value from shared storage
/// 2. **Exec**: Increments the counter by a specified amount (idempotent)
/// 3. **Post**: Stores the new value and decides whether to continue or complete
///
/// ## Features Demonstrated:
/// - Reading state from shared storage in prep phase
/// - Stateless computation in exec phase (safe for retries)
/// - Conditional action generation based on computed values
/// - State persistence for use by subsequent nodes
///
/// ## Conditional Logic:
/// - If counter >= 10: returns "complete" action (terminates workflow)
/// - If counter < 10: returns "continue" action (proceeds to next node)
struct CounterNode {
    /// The amount to increment the counter by
    increment: i32,
}

impl CounterNode {
    /// Creates a new CounterNode with the specified increment value.
    ///
    /// # Arguments
    /// * `increment` - The amount to add to the counter each time this node executes
    pub fn new(increment: i32) -> Self {
        Self { increment }
    }
}

#[async_trait]
impl<S: SharedStore + Send + Sync> Node<S> for CounterNode {
    /// Preparation phase returns the current counter value
    type PrepResult = i32;
    /// Execution phase returns the new counter value
    type ExecResult = i32;
    /// All errors are wrapped in NodeError for consistency
    type Error = NodeError;

    /// Preparation phase: Read the current counter value from shared storage.
    ///
    /// This phase retrieves the current state that will be used for computation.
    /// If no counter exists yet, it defaults to 0. This demonstrates how nodes
    /// can read and validate their inputs before execution.
    ///
    /// # Arguments
    /// * `store` - Shared storage for reading the current counter value
    /// * `_context` - Execution context (unused in this example)
    ///
    /// # Returns
    /// * `Ok(i32)` - The current counter value (0 if not found)
    /// * `Err(NodeError)` - If storage access fails
    async fn prep(
        &mut self,
        store: &S,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        // Read current counter value from storage, defaulting to 0 if not found
        let current_count: i32 = store
            .get("counter")
            .map_err(|e| NodeError::StorageError(e.to_string()))?
            .unwrap_or(0);

        println!("Current counter value: {}", current_count);
        Ok(current_count)
    }

    /// Execution phase: Increment the counter value.
    ///
    /// This is the core computation logic. It takes the current counter value
    /// from the prep phase and adds the increment. This operation is idempotent
    /// and safe to retry since it doesn't have side effects.
    ///
    /// # Arguments
    /// * `prep_result` - The current counter value from prep phase
    /// * `_context` - Execution context (unused in this example)
    ///
    /// # Returns
    /// * `Ok(i32)` - The new counter value after incrementing
    async fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        // Perform the increment operation
        let new_count = prep_result + self.increment;
        println!(
            "Incrementing counter by {}: {} -> {}",
            self.increment, prep_result, new_count
        );
        Ok(new_count)
    }

    /// Post-processing phase: Store the new value and determine next action.
    ///
    /// This phase handles the side effects (storing the result) and makes
    /// the routing decision based on the computed value. The conditional
    /// logic determines whether the workflow should continue or terminate.
    ///
    /// # Arguments
    /// * `store` - Mutable shared storage for writing the new counter value
    /// * `_prep_result` - Original counter value (unused here)
    /// * `exec_result` - New counter value from exec phase
    /// * `_context` - Execution context (unused here)
    ///
    /// # Returns
    /// * `Ok(Action)` - "complete" if counter >= 10, "continue" otherwise
    /// * `Err(NodeError)` - If storage operations fail
    async fn post(
        &mut self,
        store: &mut S,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        // Store the new counter value for use by subsequent nodes
        store
            .set("counter".to_string(), exec_result)
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        // Conditional routing: complete if we've reached the threshold
        if exec_result >= 10 {
            Ok(Action::simple("complete"))
        } else {
            Ok(Action::simple("continue"))
        }
    }

    /// Returns the human-readable name of this node for logging and debugging.
    fn name(&self) -> &str {
        "CounterNode"
    }
}

/// Main function demonstrating a multi-node workflow with state management.
///
/// This example shows how to:
/// 1. Create multiple counter nodes with different increment values
/// 2. Chain nodes together using routing
/// 3. Pass state between nodes through shared storage
/// 4. Use conditional actions to control workflow termination
///
/// The workflow demonstrates a sequential counter chain where each node
/// reads the previous counter value, increments it, and passes it to the next node.
/// The workflow terminates when the counter reaches or exceeds 10.
///
/// ## Expected Execution Flow:
/// 1. counter1: 0 + 3 = 3 → "continue" → counter2
/// 2. counter2: 3 + 3 = 6 → "continue" → counter3  
/// 3. counter3: 6 + 5 = 11 → "complete" → workflow ends
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 CosmoFlow Unified Node Trait Example");
    println!("========================================");

    // Create shared storage with MemoryStorage backend
    let mut store = MemoryStorage::new();

    // Build a multi-node workflow with sequential counter processing
    // Each counter node will:
    // 1. Read the current counter value (or 0 if first)
    // 2. Add its increment value
    // 3. Store the result for the next node
    // 4. Return "continue" or "complete" based on the final value
    let mut flow = FlowBuilder::new()
        .start_with("counter1", CounterNode::new(3)) // Start: 0 + 3 = 3
        .node("counter2", CounterNode::new(3)) // Next: 3 + 3 = 6
        .node("counter3", CounterNode::new(5)) // Final: 6 + 5 = 11
        .route("hello", "next", "counter1") // Unused route (legacy)
        .route("counter1", "continue", "counter2") // 3 → counter2
        .route("counter2", "continue", "counter3") // 6 → counter3
        .terminal_route("counter3", "complete") // 11 → terminate (explicit)
        .max_steps(20) // Safety limit
        .build();

    println!("\n📋 Executing workflow...");
    println!("------------------");

    // Execute the workflow and capture detailed results
    let result = flow.execute(&mut store).await?;

    // Display comprehensive execution results
    println!("\n✅ Workflow execution completed!");
    println!("==================");
    println!("Success: {}", result.success);
    println!("Steps executed: {}", result.steps_executed);
    println!("Final action: {}", result.final_action.name());
    println!("Execution path: {:?}", result.execution_path);

    // Inspect the final state of shared storage
    println!("\n📊 Shared storage contents:");
    println!("------------------");

    // Check for greeting (won't exist in this example)
    if let Ok(Some(greeting)) = store.get::<String>("greeting") {
        println!("greeting: {greeting}");
    }

    // Check the final counter value (should be 11)
    if let Ok(Some(counter)) = store.get::<i32>("counter") {
        println!("counter: {counter}");
    }

    Ok(())
}
