//! Flow Composition Example - CosmoFlow Sync Version
//!
//! This example demonstrates flow composition in CosmoFlow workflows using
//! synchronous execution for faster compilation.
//!
//! ## Workflow Behavior
//! - **Decision Node**: Evaluates conditions and chooses between success/error paths
//! - **Success Path**: Handles positive outcomes and continues to final processing
//! - **Error Path**: Handles negative outcomes and also continues to final processing
//! - **Final Node**: Convergence point where both paths complete the workflow
//! - **Manual Orchestration**: Explicit control flow without macro complexity
//!
//! ## Advanced Features Demonstrated
//! - **Sync Node Composition**: Multiple nodes working together without async
//! - **Decision-Based Routing**: Nodes that choose different execution paths based on logic
//! - **Path Convergence**: Multiple execution paths that merge at a common endpoint
//! - **Built-in Storage Backend**: Uses CosmoFlow's MemoryStorage for simplicity
//! - **Manual Flow Control**: Explicit action handling and routing logic
//! - **Performance Optimization**: No async overhead for CPU-intensive decision logic
//!
//! ## Performance Benefits
//! - Faster compilation compared to async flow macro version
//! - Smaller binary size (no async runtime overhead)
//! - Perfect for CPU-intensive decision workflows
//! - Explicit control flow for better debugging
//!
//! ## Execution Flow
//! 1. Decision node evaluates business logic and selects execution path
//! 2. Success path processes positive outcomes OR Error path processes negative outcomes
//! 3. Both paths converge at the final node for completion
//! 4. Manual orchestration handles all routing decisions
//!
//! This example is perfect for understanding sync node composition and manual flow control.
//!
//! To run this example:
//! ```bash
//! cargo run --bin flow_composition_sync --no-default-features --features cosmoflow/storage-memory
//! ```

#![cfg(all(feature = "sync", not(feature = "async")))]

use cosmoflow::{
    Node,
    action::Action,
    node::{ExecutionContext, NodeError},
    shared_store::SharedStore,
    shared_store::backends::MemoryStorage,
};
use rand::Rng;

/// Decision node that evaluates conditions and chooses execution paths (sync version)
///
/// This node demonstrates conditional routing by analyzing business logic
/// and returning different actions based on the evaluation results.
struct DecisionNode {
    decision_criteria: f64,
}

impl DecisionNode {
    fn new(criteria: f64) -> Self {
        Self {
            decision_criteria: criteria,
        }
    }
}

impl Node<MemoryStorage> for DecisionNode {
    type PrepResult = f64;
    type ExecResult = bool;
    type Error = NodeError;

    fn name(&self) -> &str {
        "DecisionNode"
    }

    fn prep(
        &mut self,
        _store: &MemoryStorage,
        context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        println!(
            "🔄 [PREP] DecisionNode (exec_id: {}) preparing evaluation...",
            context.execution_id()
        );

        // Generate a random value for decision making
        let mut rng = rand::thread_rng();
        let random_value: f64 = rng.gen_range(0.0..1.0);

        println!(
            "   Decision criteria: {:.2}, Random value: {:.2}",
            self.decision_criteria, random_value
        );

        Ok(random_value)
    }

    fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        println!(
            "⚡ [EXEC] DecisionNode evaluating: {:.2} > {:.2}?",
            prep_result, self.decision_criteria
        );

        // Simulate some decision computation
        std::thread::sleep(std::time::Duration::from_millis(10));

        let decision = prep_result > self.decision_criteria;
        println!(
            "   Decision result: {}",
            if decision { "SUCCESS" } else { "ERROR" }
        );

        Ok(decision)
    }

    fn post(
        &mut self,
        store: &mut MemoryStorage,
        prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        // Store decision details for later analysis
        store
            .set("decision_value".to_string(), prep_result)
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        store
            .set("decision_result".to_string(), exec_result)
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        if exec_result {
            println!("✅ [POST] DecisionNode: Routing to SUCCESS path");
            Ok(Action::simple("success"))
        } else {
            println!("✅ [POST] DecisionNode: Routing to ERROR path");
            Ok(Action::simple("error"))
        }
    }
}

/// Success path node that handles positive outcomes (sync version)
///
/// This node processes successful scenarios and continues the workflow
/// toward the final convergence point.
struct SuccessNode {
    success_count: usize,
}

impl SuccessNode {
    fn new() -> Self {
        Self { success_count: 0 }
    }
}

