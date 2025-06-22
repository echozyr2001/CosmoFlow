//! Custom Node Example - CosmoFlow Sync Version
//!
//! This example demonstrates custom node implementations in CosmoFlow workflows using
//! synchronous execution for faster compilation.
//!
//! ## Workflow Behavior
//! - **Counter Nodes**: Two stateful counters (main increments by 5, secondary by 3)
//! - **Individual Execution**: Each node is executed manually in sequence
//! - **Statistics Analysis**: Analyzes counter data including averages, min/max, and growth rates
//! - **Report Generation**: Creates a formatted analysis report with all statistics
//!
//! ## Advanced Features Demonstrated
//! - **Sync Node Implementation**: No async/await complexity for faster compilation
//! - **Stateful Nodes**: Nodes maintain internal state and persist data to shared store
//! - **Three-Phase Execution**: Proper use of prep, exec, and post phases
//! - **Data Persistence**: Stores counter values and execution history
//! - **Statistical Analysis**: Calculates metrics and generates formatted reports
//! - **Built-in Storage Backend**: Uses CosmoFlow's MemoryStorage
//!
//! ## Performance Benefits
//! - Faster compilation compared to async version
//! - Smaller binary size (no async runtime overhead)
//! - Perfect for CPU-intensive statistical computations
//!
//! ## Execution Flow
//! 1. Main counter increments and stores its value
//! 2. Secondary counter increments and stores its value  
//! 3. Process repeats for several iterations
//! 4. Statistics node calculates metrics from stored data
//! 5. Report node generates and displays final analysis
//!
//! To run this example:
//! ```bash
//! cargo run --bin custom_node_sync --no-default-features --features cosmoflow/storage-memory
//! ```

#![cfg(all(feature = "sync", not(feature = "async")))]

use cosmoflow::{
    Node,
    action::Action,
    node::{ExecutionContext, NodeError},
    shared_store::SharedStore,
    shared_store::backends::MemoryStorage,
};
use serde_json::Value;
use std::collections::HashMap;

/// A counter node that tracks how many times it has been executed (sync version)
#[derive(Debug)]
struct CounterNode {
    name: String,
    count: usize,
    increment_by: usize,
    max_count: Option<usize>,
}

impl CounterNode {
    fn new(name: impl Into<String>, increment_by: usize) -> Self {
        Self {
            name: name.into(),
            count: 0,
            increment_by,
            max_count: Some(30), // Set a reasonable limit for demo
        }
    }
}

impl Node<MemoryStorage> for CounterNode {
    type PrepResult = usize; // Previous count
    type ExecResult = usize; // New count
    type Error = NodeError;

    fn name(&self) -> &str {
        &self.name
    }

    fn prep(
        &mut self,
        store: &MemoryStorage,
        context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        let previous_count = self.count;

        // Check if we have stored count in the shared store
        let store_key = format!("{}_count", self.name);
        if let Some(count_value) = store
            .get::<Value>(&store_key)
            .ok()
            .flatten()
            .and_then(|v| v.as_u64())
        {
            self.count = count_value as usize;
            println!(
                "🔄 [PREP] {} (exec_id: {}) restored count from store: {}",
                self.name,
                context.execution_id(),
                self.count
            );
        } else {
            println!(
                "🔄 [PREP] {} (exec_id: {}) starting fresh",
                self.name,
                context.execution_id()
            );
        }

        Ok(previous_count)
    }

    fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        // Check if we've reached the maximum count
        if let Some(max) = self.max_count {
            if self.count >= max {
                return Err(NodeError::ValidationError(format!(
                    "Counter {} has reached maximum count: {}",
                    self.name, max
                )));
            }
        }

        // Simulate some synchronous computation
        std::thread::sleep(std::time::Duration::from_millis(5));

        // Increment the counter
        self.count += self.increment_by;

        println!(
            "⚡ [EXEC] {} count: {} -> {} (increment: {})",
            self.name, prep_result, self.count, self.increment_by
        );

