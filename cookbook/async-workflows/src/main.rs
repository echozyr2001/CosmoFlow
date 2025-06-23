//! Flow Builder Example - Declarative Workflow Construction with Custom Action Routing
//!
//! This example demonstrates building CosmoFlow workflows using the FlowBuilder API
//! with declarative syntax and custom action routing. The workflow showcases:
//!
//! ## Workflow Behavior
//! - **Decision Node**: Evaluates conditions and chooses between success/error paths
//! - **Success Path**: Handles positive outcomes and continues to final processing
//! - **Error Path**: Handles negative outcomes and also continues to final processing
//! - **Final Node**: Convergence point where both paths complete the workflow
//! - **Custom Actions**: Uses specific action names like "default", "error", "continue"
//!
//! ## Advanced Features Demonstrated
//! - **Declarative Syntax**: Clean, readable workflow definition using FlowBuilder
//! - **Custom Action Routing**: Specific action names for conditional workflow paths
//! - **Decision-Based Routing**: Nodes that choose different execution paths based on logic
//! - **Path Convergence**: Multiple execution paths that merge at a common endpoint
//! - **Custom Storage Backend**: Complete implementation with JSON serialization
//! - **Structured Workflow**: Explicit node and route definitions for complex flows
//!
//! ## FlowBuilder API Features
//! - **Node Registration**: `.node("id", NodeType)` syntax for clean node registration
//! - **Action Routing**: `.route("from", "action", "to")` syntax for explicit action handling
//! - **Terminal Routes**: `.terminal_route("from", "action")` for workflow termination
//! - **Type Safety**: Compile-time storage type checking and validation
//! - **Flexible Routing**: Support for multiple actions from a single node
//!
//! ## Execution Flow
//! 1. Decision node evaluates business logic and selects execution path
//! 2. Success path processes positive outcomes
//! 3. Error path processes negative outcomes (alternative branch)
//! 4. Both paths converge at the final node for completion
//! 5. Workflow demonstrates conditional routing with custom action names
//!
//! This example is perfect for understanding advanced FlowBuilder usage and conditional workflows.
//!
//! To run this example:
//! ```bash
//! cd cookbook/async-workflows && cargo run
//! ```

use cosmoflow::shared_store::backends::MemoryStorage;
use cosmoflow::{FlowBackend, SharedStore};

/// Decision node that evaluates conditions and chooses execution paths
///
/// This node demonstrates conditional routing by analyzing business logic
/// and returning different actions based on the evaluation results.
struct DecisionNode;

#[async_trait::async_trait]
impl<S: SharedStore + Send + Sync> cosmoflow::Node<S> for DecisionNode {
    type PrepResult = ();
    type ExecResult = bool;
    type Error = cosmoflow::NodeError;

    async fn prep(
        &mut self,
        _: &S,
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<(), Self::Error> {
        println!("Decision Node: Preparation phase");
        Ok(())
    }

    async fn exec(
        &mut self,
        _: (),
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<bool, Self::Error> {
        println!("Decision Node: Execution phase - simulating decision logic");
        // Simulate a decision process
        let success = true; // In real applications, this would contain actual decision logic
        Ok(success)
    }

    async fn post(
        &mut self,
        _: &mut S,
        _: (),
        result: bool,
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<cosmoflow::action::Action, Self::Error> {
        if result {
            println!("Decision Node: Choosing success path");
            Ok(cosmoflow::action::Action::simple("default"))
        } else {
            println!("Decision Node: Choosing error path");
            Ok(cosmoflow::action::Action::simple("error"))
        }
    }
}

/// Success path node that handles positive outcomes
///
/// This node processes successful scenarios and continues the workflow
/// toward the final convergence point.
struct SuccessNode;

#[async_trait::async_trait]
impl<S: SharedStore + Send + Sync> cosmoflow::Node<S> for SuccessNode {
    type PrepResult = ();
    type ExecResult = ();
    type Error = cosmoflow::NodeError;

    async fn prep(
        &mut self,
        _: &S,
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<(), Self::Error> {
        println!("Success Node: Preparation phase");
        Ok(())
    }

    async fn exec(
        &mut self,
        _: (),
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<(), Self::Error> {
        println!("Success Node: Processing successful scenario");
        Ok(())
    }

    async fn post(
        &mut self,
        _: &mut S,
        _: (),
        _: (),
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<cosmoflow::action::Action, Self::Error> {
        println!("Success Node: Continuing to final node");
        Ok(cosmoflow::action::Action::simple("continue"))
    }
}

/// Error path node that handles negative outcomes
///
/// This node processes error scenarios and provides an alternative
/// execution path that also leads to the final convergence point.
struct ErrorNode;

#[async_trait::async_trait]
impl<S: SharedStore + Send + Sync> cosmoflow::Node<S> for ErrorNode {
    type PrepResult = ();
    type ExecResult = ();
    type Error = cosmoflow::NodeError;

    async fn prep(
        &mut self,
        _: &S,
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<(), Self::Error> {
        println!("Error Node: Preparation phase");
        Ok(())
    }

    async fn exec(
        &mut self,
        _: (),
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<(), Self::Error> {
        println!("Error Node: Processing error scenario");
        Ok(())
    }

    async fn post(
        &mut self,
        _: &mut S,
        _: (),
        _: (),
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<cosmoflow::action::Action, Self::Error> {
        println!("Error Node: Continuing to final node");
        Ok(cosmoflow::action::Action::simple("continue"))
    }
}

/// Final convergence node where all execution paths complete
///
/// This node serves as the endpoint for both success and error paths,
/// demonstrating how different workflow branches can converge.
struct FinalNode;

#[async_trait::async_trait]
impl<S: SharedStore + Send + Sync> cosmoflow::Node<S> for FinalNode {
    type PrepResult = ();
    type ExecResult = ();
    type Error = cosmoflow::NodeError;

    async fn prep(
        &mut self,
        _: &S,
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<(), Self::Error> {
        println!("Final Node: Preparation phase");
        Ok(())
    }

    async fn exec(
        &mut self,
        _: (),
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<(), Self::Error> {
        println!("Final Node: Workflow ending");
        Ok(())
    }

    async fn post(
        &mut self,
        _: &mut S,
        _: (),
        _: (),
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<cosmoflow::action::Action, Self::Error> {
        println!("Final Node: Workflow completed");
        Ok(cosmoflow::action::Action::simple("complete"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use cosmoflow::FlowBuilder;

    // Create a workflow using FlowBuilder with custom action routing
    let mut workflow = FlowBuilder::<MemoryStorage>::new()
        .start_node("decision")
        .node("decision", DecisionNode)
        .node("success_path", SuccessNode)
        .node("error_path", ErrorNode)
        .node("final", FinalNode)
        .route("decision", "default", "success_path")
        .route("decision", "error", "error_path")
        .route("success_path", "continue", "final")
        .route("error_path", "continue", "final")
        .terminal_route("final", "complete")
        .build();

    let mut store = MemoryStorage::new();
    let result = workflow.execute(&mut store).await?;

    println!("Workflow execution completed!");
    println!("Steps executed: {}", result.steps_executed);
    println!("Execution path: {:?}", result.execution_path);
    println!("Success: {}", result.success);

    Ok(())
}
