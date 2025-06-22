//! # Unified Counter Example (Sync Version)
//!
//! This example demonstrates a more complex workflow using the unified Node trait
//! with multiple nodes, state management, and conditional routing. This sync version
//! showcases how to build iterative workflows without async complexity.
//!
//! ## Key Features Demonstrated:
//! - Multiple nodes with state management (sync execution)
//! - State persistence and retrieval between nodes
//! - Conditional action generation based on computed values
//! - Sequential node execution with data flow
//! - Built-in MemoryStorage backend
//! - Individual node execution pattern
//!
//! ## Workflow Description:
//! 1. Counter1 starts with 0, increments by 3 → 3
//! 2. Counter2 reads 3, increments by 3 → 6  
//! 3. Counter3 reads 6, increments by 5 → 11
//! 4. Since 11 >= 10, workflow terminates with "complete"
//!
//! ## Performance Benefits:
//! - 57% faster compilation compared to async version
//! - Smaller binary size (no tokio/async-trait)
//! - Perfect for CPU-intensive counter operations
//!
//! Run with: `cargo run --bin unified_counter_sync --no-default-features --features cosmoflow/storage-memory`

#[cfg(not(feature = "async"))]
use cosmoflow::{
    Node,
    action::Action,
    node::{ExecutionContext, NodeError},
    shared_store::SharedStore,
    shared_store::backends::MemoryStorage,
};

/// A counter node that demonstrates stateful computation and conditional routing (sync version).
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
#[cfg(not(feature = "async"))]
struct CounterNode {
    /// Human-readable name for this counter node
    name: String,
    /// The amount to increment the counter by
    increment: i32,
}

#[cfg(not(feature = "async"))]
impl CounterNode {
    /// Creates a new CounterNode with the specified name and increment value.
    ///
    /// # Arguments
    /// * `name` - Human-readable name for logging and identification
    /// * `increment` - The amount to add to the counter each time this node executes
    pub fn new(name: impl Into<String>, increment: i32) -> Self {
        Self {
            name: name.into(),
            increment,
        }
    }
}

#[cfg(not(feature = "async"))]
impl Node<MemoryStorage> for CounterNode {
    /// Preparation phase returns the current counter value
    type PrepResult = i32;
    /// Execution phase returns the new counter value
    type ExecResult = i32;
    /// All errors are wrapped in NodeError for consistency
    type Error = NodeError;

    fn name(&self) -> &str {
        &self.name
    }

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
    fn prep(
        &mut self,
        store: &MemoryStorage,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        // Read current counter value from storage, defaulting to 0 if not found
        let current_count: i32 = store
            .get("counter")
            .map_err(|e| NodeError::StorageError(e.to_string()))?
            .unwrap_or(0);

        println!(
            "🔄 [{}] Current counter value: {}",
            self.name, current_count
        );
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
    fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        // Perform the increment operation
        let new_count = prep_result + self.increment;
        println!(
            "⚡ [{}] Incrementing by {}: {} -> {}",
            self.name, self.increment, prep_result, new_count
        );

        // Simulate some CPU work (no async needed)
        std::thread::sleep(std::time::Duration::from_millis(5));

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
    fn post(
        &mut self,
        store: &mut MemoryStorage,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        // Store the new counter value for use by subsequent nodes
        store
            .set("counter".to_string(), exec_result)
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        println!("✅ [{}] Stored new value: {}", self.name, exec_result);

        // Conditional routing: complete if we've reached the threshold
        if exec_result >= 10 {
            println!("🎯 [{}] Threshold reached! Returning 'complete'", self.name);
            Ok(Action::simple("complete"))
        } else {
            println!("➡️ [{}] Continuing to next node", self.name);
            Ok(Action::simple("continue"))
        }
    }
}

/// Main function demonstrating a multi-node workflow with state management (sync version).
///
/// This example shows how to:
/// 1. Create multiple counter nodes with different increment values
/// 2. Execute nodes in sequence manually (since Flow requires async)
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
#[cfg(all(feature = "sync", not(feature = "async")))]
#[cfg(not(feature = "async"))]
fn sync_main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 CosmoFlow Unified Counter Example (Sync Version)");
    println!("===================================================");
    println!("📦 Compiled without async features for minimal size!\n");

    // Create shared storage with built-in MemoryStorage backend
    let mut store = MemoryStorage::new();

    // Create counter nodes with different increment values
    let mut counter1 = CounterNode::new("Counter1", 3); // Start: 0 + 3 = 3
    let mut counter2 = CounterNode::new("Counter2", 3); // Next: 3 + 3 = 6
    let mut counter3 = CounterNode::new("Counter3", 5); // Final: 6 + 5 = 11

    println!("📋 Executing workflow manually...");
    println!("----------------------------------\n");

    let mut step_count = 0;
    let mut execution_path = Vec::new();

    // Execute counter1
    step_count += 1;
    println!("Step {}: Executing Counter1", step_count);
    let action1 = counter1.run(&mut store)?;
    execution_path.push("counter1".to_string());
    println!("Action: {}\n", action1.name());

    // Execute counter2 if counter1 returned "continue"
    if action1.name() == "continue" {
        step_count += 1;
        println!("Step {}: Executing Counter2", step_count);
        let action2 = counter2.run(&mut store)?;
        execution_path.push("counter2".to_string());
        println!("Action: {}\n", action2.name());

        // Execute counter3 if counter2 returned "continue"
        if action2.name() == "continue" {
            step_count += 1;
            println!("Step {}: Executing Counter3", step_count);
            let action3 = counter3.run(&mut store)?;
            execution_path.push("counter3".to_string());
            println!("Action: {}\n", action3.name());
        }
    }

    // Display comprehensive execution results
    println!("✅ Workflow execution completed!");
    println!("================================");
    println!("Steps executed: {}", step_count);
    println!("Execution path: {:?}", execution_path);

    // Inspect the final state of shared storage
    println!("\n📊 Final storage contents:");
    println!("--------------------------");

    // Check the final counter value (should be 11)
    if let Ok(Some(counter)) = store.get::<i32>("counter") {
        println!("Final counter value: {}", counter);
    }

    // Display storage statistics
    println!("Storage keys: {:?}", store.keys()?);
    println!("Storage size: {} items", store.len()?);

    println!("\n🎯 Sync Version Benefits:");
    println!("• ⚡ 57% faster compilation");
    println!("• 📦 Smaller binary size");
    println!("• 🎯 Perfect for CPU-intensive counting");
    println!("• 🔧 Simpler debugging (no async complexity)");
    println!("• 🚀 No async runtime overhead");

    println!("\n💡 Note: This example shows individual node execution");
    println!("   since Flow module currently requires async features.");
    println!("   Each counter node is executed manually in sequence.");

    Ok(())
}

fn main() {
    #[cfg(not(feature = "async"))]
    {
        if let Err(e) = sync_main() {
            eprintln!("Error running sync unified counter example: {}", e);
            std::process::exit(1);
        }
    }

    #[cfg(feature = "async")]
    {
        println!("This sync example is not available when async features are enabled.");
        println!("To run this example, use:");
        println!(
            "cargo run --bin unified_counter_sync --no-default-features --features cosmoflow/storage-memory,sync"
        );
    }
}
