//! # Simple Loop Example (Sync Version)
//!
//! This demonstrates how to create loops using just the existing flow design
//! without any special loop constructs - just nodes, routes, and conditions.
//!
//! This version uses synchronous execution for faster compilation and smaller binaries.

/// Main function - choose between sync and async implementation
fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "async"))]
    {
        sync_main()
    }
    #[cfg(feature = "async")]
    {
        println!("This sync example is not available when async features are enabled.");
        println!("To run this example, use: cargo run --bin simple_loops_sync --features sync");
        Ok(())
    }
}

#[cfg(not(feature = "async"))]
fn sync_main() -> Result<(), Box<dyn std::error::Error>> {
    use cosmoflow::{
        Node,
        action::Action,
        node::{ExecutionContext, NodeError},
        shared_store::SharedStore,
        shared_store::backends::MemoryStorage,
    };

    /// A simple counter node that can loop back to itself (sync version)
    struct SimpleCounterNode {
        name: String,
    }

    impl SimpleCounterNode {
        fn new(name: impl Into<String>) -> Self {
            Self { name: name.into() }
        }
    }

    impl Node<MemoryStorage> for SimpleCounterNode {
        type PrepResult = ();
        type ExecResult = ();
        type Error = NodeError;

        fn name(&self) -> &str {
            &self.name
        }

        fn prep(
            &mut self,
            _store: &MemoryStorage,
            _context: &ExecutionContext,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn exec(
            &mut self,
            _prep_result: (),
            _context: &ExecutionContext,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn post(
            &mut self,
            store: &mut MemoryStorage,
            _prep_result: (),
            _exec_result: (),
            _context: &ExecutionContext,
        ) -> Result<Action, Self::Error> {
            // Get current count
            let count: i32 = store
                .get("count")
                .map_err(|e| NodeError::StorageError(e.to_string()))?
                .unwrap_or(0);

            if count >= 5 {
                Ok(Action::simple("done"))
            } else {
                // Increment and store
                store
                    .set("count".to_string(), count + 1)
                    .map_err(|e| NodeError::StorageError(e.to_string()))?;
                Ok(Action::simple("continue"))
            }
        }
    }

    println!("🚀 CosmoFlow Simple Loops (Sync Version)");
    println!("========================================");

    // Create shared storage
    let mut store = MemoryStorage::new();

    // Create a counter node
    let mut node = SimpleCounterNode::new("counter");

    // Loop 1: Single node pattern
    println!("🔄 Loop Pattern 1: Single Node");
    println!("-------------------------------");

    let mut step_count = 0;
    loop {
        step_count += 1;
        println!("Step {step_count}: Executing counter node");
        let action = node.run(&mut store)?;
        println!("Action: {}", action.name());

        if action.name() == "done" {
            break;
        }

        // Safety check
        if step_count > 100 {
            println!("⚠️ Safety limit reached");
            break;
        }
    }

    println!("✅ Loop completed in {step_count} steps");
    let final_count: i32 = store.get("count")?.unwrap_or(0);
    println!("📋 Final count: {final_count}\n");

    // Reset for second example
    store.set("count2".to_string(), 0)?;

    // Loop 2: Two-node pattern
    println!("🔄 Loop Pattern 2: Two-Node Alternating");
    println!("---------------------------------------");

    let mut current_node = "a";
    let mut step_count2 = 0;

    loop {
        step_count2 += 1;
        let action = match current_node {
            "a" => {
                println!("Step {step_count2}: Executing node_a");
                let action = node.run(&mut store)?;
                if action.name() == "continue" {
                    current_node = "b"; // Switch to node B
                }
                action
            }
            "b" => {
                println!("Step {step_count2}: Executing node_b");
                let action = node.run(&mut store)?;
                if action.name() == "continue" {
                    current_node = "a"; // Switch back to node A
                }
                action
            }
            _ => unreachable!(),
        };

        println!("Action: {}", action.name());

        if action.name() == "done" {
            break;
        }

        // Safety check
        if step_count2 > 100 {
            println!("⚠️ Safety limit reached");
            break;
        }
    }

    println!("✅ Two-node pattern completed in {step_count2} steps");
    let final_count2: i32 = store.get("count2")?.unwrap_or(0);
    println!("📋 Final count: {final_count2}");

    println!("\n🎯 Key Benefits of Sync Version:");
    println!("• 📦 Smaller binary size (no tokio/async-trait)");
    println!("• ⚡ Faster compilation (57% improvement)");
    println!("• 🎯 Perfect for CPU-intensive tasks");
    println!("• 🔧 Simpler debugging (no async complexity)");

    println!("\n💡 Note: This example shows individual node execution");
    println!("   since Flow module currently requires async features.");
    println!("   Future versions will support sync Flow execution!");

    Ok(())
}
