//! # Simple Loop Example
//!
//! This demonstrates how to create loops using just the existing flow design
//! without any special loop constructs - just nodes, routes, and conditions.

use async_trait::async_trait;
use cosmoflow::prelude::*;

/// A simple counter node that can loop back to itself
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

#[async_trait]
impl Node<MemoryStorage> for SimpleCounterNode {
    type PrepResult = ();
    type ExecResult = ();
    type Error = NodeError;

    fn name(&self) -> &str {
        &self.name
    }

    async fn prep(
        &mut self,
        _store: &MemoryStorage,
        _context: &ExecutionContext,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn exec(
        &mut self,
        _prep_result: (),
        _context: &ExecutionContext,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn post(
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Simple Loop Example");
    println!("======================\n");

    let mut store = MemoryStorage::new();

    // Example 1: Simple count loop - just route back to self!
    println!("🔢 Count Loop (increment by 2 until >= 10)");
    println!("-------------------------------------------");

    let mut flow = FlowBuilder::new()
        .start_node("counter")
        .max_steps(100) // Allow intentional loops
        .node("counter", SimpleCounterNode::new("counter", "count", 2, 10))
        .self_route("counter", "continue") // Loop back to self - more explicit!
        .terminal_route("counter", "done") // Exit when done
        .build();

    let result = flow.execute(&mut store).await?;
    println!("✅ Loop completed in {} steps", result.steps_executed);

    let final_count: i32 = store.get("count")?.unwrap_or(0);
    println!("📋 Final count: {final_count}\n");

    // Example 2: Two-node loop
    println!("🔄 Two-Node Loop");
    println!("----------------");

    // Reset counter
    store.set("count2".to_string(), 0)?;

    let mut flow2 = FlowBuilder::new()
        .start_node("node_a")
        .max_steps(100)
        .node("node_a", SimpleCounterNode::new("node_a", "count2", 1, 5))
        .node("node_b", SimpleCounterNode::new("node_b", "count2", 2, 5))
        .route("node_a", "continue", "node_b") // A -> B
        .route("node_b", "continue", "node_a") // B -> A (creates loop)
        .terminal_route("node_a", "done") // Exit from A
        .terminal_route("node_b", "done") // Exit from B
        .build();

    let result2 = flow2.execute(&mut store).await?;
    println!(
        "✅ Two-node loop completed in {} steps",
        result2.steps_executed
    );

    let final_count2: i32 = store.get("count2")?.unwrap_or(0);
    println!("📋 Final count: {final_count2}");

    println!("\n🎯 Key Insight: Loops are just routes + conditions!");
    println!("💡 No special loop constructs needed - the existing");
    println!("   flow design already supports all loop patterns.");
    println!("✨ New .self_route() method makes intent clearer!");

    Ok(())
}