impl Node<MemoryStorage> for SuccessNode {
    type PrepResult = ();
    type ExecResult = String;
    type Error = NodeError;

    fn name(&self) -> &str {
        "SuccessNode"
    }

    fn prep(
        &mut self,
        store: &MemoryStorage,
        context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        println!(
            "🔄 [PREP] SuccessNode (exec_id: {}) handling positive outcome...",
            context.execution_id()
        );

        // Read decision details from storage
        if let Ok(Some(decision_value)) = store.get::<f64>("decision_value") {
            println!("   Decision value was: {:.2}", decision_value);
        }

        Ok(())
    }

    fn exec(
        &mut self,
        _prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        println!("⚡ [EXEC] SuccessNode processing successful scenario...");

        self.success_count += 1;

        // Simulate success processing
        std::thread::sleep(std::time::Duration::from_millis(15));

        let success_message = format!(
            "🎉 Success processing completed! (count: {})",
            self.success_count
        );
        println!("   {}", success_message);

        Ok(success_message)
    }

    fn post(
        &mut self,
        store: &mut MemoryStorage,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        println!("✅ [POST] SuccessNode storing result and continuing...");

        // Store success result
        store
            .set("success_message".to_string(), exec_result)
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        store
            .set("path_taken".to_string(), "success".to_string())
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        Ok(Action::simple("continue"))
    }
}

/// Error path node that handles negative outcomes (sync version)
///
/// This node processes error scenarios and provides an alternative
/// execution path that also leads to the final convergence point.
struct ErrorNode {
    error_count: usize,
}

impl ErrorNode {
    fn new() -> Self {
        Self { error_count: 0 }
    }
}

impl Node<MemoryStorage> for ErrorNode {
    type PrepResult = ();
    type ExecResult = String;
    type Error = NodeError;

    fn name(&self) -> &str {
        "ErrorNode"
    }

    fn prep(
        &mut self,
        store: &MemoryStorage,
        context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        println!(
            "🔄 [PREP] ErrorNode (exec_id: {}) handling negative outcome...",
            context.execution_id()
        );

        // Read decision details from storage
        if let Ok(Some(decision_value)) = store.get::<f64>("decision_value") {
            println!("   Decision value was: {:.2}", decision_value);
        }

        Ok(())
    }

    fn exec(
        &mut self,
        _prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        println!("⚡ [EXEC] ErrorNode processing error scenario...");

        self.error_count += 1;

        // Simulate error handling
        std::thread::sleep(std::time::Duration::from_millis(12));

        let error_message = format!("⚠️ Error handled gracefully! (count: {})", self.error_count);
        println!("   {}", error_message);

        Ok(error_message)
    }

    fn post(
        &mut self,
        store: &mut MemoryStorage,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        println!("✅ [POST] ErrorNode storing result and continuing...");

        // Store error result
        store
            .set("error_message".to_string(), exec_result)
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        store
            .set("path_taken".to_string(), "error".to_string())
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        Ok(Action::simple("continue"))
    }
}

/// Final convergence node where all execution paths complete (sync version)
///
/// This node serves as the endpoint for both success and error paths,
/// demonstrating how different workflow branches can converge.
struct FinalNode;

impl Node<MemoryStorage> for FinalNode {
    type PrepResult = (String, Option<String>, Option<String>);
    type ExecResult = String;
    type Error = NodeError;

    fn name(&self) -> &str {
        "FinalNode"
    }

    fn prep(
        &mut self,
        store: &MemoryStorage,
        context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        println!(
            "🔄 [PREP] FinalNode (exec_id: {}) gathering results from all paths...",
            context.execution_id()
        );

        let path_taken: String = store
            .get("path_taken")
            .map_err(|e| NodeError::StorageError(e.to_string()))?
            .unwrap_or_else(|| "unknown".to_string());

        let success_message: Option<String> = store.get("success_message").ok().flatten();

        let error_message: Option<String> = store.get("error_message").ok().flatten();

        println!("   Path taken: {}", path_taken);

        Ok((path_taken, success_message, error_message))
    }

    fn exec(
        &mut self,
        (path_taken, success_msg, error_msg): Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        println!("⚡ [EXEC] FinalNode creating workflow summary...");

        // Simulate final processing
        std::thread::sleep(std::time::Duration::from_millis(8));

        let mut summary = String::new();
        summary.push_str("🎯 WORKFLOW SUMMARY\n");
        summary.push_str("==================\n");
        summary.push_str(&format!("Path taken: {}\n", path_taken));

        if let Some(msg) = success_msg {
            summary.push_str(&format!("Success result: {}\n", msg));
        }

        if let Some(msg) = error_msg {
            summary.push_str(&format!("Error result: {}\n", msg));
        }

        summary.push_str("Workflow completed successfully!");

        println!("   Summary generated");
        Ok(summary)
    }

