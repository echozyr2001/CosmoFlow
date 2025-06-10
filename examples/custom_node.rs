//! Custom Node Example - Advanced Iterative Workflow with Data Analysis
//!
//! This example demonstrates sophisticated custom nodes that implement an iterative
//! counter workflow with data analysis and reporting. The workflow features:
//!
//! ## Workflow Behavior
//! - **Coordinator Node**: Orchestrates the iteration logic, deciding which counter to increment
//! - **Counter Nodes**: Two stateful counters (main increments by 5, secondary by 3)
//! - **Iterative Execution**: Counters alternate execution until reaching 6 total iterations
//! - **Statistics Analysis**: Analyzes counter data including averages, min/max, and growth rates
//! - **Report Generation**: Creates a formatted analysis report with all statistics
//!
//! ## Advanced Features Demonstrated
//! - **Stateful Nodes**: Nodes maintain internal state and persist data to shared store
//! - **Complex Business Logic**: Coordinator implements sophisticated iteration control
//! - **Three-Phase Execution**: Proper use of prep, exec, and post phases
//! - **Cycle-Aware Workflows**: Disables cycle detection to allow iterative patterns
//! - **Data Persistence**: Stores counter values and execution history
//! - **Statistical Analysis**: Calculates metrics and generates formatted reports
//! - **Custom Storage Backend**: Implements a complete storage interface
//!
//! ## Execution Flow
//! 1. Coordinator analyzes current state and decides next action
//! 2. Appropriate counter increments and stores its new value
//! 3. Counter returns control to coordinator for next iteration
//! 4. After 6 iterations, workflow transitions to analysis phase
//! 5. Statistics node calculates metrics from stored data
//! 6. Report node generates and displays final analysis
//!
//! To run this example:
//! ```bash
//! cd examples && cargo run --bin custom_node --features minimal
//! ```