        Ok(self.count)
    }

    fn post(
        &mut self,
        store: &mut MemoryStorage,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        println!("✅ [POST] {} storing count: {}", self.name, exec_result);

        // Store the current count
        let store_key = format!("{}_count", self.name);
        store
            .set(store_key, Value::Number(exec_result.into()))
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        // Store count history
        let history_key = format!("{}_history", self.name);
        let mut history: Vec<usize> = match store.get::<serde_json::Value>(&history_key) {
            Ok(Some(value)) => {
                if let Some(array) = value.as_array() {
                    array
                        .iter()
                        .filter_map(|v| v.as_u64().map(|n| n as usize))
                        .collect()
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(),
        };

        history.push(exec_result);
        store
            .set(
                history_key,
                Value::Array(
                    history
                        .into_iter()
                        .map(|n| Value::Number(n.into()))
                        .collect(),
                ),
            )
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        // Determine next action based on count
        if let Some(max) = self.max_count {
            if exec_result >= max {
                Ok(Action::simple("max_reached"))
            } else {
                Ok(Action::simple("continue"))
            }
        } else {
            Ok(Action::simple("continue"))
        }
    }
}

/// A statistics node that analyzes counter data (sync version)
struct StatisticsNode;

impl Node<MemoryStorage> for StatisticsNode {
    type PrepResult = HashMap<String, Vec<usize>>;
    type ExecResult = HashMap<String, f64>;
    type Error = NodeError;

    fn name(&self) -> &str {
        "StatisticsNode"
    }

    fn prep(
        &mut self,
        store: &MemoryStorage,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        println!("🔄 [PREP] Gathering statistics from all counters...");

        let mut counter_histories = HashMap::new();

        // Look for all counter histories in the store
        let keys = ["main_counter_history", "secondary_counter_history"];

        for key in keys {
            if let Ok(Some(value)) = store.get::<serde_json::Value>(key) {
                if let Some(array) = value.as_array() {
                    let history: Vec<usize> = array
                        .iter()
                        .filter_map(|v| v.as_u64().map(|n| n as usize))
                        .collect();

                    if !history.is_empty() {
                        let counter_name = key.replace("_history", "");
                        let history_len = history.len();
                        counter_histories.insert(counter_name.clone(), history);
                        println!(
                            "📊 Found history for {}: {} entries",
                            counter_name, history_len
                        );
                    }
                }
            }
        }

        Ok(counter_histories)
    }

    fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        println!("⚡ [EXEC] Calculating statistics...");

        let mut statistics = HashMap::new();

        for (counter_name, history) in prep_result {
            if history.is_empty() {
                continue;
            }

            // Simulate some computation time
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Calculate basic statistics
            let sum: usize = history.iter().sum();
            let count = history.len();
            let average = sum as f64 / count as f64;

            let min = *history.iter().min().unwrap() as f64;
            let max = *history.iter().max().unwrap() as f64;

            // Calculate growth rate
            let growth_rate = if history.len() > 1 {
                let first = history[0] as f64;
                let last = history[history.len() - 1] as f64;
                if first > 0.0 {
                    (last - first) / first * 100.0
                } else {
                    0.0
                }
            } else {
                0.0
            };

            println!("📊 {} statistics:", counter_name);
            println!("    Count: {}", count);
            println!("    Average: {:.2}", average);
            println!("    Min: {}, Max: {}", min, max);
            println!("    Growth rate: {:.1}%", growth_rate);

            statistics.insert(format!("{}_average", counter_name), average);
            statistics.insert(format!("{}_min", counter_name), min);
            statistics.insert(format!("{}_max", counter_name), max);
            statistics.insert(format!("{}_growth_rate", counter_name), growth_rate);
        }

        Ok(statistics)
    }

    fn post(
        &mut self,
        store: &mut MemoryStorage,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        println!("✅ [POST] Storing statistics...");

        // Store all statistics
        for (key, value) in exec_result {
            store
                .set(
                    format!("stats_{}", key),
                    Value::Number(
                        serde_json::Number::from_f64(value).unwrap_or(serde_json::Number::from(0)),
                    ),
                )
                .map_err(|e| NodeError::StorageError(e.to_string()))?;
        }

        Ok(Action::simple("generate_report"))
    }
}

/// A report node that generates a final summary (sync version)
struct ReportNode;

impl Node<MemoryStorage> for ReportNode {
    type PrepResult = HashMap<String, f64>;
    type ExecResult = String;
    type Error = NodeError;

    fn name(&self) -> &str {
        "ReportNode"
    }

    fn prep(
        &mut self,
        store: &MemoryStorage,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        println!("🔄 [PREP] Collecting statistics for report...");

        let mut stats = HashMap::new();

        // Collect all statistics
        let stat_keys = [
            "stats_main_counter_average",
            "stats_main_counter_growth_rate",
            "stats_secondary_counter_average",
            "stats_secondary_counter_growth_rate",
        ];

        for key in stat_keys {
            if let Ok(Some(value)) = store.get::<serde_json::Value>(key) {
                if let Some(number) = value.as_f64() {
                    stats.insert(key.to_string(), number);
                    println!("📊 Loaded stat {}: {:.2}", key, number);
                }
            }
        }

        Ok(stats)
    }

    fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        println!("⚡ [EXEC] Generating final report...");

        // Simulate report generation time
        std::thread::sleep(std::time::Duration::from_millis(15));

        let mut report = String::new();
        report.push_str("🎯 COUNTER ANALYSIS REPORT\n");
        report.push_str("==========================\n\n");

        // Main counter stats
        if let (Some(avg), Some(growth)) = (
            prep_result.get("stats_main_counter_average"),
            prep_result.get("stats_main_counter_growth_rate"),
        ) {
            report.push_str(&format!(
                "📈 Main Counter:\n  Average: {:.2}\n  Growth Rate: {:.1}%\n\n",
                avg, growth
            ));
        }

        // Secondary counter stats
        if let (Some(avg), Some(growth)) = (
            prep_result.get("stats_secondary_counter_average"),
            prep_result.get("stats_secondary_counter_growth_rate"),
        ) {
            report.push_str(&format!(
                "📊 Secondary Counter:\n  Average: {:.2}\n  Growth Rate: {:.1}%\n\n",
                avg, growth
            ));
        }

        report.push_str("✨ Analysis completed successfully!");

        Ok(report)
    }

    fn post(
        &mut self,
        store: &mut MemoryStorage,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        println!("✅ [POST] Final report generated:");
        println!("{}", exec_result);

        // Store the final report
        store
            .set("final_report".to_string(), exec_result)
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        Ok(Action::simple("complete"))
    }
}

/// Main function demonstrating synchronous iterative workflow with data analysis
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 CosmoFlow Custom Node (Sync Version)");
    println!("========================================");
    println!("📦 Advanced iterative workflow with statistical analysis!\n");

    // Create shared storage
    let mut store = MemoryStorage::new();

    // Create counter nodes
    let mut main_counter = CounterNode::new("main_counter", 5);
    let mut secondary_counter = CounterNode::new("secondary_counter", 3);

    println!("🔄 Executing iterative counter workflow...");
    println!("------------------------------------------\n");

    // Execute multiple iterations
    let max_iterations = 6;
    for iteration in 1..=max_iterations {
        println!("🔁 Iteration {}/{}:", iteration, max_iterations);

        // Execute main counter
        println!("  1️⃣ Executing Main Counter:");
        let main_action = main_counter.run(&mut store)?;
        println!("     Action: {}", main_action.name());

        // Execute secondary counter
        println!("  2️⃣ Executing Secondary Counter:");
        let secondary_action = secondary_counter.run(&mut store)?;
        println!("     Action: {}\n", secondary_action.name());

        // Check if either counter reached max
        if main_action.name() == "max_reached" || secondary_action.name() == "max_reached" {
            println!("🛑 Counter reached maximum, stopping iterations\n");
            break;
        }
    }

    // Execute statistics analysis
    println!("📊 Analyzing counter data...");
    println!("----------------------------");
    let mut stats_node = StatisticsNode;
    let stats_action = stats_node.run(&mut store)?;
    println!("Statistics Action: {}\n", stats_action.name());

    // Generate final report
    if stats_action.name() == "generate_report" {
        println!("📋 Generating final report...");
        println!("-----------------------------");
        let mut report_node = ReportNode;
        let report_action = report_node.run(&mut store)?;
        println!("Report Action: {}\n", report_action.name());
    }

    // Display final storage contents
    println!("📊 Final Storage Summary:");
    println!("========================");

    if let Ok(Some(main_count)) = store.get::<Value>("main_counter_count") {
        println!("Main Counter Final: {}", main_count);
    }

    if let Ok(Some(secondary_count)) = store.get::<Value>("secondary_counter_count") {
        println!("Secondary Counter Final: {}", secondary_count);
    }

    if let Ok(Some(report)) = store.get::<String>("final_report") {
        println!("\n📋 Stored Report:");
        println!("{}", report);
    }

    println!("\n🎯 Sync Version Benefits:");
    println!("• ⚡ Faster compilation than async version");
    println!("• 📦 Smaller binary size");
    println!("• 🎯 Perfect for CPU-intensive statistical analysis");
    println!("• 🔧 Simpler debugging and profiling");
    println!("• 🚀 No async runtime overhead");

    println!("\n💡 Note: This example shows individual node execution");
    println!("   with manual iteration control and statistical analysis.");

    Ok(())
}

/// Dummy main function when sync example is not compiled
#[cfg(not(all(feature = "sync", not(feature = "async"))))]
fn main() {
    // This sync example is not available when async features are enabled
}