    fn post(
        &mut self,
        store: &mut MemoryStorage,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        println!("✅ [POST] FinalNode workflow completed!");

        // Store final summary
        store
            .set("workflow_summary".to_string(), exec_result.clone())
            .map_err(|e| NodeError::StorageError(e.to_string()))?;

        println!("\n{}", exec_result);

        Ok(Action::simple("complete"))
    }
}

/// Manual workflow orchestrator that handles routing between nodes
struct WorkflowOrchestrator {
    decision_node: DecisionNode,
    success_node: SuccessNode,
    error_node: ErrorNode,
    final_node: FinalNode,
}

impl WorkflowOrchestrator {
    fn new(decision_threshold: f64) -> Self {
        Self {
            decision_node: DecisionNode::new(decision_threshold),
            success_node: SuccessNode::new(),
            error_node: ErrorNode::new(),
            final_node: FinalNode,
        }
    }

    fn execute(&mut self, store: &mut MemoryStorage) -> Result<(), Box<dyn std::error::Error>> {
        println!("🎮 Starting manual workflow orchestration...");
        println!("============================================\n");

        // Step 1: Execute decision node
        println!("1️⃣ Executing Decision Phase:");
        let decision_action = self.decision_node.run(store)?;
        println!("   Decision Action: {}\n", decision_action.name());

        // Step 2: Route based on decision
        let path_action = match decision_action.name().as_str() {
            "success" => {
                println!("2️⃣ Executing Success Path:");
                let action = self.success_node.run(store)?;
                println!("   Success Action: {}\n", action.name());
                action
            }
            "error" => {
                println!("2️⃣ Executing Error Path:");
                let action = self.error_node.run(store)?;
                println!("   Error Action: {}\n", action.name());
                action
            }
            _ => {
                return Err(
                    format!("Unexpected decision action: {}", decision_action.name()).into(),
                );
            }
        };

        // Step 3: Execute final convergence if appropriate
        if path_action.name() == "continue" {
            println!("3️⃣ Executing Final Convergence:");
            let final_action = self.final_node.run(store)?;
            println!("   Final Action: {}\n", final_action.name());
        }

        Ok(())
    }
}

/// Main function demonstrating sync node composition with manual orchestration
#[cfg(all(feature = "sync", not(feature = "async")))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 CosmoFlow Flow Composition (Sync Version)");
    println!("=============================================");
    println!("📦 Manual node orchestration without async complexity!\n");

    // Create shared storage
    let mut store = MemoryStorage::new();

    // Create workflow orchestrator with decision threshold
    let mut orchestrator = WorkflowOrchestrator::new(0.5);

    // Execute the workflow
    orchestrator.execute(&mut store)?;

    // Display final storage contents
    println!("📊 Final Storage Contents:");
    println!("==========================");

    if let Ok(Some(decision_result)) = store.get::<bool>("decision_result") {
        println!("Decision Result: {}", decision_result);
    }

    if let Ok(Some(path_taken)) = store.get::<String>("path_taken") {
        println!("Path Taken: {}", path_taken);
    }

    if let Ok(Some(summary)) = store.get::<String>("workflow_summary") {
        println!("\nWorkflow Summary:");
        println!("{}", summary);
    }

    println!("\n🎯 Sync Composition Benefits:");
    println!("=============================");
    println!("• ⚡ Faster compilation than async flow macro");
    println!("• 📦 Smaller binary size (no async runtime)");
    println!("• 🎯 Perfect for CPU-intensive decision logic");
    println!("• 🔧 Explicit control flow for easier debugging");
    println!("• 🚀 No async overhead for decision workflows");
    println!("• 💡 Manual orchestration for maximum control");

    println!("\n💡 Note: This example demonstrates manual node composition");
    println!("   without relying on the flow macro or async features.");
    println!("   Each routing decision is handled explicitly for");
    println!("   maximum performance and control.");

    Ok(())
}

/// Dummy main function when sync example is not compiled
#[cfg(not(all(feature = "sync", not(feature = "async"))))]
fn main() {
    // This sync example is not available when async features are enabled
}