use async_trait::async_trait;
use cosmoflow::{
    StorageBackend,
    action::Action,
    flow::{FlowBackend, FlowBuilder},
    node::{ExecutionContext, Node, NodeBackend, NodeError},
    shared_store::SharedStore,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
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

/// A coordinator node that manages the counter workflow
struct CoordinatorNodeBackend {
    main_count: usize,
    secondary_count: usize,
    max_iterations: usize,
}

impl CoordinatorNodeBackend {
    fn new(max_iterations: usize) -> Self {
        Self {
            main_count: 0,
            secondary_count: 0,
            max_iterations,
        }
    }
}

#[async_trait]
impl NodeBackend<SimpleStorage> for CoordinatorNodeBackend {
    type PrepResult = (usize, usize); // (main_count, secondary_count)
    type ExecResult = String; // Next action to take
    type Error = NodeError;

    async fn prep(
        &mut self,
        store: &SharedStore<SimpleStorage>,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        // Get current counts from store
        if let Ok(Some(main_value)) = store.get::<Value>("main_counter_count") {
            if let Some(count) = main_value.as_u64() {
                self.main_count = count as usize;
            }
        }

        if let Ok(Some(secondary_value)) = store.get::<Value>("secondary_counter_count") {
            if let Some(count) = secondary_value.as_u64() {
                self.secondary_count = count as usize;
            }
        }

        Ok((self.main_count, self.secondary_count))
    }

    async fn exec(
        &mut self,
        (main_count, secondary_count): Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        let total_iterations = (main_count / 5) + (secondary_count / 3); // Calculate based on increments

        println!(
            "🎮 Coordinator: main={}, secondary={}, iterations={}/{}",
            main_count, secondary_count, total_iterations, self.max_iterations
        );

        if total_iterations >= self.max_iterations {
            Ok("analysis".to_string())
        } else if main_count <= secondary_count {
            Ok("increment_main".to_string())
        } else {
            Ok("increment_secondary".to_string())
        }
    }

    async fn post(
        &mut self,
        _store: &mut SharedStore<SimpleStorage>,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        Ok(Action::simple(&exec_result))
    }

    fn name(&self) -> &str {
        "CoordinatorNode"
    }
}

/// A counter node that tracks how many times it has been executed
#[derive(Debug)]
struct CounterNodeBackend {
    name: String,
    count: usize,
    increment_by: usize,
    max_count: Option<usize>,
}

impl CounterNodeBackend {
    fn new(name: impl Into<String>, increment_by: usize) -> Self {
        Self {
            name: name.into(),
            count: 0,
            increment_by,
            max_count: None,
        }
    }
}

#[async_trait]
impl NodeBackend<SimpleStorage> for CounterNodeBackend {
    type PrepResult = usize; // Previous count
    type ExecResult = usize; // New count
    type Error = NodeError;

    async fn prep(
        &mut self,
        store: &SharedStore<SimpleStorage>,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        let previous_count = self.count;

        // Check if we have stored count in the shared store
        let store_key = format!("{}_count", self.name);
        if let Ok(Some(stored_count)) = store.get::<Value>(&store_key) {
            if let Some(count_value) = stored_count.as_u64() {
                self.count = count_value as usize;
                println!("📊 {} restored count from store: {}", self.name, self.count);
            }
        }

        Ok(previous_count)
    }

    async fn exec(
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

        // Increment the counter
        self.count += self.increment_by;

        println!(
            "🔢 {} count: {} -> {} (increment: {})",
            self.name, prep_result, self.count, self.increment_by
        );

        Ok(self.count)
    }

    async fn post(
        &mut self,
        store: &mut SharedStore<SimpleStorage>,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
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

        // Determine next action based on count - return to coordinator for iteration
        if let Some(max) = self.max_count {
            if exec_result >= max {
                Ok(Action::simple("max_reached"))
            } else {
                Ok(Action::simple("return_to_coordinator"))
            }
        } else {
            Ok(Action::simple("return_to_coordinator"))
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// A statistics node that analyzes counter data
struct StatisticsNode;

#[async_trait]
impl NodeBackend<SimpleStorage> for StatisticsNode {
    type PrepResult = HashMap<String, Vec<usize>>;
    type ExecResult = HashMap<String, f64>;
    type Error = NodeError;

    async fn prep(
        &mut self,
        store: &SharedStore<SimpleStorage>,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        println!("📈 Gathering statistics from all counters...");

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
                        counter_histories.insert(counter_name, history);
                    }
                }
            }
        }

        Ok(counter_histories)
    }

    async fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        let mut statistics = HashMap::new();

        for (counter_name, history) in prep_result {
            if history.is_empty() {
                continue;
            }

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

    async fn post(
        &mut self,
        store: &mut SharedStore<SimpleStorage>,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
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

        Ok(Action::simple("report"))
    }

    fn name(&self) -> &str {
        "StatisticsNode"
    }
}

/// A report node that generates a final summary
struct ReportNode;

#[async_trait]
impl NodeBackend<SimpleStorage> for ReportNode {
    type PrepResult = HashMap<String, f64>;
    type ExecResult = String;
    type Error = NodeError;

    async fn prep(
        &mut self,
        store: &SharedStore<SimpleStorage>,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
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
                }
            }
        }

        Ok(stats)
    }

    async fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        let mut report = String::new();
        report.push_str("📋 COUNTER ANALYSIS REPORT\n");
        report.push_str("==========================\n\n");

        if prep_result.is_empty() {
            report.push_str("No statistics available.\n");
        } else {
            for (key, value) in prep_result {
                let display_key = key
                    .replace("stats_", "")
                    .replace("_", " ")
                    .split_whitespace()
                    .map(|word| {
                        let mut chars = word.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => {
                                first.to_uppercase().collect::<String>() + chars.as_str()
                            }
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                if key.contains("growth_rate") {
                    report.push_str(&format!("{}: {:.1}%\n", display_key, value));
                } else {
                    report.push_str(&format!("{}: {:.2}\n", display_key, value));
                }
            }
        }

        report.push_str("\nAnalysis completed successfully! 🎉");

        println!("{}", report);
        Ok(report)
    }

    async fn post(
        &mut self,
        store: &mut SharedStore<SimpleStorage>,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        store
            .set("final_report".to_string(), Value::String(exec_result))
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        Ok(Action::simple("complete"))
    }

    fn name(&self) -> &str {
        "ReportNode"
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Custom Node Example");
    println!("================================");

    // Create a shared store
    let mut store = SharedStore::with_storage(SimpleStorage::new());

    // Build a workflow that allows cycles for iterative behavior
    let mut flow = FlowBuilder::new()
        .start_node("coordinator")
        .max_steps(50)
        .node("coordinator", Node::new(CoordinatorNodeBackend::new(6))) // 6 total increments
        .node(
            "main_counter",
            Node::new(CounterNodeBackend::new("main_counter", 5)),
        )
        .node(
            "secondary_counter",
            Node::new(CounterNodeBackend::new("secondary_counter", 3)),
        )
        .node("statistics", Node::new(StatisticsNode))
        .node("report", Node::new(ReportNode))
        // Coordinator routes to appropriate counter or analysis
        .route("coordinator", "increment_main", "main_counter")
        .route("coordinator", "increment_secondary", "secondary_counter")
        .route("coordinator", "analysis", "statistics")
        // Counters return to coordinator for iteration
        .route("main_counter", "return_to_coordinator", "coordinator")
        .route("main_counter", "max_reached", "coordinator")
        .route("secondary_counter", "return_to_coordinator", "coordinator")
        .route("secondary_counter", "max_reached", "coordinator")
        // Analysis flow
        .route("statistics", "report", "report")
        .terminal_action("complete")
        .build();

    // Disable cycle detection to allow iterative workflows
    let mut config = flow.config().clone();
    config.detect_cycles = false;
    flow.set_config(config);

    println!("📋 Flow configuration:");
    println!("  Start node: {}", flow.config().start_node_id);
    println!("  Max steps: {}", flow.config().max_steps);
    println!();

    // Validate the flow
    if let Err(e) = flow.validate() {
        eprintln!("❌ Flow validation failed: {}", e);
        return Err(e.into());
    }
    println!("✅ Flow validation passed");
    println!();

    // Execute the workflow
    println!("⚡ Executing custom node workflow...");
    println!("------------------------------------");

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
    println!("  Execution path length: {}", result.execution_path.len());
    println!("  Duration: {:?}", duration);
    println!();

    // Show some final statistics from the store
    println!("🔍 Final store contents:");
    if let Ok(Some(main_count)) = store.get::<Value>("main_counter_count") {
        println!("  Main counter final value: {}", main_count);
    }
    if let Ok(Some(secondary_count)) = store.get::<Value>("secondary_counter_count") {
        println!("  Secondary counter final value: {}", secondary_count);
    }

    println!("\n💡 Key custom node features demonstrated:");
    println!("  - Stateful nodes that maintain internal state");
    println!("  - Complex preparation logic that reads from shared store");
    println!("  - Business logic validation in exec phase");
    println!("  - Rich post-processing that updates multiple store keys");
    println!("  - Conditional routing based on node state");
    println!("  - Data analysis and reporting capabilities");

    Ok(())
}
