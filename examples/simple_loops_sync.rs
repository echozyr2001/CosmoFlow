//! # Simple Loop Example (Sync Version)
//!
//! This demonstrates how to create loops using just the existing flow design
//! without any special loop constructs - just nodes, routes, and conditions.
//!
//! This version uses synchronous execution for faster compilation and smaller binaries.

#![cfg(all(feature = "sync", not(feature = "async")))]

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
    key: String,
    increment: i32,
    max_count: i32,
}

impl SimpleCounterNode {
    fn new(name: &str, key: &str, increment: i32, max_count: i32) -> Self {
        Self {
            name: name.to_string(),
            key: key.to_string(),
            increment,
            max_count,
        }
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

    fn exec(&mut self, _prep_result: (), _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn post(
        &mut self,
        store: &mut MemoryStorage,
        _prep_result: (),
        _exec_result: (),
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        // Get current value
        let current: i32 = store
            .get(&self.key)
            .map_err(|e| NodeError::from(e.to_string()))?
            .unwrap_or(0);
        let new_value = current + self.increment;

        // Store new value
        store
            .set(self.key.clone(), new_value)
            .map_err(|e| NodeError::from(e.to_string()))?;

        println!("📊 {}: {} -> {}", self.key, current, new_value);

        // Simple condition: continue if below max, exit if reached
        if new_value < self.max_count {
            Ok(Action::simple("continue")) // This will route back to self
        } else {
            Ok(Action::simple("done")) // This will exit the loop
        }
    }
}

/// Demonstrates individual node execution without Flow (since Flow requires async)
#[cfg(all(feature = "sync", not(feature = "async")))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Simple Counter Example (Sync Version)");
    println!("=========================================\n");

    let mut store = MemoryStorage::new();

    // Example 1: Simple count loop using individual node execution
    println!("🔢 Sync Counter Loop (increment by 2 until >= 10)");
    println!("--------------------------------------------------");

    let mut counter_node = SimpleCounterNode::new("counter", "count", 2, 10);
    let mut step_count = 0;

    loop {
        step_count += 1;
        println!("Step {}: Executing counter node", step_count);

        let action = counter_node.run(&mut store)?;
        println!("Action: {}", action.name());

        if action.name() == "done" {
            break;
        }

        // Safety check to prevent infinite loops
        if step_count > 100 {
            println!("⚠️ Safety limit reached");
            break;
        }
    }

    println!("✅ Loop completed in {} steps", step_count);
    let final_count: i32 = store.get("count")?.unwrap_or(0);
    println!("📋 Final count: {}\n", final_count);

    // Example 2: Two-node alternating pattern
    println!("🔄 Two-Node Alternating Pattern");
    println!("--------------------------------");

    // Reset counter
    store.set("count2".to_string(), 0)?;

    let mut node_a = SimpleCounterNode::new("node_a", "count2", 1, 8);
    let mut node_b = SimpleCounterNode::new("node_b", "count2", 2, 8);
    let mut current_node = "a";
    let mut step_count2 = 0;

    loop {
        step_count2 += 1;

        let action = match current_node {
            "a" => {
                println!("Step {}: Executing node_a", step_count2);
                let action = node_a.run(&mut store)?;
                if action.name() == "continue" {
                    current_node = "b"; // Switch to node B
                }
                action
            }
            "b" => {
                println!("Step {}: Executing node_b", step_count2);
                let action = node_b.run(&mut store)?;
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

    println!("✅ Two-node pattern completed in {} steps", step_count2);
    let final_count2: i32 = store.get("count2")?.unwrap_or(0);
    println!("📋 Final count: {}", final_count2);

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

/// Dummy main function when sync example is not compiled
#[cfg(not(all(feature = "sync", not(feature = "async"))))]
fn main() {
    // This sync example is not available when async features are enabled
}
